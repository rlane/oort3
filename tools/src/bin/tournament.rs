use chrono::Utc;
use clap::{Parser, Subcommand};
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use firestore::*;
use itertools::Itertools;
use oort_proto::{ShortcodeUpload, TournamentCompetitor, TournamentResults, TournamentSubmission};
use oort_simulator::{scenario, simulation};
use oort_tools::AI;
use rand::Rng;
use rayon::prelude::*;
use skillratings::{
    glicko2::{glicko2, Glicko2Config, Glicko2Rating},
    Outcomes,
};
use std::default::Default;
use std::{collections::HashMap, path::PathBuf};

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

        #[clap(short, long, default_value_t = 100)]
        rounds: i32,

        #[clap(short, long)]
        dry_run: bool,
    },
    RunUnofficial {
        scenario: String,
        shortcodes: Vec<String>,

        #[clap(short, long, default_value_t = 100)]
        rounds: i32,

        #[clap(short, long)]
        dev: bool,

        #[clap(long, default_value = "/tmp/oort-wasm-cache")]
        wasm_cache: Option<PathBuf>,
    },
    Fetch {
        scenario: String,
        out_dir: String,
    },
    Write {
        scenario: String,
        username: String,
        path: String,
    },
}

#[derive(Debug, Clone)]
struct Entrant {
    username: String,
    source_code: String,
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
            rounds,
            dry_run,
        } => cmd_run(&args.project_id, &scenario, &usernames, rounds, dry_run).await,
        SubCommand::RunUnofficial {
            scenario,
            shortcodes,
            rounds,
            dev,
            wasm_cache,
        } => cmd_run_unofficial(&scenario, &shortcodes, rounds, dev, wasm_cache).await,
        SubCommand::Fetch { scenario, out_dir } => {
            cmd_fetch(&args.project_id, &scenario, &out_dir).await
        }
        SubCommand::Write {
            scenario,
            username,
            path,
        } => cmd_write(&args.project_id, &scenario, &username, &path).await,
    }
}

async fn cmd_run(
    project_id: &str,
    scenario_name: &str,
    usernames: &[String],
    rounds: i32,
    dry_run: bool,
) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id).await?;
    scenario::load_safe(scenario_name).expect("Unknown scenario");

    let mut compiler = oort_compiler::Compiler::new();
    let entrants = get_entrants(&db, scenario_name, usernames).await?;
    let results: Vec<anyhow::Result<AI>> = entrants
        .iter()
        .map(|entrant| {
            log::info!("Compiling {:?}", entrant.username);
            let compiled_code = compiler.compile(&entrant.source_code)?;
            let compiled_code = oort_simulator::vm::precompile(&compiled_code).unwrap();
            Ok(AI {
                name: entrant.username.clone(),
                source_code: entrant.source_code.clone(),
                compiled_code,
            })
        })
        .collect();
    let ais: Vec<AI> = results.into_iter().collect::<anyhow::Result<Vec<AI>>>()?;

    log::info!("Running tournament");
    let results = run_tournament(scenario_name, &ais, rounds, run_simulation);

    display_results(&results);

    if !dry_run {
        upload_results(&db, project_id, &entrants, &results).await?;
    }

    Ok(())
}

async fn cmd_run_unofficial(
    scenario_name: &str,
    shortcodes: &[String],
    rounds: i32,
    dev: bool,
    wasm_cache: Option<PathBuf>,
) -> anyhow::Result<()> {
    scenario::load_safe(scenario_name).expect("Unknown scenario");

    let http = reqwest::Client::new();
    let ais = oort_tools::fetch_and_compile_multiple(&http, shortcodes, dev, wasm_cache.as_deref())
        .await?;

    log::info!("Running tournament");
    let results = run_tournament(scenario_name, &ais, rounds, run_simulation);

    display_results(&results);

    Ok(())
}

fn unordered_pair(name0: &str, name1: &str) -> (String, String) {
    if name0 < name1 {
        (name0.to_string(), name1.to_string())
    } else {
        (name1.to_string(), name0.to_string())
    }
}

fn run_tournament(
    scenario_name: &str,
    ais: &[AI],
    rounds: i32,
    run_sim: fn(&str, u32, &[&AI]) -> Result<Outcomes, String>,
) -> TournamentResults {
    let min_players_for_adaptive = 5;
    let min_rounds_for_adaptive = 10;

    let use_adaptive = ais.len() > min_players_for_adaptive && rounds >= min_rounds_for_adaptive;

    if use_adaptive {
        run_adaptive_tournament(scenario_name, ais, rounds, run_sim)
    } else {
        run_round_robin_tournament(scenario_name, ais, rounds, run_sim)
    }
}

