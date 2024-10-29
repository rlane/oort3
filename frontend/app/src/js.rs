pub mod filesystem {
    use wasm_bindgen::prelude::*;
    use web_sys::FileSystemFileEntry;

    #[wasm_bindgen(module = "/js/filesystem.js")]
    extern "C" {
        #[wasm_bindgen]
        #[derive(Debug, Clone)]
        pub type FileHandle;

        #[wasm_bindgen(constructor)]
        pub fn new(handle: FileSystemFileEntry) -> FileHandle;

        #[wasm_bindgen(method)]
        pub async fn name(this: &FileHandle) -> JsValue;

        #[wasm_bindgen(method, catch)]
        pub async fn read(this: &FileHandle) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(catch)]
        pub async fn open() -> Result<JsValue, JsValue>;

        #[wasm_bindgen]
        #[derive(Debug, Clone)]
        pub type DirectoryHandle;

        #[wasm_bindgen(method)]
        pub async fn getFiles(this: &DirectoryHandle) -> JsValue;

        #[wasm_bindgen(catch)]
        pub async fn openDirectory() -> Result<JsValue, JsValue>;
    }
}

pub mod golden_layout {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/golden_layout.js")]
    extern "C" {
        pub fn init();
        pub fn update_size();
        pub fn show_welcome(visible: bool);
        pub fn select_tab(id: &str);
    }
}

pub mod completion {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/completion.js")]
    extern "C" {
        pub fn init();
    }
}

pub mod clipboard {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/clipboard.js")]
    extern "C" {
        pub fn write(text: &str);
    }
}

pub mod resize {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/js/resize.js")]
    extern "C" {
        pub fn start(closure: &Closure<dyn FnMut()>);
    }
}
