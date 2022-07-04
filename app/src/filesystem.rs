pub fn load(success_callback: Box<dyn FnOnce(String)>) {
    wasm_bindgen_futures::spawn_local(async move {
        match crate::js::filesystem::load_file().await {
            Ok(data) => {
                let text = data.as_string().unwrap();
                success_callback(text);
            }
            Err(e) => log::error!("load failed: {:?}", e),
        }
    });
}

pub fn reload(success_callback: Box<dyn FnOnce(String)>) {
    wasm_bindgen_futures::spawn_local(async move {
        match crate::js::filesystem::reload_file().await {
            Ok(data) => {
                let text = data.as_string().unwrap();
                success_callback(text);
            }
            Err(e) => log::error!("reload failed: {:?}", e),
        }
    });
}
