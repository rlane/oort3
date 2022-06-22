use bytes::Bytes;
use salvo::{
    http::{self, HeaderValue},
    prelude::*,
};
use salvo_extra::cors::CorsHandler;
use tokio::process::Command;

async fn compile_internal(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    log::debug!("Got request {:?}", req);
    let payload = req.payload().await?;
    let code = std::str::from_utf8(payload)?;
    log::debug!("Code: {}", code);
    std::fs::write("ai/src/user.rs", payload)?;
    let start_time = std::time::Instant::now();
    let output = Command::new("./scripts/build-ai.sh").output().await?;
    let elapsed = std::time::Instant::now() - start_time;
    if !output.status.success() {
        log::info!("Compile failed in {:?}", elapsed);
        let stdout = std::str::from_utf8(&output.stdout)?;
        let stderr = std::str::from_utf8(&output.stderr)?;
        log::debug!("Compile failed: stderr={}\nstdout={}", stderr, stdout);
        res.render(stdout.to_string());
        return Ok(());
    }
    log::info!("Compile succeeded in {:?}", elapsed);
    log::debug!("Compile finished: {}", std::str::from_utf8(&output.stderr)?);
    let wasm = std::fs::read("target/wasm32-unknown-unknown/release/oort_reference_ai.wasm")?;
    res.write_body(Bytes::copy_from_slice(&wasm))?;
    Ok(())
}

#[fn_handler]
async fn compile(req: &mut Request, res: &mut Response) {
    if let Err(e) = compile_internal(req, res).await {
        log::error!("compile request error: {}", e);
        res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(e.to_string().to_string());
    }
}

#[tokio::main]
async fn main() {
    stackdriver_logger::init_with_cargo!();

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

    let cors_handler = CorsHandler::builder().with_allow_any_origin().build();

    let router = Router::with_path("compile")
        .hoop(cors_handler)
        .post(compile);
    log::info!("Starting oort_server");
    Server::new(TcpListener::bind(&format!("0.0.0.0:{}", port)))
        .serve(router)
        .await;
}
