use oort_simulation_worker::SimAgent;
use yew_agent::PrivateWorker;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("initializing logging");
    SimAgent::register();
}
