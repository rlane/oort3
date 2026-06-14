pub use oort_proto::analyzer::*;
use yew_agent::{HandlerId, Private, WorkerLink};

pub mod noop_backend;

pub trait CodeAnalyzer {
    fn update_file(&mut self, text: String) -> Vec<Diagnostic>;
    fn completions(&self, line: u32, col: u32) -> Vec<CompletionItem>;
}

pub struct AnalyzerAgent {
    link: WorkerLink<AnalyzerAgent>,
    backend: Box<dyn CodeAnalyzer>,
}

impl yew_agent::Worker for AnalyzerAgent {
    type Reach = Private<Self>;
    type Message = ();
    type Input = Request;
    type Output = Response;

    fn create(link: WorkerLink<Self>) -> Self {
        let backend = noop_backend::NoopBackend::new();
        Self {
            link,
            backend: Box::new(backend),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, request: Self::Input, who: HandlerId) {
        log::info!("AnalyzerAgent got message: {:?}", request);
        let response = match request {
            Request::Diagnostics(text) => {
                let diags = self.backend.update_file(text);
                Some(Response::Diagnostics(diags))
            }
            Request::Completion(line, col) => {
                let completions = self.backend.completions(line, col);
                Some(Response::Completion(completions))
            }
        };
        if let Some(msg) = response {
            self.link.respond(who, msg);
        }
    }

    fn name_of_resource() -> &'static str {
        "oort_analyzer_worker.js"
    }
}
