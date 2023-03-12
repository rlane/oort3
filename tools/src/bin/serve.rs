use anyhow::Result;
use clap::Parser as _;
use std::process::{Child, Command};

#[derive(clap::Parser, Debug)]
struct Arguments {
    #[clap(long)]
    /// Build the frontend in release mode.
    release: bool,

    #[clap(long, default_value = "oort-dev")]
    project: String,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("serve=info"))
        .init();

    let args = Arguments::parse();

    if let Ok(contents) = std::fs::read_to_string(".secrets/secrets.toml") {
        let dev_mode_secrets = ["GOOGLE_APPLICATION_CREDENTIALS"];
        let secrets = contents.parse::<toml::Table>()?;
        for (k, v) in secrets.iter() {
            if dev_mode_secrets.contains(&k.as_str()) {
                std::env::set_var(k, v.as_str().expect("invalid secret value"));
            }
        }
    } else {
        log::info!("Missing secrets file");
    }

    let services = [("compiler", 8081), ("backend", 8082)];

    std::env::set_var("COMPILER_URL", "http://localhost:8081");
    std::env::set_var("BACKEND_URL", "http://localhost:8082");

    cmd(&["cargo", "build", "--workspace", "--bins"])
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
        "-p",
        "oort_compiler_service",
        "--",
        "--prepare",
    ])
    .spawn()?
    .wait()?;

    let mut children = vec![];
    for (name, port) in services.iter() {
        let child = cmd(&["cargo", "run", "-q", "-p", &format!("oort_{name}_service")])
            .env("RUST_LOG", &format!("none,oort_{name}_service=debug"))
            .env("PROJECT_ID", &args.project)
            .env("PORT", &port.to_string())
            .spawn()?;
        children.push(ChildGuard(child));
    }

    std::fs::create_dir_all("frontend/app/dist")?;

    cmd(&[
        "trunk",
        "-v",
        "serve",
        "--dist",
        "frontend/app/dist-debug",
        "frontend/app/index.html",
        "--watch=frontend",
        "--watch=shared/ai/builtin-ai.tar.gz",
        "--watch=shared/api",
        "--watch=shared/simulator",
        "--ignore=frontend/app/dist",
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
