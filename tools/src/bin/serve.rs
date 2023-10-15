use anyhow::Result;
use clap::Parser as _;
use std::process::{Child, Command};

#[derive(clap::Parser, Debug)]
struct Arguments {
    #[clap(long)]
    /// Build the frontend in release mode.
    release: bool,

    #[clap(long)]
    /// Listen on all IP addresses.
    listen: bool,

    #[clap(long, default_value = "oort-dev")]
    project: String,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("serve=info"))
        .init();

    let args = Arguments::parse();

    for cmd in &["trunk", "wasm-opt"] {
        if !Command::new("which").arg(cmd).output()?.status.success() {
            return Err(anyhow::anyhow!("Missing dependency {}", cmd));
        }
    }

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
    let mut start_service = |name: &str, port: u16, extra_args: &[&str]| -> Result<()> {
        let s = &format!("oort_{name}_service");
        let mut c = vec!["cargo", "run", "-q", "-p", s];
        c.extend(extra_args);
        let child = cmd(&c)
            .env(
                "RUST_LOG",
                &format!("none,oort_{name}_service=debug,tower_http=trace"),
            )
            .env("PROJECT_ID", &args.project)
            .env("PORT", &port.to_string())
            .spawn()?;
        children.push(ChildGuard(child));
        Ok(())
    };

    start_service("compiler", 8081, &[])?;
    start_service("backend", 8082, &["serve"])?;

    std::fs::create_dir_all("frontend/app/dist")?;

    cmd(&[
        "trunk",
        "-v",
        "serve",
        "--dist",
        "frontend/app/dist-debug",
        "frontend/app/index.html",
        "--watch=frontend",
        "--watch=shared/builtin_ai/builtin-ai.tar.gz",
        "--watch=shared/api",
        "--watch=shared/simulator",
        "--ignore=frontend/app/dist",
        if args.release { "--release" } else { "" },
        if args.listen { "--address=0.0.0.0" } else { "" },
    ])
    .spawn()?
    .wait()?;

    Ok(())
}

fn cmd(argv: &[&str]) -> Command {
    log::info!("Executing {:?}", shell_words::join(argv));
    let mut cmd = Command::new("nice");
    let args: Vec<_> = argv.iter().filter(|x| !x.is_empty()).collect();
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
