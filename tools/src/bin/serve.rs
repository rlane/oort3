use anyhow::Result;
use clap::Parser as _;
use std::process::{Child, Command};

#[derive(clap::Parser, Debug)]
struct Arguments {
    #[clap(long)]
    /// Build the frontend in release mode.
    release: bool,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("serve=info"))
        .init();

    let args = Arguments::parse();

    if let Ok(contents) = std::fs::read_to_string(".secrets/secrets.toml") {
        let secrets = contents.parse::<toml::Table>()?;
        for (k, v) in secrets.iter() {
            std::env::set_var(k, v.as_str().expect("invalid secret value"));
        }
    } else {
        log::info!("Missing secrets file");
    }

    cmd(&[
        "cargo",
        "build",
        "-q",
        "--manifest-path",
        "services/Cargo.toml",
    ])
    .spawn()?
    .wait()?;

    let compiler_tmp_dir = "/tmp/oort-ai";
    if std::fs::metadata(compiler_tmp_dir).is_ok() {
        std::fs::remove_dir_all(compiler_tmp_dir)?;
    }

    cmd(&[
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        "services/Cargo.toml",
        "-p",
        "oort_compiler_service",
        "--",
        "--prepare",
    ])
    .spawn()?
    .wait()?;

    let services = [
        ("compiler", 8081),
        ("telemetry", 8082),
        ("leaderboard", 8083),
        ("shortcode", 8084),
    ];

    let mut children = vec![];
    for (name, port) in services.iter() {
        let child = cmd(&[
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            "services/Cargo.toml",
            "-p",
            &format!("oort_{name}_service"),
        ])
        .env("RUST_LOG", &format!("none,oort_{name}_service=debug"))
        .env("ENVIRONMENT", "dev")
        .env("PORT", &port.to_string())
        .spawn()?;
        children.push(ChildGuard(child));
    }

    cmd(&[
        "trunk",
        "serve",
        "--dist",
        "frontend/app/dist-debug",
        "frontend/app/index.html",
        "--watch=frontend",
        "--watch=shared/ai/builtin-ai.tar.gz",
        "--watch=shared/api",
        "--watch=shared/simulator",
        if args.release { "--release" } else { "" },
    ])
    .spawn()?
    .wait()?;

    Ok(())
}

fn cmd(argv: &[&str]) -> Command {
    log::info!("Executing {:?}", shell_words::join(argv));
    let mut cmd = Command::new(argv[0]);
    let args: Vec<_> = argv[1..].iter().filter(|x| !x.is_empty()).collect();
    cmd.args(&args);
    cmd
}

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Err(e) = self.0.kill() {
            println!("Could not kill child process: {e}");
        }
    }
}
