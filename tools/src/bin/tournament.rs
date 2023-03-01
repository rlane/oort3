use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use itertools::Itertools;
use oort_simulator::simulation::Code;
use oort_simulator::{scenario, simulation};
use rayon::prelude::*;
use skillratings::{
    glicko2::{glicko2, Glicko2Config, Glicko2Rating},
    Outcomes,
};
use std::collections::HashMap;
use std::default::Default;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("tournament=info"))
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        panic!("Expected arguments: SCENARIO PATH...");
    }
    let scenario_name = args[1].clone();
    let srcs = args[2..].to_vec();

    scenario::load_safe(&scenario_name).expect("Unknown scenario");

    let mut compiler = oort_compiler::Compiler::new();
    let mut competitors = vec![];
    for src in &srcs {
        log::info!("Compiling {:?}", src);
        let path = Path::new(src);
        let name = path.file_stem().unwrap().to_str().unwrap();
        let src_code = std::fs::read_to_string(src).unwrap();
        match compiler.compile(&src_code) {
            Ok(wasm) => {
                competitors.push(Competitor {
                    name: name.to_string(),
                    code: Code::Wasm(wasm),
                    rating: Default::default(),
                });
            }
            Err(e) => {
                panic!("Failed to compile {src:?}: {e}");
            }
        }
    }

    log::info!("Running tournament");
    let mut results = run_tournament(&scenario_name, competitors);

    results
        .competitors
        .sort_by_key(|c| (-c.rating.rating * 1e6) as i64);
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Name", "Rating"]);
    for competitor in &results.competitors {
        table.add_row(vec![
            competitor.name.clone(),
            format!("{:.0}", competitor.rating.rating),
        ]);
    }
    println!("Scenario: {scenario_name}");
    println!("{table}");
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    let mut header: Vec<String> = results.competitors.iter().map(|x| x.name.clone()).collect();
    header.insert(0, "Winner / Loser".to_owned());
    table.set_header(header);
    for name0 in results.competitors.iter().map(|x| &x.name) {
        let mut row = vec![name0.clone()];
        for name1 in results.competitors.iter().map(|x| &x.name) {
            if name0 == name1 {
                row.push("".to_owned());
                continue;
            }
            let frac = results
                .pairings
                .get(&(name0.clone(), name1.clone()))
                .unwrap_or(&0.0);
            row.push(format!("{}", (frac * 100.0).round()));
        }
        table.add_row(row);
    }
    println!("{table}");

    Ok(())
}

#[derive(Debug, Clone)]
struct TournamentResults {
    competitors: Vec<Competitor>,
    pairings: HashMap<(String, String), f64>,
}

#[derive(Debug, Clone)]
struct Competitor {
    name: String,
    code: Code,
    rating: Glicko2Rating,
}

fn run_tournament(scenario_name: &str, mut competitors: Vec<Competitor>) -> TournamentResults {
    let mut pairings: HashMap<(String, String), f64> = HashMap::new();
    let config = Glicko2Config::new();
    let rounds = 10;
    for round in 0..rounds {
        let pairs: Vec<_> = (0..(competitors.len()))
            .permutations(2)
            .enumerate()
            .collect();
        let base_seed = (round * pairs.len()) as u32;
        let outcomes: Vec<_> = pairs
            .par_iter()
            .map(|(seed, indices)| {
                let seed = base_seed + *seed as u32;
                let i0 = indices[0];
                let i1 = indices[1];
                (
                    indices,
                    run_simulation(scenario_name, seed, &[&competitors[i0], &competitors[i1]]),
                )
            })
            .collect();

        for (indices, outcome) in outcomes {
            let i0 = indices[0];
            let i1 = indices[1];
            let (r0, r1) = glicko2(
                &competitors[i0].rating,
                &competitors[i1].rating,
                &outcome,
                &config,
            );
            competitors[i0].rating = r0;
            competitors[i1].rating = r1;

            let increment = 1.0 / (2.0 * rounds as f64);
            if outcome == Outcomes::WIN {
                *pairings
                    .entry((competitors[i0].name.clone(), competitors[i1].name.clone()))
                    .or_default() += increment;
            } else if outcome == Outcomes::LOSS {
                *pairings
                    .entry((competitors[i1].name.clone(), competitors[i0].name.clone()))
                    .or_default() += increment;
            }
        }
    }

    TournamentResults {
        competitors,
        pairings,
    }
}

fn run_simulation(scenario_name: &str, seed: u32, competitors: &[&Competitor]) -> Outcomes {
    let codes: Vec<_> = competitors.iter().map(|c| c.code.clone()).collect();
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
