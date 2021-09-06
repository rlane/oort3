use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/editor.js")]
extern "C" {
    pub fn initialize(
        editor_div: web_sys::HtmlElement,
        action_callback: &Closure<dyn FnMut(String)>,
    );
    pub fn display_errors(errors: JsValue);
    pub fn get_text() -> String;
    pub fn set_text(code: &str);
}
