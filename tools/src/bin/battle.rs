use clap::Parser;
use oort_simulator::simulation::Code;
use oort_simulator::{scenario, simulation};
use oort_tools::AI;
use rayon::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::default::Default;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    scenario: String,
    shortcodes: Vec<String>,

    #[clap(short, long, default_value = "10")]
    rounds: u32,

    #[clap(short, long)]
    dev: bool,

    #[clap(long, default_value = "/tmp/oort-wasm-cache")]
    wasm_cache: Option<PathBuf>,

    #[clap(long)]
    local_compiler: bool,

    #[clap(short, long, help = "Output results in JSON format")]
    json: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("battle=info"))
        .init();

    let args = Arguments::parse();
    scenario::load_safe(&args.scenario).expect("Unknown scenario");
    if args.shortcodes.len() < 2 {
        panic!("Expected at least two shortcodes");
    }

    log::info!("Compiling AIs");
    let ais = if args.local_compiler {
        let mut compiler = oort_compiler::Compiler::new();
        args.shortcodes
            .iter()
            .map(|x| {
                let src = std::fs::read_to_string(x).unwrap();
                let wasm = compiler.compile(&src).unwrap();
                AI {
                    name: x.clone(),
                    source_code: src,
                    compiled_code: Code::Wasm(wasm),
                }
            })
            .collect::<Vec<_>>()
    } else {
        let http = reqwest::Client::new();
        oort_tools::fetch_and_compile_multiple(
            &http,
            &args.shortcodes,
            args.dev,
            args.wasm_cache.as_deref(),
        )
        .await?
    };

    log::info!("Running simulations");
    let player0 = &ais[0];
    let results_per_opponent = ais[1..]
        .par_iter()
        .map(|player1| {
            let codes = vec![player0.compiled_code.clone(), player1.compiled_code.clone()];
            let results = run_simulations(&args.scenario, codes, args.rounds);
            (player1, results)
        })
        .collect::<Vec<_>>();

    if args.json {
        let res = results_per_opponent
            .iter()
            .map(|(p, r)| json!({
                "opponent": p.name,
                "wins": r.team0_wins,
                "losses": r.team1_wins,
                "draws": r.draws,
                "times": r.times,
                "average_time": r.times.iter().sum::<f64>() / r.times.len() as f64,
            }))
            .collect::<Vec<_>>();
        serde_json::to_writer(std::io::stdout(), &res)?;
        return Ok(());
    }
    for (player1, results) in results_per_opponent {
        let n = 10;
        println!("{} vs {}:", player0.name, player1.name);
        println!(
            "  Wins: {} {:?}",
            results.team0_wins.len(),
            &results.team0_wins[..].iter().take(n).collect::<Vec<_>>()
        );
        println!(
            "  Losses: {} {:?}",
            results.team1_wins.len(),
            &results.team1_wins[..].iter().take(n).collect::<Vec<_>>()
        );
        println!(
            "  Draws: {} {:?}",
            results.draws.len(),
            &results.draws[..].iter().take(n).collect::<Vec<_>>()
        );
        print!("  Times: [");
        for (i, time) in results.times.iter().enumerate().take(n) {
            if i > 0 {
                print!(", ");
            }
            print!("{}: {:.3}", i, time);
        }
        println!("]");
        println!(
            "  Average time: {:.3}",
            results.times.iter().sum::<f64>() / results.times.len() as f64
        );
    }

    Ok(())
}

#[derive(Default, Debug, Serialize)]
struct Results {
    team0_wins: Vec<u32>,
    team1_wins: Vec<u32>,
    draws: Vec<u32>,
    times: Vec<f64>,
}

fn run_simulations(scenario_name: &str, codes: Vec<Code>, rounds: u32) -> Results {
    let seed_statuses: Vec<(u32, (scenario::Status, f64))> = (0..rounds)
        .into_par_iter()
        .map(|seed| (seed, run_simulation(scenario_name, seed, codes.clone())))
        .collect();
    let mut results: Results = Default::default();
    for (seed, (status, time)) in seed_statuses {
        match status {
            scenario::Status::Victory { team: 0 } => results.team0_wins.push(seed),
            scenario::Status::Victory { team: 1 } => results.team1_wins.push(seed),
            scenario::Status::Draw => results.draws.push(seed),
            scenario::Status::Failed => results.team1_wins.push(seed),
            _ => unreachable!(),
        }
        results.times.push(time);
    }
    results
}

fn run_simulation(scenario_name: &str, seed: u32, codes: Vec<Code>) -> (scenario::Status, f64) {
    let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);
    while sim.status() == scenario::Status::Running && sim.tick() < scenario::MAX_TICKS {
        sim.step();
    }
    (sim.status(), sim.score_time())
}
