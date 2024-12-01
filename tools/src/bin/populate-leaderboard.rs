use chrono::prelude::*;
use clap::Parser;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use oort_proto::{LeaderboardSubmission, Telemetry, TelemetryMsg};
use std::collections::HashMap;

const COLLECTION_NAME: &str = "telemetry";

#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    #[clap(short, long, value_parser, default_value_t = String::from("oort-319301"))]
    project_id: String,
    scenario: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("telemetry=info"))
        .init();

    let args = Arguments::parse();
    run(&args.project_id, args.scenario).await
}

async fn run(
    project_id: &str,
    scenario: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = FirestoreDb::new(project_id).await?;

    let docs: Vec<Document> = db
        .query_doc(
            FirestoreQueryParams::new(COLLECTION_NAME.into()).with_filter(
                FirestoreQueryFilter::Composite(FirestoreQueryFilterComposite::new(
                    vec![
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
                            scenario.clone().into(),
                        ))),
                    ],
                    FirestoreQueryFilterCompositeOperator::And,
                )),
            ),
        )
        .await?;

    // userid -> (time, timestamp, docid, msg)
    let mut best_times: HashMap<String, (f64, DateTime<Utc>, String, TelemetryMsg)> =
        HashMap::new();

    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<TelemetryMsg>(doc) {
            let (_, docid) = doc.name.rsplit_once('/').unwrap();
            match &msg.payload {
                Telemetry::FinishScenario { time, .. } => {
                    let insert = if let Some((ref old_time, _, _, _)) = best_times.get(&msg.userid)
                    {
                        *old_time > time.unwrap_or_default()
                    } else {
                        true
                    };
                    if insert {
                        best_times.insert(
                            msg.userid.clone(),
                            (
                                time.unwrap_or_default(),
                                msg.timestamp,
                                docid.to_owned(),
                                msg.clone(),
                            ),
                        );
                    }
                }
                _ => unreachable!(),
            }
        } else {
            log::error!("Failed to deserialize doc {}", doc.name);
        }
    }

    let mut top: Vec<_> = best_times.into_values().collect();
    top.sort_by_key(|x| (x.0 * 1000.0) as i64);

    let mut msgs: Vec<LeaderboardSubmission> = Vec::new();

    for (_, _, _, msg) in top.iter().take(20) {
        if let Telemetry::FinishScenario {
            time,
            code,
            scenario_name,
            code_size,
            ..
        } = &msg.payload
        {
            msgs.push(LeaderboardSubmission {
                userid: msg.userid.clone(),
                username: msg.username.clone(),
                timestamp: msg.timestamp,
                scenario_name: scenario_name.clone(),
                code: code.clone(),
                code_size: *code_size,
                time: time.unwrap(),
                rescored_version: None,
            });
        }
    }

    //println!("Scenario: {}", scenario);
    //println!("{:?}", msgs);

    for msg in msgs {
        let path = format!("{}.{}", msg.scenario_name, msg.userid);
        db.create_obj("leaderboard", Some(&path), &msg, None)
            .await?;
    }

    Ok(())
}
