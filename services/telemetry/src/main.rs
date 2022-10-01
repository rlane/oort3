use firestore::*;
use oort_telemetry_proto::TelemetryMsg;
use salvo::prelude::*;
use salvo_extra::cors::CorsHandler;

const COLLECTION_NAME: &'static str = "telemetry";
const PROJECT_ID: &'static str = "oort-319301";

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
    let db = FirestoreDb::new(PROJECT_ID).await?;
    log::debug!("Got request {:?}", req);
    let payload = req.payload().await?;
    log::debug!("Got payload {:?}", payload);
    let obj: TelemetryMsg = serde_json::from_slice(&payload)?;
    log::debug!("Got request obj {:?}", obj);
    db.create_obj(COLLECTION_NAME, &generate_docid(), &obj)
        .await?;
    res.render("");
    Ok(())
}

#[fn_handler]
async fn post(req: &mut Request, res: &mut Response) {
    if let Err(e) = post_internal(req, res).await {
        log::error!("error: {}", e);
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

    let router = Router::with_path("post").hoop(cors_handler).post(post);
    log::info!("Starting oort_telemetry_service");
    Server::new(TcpListener::bind(&format!("0.0.0.0:{}", port)))
        .serve(router)
        .await;
}
