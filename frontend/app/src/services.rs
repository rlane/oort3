use crate::userid;
use anyhow::anyhow;
use chrono::Utc;
use oort_proto::{LeaderboardData, LeaderboardSubmission};
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

pub fn shortcode_url() -> String {
    if is_local() {
        log::info!("Using shortcode service on localhost");
        "http://localhost:8084".to_owned()
    } else {
        "https://shortcode.oort.rs".to_owned()
    }
}

pub fn get_leaderboard(
    scenario_name: &str,
    callback: yew::Callback<Result<LeaderboardData, reqwasm::Error>>,
) {
    let url = format!(
        "{}/leaderboard?scenario_name={}",
        leaderboard_url(),
        scenario_name
    );
    wasm_bindgen_futures::spawn_local(async move {
        let result = Request::get(&url).send().await;
        match result {
            Err(e) => {
                log::warn!("Error getting leaderboard: {:?}", e);
                callback.emit(Err(e));
            }
            Ok(response) => {
                let data: Result<LeaderboardData, _> = response.json().await;
                callback.emit(data);
            }
        }
    });
}

pub fn post_leaderboard(
    msg: LeaderboardSubmission,
    callback: yew::Callback<Result<LeaderboardData, reqwasm::Error>>,
) {
    wasm_bindgen_futures::spawn_local(async move {
        let url = format!("{}/leaderboard", leaderboard_url());
        let body = oort_envelope::add(&serde_json::to_vec(&msg).unwrap());
        let jsdata = js_sys::Uint8Array::new_with_length(body.len() as u32);
        jsdata.copy_from(&body);
        let result = Request::post(&url).body(jsdata).send().await;
        match result {
            Err(e) => {
                log::warn!("Error posting to leaderboard: {:?}", e);
                callback.emit(Err(e));
            }
            Ok(response) => {
                let data: Result<LeaderboardData, _> = response.json().await;
                callback.emit(data);
            }
        }
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

pub fn format(text: String, cb: yew::Callback<String>) {
    wasm_bindgen_futures::spawn_local(async move {
        let url = format!("{}/format", compiler_url());
        let result = Request::post(&url).body(text).send().await;
        match result {
            Ok(response) => {
                if response.ok() {
                    cb.emit(response.text().await.unwrap());
                } else {
                    log::warn!(
                        "Error formatting code: {:?}",
                        response.text().await.unwrap()
                    );
                }
            }
            Err(e) => {
                log::warn!("Error formatting code: {:?}", e);
            }
        }
    });
}

pub async fn get_shortcode(shortcode: &str) -> anyhow::Result<String> {
    let response =
        reqwasm::http::Request::get(&format!("{}/shortcode/{}", shortcode_url(), shortcode))
            .send()
            .await?;
    let text = response.text().await?;
    if response.ok() {
        Ok(text)
    } else {
        Err(anyhow!(text))
    }
}
