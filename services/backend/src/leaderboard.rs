use crate::{discord, error, project_id, Error};
use axum::debug_handler;
use axum::extract::{Path, State};
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
            leaderboard.lowest_time.push(make_row(&msg));
        } else {
            log::error!("Failed to deserialize doc {}", doc.name);
        }
    }

    Ok(leaderboard)
}

pub fn make_row(submission: &LeaderboardSubmission) -> TimeLeaderboardRow {
    TimeLeaderboardRow {
        userid: submission.userid.clone(),
        username: Some(submission.username.clone()),
        time: format!("{:.3}s", submission.time),
        encrypted_code: oort_code_encryption::encrypt(&submission.code).unwrap(),
        timestamp: Some(submission.timestamp),
    }
}

pub async fn get(
    Path(scenario_name): Path<String>,
    cache: State<SharedLeaderboardCache>,
) -> Result<Json<LeaderboardData>, Error> {
    let db = FirestoreDb::new(&project_id()).await?;
    let data: LeaderboardData = cache.get(&db, &scenario_name).await?;
    Ok(Json(data))
}

#[debug_handler]
pub async fn post(
    cache: State<SharedLeaderboardCache>,
    payload: Bytes,
) -> Result<Json<LeaderboardData>, Error> {
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

    let old_leaderboard = cache.get(&db, &obj.scenario_name).await?;

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

    cache
        .update(&db, &obj.scenario_name, make_row(&obj))
        .await?;

    let new_leaderboard = cache.get(&db, &obj.scenario_name).await?;

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

    if rank_improved && new_rank.map(|x| x <= 10).unwrap_or(false) {
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

    // TODO: remove this
    if rank_improved {
        fn summarize_leaderboard(data: &LeaderboardData) -> String {
            let mut s = String::new();
            for (i, row) in data.lowest_time.iter().enumerate() {
                s.push_str(&format!(
                    "{:>3} {:>10} {:>10} {}\n",
                    i + 1,
                    row.userid,
                    row.time,
                    row.username.as_deref().unwrap_or(""),
                ));
            }
            s
        }
        let expected_leaderboard_summary = summarize_leaderboard(&new_leaderboard);
        let actual_leaderboard_summary =
            summarize_leaderboard(&fetch_leaderboard(&db, &obj.scenario_name).await?);
        if actual_leaderboard_summary != expected_leaderboard_summary {
            log::warn!(
                "Leaderboard cache mismatch:\nactual {:#?}\ncached {:#?}",
                actual_leaderboard_summary,
                expected_leaderboard_summary
            );
        }
    }

    Ok(Json(new_leaderboard))
}

pub type SharedLeaderboardCache = std::sync::Arc<LeaderboardCache>;

pub struct LeaderboardCache {
    scenarios: tokio::sync::Mutex<std::collections::HashMap<String, LeaderboardCacheScenario>>,
}

struct LeaderboardCacheScenario {
    timestamp: chrono::DateTime<Utc>,
    leaderboard: LeaderboardData,
}

impl LeaderboardCache {
    pub fn new() -> Self {
        Self {
            scenarios: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub async fn get(
        &self,
        db: &FirestoreDb,
        scenario_name: &str,
    ) -> Result<LeaderboardData, Error> {
        if let Some(cached) = self.scenarios.lock().await.get(scenario_name) {
            if cached.timestamp + chrono::Duration::minutes(1) > Utc::now() {
                log::info!("Leaderboard cache hit for {}", scenario_name);
                return Ok(cached.leaderboard.clone());
            }
        }
        log::info!("Leaderboard cache miss for {}", scenario_name);
        let leaderboard = fetch_leaderboard(db, scenario_name).await?;
        self.scenarios.lock().await.insert(
            scenario_name.to_owned(),
            LeaderboardCacheScenario {
                timestamp: Utc::now(),
                leaderboard: leaderboard.clone(),
            },
        );
        Ok(leaderboard)
    }

    pub async fn update(
        &self,
        db: &FirestoreDb,
        scenario_name: &str,
        row: TimeLeaderboardRow,
    ) -> Result<(), Error> {
        log::info!("Leaderboard cache update for {}", scenario_name);
        let has_cache_entry = {
            let scenarios = self.scenarios.lock().await;
            scenarios.contains_key(scenario_name)
        };

        if !has_cache_entry {
            let leaderboard = fetch_leaderboard(db, scenario_name).await?;
            let mut scenarios = self.scenarios.lock().await;
            scenarios.insert(
                scenario_name.to_owned(),
                LeaderboardCacheScenario {
                    timestamp: Utc::now(),
                    leaderboard,
                },
            );
        }

        let mut scenarios = self.scenarios.lock().await;
        let cached = scenarios.get_mut(scenario_name).unwrap();
        cached
            .leaderboard
            .lowest_time
            .retain(|x| x.userid != row.userid);
        cached.leaderboard.lowest_time.push(row);
        cached
            .leaderboard
            .lowest_time
            .sort_by_key(|x| (x.time.clone(), x.timestamp));
        Ok(())
    }
}

impl Default for LeaderboardCache {
    fn default() -> Self {
        Self::new()
    }
}
