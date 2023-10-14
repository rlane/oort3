use anyhow::{anyhow, bail, Result};
use clap::Parser as _;
use indicatif::{MultiProgress, ProgressBar};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::process::{ExitStatus, Output};
use tokio::process::Command;
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::Retry;

const REGION: &str = "us-west1";
const WORKSPACES: &[&str] = &[".", "frontend"];
static PROGRESS: Lazy<MultiProgress> = Lazy::new(MultiProgress::new);

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
enum Component {
    App,
    Backend,
    Compiler,
    Doc,
    Tools,
}

const ALL_COMPONENTS: &[Component] = &[
    Component::App,
    Component::Backend,
    Component::Compiler,
    Component::Doc,
    Component::Tools,
];

#[derive(clap::Parser, Debug)]
struct Arguments {
    #[clap(
        short,
        long,
        value_enum,
        value_delimiter = ',',
        default_value = "app,backend,compiler,doc,tools"
    )]
    /// Components to push.
    components: Vec<Component>,

    #[clap(short)]
    /// Skip bumping version.
    skip_version_bump: bool,

    #[clap(short = 'n')]
    /// Skip pushing.
    dry_run: bool,

    #[clap(long)]
    /// Allow pushing with uncommitted changes or on a non-master branch.
    skip_git_checks: bool,

    #[clap(long)]
    skip_github: bool,

    #[clap(long)]
    skip_discord: bool,

    #[clap(long)]
    skip_components_check: bool,

    #[clap(long, default_value = "oort-319301")]
    project: String,

    #[clap(long)]
    no_secrets: bool,
}

