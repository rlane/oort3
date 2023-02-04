use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use itertools::Itertools;
use oort_simulator::simulation::Code;
use oort_simulator::{scenario, simulation};
use rayon::prelude::*;
use skillratings::{
    config::Glicko2Config, glicko2::glicko2, outcomes::Outcomes, rating::Glicko2Rating,
};
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

    let http_client = reqwest::Client::new();
    let mut competitors = vec![];
    for src in &srcs {
        log::info!("Compiling {:?}", src);
        let path = Path::new(src);
        let name = path.file_stem().unwrap().to_str().unwrap();
        let src_code = std::fs::read_to_string(src).unwrap();
        if let Some(wasm) = compile(&http_client, src.to_string(), src_code).await {
            competitors.push(Competitor {
                name: name.to_string(),
                code: Code::Wasm(wasm),
                rating: Default::default(),
            });
        } else {
            panic!("Failed to compile {src:?}");
        }
    }

    log::info!("Running tournament");
    let mut competitors = run_tournament(&scenario_name, competitors);

    competitors.sort_by_key(|c| (-c.rating.rating * 1e6) as i64);
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Name", "Rating"]);
    for competitor in &competitors {
        table.add_row(vec![
            competitor.name.clone(),
            format!("{:.0}", competitor.rating.rating),
        ]);
    }
    println!("Scenario: {scenario_name}");
    println!("{table}");

    Ok(())
}

async fn compile(client: &reqwest::Client, name: String, code: String) -> Option<Vec<u8>> {
    let url = "http://localhost:8081/compile";
    let result = client.post(url).body(code).send().await;
    let response = result.unwrap().error_for_status();
    match response {
        Ok(response) => Some(response.bytes().await.unwrap().as_ref().into()),
        Err(e) => {
            log::warn!("Failed to compile {:?}: {}", name, e);
            None
        }
    }
}

#[derive(Debug, Clone)]
struct Competitor {
    name: String,
    code: Code,
    rating: Glicko2Rating,
}

fn run_tournament(scenario_name: &str, mut competitors: Vec<Competitor>) -> Vec<Competitor> {
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
                competitors[i0].rating,
                competitors[i1].rating,
                outcome,
                &config,
            );
            competitors[i0].rating = r0;
            competitors[i1].rating = r1;
        }
    }

    competitors
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
