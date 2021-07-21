use crate::api;
use log::warn;
use serde::{Deserialize, Serialize};

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
        ticks: u64,
        code_size: usize,
    },
    Crash {
        msg: String,
    },
}

pub fn send(msg: Telemetry) {
    match serde_json::to_string(&msg) {
        Ok(payload) => api::send_telemetry(&payload),
        Err(msg) => warn!("Failed to serialize telemetry: {}", msg),
    };
}
