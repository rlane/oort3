use chrono::prelude::*;
use firestore::*;
use oort_proto::TelemetryMsg;
use salvo::prelude::*;
use salvo_extra::cors::Cors;

fn project_id() -> &'static str {
    match std::env::var("ENVIRONMENT") {
        Ok(x) if x == "dev" => { "oort-dev" }
        Ok(x) if x == "prod" => { "oort-319301" }
        _ => { panic!("Invalid ENVIRONMENT") }
    }
}

fn generate_docid() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();

    (0..16)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

async fn post_internal(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id()).await?;
    log::debug!("Got request {:?}", req);
    let payload = req.payload().await?;
    log::debug!("Got payload {:?}", payload);
    let mut obj: TelemetryMsg = serde_json::from_slice(payload)?;
    obj.timestamp = Utc::now();
    log::debug!("Got request obj {:?}", obj);
    db.create_obj("telemetry", &generate_docid(), &obj)
        .await?;
    res.render("");
    Ok(())
}

#[handler]
async fn post(req: &mut Request, res: &mut Response) {
    if let Err(e) = post_internal(req, res).await {
        log::error!("error: {}", e);
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

    let router =
        Router::with_hoop(cors_handler).push(Router::with_path("/post").post(post).options(nop));

    log::info!("Starting oort_telemetry_service");
    log::info!("Using project ID {}", project_id());
    Server::new(TcpListener::bind(&format!("0.0.0.0:{}", port)))
        .serve(router)
        .await;
}
