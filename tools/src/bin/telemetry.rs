use chrono::prelude::*;
use firestore::*;
use gcloud_sdk::google::firestore::v1::Document;
use serde::{Deserialize, Serialize};

pub fn config_env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

#[derive(Serialize, Deserialize, Debug)]
struct TelemetryMsg {
    #[serde(flatten)]
    payload: Telemetry,
    build: String,
    userid: String,
    username: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Telemetry {
    StartScenario {
        scenario_name: String,
        code: String,
    },
    FinishScenario {
        scenario_name: String,
        code: String,
        ticks: u32,
        code_size: usize,
        success: Option<bool>,
    },
    Crash {
        msg: String,
    },
    SubmitToTournament {
        scenario_name: String,
        code: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("telemetry=info"))
        .init();

    let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;

    const COLLECTION_NAME: &'static str = "telemetry";

    let docs: Vec<Document> = db
        .query_doc(FirestoreQueryParams::new(COLLECTION_NAME.into()).clone())
        .await?;
    for doc in &docs {
        if let Ok(msg) = FirestoreDb::deserialize_doc_to::<TelemetryMsg>(doc) {
            let (_, docid) = doc.name.rsplit_once("/").unwrap();
            let user = msg.username.as_ref().unwrap_or(&msg.userid);
            let epoch_time = doc.create_time.clone().map(|x| x.seconds).unwrap_or(0);
            let time = Local.timestamp(epoch_time, 0);
            let prefix = format!("{docid} {}", time.format("%Y-%m-%d %H:%M:%S"));
            match &msg.payload {
                Telemetry::StartScenario { scenario_name, .. } => {
                    println!("{prefix} StartScenario user={user} scenario={scenario_name}")
                }
                Telemetry::FinishScenario {
                    scenario_name,
                    success,
                    ticks,
                    ..
                } => {
                    let ticks = if success.unwrap_or(false) {
                        ticks.to_string()
                    } else {
                        "failed".to_string()
                    };
                    println!(
                        "{prefix} FinishScenario user={user} scenario={scenario_name} ticks={ticks}"
                    );
                }
                Telemetry::Crash { .. } => println!("{prefix} Crash user={user}"),
                Telemetry::SubmitToTournament { scenario_name, .. } => {
                    println!("{prefix} SubmitToTournament user={user} scenario={scenario_name}")
                }
            }
        } else {
            log::error!("Failed to deserialize doc {}", doc.name);
        }
    }

    Ok(())
}
