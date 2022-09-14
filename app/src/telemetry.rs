use super::userid::get_userid;
use crate::{js, userid::get_username};
use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TelemetryMsg {
    #[serde(flatten)]
    payload: Telemetry,
    build: String,
    userid: String,
    username: String,
}

#[derive(Serialize, Deserialize)]
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
    },
    Crash {
        msg: String,
    },
    SubmitToTournament {
        scenario_name: String,
        code: String,
    },
}

pub fn send(payload: Telemetry) {
    let userid = get_userid();
    let username = get_username();
    let msg = TelemetryMsg {
        payload,
        build: crate::version(),
        userid,
        username,
    };
    match serde_json::to_string(&msg) {
        Ok(serialized) => js::telemetry::send_telemetry(&serialized),
        Err(msg) => warn!("Failed to serialize telemetry: {}", msg),
    };
}
