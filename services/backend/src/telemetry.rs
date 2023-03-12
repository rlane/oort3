use crate::{discord, project_id};
use chrono::prelude::*;
use firestore::*;
use oort_proto::{Telemetry, TelemetryMsg};
use salvo::prelude::*;

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

async fn post_internal(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    let db = FirestoreDb::new(&project_id()).await?;
    let payload = req.payload().await?;
    let mut obj: TelemetryMsg = serde_json::from_slice(payload)?;
    obj.timestamp = Utc::now();
    log::debug!("Got request obj {:?}", obj);
    let docid = generate_docid();
    db.create_obj("telemetry", &docid, &obj).await?;
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
    res.render("");
    Ok(())
}

#[handler]
pub async fn post_telemetry(req: &mut Request, res: &mut Response) {
    if let Err(e) = post_internal(req, res).await {
        log::error!("error: {}", e);
        res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(e.to_string());
    }
}
