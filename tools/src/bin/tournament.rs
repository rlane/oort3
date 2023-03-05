use chrono::Utc;
use clap::{Parser, Subcommand};
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use firestore::*;
use itertools::Itertools;
use oort_proto::{ShortcodeUpload, TournamentCompetitor, TournamentResults, TournamentSubmission};
use oort_simulator::simulation::Code;
use oort_simulator::{scenario, simulation};
use rand::Rng;
use rayon::prelude::*;
use skillratings::{
    glicko2::{glicko2, Glicko2Config, Glicko2Rating},
    Outcomes,
};
use std::collections::HashMap;
use std::default::Default;

#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    #[clap(short, long, value_parser, default_value_t = String::from("oort-319301"))]
    project_id: String,

    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    Run {
        scenario: String,
        usernames: Vec<String>,

        #[clap(short, long)]
        dry_run: bool,
    },
    Fetch {
        scenario: String,
        out_dir: String,
    },
}

#[derive(Debug, Clone)]
struct Entrant {
    username: String,
    source_code: String,
    compiled_code: Option<Code>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("tournament=info"))
        .init();

    let args = Arguments::parse();
    match args.cmd {
        SubCommand::Run {
            scenario,
            usernames,
            dry_run,
        } => cmd_run(&args.project_id, &scenario, &usernames, dry_run).await,
        SubCommand::Fetch { scenario, out_dir } => {
            cmd_fetch(&args.project_id, &scenario, &out_dir).await
        }
    }
}

async fn cmd_run(
    project_id: &str,
    scenario_name: &str,
    usernames: &[String],
    dry_run: bool,
) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id).await?;
    scenario::load_safe(scenario_name).expect("Unknown scenario");

    let tournament_id = format!(
        "{}.{}.{}",
        scenario_name,
        Utc::now().format("%Y%m%d"),
        rand::thread_rng().gen_range(0..10000)
    );
    log::info!("Running tournament {}", tournament_id);

    let mut compiler = oort_compiler::Compiler::new();
    let mut entrants = get_entrants(&db, scenario_name, usernames).await?;
    for entrant in entrants.iter_mut() {
        log::info!("Compiling {:?}", entrant.username);
        match compiler.compile(&entrant.source_code) {
            Ok(wasm) => entrant.compiled_code = Some(Code::Wasm(wasm)),
            Err(e) => {
                panic!("Failed to compile {:?}: {e}", entrant.username);
            }
        }
    }

    log::info!("Running tournament");
    let mut results = run_tournament(scenario_name, &entrants);

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Name", "Rating"]);
    for competitor in &results.competitors {
        table.add_row(vec![
            competitor.username.clone(),
            format!("{:.0}", competitor.rating),
        ]);
    }
    println!("Scenario: {scenario_name}");
    println!("{table}");
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    let mut header: Vec<String> = results
        .competitors
        .iter()
        .map(|x| x.username.clone())
        .collect();
    header.insert(0, "Winner / Loser".to_owned());
    table.set_header(header);
    let mut index = 0;
    for name0 in results.competitors.iter().map(|x| &x.username) {
        let mut row = vec![name0.clone()];
        for name1 in results.competitors.iter().map(|x| &x.username) {
            if name0 == name1 {
                row.push("".to_owned());
                index += 1;
                continue;
            }
            let frac = results.win_matrix[index];
            row.push(format!("{}", (frac * 100.0).round()));
            index += 1;
        }
        table.add_row(row);
    }
    println!("{table}");

    if !dry_run {
        log::info!("Uploading to database...");
        for competitor in results.competitors.iter_mut() {
            let entrant = entrants
                .iter()
                .find(|x| x.username == competitor.username)
                .unwrap();
            let obj = ShortcodeUpload {
                userid: "".to_string(), // TODO
                username: competitor.username.clone(),
                timestamp: Utc::now(),
                code: entrant.source_code.clone(),
            };
            let shortcode = format!("{tournament_id}.{}", competitor.username);
            db.create_obj("shortcode", &shortcode, &obj).await?;
            competitor.shortcode = shortcode;
        }
        db.create_obj("tournament_results", &tournament_id, &results)
            .await?;
        println!();
        if project_id == "oort-dev" {
            println!("Uploaded to http://localhost:8080/tournament/{tournament_id}");
        } else {
            println!("Uploaded to https://oort.rs/tournament/{tournament_id}");
        }
    }

    Ok(())
}

