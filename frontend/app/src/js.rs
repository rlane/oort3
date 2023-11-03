pub mod filesystem {
    use wasm_bindgen::prelude::*;
    use web_sys::FileSystemFileEntry;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct DirectoryValidateResponseEntry {
        pub name: String,
        #[serde(rename = "lastModified")]
        pub last_modified: u64,
        pub contents: String,
    }

    impl std::fmt::Debug for DirectoryValidateResponseEntry {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("DirectoryValidateResponseEntry")
                .field("name", &self.name)
                .field("last_modified", &self.last_modified)
                .field("contents", &format!("<{} bytes>", &self.contents.len()))
                .finish()
        }
    }

    #[wasm_bindgen(module = "/js/filesystem.js")]
    extern "C" {
        #[wasm_bindgen]
        #[derive(Debug, Clone)]
        pub type FileHandle;

        #[wasm_bindgen(constructor)]
        pub fn new(handle: FileSystemFileEntry) -> FileHandle;

        #[wasm_bindgen]
        #[derive(Debug, Clone)]
        pub type DirectoryHandle;

        #[wasm_bindgen(method, catch)]
        pub async fn load_files(this: &DirectoryHandle) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub async fn read(this: &FileHandle) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(catch)]
        pub async fn open() -> Result<JsValue, JsValue>;

        #[wasm_bindgen(catch)]
        pub async fn open_directory() -> Result<JsValue, JsValue>;

        #[wasm_bindgen(catch)]
        pub async fn pick(editor: JsValue, items: Vec<JsValue>) -> Result<JsValue, JsValue>;
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
