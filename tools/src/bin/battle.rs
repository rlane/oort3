use oort_simulator::simulation::Code;
use oort_simulator::{scenario, simulation};
use rayon::prelude::*;
use std::default::Default;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("battle=info"))
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        panic!("Expected arguments: SCENARIO PATH PATH");
    }
    let scenario_name = args[1].clone();
    let srcs = args[2..].to_vec();

    let http_client = reqwest::Client::new();
    let mut codes = vec![];
    for src in &srcs {
        log::info!("Compiling {:?}", src);
        let src_code = std::fs::read_to_string(src).unwrap();
        if let Some(wasm) = compile(&http_client, src.to_string(), src_code).await {
            codes.push(Code::Wasm(wasm));
        } else {
            panic!("Failed to compile {src:?}");
        }
    }

    log::info!("Running simulations");
    let results = run_simulations(&scenario_name, codes);
    log::info!("Results: {:?}", results);
    match results.team0_wins.len().cmp(&results.team1_wins.len()) {
        std::cmp::Ordering::Greater => log::info!("Team 0 ({:?}) wins", srcs[0]),
        std::cmp::Ordering::Less => log::info!("Team 1 ({:?}) wins", srcs[1]),
        _ => log::info!("Draw"),
    }
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

#[derive(Default, Debug)]
struct Results {
    team0_wins: Vec<u32>,
    team1_wins: Vec<u32>,
    draws: Vec<u32>,
}

fn run_simulations(scenario_name: &str, codes: Vec<Code>) -> Results {
    let seed_statuses: Vec<(u32, scenario::Status)> = (0..10u32)
        .into_par_iter()
        .map(|seed| (seed, run_simulation(scenario_name, seed, codes.clone())))
        .collect();
    let mut results: Results = Default::default();
    for (seed, status) in seed_statuses {
        match status {
            scenario::Status::Victory { team: 0 } => results.team0_wins.push(seed),
            scenario::Status::Victory { team: 1 } => results.team1_wins.push(seed),
            scenario::Status::Draw => results.draws.push(seed),
            _ => unreachable!(),
        }
    }
    results
}

fn run_simulation(scenario_name: &str, seed: u32, codes: Vec<Code>) -> scenario::Status {
    let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);
    while sim.status() == scenario::Status::Running && sim.tick() < scenario::MAX_TICKS {
        sim.step();
    }
    sim.status()
}
