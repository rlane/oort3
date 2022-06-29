use oort_worker::SimAgent;
use yew_agent::PrivateAgent;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("initializing logging");
    log::info!("starting worker");
    SimAgent::register();
}