#[derive(Deserialize, Clone, Default)]
struct Secrets {
    oort_envelope_secret: Option<String>,
    oort_code_encryption_secret: Option<String>,
    discord_changelog_webhook: Option<String>,
    discord_telemetry_webhook: Option<String>,
    discord_leaderboard_webhook: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("release=info"))
        .init();

    let start_time = std::time::Instant::now();

    let args = Arguments::parse();
    let dry_run = args.dry_run;

    let mut secrets: Secrets = Secrets::default();
    if !args.no_secrets && std::fs::metadata(".secrets/secrets.toml").is_ok() {
        secrets = toml::from_str(&std::fs::read_to_string(".secrets/secrets.toml")?)?;
        std::env::set_var(
            "OORT_ENVELOPE_SECRET",
            &secrets.oort_envelope_secret.clone().unwrap_or_default(),
        );
    }

    std::env::set_var("DOCKER_BUILDKIT", "1");

    if !args.skip_git_checks {
        sync_cmd_ok(&["git", "diff", "HEAD", "--quiet"])
            .await
            .map_err(|_| anyhow!("Uncommitted changes, halting release"))?;
    }

    if !dry_run && !args.skip_github && !args.skip_git_checks {
        sync_cmd_ok(&["git", "fetch"]).await?;
    }

    let mut version = "unknown".to_string();
    let mut changelog = "unknown".to_string();
    let bump_version = !args.skip_version_bump;
    if bump_version {
        if args.components != ALL_COMPONENTS && !args.skip_components_check {
            bail!("Attempted to bump version without pushing all components");
        }

        if sync_cmd_ok(&["git", "rev-parse", "--abbrev-ref", "HEAD"])
            .await?
            .stdout_string()
            .trim()
            != "master"
            && !args.skip_git_checks
        {
            bail!("Not on master branch, halting release");
        }

        changelog = sync_cmd_ok(&["sed", "/^#/Q", "CHANGELOG.md"])
            .await?
            .stdout_string();
        if changelog.is_empty() {
            bail!("Changelog empty, halting release");
        }

        println!("Changelog:\n{}", changelog.trim());

        cmd_argv(&[
            "cargo",
            "workspaces",
            "version",
            "--all",
            "--force=*",
            "--no-git-commit",
            "--yes",
        ])
        .current_dir("frontend")
        .status()
        .await?
        .check_success()?;

        version = {
            let manifest = std::fs::read_to_string("frontend/app/Cargo.toml")?;
            let manifest = manifest.parse::<toml::Table>()?;
            manifest["package"]["version"]
                .as_str()
                .ok_or_else(|| anyhow!("Failed to find version"))?
                .to_string()
        };
        log::info!("Version {}", version);

        for workspace in WORKSPACES {
            sync_cmd_ok(&[
                "cargo",
                "workspaces",
                "--manifest-path",
                &format!("{workspace}/Cargo.toml"),
                "version",
                "--all",
                "--force=*",
                "--no-git-commit",
                "--yes",
                "custom",
                &version,
            ])
            .await?;
        }

        for workspace in WORKSPACES {
            sync_cmd_ok(&[
                "cargo",
                "update",
                "--manifest-path",
                &format!("{workspace}/Cargo.toml"),
                "--workspace",
            ])
            .await?;
        }

        for workspace in WORKSPACES {
            sync_cmd_ok(&[
                "cargo",
                "verify-project",
                "--manifest-path",
                &format!("{workspace}/Cargo.toml"),
                "--frozen",
                "--locked",
            ])
            .await?;
        }

        let previous_changelog_contents =
            std::str::from_utf8(&std::fs::read("CHANGELOG.md")?)?.to_owned();
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        std::fs::write(
            "CHANGELOG.md",
            &format!("### {version} - {date}\n\n{previous_changelog_contents}"),
        )?;

        sync_cmd_ok(&[
            "git",
            "commit",
            "-n",
            "-a",
            "-m",
            &format!("bump version to {version}"),
        ])
        .await?;

        sync_cmd_ok(&["git", "tag", &format!("v{version}")]).await?;
    }

    let mut tasks = tokio::task::JoinSet::new();

    if args.components.contains(&Component::App) {
        let project: String = args.project.to_string();

        let backend_url = sync_cmd_ok(&[
            "gcloud",
            "--project",
            &project,
            "run",
            "services",
            "describe",
            "oort-backend-service",
            "--format=value(status.url)",
        ])
        .await?
        .stdout_string();
        std::env::set_var("BACKEND_URL", &backend_url);

        let compiler_url = sync_cmd_ok(&[
            "gcloud",
            "--project",
            &project,
            "run",
            "services",
            "describe",
            "oort-compiler-service",
            "--format=value(status.url)",
        ])
        .await?
        .stdout_string();
        std::env::set_var("COMPILER_URL", &compiler_url);

        tasks.spawn(Retry::spawn(retry_strategy(), move || {
            let project = project.clone();
            async move {
                let progress = create_progress_bar("frontend");

                progress.set_message("prebuild (app)");
                sync_cmd_ok(&[
                    "cargo",
                    "build",
                    "--target=wasm32-unknown-unknown",
                    "--manifest-path",
                    "frontend/app/Cargo.toml",
                    "--release",
                    "--bin",
                    "app",
                    "--bin",
                    "oort_simulation_worker",
                ])
                .await?;

                progress.set_message("prebuild (analyzer)");
                sync_cmd_ok(&[
                    "cargo",
                    "build",
                    "--target=wasm32-unknown-unknown",
                    "--manifest-path",
                    "frontend/analyzer_worker/Cargo.toml",
                    "--release",
                    "--bin",
                    "oort_analyzer_worker",
                ])
                .await?;

                progress.set_message("running trunk");
                if std::fs::metadata("frontend/app/dist").is_ok() {
                    std::fs::remove_dir_all("frontend/app/dist")?;
                }
                sync_cmd_ok(&[
                    "trunk",
                    "build",
                    "--release",
                    "--dist",
                    "frontend/app/dist",
                    "frontend/app/index.html",
                ])
                .await?;

                let du_output = sync_cmd_ok(&["du", "-sh", "frontend/app/dist"])
                    .await?
                    .stdout_string();
                PROGRESS.suspend(|| {
                    log::info!(
                        "Output size: {}",
                        du_output.split_whitespace().next().unwrap_or_default()
                    )
                });

                if !dry_run {
                    progress.set_message("deploying");
                    sync_cmd_ok(&[
                        "sh",
                        "-c",
                        &format!(r#"cd firebase && eval "$(fnm env)" && fnm use && npx firebase --project {project} deploy"#),
                    ])
                    .await?;
                }

                progress.finish_with_message("done");
                anyhow::Ok(())
        }}));
    }

    if args.components.contains(&Component::Compiler) {
        let secrets = secrets.clone();
        let project = args.project.clone();
        tasks.spawn(Retry::spawn(retry_strategy(), move || {
            let secrets = secrets.clone();
            let project = project.clone();
            async move {
                let progress = create_progress_bar("compiler");

                progress.set_message("building");
                sync_cmd_ok(&[
                    "docker",
                    "build",
                    "-f",
                    "services/compiler/Dockerfile",
                    "--tag",
                    "oort_compiler_service",
                    "--build-arg",
                    &format!(
                        "OORT_CODE_ENCRYPTION_SECRET={}",
                        secrets.oort_code_encryption_secret.unwrap_or_default()
                    ),
                    ".",
                ])
                .await?;

                if !dry_run {
                    let container_image = format!(
                        "{REGION}-docker.pkg.dev/{}/services/oort_compiler_service",
                        project
                    );

                    progress.set_message("tagging");
                    sync_cmd_ok(&[
                        "docker",
                        "tag",
                        "oort_compiler_service:latest",
                        &container_image,
                    ])
                    .await?;

                    progress.set_message("pushing image");
                    sync_cmd_ok(&["docker", "push", &container_image]).await?;

                    progress.set_message("deploying to Cloud Run");
                    sync_cmd_ok(&[
                        "gcloud",
                        "--project",
                        &project,
                        "run",
                        "deploy",
                        "oort-compiler-service",
                        "--image",
                        &container_image,
                        "--allow-unauthenticated",
                        "--region", REGION,
                        "--execution-environment=gen2",
                        "--cpu=2",
                        "--memory=2G",
                        "--timeout=20s",
                        "--concurrency=1",
                        "--max-instances=10",
                        &format!("--service-account=oort-compiler-service@{project}.iam.gserviceaccount.com"),
                    ])
                    .await?;
                }

                progress.finish_with_message("done");
                Ok(())
            }
        }));
    }

    if args.components.contains(&Component::Backend) {
        let secrets = secrets.clone();
        let project = args.project.clone();
        tasks.spawn(Retry::spawn(retry_strategy(), move || {
            let secrets = secrets.clone();
            let project = project.clone();
            async move {
                let progress = create_progress_bar("backend");

                progress.set_message("building");
                sync_cmd_ok(&[
                    "docker",
                    "build",
                    "-f",
                    "services/backend/Dockerfile",
                    "--tag",
                    "oort_backend_service",
                    "--build-arg",
                    &format!(
                        "PROJECT_ID={project}",
                    ),
                    "--build-arg",
                    &format!(
                        "OORT_CODE_ENCRYPTION_SECRET={}",
                        secrets.oort_code_encryption_secret.unwrap_or_default()
                    ),
                    "--build-arg",
                    &format!(
                        "OORT_ENVELOPE_SECRET={}",
                        secrets.oort_envelope_secret.unwrap_or_default()
                    ),
                    "--build-arg",
                    &format!(
                        "DISCORD_TELEMETRY_WEBHOOK={}",
                        secrets.discord_telemetry_webhook.unwrap_or_default()
                    ),
                    "--build-arg",
                    &format!(
                        "DISCORD_LEADERBOARD_WEBHOOK={}",
                        secrets.discord_leaderboard_webhook.unwrap_or_default()
                    ),
                    ".",
                ])
                .await?;

                if !dry_run {
                    let container_image = format!(
                        "{REGION}-docker.pkg.dev/{}/services/oort_backend_service",
                        project
                    );

                    progress.set_message("tagging");
                    sync_cmd_ok(&[
                        "docker",
                        "tag",
                        "oort_backend_service:latest",
                        &container_image,
                    ])
                    .await?;

                    progress.set_message("pushing image");
                    sync_cmd_ok(&["docker", "push", &container_image]).await?;

                    progress.set_message("deploying");
                    sync_cmd_ok(&[
                        "gcloud",
                        "--project",
                        &project,
                        "run",
                        "deploy",
                        "oort-backend-service",
                        "--image",
                        &container_image,
                        "--allow-unauthenticated",
                        "--region",
                        REGION,
                        "--cpu=1",
                        "--memory=1G",
                        "--timeout=20s",
                        "--concurrency=10",
                        "--max-instances=1",
                        &format!("--service-account=oort-backend-service@{project}.iam.gserviceaccount.com"),
                        &format!("--set-env-vars=PROJECT_ID={project}"),
                    ])
                    .await?;
                }

                progress.finish_with_message("done");
                Ok(())
            }
        }));
    }

    if args.components.contains(&Component::Tools) {
        tasks.spawn(async move {
            use std::os::unix::fs::MetadataExt;
            use std::path::PathBuf;

            let progress = create_progress_bar("tools");

            progress.set_message("building");
            sync_cmd_ok(&["cargo", "build", "--bins"]).await?;

            std::fs::create_dir_all("scratch/tools")?;

            for entry in std::fs::read_dir("target/debug")? {
                let entry = entry?;
                if entry.metadata()?.is_file() && entry.metadata()?.mode() & 1 != 0 {
                    let dst: PathBuf = [
                        std::ffi::OsStr::new("scratch/tools"),
                        entry.path().file_name().unwrap(),
                    ]
                    .iter()
                    .collect();
                    std::fs::copy(entry.path(), dst)?;
                }
            }

            progress.finish_with_message("done");
            Ok(())
        });
    }

    let mut failed = false;
    while let Some(res) = tasks.join_next().await {
        let res = res.map_err(anyhow::Error::msg).and_then(|x| x);
        if let Err(e) = &res {
            PROGRESS.suspend(|| log::error!("Task failed: {}", e));
            failed = true;
        }
    }
    if failed {
        bail!("Release task failed");
    }

    if args.components.contains(&Component::Doc) {
        log::info!("Building docs");
        sync_cmd_ok(&["cargo", "doc", "-p", "oort_api"]).await?;

        if !dry_run && bump_version {
            log::info!("Publishing docs");
            sync_cmd_ok(&["cargo", "publish", "-p", "oort_api"]).await?;
        }
    }

    if !dry_run && !args.skip_github && !args.skip_git_checks {
        log::info!("Pushing to github");
        sync_cmd_ok(&["git", "push"]).await?;
    }

    if bump_version && !dry_run && !args.skip_discord {
        log::info!("Sending Discord message");
        let mut map = std::collections::HashMap::new();
        map.insert(
            "content",
            format!("Released version {version}:\n{changelog}"),
        );
        let client = reqwest::Client::new();
        if let Some(url) = secrets.discord_changelog_webhook {
            let response = client.post(url).json(&map).send().await?;
            response.error_for_status()?;
        }
    }

    let end_time = std::time::Instant::now();
    log::info!("Finished in {:?}", end_time - start_time);
    if args.components.contains(&Component::App) {
        log::info!("Oort should be running at https://{}.web.app", args.project);
    }
    Ok(())
}

