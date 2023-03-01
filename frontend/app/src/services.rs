use crate::userid;
use anyhow::anyhow;
use chrono::Utc;
use oort_proto::{LeaderboardData, LeaderboardSubmission};
use oort_proto::{ShortcodeUpload, TournamentSubmission};
use oort_proto::{Telemetry, TelemetryMsg};
use reqwasm::http::{Request, Response};

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

async fn send_request(request: Request) -> anyhow::Result<Response> {
    match request.send().await {
        Ok(response) if response.ok() => Ok(response),
        Ok(response) => Err(anyhow!(
            "Request to {} failed with status {}: {}",
            response.url(),
            response.status(),
            response.text().await.unwrap_or_else(|e| format!("{e:?}"))
        )),
        Err(e) => Err(anyhow!("Request failed: {:?}", e)),
    }
}

pub fn get_leaderboard(
    scenario_name: &str,
    callback: yew::Callback<anyhow::Result<LeaderboardData>>,
) {
    let url = format!(
        "{}/leaderboard?scenario_name={}",
        leaderboard_url(),
        scenario_name
    );
    wasm_bindgen_futures::spawn_local(async move {
        match send_request(Request::get(&url)).await {
            Err(e) => {
                callback.emit(Err(e));
            }
            Ok(response) => {
                let data: Result<LeaderboardData, anyhow::Error> =
                    response.json().await.map_err(|e| e.into());
                callback.emit(data);
            }
        }
    });
}

pub fn post_leaderboard(
    msg: LeaderboardSubmission,
    callback: yew::Callback<Result<LeaderboardData, anyhow::Error>>,
) {
    wasm_bindgen_futures::spawn_local(async move {
        let url = format!("{}/leaderboard", leaderboard_url());
        let body = oort_envelope::add(&serde_json::to_vec(&msg).unwrap());
        let jsdata = js_sys::Uint8Array::new_with_length(body.len() as u32);
        jsdata.copy_from(&body);
        let result = send_request(Request::post(&url).body(jsdata)).await;
        match result {
            Err(e) => {
                log::warn!("Error posting to leaderboard: {:?}", e);
                callback.emit(Err(e));
            }
            Ok(response) => {
                let data: Result<LeaderboardData, _> = response.json().await.map_err(|e| e.into());
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
        let result = send_request(Request::post(&url).body(body)).await;
        if let Err(e) = result {
            log::warn!("Error posting telemetry: {:?}", e);
        }
    });
}

pub fn format(text: String, cb: yew::Callback<String>) {
    wasm_bindgen_futures::spawn_local(async move {
        let url = format!("{}/format", compiler_url());
        let result = send_request(Request::post(&url).body(text)).await;
        match result {
            Ok(response) => {
                cb.emit(response.text().await.unwrap());
            }
            Err(e) => {
                log::warn!("Error formatting code: {:?}", e);
            }
        }
    });
}

pub async fn get_shortcode(shortcode: &str) -> anyhow::Result<String> {
    let response = send_request(Request::get(&format!(
        "{}/shortcode/{}",
        shortcode_url(),
        shortcode
    )))
    .await?;
    response.text().await.map_err(|e| e.into())
}

pub async fn upload_shortcode(code: &str) -> anyhow::Result<String> {
    let userid = userid::get_userid();
    let username = userid::get_username();
    let msg = ShortcodeUpload {
        userid,
        username,
        timestamp: Utc::now(),
        code: code.to_string(),
    };
    let body = serde_json::to_string(&msg).unwrap();
    let response =
        send_request(Request::post(&format!("{}/shortcode", shortcode_url())).body(body)).await?;
    response.text().await.map_err(|e| e.into())
}

pub async fn submit_to_tournament(scenario_name: &str, code: &str) -> anyhow::Result<()> {
    let userid = userid::get_userid();
    let username = userid::get_username();
    let msg = TournamentSubmission {
        userid,
        username,
        timestamp: Utc::now(),
        scenario_name: scenario_name.to_string(),
        code: code.to_string(),
    };
    let body = serde_json::to_string(&msg).unwrap();
    send_request(Request::post(&format!("{}/tournament", shortcode_url())).body(body)).await?;
    Ok(())
}
