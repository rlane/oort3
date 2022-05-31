pub mod telemetry {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/telemetry.js")]
    extern "C" {
        pub fn send_telemetry(data: &str);
    }
}
