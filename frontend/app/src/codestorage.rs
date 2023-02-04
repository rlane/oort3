use super::game::{code_to_string, str_to_code};
use log::{error, info};
use oort_simulator::scenario;
use oort_simulator::simulation::Code;

pub fn load(scenario_name: &str) -> Vec<Code> {
    let window = web_sys::window().expect("no global `window` exists");
    let storage = window
        .local_storage()
        .expect("failed to get local storage")
        .unwrap();
    let mut result = scenario::load(scenario_name).initial_code();
    match storage.get_item(&format!("/code/{scenario_name}")) {
        Ok(Some(code)) => result[0] = str_to_code(&code),
        Ok(None) => {
            info!("No saved code, using starter code");
        }
        Err(msg) => {
            error!("Failed to load code: {:?}", msg);
        }
    }
    result
}

pub fn save(scenario_name: &str, code: &Code) {
    let window = web_sys::window().expect("no global `window` exists");
    let storage = window
        .local_storage()
        .expect("failed to get local storage")
        .unwrap();
    if let Err(msg) = storage.set_item(&format!("/code/{scenario_name}"), &code_to_string(code)) {
        error!("Failed to save code: {:?}", msg);
    }
}
