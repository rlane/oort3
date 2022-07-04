pub mod telemetry {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/telemetry.js")]
    extern "C" {
        pub fn send_telemetry(data: &str);
    }
}

pub mod filesystem {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/filesystem.js")]
    extern "C" {
        #[wasm_bindgen(catch)]
        pub async fn load_file() -> Result<JsValue, JsValue>;

        #[wasm_bindgen(catch)]
        pub async fn reload_file() -> Result<JsValue, JsValue>;
    }
}
