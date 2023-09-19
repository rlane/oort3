use clap::Parser;
use oort_simulator::{scenario, simulation};
use std::default::Default;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    scenario: String,
    shortcodes: Vec<String>,

    #[clap(short, long)]
    seed: u32,

    #[clap(short, long)]
    dev: bool,

    #[clap(long, default_value = "/tmp/oort-wasm-cache")]
    wasm_cache: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Arguments::parse();
    scenario::load_safe(&args.scenario).expect("Unknown scenario");
    if args.shortcodes.len() != 2 {
        panic!("Expected two shortcodes");
    }

    log::info!("Compiling AIs");
    let http = reqwest::Client::new();
    let ais = oort_tools::fetch_and_compile_multiple(
        &http,
        &args.shortcodes,
        args.dev,
        args.wasm_cache.as_deref(),
    )
    .await?;
    let codes = vec![ais[0].compiled_code.clone(), ais[1].compiled_code.clone()];

    let mut sims = [0, 1]
        .iter()
        .map(|_| simulation::Simulation::new(&args.scenario, args.seed, &codes))
        .collect::<Vec<_>>();
    while sims[0].status() == scenario::Status::Running && sims[0].tick() < scenario::MAX_TICKS {
        let hashes = sims.iter().map(|sim| sim.hash()).collect::<Vec<_>>();
        if hashes[0] != hashes[1] {
            println!("hashes differ at tick {}", sims[0].tick());
            break;
        }
        sims.iter_mut().for_each(|sim| sim.step());
    }

    Ok(())
}
