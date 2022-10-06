use anyhow::anyhow;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_telemetry_proto::{LeaderboardData, Telemetry, TelemetryMsg, TimeLeaderboardRow};
use salvo::prelude::*;
use salvo_extra::cors::Cors;

const COLLECTION_NAME: &str = "telemetry";
const PROJECT_ID: &str = "oort-319301";

async fn get_leaderboard_internal(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    let db = FirestoreDb::new(PROJECT_ID).await?;
    log::debug!("Got request {:?}", req);

    let scenario_name: String = req
        .query("scenario_name")
        .ok_or_else(|| anyhow!("missing scenario_name parameter"))?;

    let docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new(COLLECTION_NAME.into())
                .with_filter(FirestoreQueryFilter::Composite(
                    FirestoreQueryFilterComposite::new(vec![
                        FirestoreQueryFilter::Compare(Some(FirestoreQueryFilterCompare::Equal(
                            "type".into(),
                            "FinishScenario".into(),
                        ))),
                        FirestoreQueryFilter::Compare(Some(FirestoreQueryFilterCompare::Equal(
                            "success".into(),
                            true.into(),
                        ))),
                        FirestoreQueryFilter::Compare(Some(FirestoreQueryFilterCompare::Equal(
                            "scenario_name".into(),
                            scenario_name.clone().into(),
                        ))),
                    ]),
                ))
                .with_order_by(vec![FirestoreQueryOrder::new(
                    "time".to_owned(),
                    FirestoreQueryDirection::Ascending,
                )])
                .with_limit(100),
        )
        .await?;

    // userid -> (ticks, creation_time, docid, msg)
    let mut seen = std::collections::HashSet::new();
    let mut leaderboard = LeaderboardData::default();

    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<TelemetryMsg>(doc) {
            if seen.contains(&msg.userid) {
                continue;
            }
            seen.insert(msg.userid.clone());
            if msg.username.is_none() {
                continue;
            }
            let user = msg.username.unwrap();
            if let Telemetry::FinishScenario { time, .. } = &msg.payload {
                leaderboard.lowest_time.push(TimeLeaderboardRow {
                    userid: msg.userid.clone(),
                    username: Some(user.clone()),
                    time: format!("{:.2}s", time.unwrap_or_default()),
                })
            }
            if leaderboard.lowest_time.len() >= 20 {
                break;
            }
        } else {
            log::error!("Failed to deserialize doc {}", doc.name);
        }
    }

    res.render(&serde_json::to_string_pretty(&leaderboard)?);

    Ok(())
}

#[handler]
async fn get_leaderboard(req: &mut Request, res: &mut Response) {
    if let Err(e) = get_leaderboard_internal(req, res).await {
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

    let router = Router::with_hoop(cors_handler).push(
        Router::with_path("/leaderboard")
            .get(get_leaderboard)
            .options(nop),
    );

    log::info!("Starting oort_leaderboard_service");
    Server::new(TcpListener::bind(&format!("0.0.0.0:{}", port)))
        .serve(router)
        .await;
}
