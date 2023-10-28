use clap::Parser;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use oort_simulator::simulation::Code;
use oort_simulator::{scenario, simulation};
use rayon::prelude::*;
use std::default::Default;

#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    scenario: Option<String>,

    #[clap(short, long, default_value = "100")]
    rounds: u32,

    /// Base seed
    #[clap(short, long, default_value = "0")]
    seed: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("check_solution_reliability=info"),
    )
    .init();

    let args = Arguments::parse();

    let scenarios = if let Some(scenario_name) = args.scenario {
        vec![scenario_name.clone()]
    } else {
        let categories = scenario::list();
        categories
            .iter()
            .find(|(category, _)| category == "Tutorial")
            .unwrap()
            .1
            .clone()
    };

    let progress = indicatif::ProgressBar::new(args.rounds as u64 * scenarios.len() as u64);

    let failures: Vec<(String, Vec<u32>)> = scenarios
        .par_iter()
        .map(|scenario_name| {
            let scenario =
                scenario::load_safe(scenario_name).expect("Unknown scenario {scenario_name}");
            let codes = scenario.solution_codes();
            let failed_seeds: Vec<u32> = (args.seed..(args.seed + args.rounds))
                .into_par_iter()
                .filter_map(|seed| {
                    let status = run_simulation(scenario_name, seed, codes.clone());
                    progress.inc(1);
                    match status {
                        scenario::Status::Victory { team: 0 } => None,
                        _ => Some(seed),
                    }
                })
                .collect();
            (scenario_name.clone(), failed_seeds)
        })
        .collect();
    progress.finish_and_clear();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Scenario", "Failure Count", "Sample Seeds"]);
    for (scenario_name, failed_seeds) in failures {
        if !failed_seeds.is_empty() {
            table.add_row(vec![
                scenario_name.clone(),
                failed_seeds.len().to_string(),
                failed_seeds
                    .iter()
                    .take(10)
                    .map(|seed| seed.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ]);
        }
    }
    println!("{table}");

    Ok(())
}

fn run_simulation(scenario_name: &str, seed: u32, codes: Vec<Code>) -> scenario::Status {
    let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);
    while sim.status() == scenario::Status::Running && sim.tick() < scenario::MAX_TICKS {
        sim.step();
    }
    sim.status()
}
