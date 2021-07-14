use crate::ui::UI;
use lazy_static::lazy_static;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

lazy_static! {
    static ref OORT_UI: Mutex<Option<UI>> = Mutex::new(None);
}

#[wasm_bindgen]
pub fn initialize() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("initializing logging");
}

#[wasm_bindgen]
pub fn start(scenario_name: &str) {
    let mut ui = OORT_UI.lock().unwrap();
    *ui = Some(UI::new(scenario_name));
}

#[wasm_bindgen]
pub fn render() {
    let mut ui = OORT_UI.lock().unwrap();
    if ui.is_some() {
        ui.as_mut().unwrap().render();
    }
}

#[wasm_bindgen]
pub fn upload_code(code: &str) {
    let mut ui = OORT_UI.lock().unwrap();
    if ui.is_some() {
        ui.as_mut().unwrap().upload_code(code);
    }
}
#[wasm_bindgen]
extern "C" {
    pub fn set_editor_code(code: &str);
}
