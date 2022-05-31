use log::{error, info};
use oort_simulator::scenario;

pub fn load(scenario_name: &str) -> String {
    let window = web_sys::window().expect("no global `window` exists");
    let storage = window
        .local_storage()
        .expect("failed to get local storage")
        .unwrap();
    let initial_code = scenario::load(scenario_name).initial_code();
    match storage.get_item(&format!("/code/{}", scenario_name)) {
        Ok(Some(code)) => code,
        Ok(None) => {
            info!("No saved code, using starter code");
            initial_code
        }
        Err(msg) => {
            error!("Failed to load code: {:?}", msg);
            initial_code
        }
    }
}

pub fn save(scenario_name: &str, code: &str) {
    let window = web_sys::window().expect("no global `window` exists");
    if !code.is_empty() {
        let storage = window
            .local_storage()
            .expect("failed to get local storage")
            .unwrap();
        if let Err(msg) = storage.set_item(&format!("/code/{}", scenario_name), code) {
            error!("Failed to save code: {:?}", msg);
        }
    }
}
