use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn start_background_simulations(scenario_name: &str, code: &str, n: i32);

    pub fn display_background_simulation_results(wins: i32, total: i32);
}

pub mod telemetry {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/telemetry.js")]
    extern "C" {
        pub fn send_telemetry(data: &str);
    }
}

pub mod editor {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/editor.js")]
    extern "C" {
        pub fn display_errors(errors: JsValue);
    }
}
