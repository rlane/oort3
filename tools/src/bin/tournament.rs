use oort_simulator::simulation::Code;
use oort_simulator::{scenario, simulation};
use rayon::prelude::*;
use std::default::Default;

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

    let http_client = reqwest::Client::new();
    let mut competitors = vec![];
    for src in &srcs {
        log::info!("Compiling {:?}", src);
        let name = src;
        let src_code = std::fs::read_to_string(src.to_string()).unwrap();
        if let Some(wasm) = compile(&http_client, src.to_string(), src_code).await {
            competitors.push(Competitor {
                name: name.to_string(),
                code: Code::Wasm(wasm),
            });
        } else {
            panic!("Failed to compile {:?}", src);
        }
    }

    log::info!("Running tournament");
    run_tournament(&scenario_name, competitors);
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
}

fn run_tournament(scenario_name: &str, mut competitors: Vec<Competitor>) {
    let mut new_competitors = vec![];
    while competitors.len() > 1 {
        log::info!(
            "Running tournament iteration with {} competitors",
            competitors.len()
        );
        for cs in competitors.chunks(2) {
            log::info!("Competitors: {} vs {}", cs[0].name, cs[1].name);
            if let Some(winner) = run_simulations(scenario_name, cs, 3) {
                log::info!("Winner: {}", winner.name);
                new_competitors.push(winner.clone());
            } else {
                log::info!("Draw");
                // TODO handle draws
            }
        }
        competitors = new_competitors;
        new_competitors = vec![];
    }
    log::info!("Overall winner: {}", competitors[0].name);
}

fn run_simulations<'a>(
    scenario_name: &str,
    competitors: &'a [Competitor],
    n: u32,
) -> Option<&'a Competitor> {
    let codes: Vec<_> = competitors.iter().map(|c| c.code.clone()).collect();
    let seed_statuses: Vec<(u32, scenario::Status)> = (0..n)
        .into_par_iter()
        .map(|seed| (seed, run_simulation(scenario_name, seed, &codes)))
        .collect();
    let mut wins = vec![];
    wins.resize(competitors.len(), 0);
    for (_, status) in seed_statuses {
        match status {
            scenario::Status::Victory { team } => wins[team as usize] += 1,
            scenario::Status::Draw => {}
            _ => unreachable!(),
        }
    }
    let winner_index = wins.iter().enumerate().max_by_key(|x| x.1).map(|x| x.0);
    if let Some(i) = winner_index {
        Some(&competitors[i])
    } else {
        None
    }
}

fn run_simulation(scenario_name: &str, seed: u32, codes: &[Code]) -> scenario::Status {
    let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);
    while sim.status() == scenario::Status::Running && sim.tick() < scenario::MAX_TICKS {
        sim.step();
    }
    sim.status()
}
