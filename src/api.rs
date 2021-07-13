use crate::ui::UI;
use lazy_static::lazy_static;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

lazy_static! {
    static ref OORT_UI: Mutex<UI> = Mutex::new(UI::new());
}

#[wasm_bindgen]
pub fn render() {
    let mut ui = OORT_UI.lock().unwrap();
    ui.render();
}

#[wasm_bindgen]
pub fn upload_code(code: &str) {
    let mut ui = OORT_UI.lock().unwrap();
    ui.upload_code(code);
}
