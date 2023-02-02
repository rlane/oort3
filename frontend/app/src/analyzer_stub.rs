pub use oort_proto::analyzer::*;
use yew_agent::{HandlerId, Private, WorkerLink};

pub struct AnalyzerAgent {}

impl yew_agent::Worker for AnalyzerAgent {
    type Reach = Private<Self>;
    type Message = ();
    type Input = Request;
    type Output = Response;

    fn create(_link: WorkerLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, request: Self::Input, _who: HandlerId) {
        log::error!("AnalyzerAgent stub got message: {:?}", request);
    }

    fn name_of_resource() -> &'static str {
        "oort_analyzer_worker.js"
    }
}
