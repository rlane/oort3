use crate::simulation::scenario;
use crate::ui::telemetry;
use crate::ui::userid;
use crate::ui::UI;
use lazy_static::lazy_static;
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

lazy_static! {
    static ref OORT_UI: Mutex<Option<UI>> = Mutex::new(None);
}

static PANICKED: AtomicBool = AtomicBool::new(false);

fn has_panicked() -> bool {
    PANICKED.load(Ordering::SeqCst)
}

#[wasm_bindgen]
pub fn initialize() {
    std::panic::set_hook(Box::new(|panic_info| {
        console_error_panic_hook::hook(panic_info);
        telemetry::send(telemetry::Telemetry::Crash {
            msg: panic_info.to_string(),
        });
        PANICKED.store(true, Ordering::SeqCst);
    }));
    console_log::init_with_level(log::Level::Info).expect("initializing logging");
    log::info!("Version {}", &crate::version());
}

#[wasm_bindgen]
pub fn start(scenario_name: &str, code: &str) {
    if has_panicked() {
        return;
    }
    let mut ui = OORT_UI.lock().unwrap();
    *ui = Some(UI::new(scenario_name, code));
}

#[wasm_bindgen]
pub fn render() {
    if has_panicked() {
        return;
    }
    let mut ui = OORT_UI.lock().unwrap();
    if ui.is_some() {
        ui.as_mut().unwrap().render();
    }
}

#[wasm_bindgen]
pub fn on_snapshot(value: &[u8]) {
    if has_panicked() {
        return;
    }
    let mut ui = OORT_UI.lock().unwrap();
    if ui.is_some() {
        ui.as_mut()
            .unwrap()
            .on_snapshot(bincode::deserialize(value).unwrap());
    }
}

#[wasm_bindgen]
pub fn on_key_event(e: web_sys::KeyboardEvent) {
    if has_panicked() {
        return;
    }
    let mut ui = OORT_UI.lock().unwrap();
    if ui.is_some() {
        ui.as_mut().unwrap().on_key_event(e);
    }
}

#[wasm_bindgen]
pub fn on_wheel_event(e: web_sys::WheelEvent) {
    if has_panicked() {
        return;
    }
    let mut ui = OORT_UI.lock().unwrap();
    if ui.is_some() {
        ui.as_mut().unwrap().on_wheel_event(e);
    }
}

#[wasm_bindgen]
pub fn get_initial_code(scenario_name: &str) -> String {
    if has_panicked() {
        return "".to_string();
    }
    scenario::load(scenario_name).initial_code()
}

#[wasm_bindgen]
pub fn get_solution_code(scenario_name: &str) -> String {
    if has_panicked() {
        return "".to_string();
    }
    scenario::load(scenario_name).solution()
}

#[wasm_bindgen]
pub fn get_saved_code(scenario_name: &str) -> String {
    if has_panicked() {
        return "".to_string();
    }
    let window = web_sys::window().expect("no global `window` exists");
    let storage = window
        .local_storage()
        .expect("failed to get local storage")
        .unwrap();
    let initial_code = scenario::load(scenario_name).initial_code();
    match storage.get_item(&format!("/code/{}", scenario_name)) {
        Ok(Some(code)) => code,
        Ok(None) => {
            info!("No saved code, using starter code");
            initial_code
        }
        Err(msg) => {
            error!("Failed to load code: {:?}", msg);
            initial_code
        }
    }
}

#[wasm_bindgen]
pub fn save_code(scenario_name: &str, code: &str) {
    if has_panicked() {
        return;
    }
    let window = web_sys::window().expect("no global `window` exists");
    if !code.is_empty() {
        let storage = window
            .local_storage()
            .expect("failed to get local storage")
            .unwrap();
        if let Err(msg) = storage.set_item(&format!("/code/{}", scenario_name), code) {
            error!("Failed to save code: {:?}", msg);
        }
    }
}

#[wasm_bindgen]
pub fn get_userid() -> String {
    userid::get_userid()
}

#[wasm_bindgen]
pub fn get_username(userid: &str) -> String {
    userid::get_username(userid)
}

#[wasm_bindgen]
pub fn get_scenarios() -> JsValue {
    JsValue::from_serde(&scenario::list()).unwrap()
}

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

    pub fn request_snapshot(nonce: u64);
}