trait ExtendedOutput {
    fn stdout_string(&self) -> String;
    fn stderr_string(&self) -> String;
    fn check_success(&self) -> Result<&Self>;
}

impl ExtendedOutput for Output {
    fn stdout_string(&self) -> String {
        std::str::from_utf8(&self.stdout)
            .expect("invalid utf8")
            .to_string()
    }

    fn stderr_string(&self) -> String {
        std::str::from_utf8(&self.stderr)
            .expect("invalid utf8")
            .to_string()
    }

    fn check_success(&self) -> Result<&Self> {
        if !self.status.success() {
            log::error!(
                "Command failed with status {}.\nstderr:\n{}",
                self.status,
                self.stderr_string(),
            );
            bail!(
                "Command failed with status {}.\nstderr:\n{}",
                self.status,
                self.stderr_string(),
            );
        }
        Ok(self)
    }
}

trait ExtendedExitStatus {
    fn check_success(&self) -> Result<&Self>;
}

impl ExtendedExitStatus for ExitStatus {
    fn check_success(&self) -> Result<&Self> {
        if !self.success() {
            bail!("Command failed with status {}", self);
        }
        Ok(self)
    }
}

fn cmd_argv(argv: &[&str]) -> Command {
    PROGRESS.suspend(|| log::info!("Executing {:?}", shell_words::join(argv)));
    let mut cmd = Command::new("nice");
    cmd.kill_on_drop(true);
    cmd.args(argv);
    cmd
}

