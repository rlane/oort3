use clap::Parser;
use oort_simulator::snapshot::Snapshot;
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
            diff_snapshots(&sims[0].snapshot(0), &sims[1].snapshot(0));
            break;
        }
        sims.iter_mut().for_each(|sim| sim.step());
    }

    Ok(())
}

fn diff_snapshots(a: &Snapshot, b: &Snapshot) {
    if a.ships.len() != b.ships.len() {
        println!("ship count differs");
        println!("  a: {}", a.ships.len());
        println!("  b: {}", b.ships.len());
    }

    for (i, (a, b)) in a.ships.iter().zip(b.ships.iter()).enumerate() {
        let epsilon = 0.0;

        if (a.position - b.position).magnitude() > epsilon {
            println!("ship {} position differs", i);
            println!("  a: {:?}", a.position);
            println!("  b: {:?}", b.position);
        }

        if (a.velocity - b.velocity).magnitude() > epsilon {
            println!("ship {} velocity differs", i);
            println!("  a: {:?}", a.velocity);
            println!("  b: {:?}", b.velocity);
        }

        if (a.acceleration - b.acceleration).magnitude() > epsilon {
            println!("ship {} acceleration differs", i);
            println!("  a: {:?}", a.acceleration);
            println!("  b: {:?}", b.acceleration);
        }

        if (a.heading - b.heading).abs() > epsilon {
            println!("ship {} heading differs", i);
            println!("  a: {:?}", a.heading);
            println!("  b: {:?}", b.heading);
        }

        if (a.angular_velocity - b.angular_velocity).abs() > epsilon {
            println!("ship {} angular_velocity differs", i);
            println!("  a: {:?}", a.angular_velocity);
            println!("  b: {:?}", b.angular_velocity);
        }

        if (a.health - b.health).abs() > epsilon {
            println!("ship {} health differs", i);
            println!("  a: {:?}", a.health);
            println!("  b: {:?}", b.health);
        }

        if a.fuel != b.fuel {
            println!("ship {} fuel differs", i);
            println!("  a: {:?}", a.fuel);
            println!("  b: {:?}", b.fuel);
        }
    }
}
