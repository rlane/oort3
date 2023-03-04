use anyhow::anyhow;
use chrono::Utc;
use firestore::*;
use oort_proto::{TournamentResults, TournamentSubmission};
use salvo::cors::Cors;
use salvo::prelude::*;

fn project_id() -> &'static str {
    match std::env::var("ENVIRONMENT") {
        Ok(x) if x == "dev" => "oort-dev",
        Ok(x) if x == "prod" => "oort-319301",
        _ => {
            panic!("Invalid ENVIRONMENT")
        }
    }
}

async fn submit_internal(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id()).await?;
    let payload = req.payload().await?;
    let mut obj: TournamentSubmission = serde_json::from_slice(&payload)?;
    obj.timestamp = Utc::now();
    log::debug!("{:?}", obj);
    let docid = format!("{}.{}", obj.scenario_name, obj.userid);
    db.update_obj("tournament", &docid, &obj, None).await?;
    res.render(docid);
    Ok(())
}

#[handler]
async fn submit(req: &mut Request, res: &mut Response) {
    if let Err(e) = submit_internal(req, res).await {
        log::error!("error: {}", e);
        res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(e.to_string());
    }
}

#[handler]
async fn get_tournament_results(
    req: &mut Request,
) -> anyhow::Result<salvo::writer::Json<TournamentResults>> {
    let db = FirestoreDb::new(project_id()).await?;
    let id: String = req.param("id").ok_or(anyhow!("missing id parameter"))?;
    let tournament_results = db
        .get_obj::<TournamentResults>("tournament_results", &id)
        .await?;
    Ok(Json(tournament_results))
}

#[handler]
async fn nop(_req: &mut Request, res: &mut Response) {
    res.render("");
}

#[tokio::main]
pub async fn main() {
    stackdriver_logger::init_with_cargo!();

    let mut port: u16 = 8085;
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

    log::info!("Starting oort_tournament_service");
    log::info!("Using project ID {}", project_id());

    let cors_handler = Cors::builder()
        .allow_any_origin()
        .allow_methods(vec!["POST", "OPTIONS"])
        .allow_header("content-type")
        .build();

    let router = Router::with_hoop(cors_handler)
        .push(
            Router::with_path("/tournament/submit")
                .post(submit)
                .options(nop),
        )
        .push(
            Router::with_path("/tournament/results/<id>")
                .get(get_tournament_results)
                .options(nop),
        );

    Server::new(TcpListener::bind(&format!("0.0.0.0:{port}")))
        .serve(router)
        .await;
}