async fn sync_cmd(argv: &[&str]) -> Result<Output> {
    let result = cmd_argv(argv).output().await;
    if let Ok(output) = &result {
        if log::log_enabled!(log::Level::Debug) {
            if !output.stdout.is_empty() {
                PROGRESS.suspend(|| log::debug!("stdout:\n{}", output.stdout_string()));
            }
            if !output.stderr.is_empty() {
                PROGRESS.suspend(|| log::debug!("stderr:\n{}", output.stderr_string()));
            }
        }
    }
    result.map_err(anyhow::Error::msg)
}

async fn sync_cmd_ok(argv: &[&str]) -> Result<Output> {
    let output = sync_cmd(argv).await?;
    if !output.status.success() {
        log::error!(
            "Command {:?} failed with status {}.\nstderr:\n{}",
            argv,
            output.status,
            output.stderr_string(),
        );
        bail!("Command {:?} failed", argv);
    }
    Ok(output)
}

fn create_progress_bar(prefix: &'static str) -> ProgressBar {
    let progress = PROGRESS.add(ProgressBar::new_spinner());
    progress.enable_steady_tick(std::time::Duration::from_millis(66));
    progress.set_prefix(prefix);
    progress.set_message("starting");
    progress.set_style(
        progress
            .style()
            .template("[{elapsed_precise}] {prefix}: {msg} {spinner}")
            .unwrap(),
    );
    progress
}

fn retry_strategy() -> std::iter::Take<ExponentialBackoff> {
    ExponentialBackoff::from_millis(1000).take(3)
}
