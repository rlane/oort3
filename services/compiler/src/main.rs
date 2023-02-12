use bytes::Bytes;
use clap::Parser as _;
use once_cell::sync::Lazy;
use oort_compiler::Compiler;
use salvo::prelude::*;
use salvo_extra::cors::Cors;
use std::sync::{Arc, Mutex};
use tokio::process::Command;

static LOCK: Lazy<tokio::sync::Mutex<()>> = Lazy::new(|| tokio::sync::Mutex::new(()));

async fn compile_internal(
    compiler: Arc<Mutex<Compiler>>,
    req: &mut Request,
    res: &mut Response,
) -> anyhow::Result<()> {
    log::debug!("Got compile request {:?}", req);
    let payload = req.payload().await?;
    let mut code = std::str::from_utf8(payload)?.to_string();
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
            res.write_body(Bytes::copy_from_slice(&wasm))?;
            Ok(())
        }
        Err(e) => {
            log::info!("Compile failed in {:?}", elapsed);
            res.set_status_code(StatusCode::BAD_REQUEST);
            log::debug!("Compile failed: {}", e);
            res.render(e.to_string());
            Ok(())
        }
    }
}

struct CompileHandler {
    compiler: Arc<Mutex<Compiler>>,
}

#[async_trait]
impl Handler for CompileHandler {
    async fn handle(
        &self,
        req: &mut Request,
        _depot: &mut Depot,
        res: &mut Response,
        _ctrl: &mut FlowCtrl,
    ) {
        if let Err(e) = compile_internal(self.compiler.clone(), req, res).await {
            log::error!("compile request error: {}", e);
            res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(e.to_string());
        }
    }
}

async fn format_internal(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    let _guard = LOCK.lock().await;
    log::debug!("Got format request {:?}", req);
    let payload = req.payload().await?;
    let code = std::str::from_utf8(payload)?;
    log::debug!("Code: {}", code);
    let filename = "ai/src/format.rs";
    std::fs::write(filename, payload)?;
    let start_time = std::time::Instant::now();
    let output = Command::new("rustfmt").args([filename]).output().await?;
    let elapsed = std::time::Instant::now() - start_time;
    if !output.status.success() {
        log::info!("Format failed in {:?}", elapsed);
        res.set_status_code(StatusCode::BAD_REQUEST);
        let stdout = std::str::from_utf8(&output.stdout)?;
        let stderr = std::str::from_utf8(&output.stderr)?;
        log::debug!("Format failed: stderr={}\nstdout={}", stderr, stdout);
        res.render(stderr.to_string());
        return Ok(());
    }
    log::info!("Format succeeded in {:?}", elapsed);
    log::debug!("Format finished: {}", std::str::from_utf8(&output.stderr)?);
    let formatted = std::fs::read(filename)?;
    res.write_body(Bytes::copy_from_slice(&formatted))?;
    Ok(())
}

#[handler]
async fn format(req: &mut Request, res: &mut Response) {
    if let Err(e) = format_internal(req, res).await {
        log::error!("Format request error: {}", e);
        res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(e.to_string());
    }
}

#[handler]
async fn nop(_req: &mut Request, res: &mut Response) {
    res.render("");
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
            .compile(include_str!("../../../shared/ai/empty.rs"))
            .unwrap();
        return;
    }

    let cors_handler = Cors::builder()
        .allow_any_origin()
        .allow_methods(vec!["POST", "OPTIONS"])
        .allow_header("content-type")
        .build();

    let router = Router::with_hoop(cors_handler)
        .push(
            Router::with_path("/compile")
                .post(CompileHandler {
                    compiler: Arc::new(Mutex::new(compiler)),
                })
                .options(nop),
        )
        .push(Router::with_path("/format").post(format).options(nop));
    log::info!("Starting oort_compiler_service v1");
    Server::new(TcpListener::bind(&format!("0.0.0.0:{port}")))
        .serve(router)
        .await;
}
