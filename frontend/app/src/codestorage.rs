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
    let scenario = scenario::load(scenario_name);
    let mut result = scenario.initial_code();
    let mut names = vec![];
    names.push(scenario_name.to_string());
    names.append(&mut scenario.previous_names());
    let player_code = names
        .iter()
        .find_map(|name| storage.get_item(&format!("/code/{name}")).unwrap());
    match player_code {
        Some(code) => result[0] = str_to_code(&code),
        None => info!("No saved code, using starter code"),
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

    let scenario_name = scenario_name.to_string();
    let code = code.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let version_control = oort_version_control::VersionControl::new().await.unwrap();
        let version = oort_version_control::CreateVersionParams {
            code: code_to_string(&code),
            scenario_name,
            label: None,
        };
        version_control.create_version(&version).await.unwrap();
    });
}