fn run_tournament(scenario_name: &str, entrants: &[Entrant]) -> TournamentResults {
    let mut pairings: HashMap<(String, String), f64> = HashMap::new();
    let config = Glicko2Config::new();
    let mut ratings: Vec<Glicko2Rating> = Vec::new();
    ratings.resize_with(entrants.len(), Default::default);
    let pairs: Vec<_> = (0..(entrants.len())).permutations(2).collect();
    let rounds = 10;
    for round in 0..rounds {
        let outcomes: Vec<_> = pairs
            .par_iter()
            .map(|indices| {
                let seed = round as u32;
                let i0 = indices[0];
                let i1 = indices[1];
                (
                    indices,
                    run_simulation(scenario_name, seed, &[&entrants[i0], &entrants[i1]]),
                )
            })
            .collect();

        for (indices, outcome) in outcomes {
            let i0 = indices[0];
            let i1 = indices[1];
            let (r0, r1) = glicko2(&ratings[i0], &ratings[i1], &outcome, &config);
            ratings[i0] = r0;
            ratings[i1] = r1;

            let increment = 1.0 / (2.0 * rounds as f64);
            if outcome == Outcomes::WIN {
                *pairings
                    .entry((entrants[i0].username.clone(), entrants[i1].username.clone()))
                    .or_default() += increment;
            } else if outcome == Outcomes::LOSS {
                *pairings
                    .entry((entrants[i1].username.clone(), entrants[i0].username.clone()))
                    .or_default() += increment;
            }
        }
    }

    let mut competitors: Vec<_> = entrants
        .iter()
        .enumerate()
        .map(|(i, x)| TournamentCompetitor {
            username: x.username.clone(),
            shortcode: "".to_string(),
            rating: ratings[i].rating,
        })
        .collect();
    competitors.sort_by_key(|c| (-c.rating * 1e6) as i64);

    let mut win_matrix: Vec<f64> = vec![];
    for competitor in &competitors {
        for other_competitor in &competitors {
            win_matrix.push(
                pairings
                    .get(&(
                        competitor.username.clone(),
                        other_competitor.username.clone(),
                    ))
                    .copied()
                    .unwrap_or_default(),
            );
        }
    }

    TournamentResults {
        scenario_name: scenario_name.to_string(),
        competitors,
        win_matrix,
    }
}

fn run_simulation(scenario_name: &str, seed: u32, entrants: &[&Entrant]) -> Outcomes {
    let codes: Vec<_> = entrants
        .iter()
        .map(|x| x.compiled_code.as_ref().unwrap().clone())
        .collect();
    let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);
    while sim.status() == scenario::Status::Running && sim.tick() < scenario::MAX_TICKS {
        sim.step();
    }
    match sim.status() {
        scenario::Status::Victory { team: 0 } => Outcomes::WIN,
        scenario::Status::Victory { team: 1 } => Outcomes::LOSS,
        scenario::Status::Draw => Outcomes::DRAW,
        _ => unreachable!(),
    }
}

async fn get_entrants(
    db: &FirestoreDb,
    scenario_name: &str,
    usernames: &[String],
) -> anyhow::Result<Vec<Entrant>> {
    let msgs: Vec<TournamentSubmission> = db
        .query_obj(
            FirestoreQueryParams::new("tournament".into())
                .with_filter(FirestoreQueryFilter::Composite(
                    FirestoreQueryFilterComposite::new(vec![FirestoreQueryFilter::Compare(Some(
                        FirestoreQueryFilterCompare::Equal(
                            "scenario_name".into(),
                            scenario_name.into(),
                        ),
                    ))]),
                ))
                .with_order_by(vec![
                    FirestoreQueryOrder::new(
                        "username".to_owned(),
                        FirestoreQueryDirection::Ascending,
                    ),
                    FirestoreQueryOrder::new(
                        "timestamp".to_owned(),
                        FirestoreQueryDirection::Ascending,
                    ),
                ]),
        )
        .await?;

    let mut map: HashMap<String, TournamentSubmission> = HashMap::new();
    for msg in msgs {
        if usernames.is_empty() || usernames.contains(&msg.username) {
            map.insert(msg.username.clone(), msg);
        }
    }

    let mut entrants = vec![];
    for msg in map.into_values() {
        entrants.push(Entrant {
            username: msg.username,
            source_code: msg.code,
            compiled_code: None,
        });
    }
    entrants.sort_by_key(|x| x.username.clone());
    Ok(entrants)
}

async fn cmd_fetch(project_id: &str, scenario_name: &str, out_dir: &str) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id).await?;
    let entrants = get_entrants(&db, scenario_name, &[]).await?;
    std::fs::create_dir_all(out_dir).unwrap();
    for entrant in entrants {
        let filename = format!("{}/{}.rs", &out_dir, entrant.username);
        std::fs::write(&filename, &entrant.source_code).unwrap();
        print!("{filename} ");
    }
    println!();

    Ok(())
}
