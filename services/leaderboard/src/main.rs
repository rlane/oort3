mod discord;

use anyhow::anyhow;
use chrono::Utc;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_proto::{LeaderboardData, LeaderboardSubmission, TimeLeaderboardRow};
use salvo::prelude::*;
use salvo_extra::cors::Cors;
use tokio::sync::mpsc;

fn project_id() -> &'static str {
    match std::env::var("ENVIRONMENT") {
        Ok(x) if x == "dev" => "oort-dev",
        Ok(x) if x == "prod" => "oort-319301",
        _ => {
            panic!("Invalid ENVIRONMENT")
        }
    }
}

async fn fetch_leaderboard(
    db: &FirestoreDb,
    scenario_name: &str,
) -> anyhow::Result<LeaderboardData> {
    let docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new("leaderboard".into())
                .with_filter(FirestoreQueryFilter::Composite(
                    FirestoreQueryFilterComposite::new(vec![FirestoreQueryFilter::Compare(Some(
                        FirestoreQueryFilterCompare::Equal(
                            "scenario_name".into(),
                            scenario_name.into(),
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
                .with_limit(10),
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

    Ok(leaderboard)
}

async fn render_leaderboard(
    leaderboard: &LeaderboardData,
    res: &mut Response,
) -> anyhow::Result<()> {
    res.render(&serde_json::to_string_pretty(leaderboard)?);

    Ok(())
}

async fn get_leaderboard_internal(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id()).await?;
    log::debug!("Got request {:?}", req);

    let scenario_name: String = req
        .query("scenario_name")
        .ok_or_else(|| anyhow!("missing scenario_name parameter"))?;

    let leaderboard = fetch_leaderboard(&db, &scenario_name).await?;
    render_leaderboard(&leaderboard, res).await
}

#[handler]
async fn get_leaderboard(req: &mut Request, res: &mut Response) {
    if let Err(e) = get_leaderboard_internal(req, res).await {
        log::error!("error: {}", e);
        res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(e.to_string());
    }
}

async fn post_leaderboard_internal(
    req: &mut Request,
    res: &mut Response,
    discord_tx: &mpsc::Sender<discord::Msg>,
) -> anyhow::Result<()> {
    let db = FirestoreDb::new(project_id()).await?;
    log::debug!("Got request {:?}", req);
    let payload = match oort_envelope::remove(req.payload().await?) {
        Some(x) => x,
        None => {
            log::warn!("Failed to remove envelope");
            return Err(anyhow!("failed"));
        }
    };
    log::debug!("Got payload {:?}", payload);
    let mut obj: LeaderboardSubmission = serde_json::from_slice(&payload)?;
    obj.timestamp = Utc::now();
    log::debug!("Got request obj {:?}", obj);
    let path = format!("{}.{}", obj.scenario_name, obj.userid);

    let old_leaderboard = fetch_leaderboard(&db, &obj.scenario_name).await?;

    if let Ok(existing_obj) = db
        .get_obj::<LeaderboardSubmission>("leaderboard", &path)
        .await
    {
        log::debug!("Got existing obj {:?}", existing_obj);
        if existing_obj.time <= obj.time {
            log::debug!("Ignoring slower time");
            return render_leaderboard(&old_leaderboard, res).await;
        }
    }

    db.update_obj("leaderboard", &path, &obj, None).await?;

    let new_leaderboard = fetch_leaderboard(&db, &obj.scenario_name).await?;

    let get_rank = |leaderboard: &LeaderboardData, userid: &str| -> Option<usize> {
        leaderboard.lowest_time.iter().enumerate().find(|(_,entry)| entry.userid == userid).map(|(i,_)| i + 1)
    };

    let old_rank = get_rank(&old_leaderboard, &obj.userid);
    let new_rank = get_rank(&new_leaderboard, &obj.userid);

    let rank_improved = match (old_rank, new_rank) {
        (Some(old_rank), Some(new_rank)) if old_rank > new_rank => true,
        (None, Some(_)) => true,
        _ => false,
    };

    if rank_improved {
        if let Err(e) = discord_tx
            .send_timeout(
                discord::Msg {
                    text: format!(
                        "{} achieved leaderboard rank {} on scenario {} with time {:.2}s",
                        obj.username, new_rank.unwrap(), obj.scenario_name, obj.time
                    ),
                },
                std::time::Duration::from_secs(1),
            )
        .await
        {
            log::error!("Failed to send Discord message: {:?}", e);
        }
    }

    render_leaderboard(&new_leaderboard, res).await
}

struct PostLeaderboard {
    discord_tx: mpsc::Sender<discord::Msg>,
}

#[async_trait]
impl Handler for PostLeaderboard {
    async fn handle(
        &self,
        req: &mut Request,
        _depot: &mut Depot,
        res: &mut Response,
        _ctrl: &mut FlowCtrl,
    ) {
        if let Err(e) = post_leaderboard_internal(req, res, &self.discord_tx).await {
            log::error!("error: {}", e);
            res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(e.to_string());
        }
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

    log::info!("Starting oort_leaderboard_service");
    log::info!("Using project ID {}", project_id());

    let discord_tx = discord::start().await.expect("starting Discord bot");

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
                .post(PostLeaderboard { discord_tx })
                .options(nop),
        );

    Server::new(TcpListener::bind(&format!("0.0.0.0:{}", port)))
        .serve(router)
        .await;
}
