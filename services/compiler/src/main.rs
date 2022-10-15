use bytes::Bytes;
use salvo::prelude::*;
use salvo_extra::cors::Cors;
use tokio::process::Command;

async fn compile_internal(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    log::debug!("Got request {:?}", req);
    let payload = req.payload().await?;
    let code = std::str::from_utf8(payload)?;
    log::debug!("Code: {}", code);
    std::fs::write("ai/src/user.rs", payload)?;
    let start_time = std::time::Instant::now();
    let output = Command::new("./scripts/build-ai-fast.sh").output().await?;
    let elapsed = std::time::Instant::now() - start_time;
    if !output.status.success() {
        log::info!("Compile failed in {:?}", elapsed);
        res.set_status_code(StatusCode::BAD_REQUEST);
        let stdout = std::str::from_utf8(&output.stdout)?;
        let stderr = std::str::from_utf8(&output.stderr)?;
        log::debug!("Compile failed: stderr={}\nstdout={}", stderr, stdout);
        res.render(stderr.to_string());
        return Ok(());
    }
    log::info!("Compile succeeded in {:?}", elapsed);
    log::debug!("Compile finished: {}", std::str::from_utf8(&output.stderr)?);
    let wasm = std::fs::read("output.wasm")?;
    res.write_body(Bytes::copy_from_slice(&wasm))?;
    Ok(())
}

#[handler]
async fn compile(req: &mut Request, res: &mut Response) {
    if let Err(e) = compile_internal(req, res).await {
        log::error!("compile request error: {}", e);
        res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(e.to_string());
    }
}

async fn format_internal(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    log::debug!("Got request {:?}", req);
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

    let cors_handler = Cors::builder()
        .allow_any_origin()
        .allow_methods(vec!["POST", "OPTIONS"])
        .allow_header("content-type")
        .build();

    let router = Router::with_hoop(cors_handler)
        .push(Router::with_path("/compile").post(compile).options(nop))
        .push(Router::with_path("/format").post(format).options(nop));
    log::info!("Starting oort_compiler_service v1");
    Server::new(TcpListener::bind(&format!("0.0.0.0:{}", port)))
        .serve(router)
        .await;
}
