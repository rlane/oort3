use super::userid::get_userid;
use crate::api;
use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TelemetryMsg {
    #[serde(flatten)]
    payload: Telemetry,
    build: String,
    userid: String,
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
}

pub fn send(payload: Telemetry) {
    let msg = TelemetryMsg {
        payload,
        build: crate::version(),
        userid: get_userid(),
    };
    match serde_json::to_string(&msg) {
        Ok(serialized) => api::send_telemetry(&serialized),
        Err(msg) => warn!("Failed to serialize telemetry: {}", msg),
    };
}
