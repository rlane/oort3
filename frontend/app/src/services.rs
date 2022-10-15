use crate::userid;
use chrono::Utc;
use oort_proto::LeaderboardSubmission;
use oort_proto::{Telemetry, TelemetryMsg};
use reqwasm::http::Request;

pub fn is_local() -> bool {
    gloo_utils::document()
        .location()
        .unwrap()
        .hostname()
        .unwrap()
        == "localhost"
}

pub fn compiler_vm_url() -> String {
    if is_local() {
        log::info!("Using compiler service on localhost");
        "http://localhost:8081".to_owned()
    } else {
        "https://compiler-vm.oort.rs".to_owned()
    }
}

pub fn compiler_url() -> String {
    if is_local() {
        log::info!("Using compiler service on localhost");
        "http://localhost:8081".to_owned()
    } else {
        "https://compiler.oort.rs".to_owned()
    }
}

pub fn telemetry_url() -> String {
    if is_local() {
        log::info!("Using telemetry service on localhost");
        "http://localhost:8082".to_owned()
    } else {
        "https://telemetry.oort.rs".to_owned()
    }
}

pub fn leaderboard_url() -> String {
    if is_local() {
        log::info!("Using leaderboard service on localhost");
        "http://localhost:8083".to_owned()
    } else {
        "https://leaderboard.oort.rs".to_owned()
    }
}

pub fn post_leaderboard(msg: LeaderboardSubmission) {
    wasm_bindgen_futures::spawn_local(async move {
        let url = format!("{}/leaderboard", leaderboard_url());
        let body = serde_json::to_string(&msg).unwrap();
        let result = Request::post(&url).body(body).send().await;
        if let Err(e) = result {
            log::warn!("Error posting to leaderboard: {:?}", e);
        }
        // TODO refresh displayed leaderboard
    });
}

pub fn send_telemetry(payload: Telemetry) {
    let userid = userid::get_userid();
    let username = userid::get_username();
    let msg = TelemetryMsg {
        timestamp: Utc::now(),
        payload,
        build: crate::version(),
        userid,
        username,
    };
    wasm_bindgen_futures::spawn_local(async move {
        let url = format!("{}/post", telemetry_url());
        let body = serde_json::to_string(&msg).unwrap();
        log::info!("Sending telemetry: {}", body);
        let result = Request::post(&url).body(body).send().await;
        if let Err(e) = result {
            log::warn!("Error posting telemetry: {:?}", e);
        }
    });
}
