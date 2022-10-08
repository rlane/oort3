use anyhow::anyhow;
use chrono::Utc;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_proto::{LeaderboardData, LeaderboardSubmission, TimeLeaderboardRow};
use salvo::prelude::*;
use salvo_extra::cors::Cors;

const LEADERBOARD_COLLECTION_NAME: &str = "leaderboard";
const PROJECT_ID: &str = "oort-319301";

async fn get_leaderboard_internal(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    let db = FirestoreDb::new(PROJECT_ID).await?;
    log::debug!("Got request {:?}", req);

    let scenario_name: String = req
        .query("scenario_name")
        .ok_or_else(|| anyhow!("missing scenario_name parameter"))?;

    let docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new(LEADERBOARD_COLLECTION_NAME.into())
                .with_filter(FirestoreQueryFilter::Composite(
                    FirestoreQueryFilterComposite::new(vec![FirestoreQueryFilter::Compare(Some(
                        FirestoreQueryFilterCompare::Equal(
                            "scenario_name".into(),
                            scenario_name.clone().into(),
                        ),
                    ))]),
                ))
                .with_order_by(vec![
                    FirestoreQueryOrder::new("time".to_owned(), FirestoreQueryDirection::Ascending),
                    FirestoreQueryOrder::new(
                        "timestamp".to_owned(),
                        FirestoreQueryDirection::Ascending,
                    ),
                ])
                .with_limit(20),
        )
        .await?;

    let mut leaderboard = LeaderboardData::default();

    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<LeaderboardSubmission>(doc) {
            leaderboard.lowest_time.push(TimeLeaderboardRow {
                userid: msg.userid.clone(),
                username: Some(msg.username.clone()),
                time: format!("{:.2}s", msg.time),
            });
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

async fn post_leaderboard_internal(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    let db = FirestoreDb::new(PROJECT_ID).await?;
    log::debug!("Got request {:?}", req);
    let payload = req.payload().await?;
    log::debug!("Got payload {:?}", payload);
    let mut obj: LeaderboardSubmission = serde_json::from_slice(payload)?;
    obj.timestamp = Utc::now();
    log::debug!("Got request obj {:?}", obj);
    let path = format!("{}.{}", obj.scenario_name, obj.userid);

    if let Ok(existing_obj) = db
        .get_obj::<LeaderboardSubmission>(LEADERBOARD_COLLECTION_NAME, &path)
        .await
    {
        log::debug!("Got existing obj {:?}", existing_obj);
        if existing_obj.time <= obj.time {
            log::debug!("Ignoring slower time");
            res.render("");
            return Ok(());
        }
    }

    db.update_obj(LEADERBOARD_COLLECTION_NAME, &path, &obj, None)
        .await?;
    res.render("");
    Ok(())
}

#[handler]
async fn post_leaderboard(req: &mut Request, res: &mut Response) {
    if let Err(e) = post_leaderboard_internal(req, res).await {
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

    let router = Router::with_hoop(cors_handler)
        .push(
            Router::with_path("/leaderboard")
                .get(get_leaderboard)
                .options(nop),
        )
        .push(
            Router::with_path("/leaderboard")
                .post(post_leaderboard)
                .options(nop),
        );

    log::info!("Starting oort_leaderboard_service");
    Server::new(TcpListener::bind(&format!("0.0.0.0:{}", port)))
        .serve(router)
        .await;
}
