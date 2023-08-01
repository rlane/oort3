use crate::{project_id, Error};
use anyhow::bail;
use axum::extract::{Json, Path};
use chrono::Utc;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_proto::{LeaderboardSubmission, ShortcodeUpload, TournamentSubmission};
use regex::Regex;

#[derive(Clone, Debug)]
enum Shortcode {
    Leaderboard {
        username: String,
        scenario_name: String,
    },
    Uploaded {
        docid: String,
    },
    Tournament {
        username: String,
        scenario_name: String,
    },
}

fn parse_id(id: &str) -> anyhow::Result<Shortcode> {
    let leaderboard_re = Regex::new(r"^leaderboard:([a-zA-Z0-9_-]+):(\w+)$")?;
    let tournament_re = Regex::new(r"^tournament:([a-zA-Z0-9_-]+):(\w+)$")?;
    let uploaded_re = Regex::new(r"^([a-zA-Z0-9_.-]+)$")?;
    if let Some(caps) = leaderboard_re.captures(id) {
        let username = caps.get(1).unwrap().as_str().to_string();
        let scenario_name = caps.get(2).unwrap().as_str().to_string();
        Ok(Shortcode::Leaderboard {
            username,
            scenario_name,
        })
    } else if let Some(caps) = tournament_re.captures(id) {
        let username = caps.get(1).unwrap().as_str().to_string();
        let scenario_name = caps.get(2).unwrap().as_str().to_string();
        Ok(Shortcode::Tournament {
            username,
            scenario_name,
        })
    } else if let Some(caps) = uploaded_re.captures(id) {
        let docid = caps.get(1).unwrap().as_str().to_string();
        Ok(Shortcode::Uploaded { docid })
    } else {
        bail!("id did not match any known formats")
    }
}

async fn fetch_leaderboard(
    db: &FirestoreDb,
    scenario_name: &str,
    username: &str,
) -> anyhow::Result<String> {
    let docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new("leaderboard".into())
                .with_filter(FirestoreQueryFilter::Composite(
                    FirestoreQueryFilterComposite::new(
                        vec![
                            FirestoreQueryFilter::Compare(Some(
                                FirestoreQueryFilterCompare::Equal(
                                    "scenario_name".into(),
                                    scenario_name.into(),
                                ),
                            )),
                            FirestoreQueryFilter::Compare(Some(
                                FirestoreQueryFilterCompare::Equal(
                                    "username".into(),
                                    username.into(),
                                ),
                            )),
                        ],
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
                .with_limit(1),
        )
        .await?;

    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<LeaderboardSubmission>(doc) {
            return oort_code_encryption::encrypt(&msg.code);
        }
    }

    bail!("no matching leaderboard entry found");
}

async fn fetch_tournament(
    db: &FirestoreDb,
    scenario_name: &str,
    username: &str,
) -> anyhow::Result<String> {
    let docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new("tournament".into())
                .with_filter(FirestoreQueryFilter::Composite(
                    FirestoreQueryFilterComposite::new(
                        vec![
                            FirestoreQueryFilter::Compare(Some(
                                FirestoreQueryFilterCompare::Equal(
                                    "scenario_name".into(),
                                    scenario_name.into(),
                                ),
                            )),
                            FirestoreQueryFilter::Compare(Some(
                                FirestoreQueryFilterCompare::Equal(
                                    "username".into(),
                                    username.into(),
                                ),
                            )),
                        ],
                        FirestoreQueryFilterCompositeOperator::And,
                    ),
                ))
                .with_order_by(vec![FirestoreQueryOrder::new(
                    "timestamp".to_owned(),
                    FirestoreQueryDirection::Ascending,
                )])
                .with_limit(1),
        )
        .await?;

    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<TournamentSubmission>(doc) {
            return oort_code_encryption::encrypt(&msg.code);
        }
    }

    bail!("no matching tournament entry found");
}

pub async fn get(Path(id): Path<String>) -> Result<String, Error> {
    let db = FirestoreDb::new(&project_id()).await?;
    let code = match parse_id(&id)? {
        Shortcode::Leaderboard {
            username,
            scenario_name,
        } => fetch_leaderboard(&db, &scenario_name, &username).await?,
        Shortcode::Tournament {
            username,
            scenario_name,
        } => fetch_tournament(&db, &scenario_name, &username).await?,
        Shortcode::Uploaded { docid } => {
            let obj = db
                .get_obj::<ShortcodeUpload, _>("shortcode", &docid)
                .await?;
            oort_code_encryption::encrypt(&obj.code)?
        }
    };

    Ok(code)
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

pub async fn post(Json(mut obj): Json<ShortcodeUpload>) -> Result<String, Error> {
    let db = FirestoreDb::new(&project_id()).await?;
    obj.timestamp = Utc::now();
    let docid = generate_docid();
    db.create_obj("shortcode", Some(&docid), &obj, None).await?;
    Ok(docid)
}
