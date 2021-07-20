use crate::api;
use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct StartScenario {
    scenario_name: String,
    code: String,
}

pub fn send_start_scenario(scenario_name: &str, code: &str) {
    let msg = StartScenario {
        scenario_name: scenario_name.into(),
        code: code.into(),
    };
    match serde_json::to_string(&msg) {
        Ok(payload) => api::send_telemetry(&payload),
        Err(msg) => warn!("Failed to serialize telemetry: {}", msg),
    };
}