fn run_round_robin_tournament(
    scenario_name: &str,
    ais: &[AI],
    rounds: i32,
    run_sim: fn(&str, u32, &[&AI]) -> Result<Outcomes, String>,
) -> TournamentResults {
    let seeds: Vec<u32> = (0..rounds).map(|_| rand::thread_rng().gen()).collect();
    let config = Glicko2Config::new();
    let mut pairings: HashMap<(String, String), f64> = HashMap::new();
    let mut ratings: Vec<Glicko2Rating> = Vec::new();
    ratings.resize_with(ais.len(), Default::default);
    let pairs: Vec<(i32, Vec<_>)> = (0..rounds)
        .flat_map(|round| (0..(ais.len())).permutations(2).map(move |x| (round, x)))
        .collect();
    let progress = indicatif::ProgressBar::new(pairs.len() as u64);
    progress.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{wide_bar} {pos}/{len} Elapsed: {elapsed_precise} ETA: {eta_precise}")
            .unwrap(),
    );
    let outcomes: Vec<(i32, Vec<_>, Result<Outcomes, String>)> = pairs
        .par_iter()
        .map(|(round, indices)| {
            let seed = seeds[*round as usize];
            let ai0: &AI = &ais[indices[0]];
            let ai1: &AI = &ais[indices[1]];
            let res = run_sim(scenario_name, seed, &[ai0, ai1]);
            progress.inc(1);
            (*round, indices.clone(), res)
        })
        .collect();
    progress.finish_and_clear();

    let mut crashes = Vec::new();
    for (round, indices, outcome_res) in outcomes {
        let outcome = match outcome_res {
            Ok(outcome) => outcome,
            Err(_) => {
                crashes.push(oort_proto::TournamentCrash {
                    seed: seeds[round as usize],
                    ais: indices.iter().map(|&idx| ais[idx].name.clone()).collect(),
                });
                Outcomes::DRAW
            }
        };
        let i0 = indices[0];
        let i1 = indices[1];
        log::debug!(
            "{} vs {} seed {}: {:?}",
            ais[i0].name,
            ais[i1].name,
            round,
            outcome
        );
        let (r0, r1) = glicko2(&ratings[i0], &ratings[i1], &outcome, &config);
        ratings[i0] = r0;
        ratings[i1] = r1;

        let increment = 1.0 / (2.0 * rounds as f64);
        if outcome == Outcomes::WIN {
            *pairings
                .entry((ais[i0].name.clone(), ais[i1].name.clone()))
                .or_default() += increment;
        } else if outcome == Outcomes::LOSS {
            *pairings
                .entry((ais[i1].name.clone(), ais[i0].name.clone()))
                .or_default() += increment;
        }
    }

    let mut competitors: Vec<_> = ais
        .iter()
        .enumerate()
        .map(|(i, x)| TournamentCompetitor {
            username: x.name.clone(),
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
        crashes,
    }
}

