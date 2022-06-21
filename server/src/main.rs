use bytes::Bytes;
use salvo::prelude::*;
use std::process::Command;

#[fn_handler]
async fn compile(req: &mut Request, res: &mut Response) {
    log::info!("Got request {:?}", req);
    let payload = req.payload().await.expect("Reading payload failed");
    let slice: &[u8] = payload;
    let code = std::str::from_utf8(slice).unwrap();
    log::debug!("Code: {}", code);

    std::fs::write("ai/src/user.rs", slice).unwrap();

    let output = Command::new("./scripts/build-ai.sh").output().unwrap();

    if !output.status.success() {
        log::info!(
            "Compile failed: {}",
            std::str::from_utf8(&output.stderr).unwrap()
        );
        return;
    }

    log::info!(
        "Compile finished: {}",
        std::str::from_utf8(&output.stderr).unwrap()
    );

    let wasm =
        std::fs::read("target/wasm32-unknown-unknown/release/oort_reference_ai.wasm").unwrap();

    res.write_body(Bytes::copy_from_slice(&wasm))
        .expect("Response::write_body failed");
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

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
    let router = Router::with_path("compile").post(compile);
    log::info!("Starting oort_server 1");
    Server::new(TcpListener::bind(&format!("0.0.0.0:{}", port)))
        .serve(router)
        .await;
}
