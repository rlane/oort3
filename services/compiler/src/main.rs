use axum::extract::State;
use axum::Router;
use bytes::Bytes;
use clap::Parser as _;
use http::{Method, StatusCode};
use once_cell::sync::Lazy;
use oort_compiler::Compiler;
use oort_compiler_service::{error, Error};
use std::io::Write;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use tokio::process::Command;
use tower_http::cors::{Any, CorsLayer};

const MAX_CONCURRENCY: usize = 3;
static FORMAT_LOCK: Lazy<tokio::sync::Mutex<()>> = Lazy::new(|| tokio::sync::Mutex::new(()));
static SEMAPHORE: Lazy<tokio::sync::Semaphore> =
    Lazy::new(|| tokio::sync::Semaphore::new(MAX_CONCURRENCY));

async fn post_compile(
    State(compiler): State<Arc<Mutex<Compiler>>>,
    mut code: String,
) -> Result<Bytes, Error> {
    let permit = SEMAPHORE.try_acquire();
    if permit.is_err() {
        Err(anyhow::anyhow!("Service overloaded"))?
    }

    if oort_code_encryption::is_encrypted(&code) {
        log::debug!("Encrypted code: {}", code);
        code = oort_code_encryption::decrypt(&code)?;
    }
    log::debug!("Code: {}", code);
    oort_compiler_service::sanitizer::check(&code)?;
    let start_time = std::time::Instant::now();
    let result = tokio::runtime::Handle::current()
        .spawn_blocking(move || compiler.lock().unwrap().compile(&code))
        .await?;
    let elapsed = std::time::Instant::now() - start_time;
    match result {
        Ok(wasm) => {
            log::info!("Compile succeeded in {:?}", elapsed);
            Ok(Bytes::copy_from_slice(&wasm))
        }
        Err(e) => {
            log::info!("Compile failed in {:?}", elapsed);
            log::debug!("Compile failed: {}", e);
            Err(error(StatusCode::BAD_REQUEST, e.to_string()))
        }
    }
}

async fn post_format(code: String) -> Result<String, Error> {
    let _guard = FORMAT_LOCK.lock().await;
    let mut tmpfile = NamedTempFile::new()?;
    tmpfile.write_all(code.as_bytes())?;
    let start_time = std::time::Instant::now();
    let output = Command::new("rustfmt")
        .args([tmpfile.path()])
        .output()
        .await?;
    let elapsed = std::time::Instant::now() - start_time;
    if !output.status.success() {
        log::info!("Format failed in {:?}", elapsed);
        let stdout = std::str::from_utf8(&output.stdout)?;
        let stderr = std::str::from_utf8(&output.stderr)?;
        log::debug!("Format failed: stderr={}\nstdout={}", stderr, stdout);
        return Err(error(StatusCode::BAD_REQUEST, stderr.to_string()));
    }
    log::info!("Format succeeded in {:?}", elapsed);
    let formatted = std::fs::read_to_string(tmpfile.path())?;
    Ok(formatted)
}

#[tokio::main]
async fn main() {
    stackdriver_logger::init_with_cargo!();

    #[derive(clap::Parser, Debug)]
    struct Arguments {
        #[clap(short, long)]
        prepare: bool,
    }
    let args = Arguments::parse();

    let mut port: u16 = 8080;
    match std::env::var("PORT") {
        Ok(p) => {
            match p.parse::<u16>() {
                Ok(n) => {
                    port = n;
                }
                Err(_e) => {}
            };
        }
        Err(_e) => {}
    };

    let dir = "/tmp/oort-ai";
    std::fs::create_dir_all(dir).unwrap();
    let mut compiler = Compiler::new_with_dir(std::path::Path::new(dir));

    if args.prepare {
        compiler.enable_online();
        compiler
            .compile(include_str!("../../../shared/builtin_ai/src/empty.rs"))
            .unwrap();
        return;
    }

    log::info!("Starting oort_compiler_service v1");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin(Any)
        .allow_headers(Any);

    let router = {
        use axum::routing::post;
        Router::new()
            .route("/compile", post(post_compile))
            .route("/format", post(post_format))
            .layer(cors)
            .layer(tower_http::trace::TraceLayer::new_for_http())
            .with_state(Arc::new(Mutex::new(compiler)))
    };

    axum::Server::bind(&format!("0.0.0.0:{port}").parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
