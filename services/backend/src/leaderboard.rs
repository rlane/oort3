use crate::{discord, error, project_id, Error};
use axum::extract::Path;
use axum::Json;
use bytes::Bytes;
use chrono::Utc;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_proto::{LeaderboardData, LeaderboardSubmission, TimeLeaderboardRow};

async fn fetch_leaderboard(
    db: &FirestoreDb,
    scenario_name: &str,
) -> anyhow::Result<LeaderboardData> {
    let docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new("leaderboard".into())
                .with_filter(FirestoreQueryFilter::Composite(
                    FirestoreQueryFilterComposite::new(
                        vec![FirestoreQueryFilter::Compare(Some(
                            FirestoreQueryFilterCompare::Equal(
                                "scenario_name".into(),
                                scenario_name.into(),
                            ),
                        ))],
                        FirestoreQueryFilterCompositeOperator::And,
                    ),
                ))
                .with_order_by(vec![
                    FirestoreQueryOrder::new("time".to_owned(), FirestoreQueryDirection::Ascending),
                    FirestoreQueryOrder::new(
                        "timestamp".to_owned(),
                        FirestoreQueryDirection::Ascending,
                    ),
                ])
                .with_limit(1000),
        )
        .await?;

    let mut leaderboard = LeaderboardData::default();

    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<LeaderboardSubmission>(doc) {
            leaderboard.lowest_time.push(TimeLeaderboardRow {
                userid: msg.userid.clone(),
                username: Some(msg.username.clone()),
                time: format!("{:.3}s", msg.time),
                encrypted_code: oort_code_encryption::encrypt(&msg.code)?,
            });
        } else {
            log::error!("Failed to deserialize doc {}", doc.name);
        }
    }

    Ok(leaderboard)
}

pub async fn get(Path(scenario_name): Path<String>) -> Result<Json<LeaderboardData>, Error> {
    let db = FirestoreDb::new(&project_id()).await?;
    let data: LeaderboardData = fetch_leaderboard(&db, &scenario_name).await?;
    Ok(Json(data))
}

pub async fn post(payload: Bytes) -> Result<Json<LeaderboardData>, Error> {
    let db = FirestoreDb::new(&project_id()).await?;

    let payload = match oort_envelope::remove(payload.as_ref()) {
        Some(x) => x,
        None => {
            return Err(error(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid envelope".into(),
            ));
        }
    };
    let mut obj: LeaderboardSubmission = serde_json::from_slice(&payload)?;

    obj.timestamp = Utc::now();
    let path = format!("{}.{}", obj.scenario_name, obj.userid);

    let old_leaderboard = fetch_leaderboard(&db, &obj.scenario_name).await?;

    if let Ok(existing_obj) = db
        .get_obj::<LeaderboardSubmission, _>("leaderboard", &path)
        .await
    {
        log::debug!("Got existing obj {:?}", existing_obj);
        if existing_obj.time <= obj.time {
            log::debug!("Ignoring slower time");
            return Ok(Json(old_leaderboard));
        }
    }

    db.update_obj("leaderboard", &path, &obj, None, None, None)
        .await?;

    let new_leaderboard = fetch_leaderboard(&db, &obj.scenario_name).await?;

    let get_rank = |leaderboard: &LeaderboardData, userid: &str| -> Option<usize> {
        leaderboard
            .lowest_time
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.userid == userid)
            .map(|(i, _)| i + 1)
    };

    let old_rank = get_rank(&old_leaderboard, &obj.userid);
    let new_rank = get_rank(&new_leaderboard, &obj.userid);

    let rank_improved = match (old_rank, new_rank) {
        (Some(old_rank), Some(new_rank)) if old_rank > new_rank => true,
        (None, Some(_)) => true,
        _ => false,
    };

    if rank_improved {
        discord::send_message(
            discord::Channel::Leaderboard,
            format!(
                "{} achieved leaderboard rank {} on scenario {} with time {:.3}s",
                obj.username,
                new_rank.unwrap(),
                obj.scenario_name,
                obj.time
            ),
        );
    }

    Ok(Json(new_leaderboard))
}
