use clap::Parser as _;
use metaheuristics_nature::{Bounded, ObjFunc, Solver};
use oort_simulator::simulation::Code;
use oort_simulator::{scenario, simulation};
use rayon::prelude::*;
use std::default::Default;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("tune=info")).init();

    #[derive(clap::Parser, Debug)]
    struct Arguments {
        #[clap(short, long, value_parser, default_value = "20")]
        population: usize,

        #[clap(short, long, value_parser, default_value = "5")]
        generations: u64,

        scenario_name: String,
        player_code: String,
        enemy_code: String,
    }

    let args = Arguments::parse();

    let http_client = reqwest::blocking::Client::new();
    let mut codes = vec![];
    for src in &[&args.player_code, &args.enemy_code] {
        log::info!("Compiling {:?}", src);
        let src_code = std::fs::read_to_string(src).unwrap();
        if let Some(wasm) = compile(&http_client, src.to_string(), src_code) {
            codes.push(Code::Wasm(wasm));
        } else {
            panic!("Failed to compile {:?}", src);
        }
    }

    log::info!("Running initial simulations");
    let fitness = run_simulations(&args.scenario_name, codes.clone());
    log::info!("Initial fitness: {}", fitness);

    let player_src_code = std::fs::read_to_string(&args.player_code).unwrap();
    let (initial_values, bounds) = extract_tunables(&player_src_code);
    assert!(!initial_values.is_empty());

    let objective_function = ObjectiveFunction {
        scenario_name: args.scenario_name.to_string(),
        player_src_code: player_src_code.clone(),
        bounds: bounds.to_vec(),
        enemy_code: codes[1].clone(),
    };

    let std: Vec<f64> = bounds.iter().map(|b| (b[1] - b[0]) / 2.0).collect();

    let pool = metaheuristics_nature::utility::gaussian_pool_inclusive(&initial_values, &std);

    let s = Solver::build(metaheuristics_nature::Rga::default(), objective_function)
        .pop_num(args.population)
        .pool(pool)
        .task(|ctx| ctx.gen == args.generations)
        .callback(|ctx| {
            log::info!(
                "Generation {}. Best fitness {} for {:?}",
                ctx.gen,
                ctx.best_f,
                ctx.best.iter().cloned().collect::<Vec<f64>>()
            )
        })
        .solve()
        .unwrap();

    log::info!(
        "Result: fitness={:?} parameters={:?}",
        s.best_fitness(),
        s.best_parameters()
    );

    Ok(())
}

struct ObjectiveFunction {
    scenario_name: String,
    player_src_code: String,
    bounds: Vec<[f64; 2]>,
    enemy_code: Code,
}

impl Bounded for ObjectiveFunction {
    fn bound(&self) -> &[[f64; 2]] {
        self.bounds.as_slice()
    }
}

impl ObjFunc for ObjectiveFunction {
    type Fitness = f64;

    fn fitness(&self, x: &[f64]) -> Self::Fitness {
        log::info!("Evaluating candidate {:?}", x);
        let start_time = std::time::Instant::now();
        let player_src_code = rewrite_tunables(&self.player_src_code, x);

        let compile_start_time = std::time::Instant::now();
        let http_client = reqwest::blocking::Client::new();
        let player_code = if let Some(wasm) = compile(
            &http_client,
            "player code".to_string(),
            player_src_code.clone(),
        ) {
            Code::Wasm(wasm)
        } else {
            panic!("Failed to compile player source code");
        };
        let compile_duration = std::time::Instant::now() - compile_start_time;

        let sim_start_time = std::time::Instant::now();
        let fitness = run_simulations(
            &self.scenario_name,
            vec![player_code, self.enemy_code.clone()],
        );
        let sim_duration = std::time::Instant::now() - sim_start_time;

        log::info!(
            "Got fitness {} in {:?} (compile {:?}, sim {:?})",
            fitness,
            std::time::Instant::now() - start_time,
            compile_duration,
            sim_duration
        );
        fitness
    }
}

const TUNABLE_RE: &str = r"/\* ?tune from ([0-9.-]+) to ([0-9.-]+) ?\*/ ?([0-9.-]+)";

fn extract_tunables(src_code: &str) -> (Vec<f64>, Vec<[f64; 2]>) {
    let mut initial_values = vec![];
    let mut bounds = vec![];
    let re = regex::Regex::new(TUNABLE_RE).unwrap();
    for cap in re.captures_iter(src_code) {
        initial_values.push(cap[3].parse().unwrap());
        bounds.push([cap[1].parse().unwrap(), cap[2].parse().unwrap()]);
    }
    (initial_values, bounds)
}

fn rewrite_tunables(src_code: &str, values: &[f64]) -> String {
    let re = regex::Regex::new(TUNABLE_RE).unwrap();
    let mut i = 0;
    re.replace_all(src_code, move |_: &regex::Captures| {
        let r = format!("{:?}", values[i]);
        i += 1;
        r
    })
    .to_string()
}

fn compile(client: &reqwest::blocking::Client, name: String, code: String) -> Option<Vec<u8>> {
    let url = "http://localhost:8081/compile";
    let result = client.post(url).body(code).send();
    let response = result.unwrap().error_for_status();
    match response {
        Ok(response) => Some(response.bytes().unwrap().as_ref().into()),
        Err(e) => {
            log::warn!("Failed to compile {:?}: {}", name, e);
            None
        }
    }
}

fn run_simulations(scenario_name: &str, codes: Vec<Code>) -> f64 {
    let mut total_time = 0.0;
    let mut losses = 0;
    let results: Vec<(scenario::Status, f64)> = (0..10u32)
        .into_par_iter()
        .map(|seed| run_simulation(scenario_name, seed, codes.clone()))
        .collect();
    for (status, time) in results {
        total_time += time;
        match status {
            scenario::Status::Victory { team: 0 } => {}
            _ => {
                losses += 1;
            }
        }
    }
    losses as f64 * 1000.0 + total_time
}

fn run_simulation(scenario_name: &str, seed: u32, codes: Vec<Code>) -> (scenario::Status, f64) {
    let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);
    while sim.status() == scenario::Status::Running && sim.tick() < scenario::MAX_TICKS {
        sim.step();
    }
    (sim.status(), sim.time())
}
