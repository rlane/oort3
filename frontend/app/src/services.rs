pub fn is_local() -> bool {
    gloo_utils::document()
        .location()
        .unwrap()
        .hostname()
        .unwrap()
        == "localhost"
}

pub fn compiler_vm_url() -> String {
    if is_local() {
        log::info!("Using compiler service on localhost");
        "http://localhost:8081".to_owned()
    } else {
        "https://compiler-vm.oort.rs".to_owned()
    }
}

pub fn compiler_url() -> String {
    if is_local() {
        log::info!("Using compiler service on localhost");
        "http://localhost:8081".to_owned()
    } else {
        "https://compiler.oort.rs".to_owned()
    }
}

pub fn telemetry_url() -> String {
    if is_local() {
        log::info!("Using telemetry service on localhost");
        "http://localhost:8082".to_owned()
    } else {
        "https://telemetry.oort.rs".to_owned()
    }
}

pub fn leaderboard_url() -> String {
    if is_local() {
        log::info!("Using leaderboard service on localhost");
        "http://localhost:8083".to_owned()
    } else {
        "https://leaderboard.oort.rs".to_owned()
    }
}
