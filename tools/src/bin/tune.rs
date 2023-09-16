use clap::Parser as _;
use metaheuristics_nature::utility::prelude::*;
use metaheuristics_nature::{Bounded, ObjFunc, Solver};
use oort_simulator::simulation::Code;
use oort_simulator::{scenario, simulation};
use std::cell::RefCell;
use std::default::Default;
use std::path::PathBuf;

thread_local! {
  static COMPILERS: std::cell::RefCell<oort_compiler::Compiler> = RefCell::new(oort_compiler::Compiler::new());
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("tune=info")).init();

    #[derive(clap::Parser, Debug)]
    struct Arguments {
        #[clap(short, long, value_parser, default_value = "20")]
        population: usize,

        #[clap(short, long, value_parser, default_value = "5")]
        generations: u64,

        #[clap(short, long, value_parser, default_value = "10")]
        num_seeds: u32,

        #[clap(short, long)]
        dry_run: bool,

        #[clap(long)]
        wasm_cache: Option<PathBuf>,

        scenario_name: String,
        player_code: String,
        enemy_code: String,
    }

    let args = Arguments::parse();

    let player_src_code = std::fs::read_to_string(&args.player_code).unwrap();
    let (initial_values, bounds) = extract_tunables(&player_src_code);
    assert!(!initial_values.is_empty());

    let ais = oort_tools::fetch_and_compile_multiple(
        &reqwest::Client::new(),
        &[args.player_code.clone(), args.enemy_code.clone()],
        false,
        args.wasm_cache.as_deref(),
    )
    .await?;
    let codes = ais
        .into_iter()
        .map(|ai| ai.compiled_code)
        .collect::<Vec<_>>();

    log::info!("Running initial simulations");
    let initial_fitness = run_simulations(&args.scenario_name, codes.clone(), args.num_seeds);
    log::info!(
        "Initial fitness: {} for {:?}",
        initial_fitness,
        initial_values
    );

    {
        let test_src_code = rewrite_tunables(&player_src_code, &initial_values);
        let (test_initial_values, test_bounds) = extract_tunables(&test_src_code);
        assert_eq!(test_initial_values, initial_values);
        assert_eq!(test_bounds, bounds);
    }

    let objective_function = ObjectiveFunction {
        scenario_name: args.scenario_name.to_string(),
        player_src_code: player_src_code.clone(),
        bounds: bounds.to_vec(),
        enemy_code: codes[1].clone(),
        num_seeds: args.num_seeds,
    };

    let pool = generate_pool(&initial_values);

    let s = Solver::build(metaheuristics_nature::Rga::default(), objective_function)
        .pop_num(args.population)
        .pool(pool)
        .task(|ctx| ctx.gen + 1 == args.generations)
        .callback(|ctx| {
            log::info!(
                "Generation {}/{}. Best fitness {} for {:?}",
                ctx.gen + 1,
                args.generations,
                ctx.best_f,
                ctx.best.iter().cloned().collect::<Vec<f64>>()
            );
            if ctx.best_f < initial_fitness {
                log::info!("Writing back to {}", args.player_code);
                let new_src_code =
                    rewrite_tunables(&player_src_code, ctx.best.slice(s![..]).as_slice().unwrap());
                std::fs::write(&args.player_code, new_src_code).unwrap();
            }
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
    num_seeds: u32,
}

impl Bounded for ObjectiveFunction {
    fn bound(&self) -> &[[f64; 2]] {
        self.bounds.as_slice()
    }
}

impl ObjFunc for ObjectiveFunction {
    type Fitness = f64;

    fn fitness(&self, x: &[f64]) -> Self::Fitness {
        log::debug!("Evaluating candidate {:?}", x);
        let start_time = std::time::Instant::now();
        let player_src_code = rewrite_tunables(&self.player_src_code, x);

        let compile_start_time = std::time::Instant::now();
        let player_code = if let Some(wasm) = compile("player code".to_string(), player_src_code) {
            oort_simulator::vm::precompile(&wasm).unwrap()
        } else {
            panic!("Failed to compile player source code");
        };
        let compile_duration = std::time::Instant::now() - compile_start_time;

        let sim_start_time = std::time::Instant::now();
        let fitness = run_simulations(
            &self.scenario_name,
            vec![player_code, self.enemy_code.clone()],
            self.num_seeds,
        );
        let sim_duration = std::time::Instant::now() - sim_start_time;

        log::debug!(
            "Got fitness {} in {:?} (compile {:?}, sim {:?})",
            fitness,
            std::time::Instant::now() - start_time,
            compile_duration,
            sim_duration
        );
        fitness
    }
}

fn generate_pool<F: ObjFunc>(initial_values: &[f64]) -> impl Fn(&Ctx<F>, &Rng) -> Array2<f64> {
    let initial_values = initial_values.to_owned();
    move |ctx, rng| {
        let mut pool = Array2::from_shape_fn(ctx.pool_size(), |(_, s)| initial_values[s]);
        for i in 0..(ctx.pool_size()[0] - 1) {
            let s = i % initial_values.len();
            for j in 0..initial_values.len() {
                pool[[i + 1, j]] =
                    ctx.clamp(j, rng.normal(initial_values[j], ctx.bound_width(j) / 32.0));
            }
            pool[[i + 1, s]] =
                ctx.clamp(s, rng.normal(initial_values[s], ctx.bound_width(s) / 4.0));
        }
        pool
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
    let value_re = regex::Regex::new("([0-9.-]+)$").unwrap();
    let mut i = 0;
    re.replace_all(src_code, move |caps: &regex::Captures| {
        let r = value_re.replace(caps.get(0).unwrap().as_str(), format!("{:?}", values[i]));
        i += 1;
        r.to_string()
    })
    .to_string()
}

fn compile(name: String, code: String) -> Option<Vec<u8>> {
    COMPILERS.with(|compiler_cell| {
        let mut compiler = compiler_cell.borrow_mut();
        match compiler.compile(&code) {
            Ok(wasm) => Some(wasm),
            Err(e) => {
                log::warn!("Failed to compile {:?}: {}", name, e);
                None
            }
        }
    })
}

fn run_simulations(scenario_name: &str, codes: Vec<Code>, num_seeds: u32) -> f64 {
    let reverse_codes = codes.iter().rev().cloned().collect::<Vec<_>>();
    (0..num_seeds)
        .into_par_iter()
        .flat_map(|seed| [(seed, false), (seed, true)])
        .map(|(seed, reverse)| {
            let codes = if reverse { &reverse_codes } else { &codes };
            let player_team = if reverse { 1 } else { 0 };
            let (status, time) = run_simulation(scenario_name, seed, codes.clone());
            match status {
                scenario::Status::Victory { team } if team == player_team => time,
                _ => 1e6,
            }
        })
        .sum()
}

fn run_simulation(scenario_name: &str, seed: u32, codes: Vec<Code>) -> (scenario::Status, f64) {
    let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);
    while sim.status() == scenario::Status::Running && sim.tick() < scenario::MAX_TICKS {
        sim.step();
    }
    (sim.status(), sim.time())
}
