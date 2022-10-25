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

pub mod golden_layout {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/golden_layout.js")]
    extern "C" {
        pub fn init();
        pub fn update_size();
    }
}

pub mod completion {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/completion.js")]
    extern "C" {
        pub fn init();
    }
}