fn run_adaptive_tournament(
    scenario_name: &str,
    ais: &[AI],
    rounds: i32,
    run_sim: fn(&str, u32, &[&AI]) -> Result<Outcomes, String>,
) -> TournamentResults {
    let seeds: Vec<u32> = (0..rounds).map(|_| rand::thread_rng().gen()).collect();
    let config = Glicko2Config::new();
    let s_base = 5;

    log::info!(
        "Running adaptive tournament for {} players (s_base = {}, rounds = {})",
        ais.len(),
        s_base,
        rounds
    );

    // 1. Base Phase (First s_base rounds)
    let base_pairs: Vec<(i32, Vec<usize>)> = (0..s_base)
        .flat_map(|round| (0..(ais.len())).permutations(2).map(move |x| (round, x)))
        .collect();

    let progress = indicatif::ProgressBar::new(base_pairs.len() as u64);
    progress.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("[Base Phase] {wide_bar} {pos}/{len} Elapsed: {elapsed_precise} ETA: {eta_precise}")
            .unwrap(),
    );

    let base_outcomes: Vec<(i32, Vec<usize>, Result<Outcomes, String>)> = base_pairs
        .par_iter()
        .map(|(round, indices)| {
            let seed = seeds[*round as usize];
            let ai0: &AI = &ais[indices[0]];
            let ai1: &AI = &ais[indices[1]];
            let res = run_sim(scenario_name, seed, &[ai0, ai1]);
            progress.inc(1);
            (*round, indices.clone(), res)
        })
        .collect();
    progress.finish_and_clear();

    // Track stats for the base phase
    let mut wins: HashMap<(String, String), usize> = HashMap::new();
    let mut total_played: HashMap<(String, String), usize> = HashMap::new();
    let mut ratings: Vec<Glicko2Rating> = vec![Default::default(); ais.len()];

    for (_round, indices, outcome_res) in &base_outcomes {
        let outcome = match outcome_res {
            Ok(o) => o,
            Err(_) => &Outcomes::DRAW,
        };
        let i0 = indices[0];
        let i1 = indices[1];
        let (r0, r1) = glicko2(&ratings[i0], &ratings[i1], outcome, &config);
        ratings[i0] = r0;
        ratings[i1] = r1;

        let name0 = &ais[i0].name;
        let name1 = &ais[i1].name;

        let key = unordered_pair(name0, name1);
        *total_played.entry(key.clone()).or_default() += 1;

        if *outcome == Outcomes::WIN {
            *wins.entry((name0.clone(), name1.clone())).or_default() += 1;
        } else if *outcome == Outcomes::LOSS {
            *wins.entry((name1.clone(), name0.clone())).or_default() += 1;
        }
    }

    // Identify top 8 players based on intermediate ratings
    let mut intermediate_indices: Vec<usize> = (0..ais.len()).collect();
    intermediate_indices.sort_by(|&a, &b| {
        ratings[b].rating.partial_cmp(&ratings[a].rating).unwrap()
    });
    let top_tier_count = std::cmp::min(8, ais.len());
    let top_tier: Vec<usize> = intermediate_indices[0..top_tier_count].to_vec();

    // 2. Identify which pairs need refinement
    let mut refined_pairs = Vec::new();
    for i in 0..ais.len() {
        for j in (i + 1)..ais.len() {
            let name_i = &ais[i].name;
            let name_j = &ais[j].name;
            let key = unordered_pair(name_i, name_j);

            let played = total_played.get(&key).copied().unwrap_or(0);
            if played == 0 {
                continue;
            }

            let wins_i = wins.get(&(name_i.clone(), name_j.clone())).copied().unwrap_or(0);
            let wins_j = wins.get(&(name_j.clone(), name_i.clone())).copied().unwrap_or(0);
            let win_rate_i = wins_i as f64 / played as f64;
            let win_rate_j = wins_j as f64 / played as f64;

            let rating_diff = (ratings[i].rating - ratings[j].rating).abs();

            let close_ratings = rating_diff < 100.0;

            let unexpected_outcome = if ratings[i].rating < ratings[j].rating - 150.0 {
                win_rate_i >= 0.30
            } else if ratings[j].rating < ratings[i].rating - 150.0 {
                win_rate_j >= 0.30
            } else {
                false
            };

            let both_top_tier = top_tier.contains(&i) && top_tier.contains(&j);

            if close_ratings || unexpected_outcome || both_top_tier {
                refined_pairs.push((i, j));
            }
        }
    }

    // 3. Refinement Phase (Remaining rounds for selected pairs)
    let mut refinement_matchups = Vec::new();
    for &(i, j) in &refined_pairs {
        for round in s_base..rounds {
            refinement_matchups.push((round, vec![i, j]));
            refinement_matchups.push((round, vec![j, i]));
        }
    }

    let refinement_outcomes: Vec<(i32, Vec<usize>, Result<Outcomes, String>)> = if !refinement_matchups.is_empty() {
        let progress = indicatif::ProgressBar::new(refinement_matchups.len() as u64);
        progress.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("[Refinement Phase] {wide_bar} {pos}/{len} Elapsed: {elapsed_precise} ETA: {eta_precise}")
                .unwrap(),
        );

        let outcomes: Vec<(i32, Vec<usize>, Result<Outcomes, String>)> = refinement_matchups
            .par_iter()
            .map(|(round, indices)| {
                let seed = seeds[*round as usize];
                let ai0: &AI = &ais[indices[0]];
                let ai1: &AI = &ais[indices[1]];
                let res = run_sim(scenario_name, seed, &[ai0, ai1]);
                progress.inc(1);
                (*round, indices.clone(), res)
            })
            .collect();
        progress.finish_and_clear();
        outcomes
    } else {
        Vec::new()
    };

    // 4. Combine and update final ratings
    let mut all_outcomes = base_outcomes;
    all_outcomes.extend(refinement_outcomes);
    all_outcomes.sort_by_key(|x| x.0);

    let mut final_ratings: Vec<Glicko2Rating> = vec![Default::default(); ais.len()];
    wins.clear();
    total_played.clear();
    let mut crashes = Vec::new();

    for (_round, indices, outcome_res) in &all_outcomes {
        let outcome = match outcome_res {
            Ok(o) => o,
            Err(_) => {
                crashes.push(oort_proto::TournamentCrash {
                    seed: seeds[*_round as usize],
                    ais: indices.iter().map(|&idx| ais[idx].name.clone()).collect(),
                });
                &Outcomes::DRAW
            }
        };
        let i0 = indices[0];
        let i1 = indices[1];
        let (r0, r1) = glicko2(&final_ratings[i0], &final_ratings[i1], outcome, &config);
        final_ratings[i0] = r0;
        final_ratings[i1] = r1;

        let name0 = &ais[i0].name;
        let name1 = &ais[i1].name;

        let key = unordered_pair(name0, name1);
        *total_played.entry(key.clone()).or_default() += 1;

        if *outcome == Outcomes::WIN {
            *wins.entry((name0.clone(), name1.clone())).or_default() += 1;
        } else if *outcome == Outcomes::LOSS {
            *wins.entry((name1.clone(), name0.clone())).or_default() += 1;
        }
    }

    let mut competitors: Vec<_> = ais
        .iter()
        .enumerate()
        .map(|(i, x)| TournamentCompetitor {
            username: x.name.clone(),
            shortcode: "".to_string(),
            rating: final_ratings[i].rating,
        })
        .collect();
    competitors.sort_by_key(|c| (-c.rating * 1e6) as i64);

    let mut win_matrix: Vec<f64> = vec![];
    for competitor in &competitors {
        for other_competitor in &competitors {
            let name0 = &competitor.username;
            let name1 = &other_competitor.username;

            if name0 == name1 {
                win_matrix.push(0.0);
                continue;
            }

            let key = unordered_pair(name0, name1);

            let played = total_played.get(&key).copied().unwrap_or(0);
            let w = wins.get(&(name0.clone(), name1.clone())).copied().unwrap_or(0);

            let win_rate = if played > 0 {
                w as f64 / played as f64
            } else {
                0.0
            };
            win_matrix.push(win_rate);
        }
    }

    let total_rr_sims = rounds * (ais.len() as i32) * ((ais.len() - 1) as i32);
    let actual_sims = all_outcomes.len();
    let savings = (1.0 - (actual_sims as f64 / total_rr_sims as f64)) * 100.0;
    log::info!(
        "Adaptive tournament complete. Ran {} simulations instead of {} ({:.1}% saved). Refined {}/{} pairs.",
        actual_sims,
        total_rr_sims,
        savings,
        refined_pairs.len(),
        (ais.len() * (ais.len() - 1)) / 2
    );

    TournamentResults {
        scenario_name: scenario_name.to_string(),
        competitors,
        win_matrix,
        crashes,
    }
}

