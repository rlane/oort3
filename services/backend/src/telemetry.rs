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
        Telemetry::Crash { msg } => {
            discord::send_message(
                discord::Channel::Telemetry,
                format!("User {} reported crash {}: {}", obj.username, docid, msg),
            );
        }
        Telemetry::SubmitToTournament { scenario_name, .. } => {
            discord::send_message(
                discord::Channel::Telemetry,
                format!(
                    "User {} submitted AI {} to tournament scenario {}",
                    obj.username, docid, scenario_name
                ),
            );
        }
        Telemetry::Feedback { text } => {
            discord::send_message(
                discord::Channel::Telemetry,
                format!(
                    "User {} submitted feedback {}: {}",
                    obj.username, docid, text
                ),
            );
        }
        _ => {}
    }
    Ok(())
}
