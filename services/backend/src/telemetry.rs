use crate::{discord, project_id, Error};
use axum::extract::Json;
use chrono::prelude::*;
use firestore::*;
use oort_proto::{Telemetry, TelemetryMsg};

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

pub async fn post(Json(mut obj): Json<TelemetryMsg>) -> Result<(), Error> {
    let db = FirestoreDb::new(&project_id()).await?;
    obj.timestamp = Utc::now();
    log::debug!("Got request obj {:?}", obj);
    let docid = generate_docid();
    db.create_obj("telemetry", Some(&docid), &obj, None).await?;
    match obj.payload {
        Telemetry::StartScenario { scenario_name, .. } => {
            log::info!("User {} started scenario {}", obj.username, scenario_name);
        }
        Telemetry::FinishScenario {
            scenario_name,
            success,
            time,
            ..
        } => {
            if success {
                log::info!(
                    "User {} completed scenario {} in {}s",
                    obj.username,
                    scenario_name,
                    time.unwrap_or_default(),
                );
            } else {
                log::info!("User {} failed scenario {}", obj.username, scenario_name);
            }
        }
        Telemetry::Crash { msg } => {
            log::info!("User {} reported crash {}: {}", obj.username, docid, msg);
            discord::send_message(
                discord::Channel::Telemetry,
                format!("User {} reported crash {}: {}", obj.username, docid, msg),
            );
        }
        Telemetry::SubmitToTournament { scenario_name, .. } => {
            log::info!(
                "User {} submitted AI {} to tournament scenario {}",
                obj.username,
                docid,
                scenario_name
            );
            discord::send_message(
                discord::Channel::Telemetry,
                format!(
                    "User {} submitted AI {} to tournament scenario {}",
                    obj.username, docid, scenario_name
                ),
            );
        }
        Telemetry::Feedback { text } => {
            log::info!(
                "User {} submitted feedback {}: {}",
                obj.username,
                docid,
                text
            );
            discord::send_message(
                discord::Channel::Telemetry,
                format!(
                    "User {} submitted feedback {}: {}",
                    obj.username, docid, text
                ),
            );
        }
    }
    Ok(())
}
