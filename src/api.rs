use crate::ui::telemetry;
use crate::ui::userid;
use crate::ui::UI;
use lazy_static::lazy_static;
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
pub fn start(scenario_name: &str) {
    if has_panicked() {
        return;
    }
    let mut ui = OORT_UI.lock().unwrap();
    *ui = Some(UI::new(scenario_name));
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
pub fn upload_code(code: &str) {
    if has_panicked() {
        return;
    }
    let mut ui = OORT_UI.lock().unwrap();
    if ui.is_some() {
        ui.as_mut().unwrap().upload_code(code);
    }
}

#[wasm_bindgen]
pub fn get_initial_code() -> String {
    if has_panicked() {
        return "".to_string();
    }
    let mut ui = OORT_UI.lock().unwrap();
    if ui.is_some() {
        ui.as_mut().unwrap().get_initial_code()
    } else {
        "".to_string()
    }
}

#[wasm_bindgen]
pub fn get_username() -> String {
    userid::get_username(&userid::get_userid())
}

#[wasm_bindgen]
extern "C" {
    pub fn send_telemetry(data: &str);

    pub fn display_splash(contents: &str);

    pub fn display_mission_complete_overlay(time: f64, code_size: usize, next_scenario: &str);
}
