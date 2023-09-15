use gloo_utils::format::JsValueSerdeExt;
use serde_json::json;

fn gtag(event: &str, params: &serde_json::Value) {
    let params = <wasm_bindgen::JsValue as JsValueSerdeExt>::from_serde(params).unwrap();
    log::debug!("gtag {:?} {:?}", event, js_sys::JSON::stringify(&params).unwrap());
    gtag_js_sys::gtag_with_parameters("event", event, &params);
}

pub fn discord() {
    gtag("discord", &json!({}));
}

pub fn mission_complete(scenario: &str) {
    gtag(
        "mission_complete",
        &json!({
            "scenario": scenario,
        }),
    );
}
