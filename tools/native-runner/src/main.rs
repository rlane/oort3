use clap::Parser;
use oort_simulator::scenario;
use oort_simulator::simulation::{Code, NativeShip, Simulation};
use std::sync::Arc;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "oort-native-runner", about = "Run Oort AI natively (no WASM) for debugging")]
struct Cli {
    /// Scenario name (e.g. tutorial_guns, tutorial_acceleration)
    scenario: Option<String>,

    /// Random seed (default: 0)
    #[arg(short, long, default_value_t = 0)]
    seed: u32,

    /// Number of seeds to test (runs seed..seed+count)
    #[arg(short, long, default_value_t = 1)]
    count: u32,

    /// Maximum ticks before stopping
    #[arg(short, long, default_value_t = 10000)]
    max_ticks: u32,

    /// Print debug text from ships each tick
    #[arg(long)]
    debug: bool,

    /// List all available scenarios
    #[arg(long)]
    list: bool,
}

struct NativeShipWrapper {
    ship: oort_ai::Ship,
}

impl NativeShip for NativeShipWrapper {
    fn tick(&mut self) {
        self.ship.tick();
    }
}

fn list_scenarios() {
    for (category, names) in scenario::list() {
        println!("{}:", category);
        for name in names {
            println!("  {}", name);
        }
    }
}

fn run_simulation(
    scenario_name: &str,
    code: &Code,
    seed: u32,
    max_ticks: u32,
    debug: bool,
) -> SimResult {
    let scn = scenario::load(scenario_name);
    let mut codes = scn.initial_code();
    codes[0] = code.clone();
    let mut sim = Simulation::new(scenario_name, seed, &codes);

    let start = Instant::now();
    while sim.status() == scenario::Status::Running && sim.tick() < max_ticks {
        sim.step();
        if debug {
            let snapshot = sim.snapshot(0);
            for (_ship_id, text) in &snapshot.debug_text {
                if !text.is_empty() {
                    println!("{}", text.trim());
                }
            }
            for err in &snapshot.errors {
                eprintln!("[tick {} ERROR] {}", sim.tick(), err.msg);
            }
        }
    }
    let elapsed = start.elapsed();

    let snapshot = sim.snapshot(0);
    let final_errors: Vec<String> = snapshot.errors.iter().map(|e| e.msg.clone()).collect();

    SimResult {
        seed,
        status: sim.status(),
        score_time: sim.score_time(),
        ticks: sim.tick(),
        wall_time: elapsed,
        errors: final_errors,
    }
}

struct SimResult {
    seed: u32,
    status: scenario::Status,
    score_time: f64,
    ticks: u32,
    wall_time: std::time::Duration,
    errors: Vec<String>,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    if cli.list {
        list_scenarios();
        return;
    }

    let scenario_name = match cli.scenario.as_deref() {
        Some(name) => name,
        None => {
            eprintln!("Scenario name required. Use --list to see available scenarios.");
            std::process::exit(1);
        }
    };

    if scenario::load_safe(scenario_name).is_none() {
        eprintln!("Unknown scenario: '{}'. Use --list to see available scenarios.", scenario_name);
        std::process::exit(1);
    }

    let factory = Arc::new(|| -> Box<dyn NativeShip> {
        Box::new(NativeShipWrapper {
            ship: oort_ai::Ship::new(),
        })
    });
    let code = Code::Native(factory);

    println!("Scenario: {} (native mode)", scenario_name);
    println!("Seeds: {}..{}", cli.seed, cli.seed + cli.count);
    println!("---");

    let mut wins = 0u32;
    let mut fails = 0u32;
    let mut draws = 0u32;

    for seed in cli.seed..(cli.seed + cli.count) {
        let result = run_simulation(scenario_name, &code, seed, cli.max_ticks, cli.debug);

        let status_str = match result.status {
            scenario::Status::Victory { team: 0 } => {
                wins += 1;
                "VICTORY".to_string()
            }
            scenario::Status::Victory { team } => {
                fails += 1;
                format!("ENEMY WIN (team {})", team)
            }
            scenario::Status::Draw => {
                draws += 1;
                "DRAW".to_string()
            }
            scenario::Status::Failed => {
                fails += 1;
                "FAILED".to_string()
            }
            scenario::Status::Running => {
                draws += 1;
                "TIMEOUT".to_string()
            }
        };

        println!(
            "Seed {:>3}: {:<12} time={:.2}s  ticks={}  wall={:.1?}",
            result.seed, status_str, result.score_time, result.ticks, result.wall_time,
        );

        for err in &result.errors {
            eprintln!("  ERROR: {}", err);
        }
    }

    if cli.count > 1 {
        println!("---");
        println!(
            "Summary: {} wins, {} fails, {} draws/timeouts out of {} runs",
            wins, fails, draws, cli.count,
        );
    }

    if fails > 0 || draws > 0 {
        std::process::exit(1);
    }
}