fn run_simulation(scenario_name: &str, seed: u32, ais: &[&AI]) -> Result<Outcomes, String> {
    let f = move || {
        let codes: Vec<_> = ais.iter().map(|x| x.compiled_code.clone()).collect();
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
    };
    match ::std::panic::catch_unwind(f) {
        Ok(x) => Ok(x),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            log::error!("Simulation panicked: {}", msg);
            Err(msg)
        }
    }
}

fn display_results(results: &TournamentResults) {
    if !results.crashes.is_empty() {
        let num_crashes = results.crashes.len();
        let displayed_crashes = std::cmp::min(10, num_crashes);
        println!("Crashes ({displayed_crashes} of {num_crashes}):");
        for crash in results.crashes.iter().take(displayed_crashes) {
            println!("  Seed: {}, AIs: {:?}", crash.seed, crash.ais);
        }
        println!();
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Name", "Rating"]);
    for competitor in &results.competitors {
        table.add_row(vec![
            competitor.username.clone(),
            format!("{:.0}", competitor.rating),
        ]);
    }
    println!("Scenario: {}", results.scenario_name);
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
}

async fn upload_results(
    db: &FirestoreDb,
    project_id: &str,
    entrants: &[Entrant],
    results: &TournamentResults,
) -> anyhow::Result<()> {
    log::info!("Uploading to database...");

    let tournament_id = format!(
        "{}.{}.{}",
        results.scenario_name,
        Utc::now().format("%Y%m%d"),
        rand::thread_rng().gen_range(0..10000)
    );

    let mut results = results.clone();
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
        db.create_obj::<_, (), _>("shortcode", Some(&shortcode), &obj, None)
            .await?;
        competitor.shortcode = shortcode;
    }
    db.create_obj::<_, (), _>("tournament_results", Some(&tournament_id), &results, None)
        .await?;
    println!();
    if project_id == "oort-dev" {
        println!("Uploaded to http://localhost:8080/tournament/{tournament_id}");
    } else {
        println!("Uploaded to https://oort.rs/tournament/{tournament_id}");
    }
    Ok(())
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
                    FirestoreQueryFilterComposite::new(
                        vec![FirestoreQueryFilter::Compare(Some(
                            FirestoreQueryFilterCompare::Equal(
                                "scenario_name".into(),
                                scenario_name.into(),
                            ),
                        ))],
                        FirestoreQueryFilterCompositeOperator::And,
                    ),
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

async fn cmd_write(
    project_id: &str,
    scenario_name: &str,
    username: &str,
    path: &str,
) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id).await?;
    let userid = format!("admin-{}", username);
    let docid = format!("{}.{}", scenario_name, userid);
    let code = std::fs::read_to_string(path)?;
    let msg = TournamentSubmission {
        userid,
        username: username.to_owned(),
        timestamp: Utc::now(),
        scenario_name: scenario_name.to_owned(),
        code,
    };
    db.update_obj::<_, (), _>("tournament", docid, &msg, None, None, None)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_run_simulation(_scenario_name: &str, _seed: u32, ais: &[&AI]) -> Result<Outcomes, String> {
        let name0 = &ais[0].name;
        let name1 = &ais[1].name;

        if (name0 == "bot4" && name1 == "bot5") || (name0 == "bot5" && name1 == "bot4") {
            return Err("mock panic".to_string());
        }

        let outcome = if name0 == "bot0" && name1 == "bot5" {
            Outcomes::WIN
        } else if name0 == "bot5" && name1 == "bot0" {
            Outcomes::LOSS
        } else {
            if name0 < name1 {
                Outcomes::WIN
            } else {
                Outcomes::LOSS
            }
        };
        Ok(outcome)
    }

    #[test]
    fn test_adaptive_tournament() {
        let _ = env_logger::builder().is_test(true).try_init();

        let names = vec![
            "bot0".to_string(),
            "bot1".to_string(),
            "bot2".to_string(),
            "bot3".to_string(),
            "bot4".to_string(),
            "bot5".to_string(),
        ];

        let ais: Vec<AI> = names
            .into_iter()
            .map(|name| AI {
                name,
                source_code: String::new(),
                compiled_code: oort_simulator::simulation::Code::None,
            })
            .collect();

        let results = run_tournament("fighter_duel", &ais, 12, mock_run_simulation);

        assert_eq!(results.competitors.len(), 6);
        assert_eq!(results.win_matrix.len(), 36);
        for competitor in &results.competitors {
            assert!(competitor.rating > 0.0);
        }

        // We expect bot4 vs bot5 matches to crash.
        // Base phase (5 rounds) + Refinement phase (7 rounds) = 12 rounds * 2 directions = 24 crashes.
        assert_eq!(results.crashes.len(), 24);
        for crash in &results.crashes {
            assert!(crash.ais.contains(&"bot4".to_string()));
            assert!(crash.ais.contains(&"bot5".to_string()));
        }

        let ranking: Vec<String> = results.competitors.iter().map(|c| c.username.clone()).collect();
        assert_eq!(ranking, vec!["bot0", "bot1", "bot2", "bot3", "bot5", "bot4"]);
    }

    #[test]
    fn test_round_robin_tournament() {
        let _ = env_logger::builder().is_test(true).try_init();

        let names = vec![
            "bot0".to_string(),
            "bot1".to_string(),
            "bot2".to_string(),
            "bot3".to_string(),
            "bot4".to_string(),
            "bot5".to_string(),
        ];

        let ais: Vec<AI> = names
            .into_iter()
            .map(|name| AI {
                name,
                source_code: String::new(),
                compiled_code: oort_simulator::simulation::Code::None,
            })
            .collect();

        let results = run_tournament("fighter_duel", &ais, 3, mock_run_simulation);

        assert_eq!(results.competitors.len(), 6);
        assert_eq!(results.win_matrix.len(), 36);
        for competitor in &results.competitors {
            assert!(competitor.rating > 0.0);
        }

        // We expect bot4 vs bot5 matches to crash.
        // 3 rounds * 2 directions = 6 crashes.
        assert_eq!(results.crashes.len(), 6);
        for crash in &results.crashes {
            assert!(crash.ais.contains(&"bot4".to_string()));
            assert!(crash.ais.contains(&"bot5".to_string()));
        }

        let ranking: Vec<String> = results.competitors.iter().map(|c| c.username.clone()).collect();
        assert_eq!(ranking, vec!["bot0", "bot1", "bot2", "bot3", "bot5", "bot4"]);
    }
}
