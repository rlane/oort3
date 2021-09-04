use oort_app::sim_agent::SimAgent;
use yew::agent::Threaded;

fn main() {
    yew::initialize();
    SimAgent::register();
}
