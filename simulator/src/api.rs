use crate::worker_api::WorkerRequest;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn send_telemetry(data: &str);

    pub fn display_splash(contents: &str);

    pub fn display_mission_complete_overlay(
        scenario_name: &str,
        time: f64,
        code_size: usize,
        next_scenario: &str,
    );

    pub fn display_errors(errors: JsValue);

    pub fn start_background_simulations(scenario_name: &str, code: &str, n: i32);

    pub fn display_background_simulation_results(wins: i32, total: i32);

    #[wasm_bindgen(js_name = "send_worker_request")]
    pub fn js_send_worker_request(request: JsValue);
}

pub fn send_worker_request(request: &WorkerRequest) {
    js_send_worker_request(
        serde_wasm_bindgen::to_value(&request).expect("serializing worker request"),
    );
}
