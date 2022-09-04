use serde::{Deserialize, Serialize};
use std::default::Default;
use std::sync::Arc;
use yew_agent::{Agent, AgentLink, HandlerId, Private};

use cfg::CfgOptions;
use ide::{
    AnalysisHost, Change, CrateGraph, CrateId, DiagnosticsConfig, Edition, FileId, LineIndex,
    SourceRoot,
};
use ide_db::base_db::{CrateName, CrateOrigin, Dependency, Env, FileSet, VfsPath};

#[derive(Serialize, Deserialize, Debug)]
pub struct Diagnostic {
    pub message: String,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Analyze(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    AnalyzeFinished(Vec<Diagnostic>),
}

pub fn create_source_root(name: &str, f: FileId) -> SourceRoot {
    let mut file_set = FileSet::default();
    file_set.insert(
        f,
        VfsPath::new_virtual_path(format!("/{}/src/lib.rs", name)),
    );
    SourceRoot::new_library(file_set)
}

pub fn create_crate(crate_graph: &mut CrateGraph, f: FileId) -> CrateId {
    let mut cfg = CfgOptions::default();
    cfg.insert_atom("unix".into());
    cfg.insert_key_value("target_arch".into(), "x86_64".into());
    cfg.insert_key_value("target_pointer_width".into(), "64".into());
    crate_graph.add_crate_root(
        f,
        Edition::Edition2018,
        None,
        None,
        cfg,
        Default::default(),
        Env::default(),
        Ok(Vec::new()),
        false,
        CrateOrigin::CratesIo { repo: None },
    )
}

pub struct AnalyzerAgent {
    link: AgentLink<AnalyzerAgent>,
    analysis_host: ide::AnalysisHost,
}

impl Agent for AnalyzerAgent {
    type Reach = Private<Self>;
    type Message = ();
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let text = "".to_string();
        let fake_std = "".to_string();
        let fake_core = "".to_string();
        let fake_alloc = "".to_string();
        let fake_oort_api = "pub mod prelude {}".to_string();

        let mut host = AnalysisHost::default();
        let file_id = FileId(0);
        let std_id = FileId(1);
        let core_id = FileId(2);
        let alloc_id = FileId(3);
        let oort_api_id = FileId(4);

        let mut file_set = FileSet::default();
        file_set.insert(
            file_id,
            VfsPath::new_virtual_path("/my_crate/main.rs".to_string()),
        );
        let source_root = SourceRoot::new_local(file_set);

        let mut change = Change::new();
        change.set_roots(vec![
            source_root,
            create_source_root("std", std_id),
            create_source_root("core", core_id),
            create_source_root("alloc", alloc_id),
            create_source_root("oort_api", oort_api_id),
        ]);
        let mut crate_graph = CrateGraph::default();
        let my_crate = create_crate(&mut crate_graph, file_id);
        let std_crate = create_crate(&mut crate_graph, std_id);
        let core_crate = create_crate(&mut crate_graph, core_id);
        let alloc_crate = create_crate(&mut crate_graph, alloc_id);
        let oort_api_crate = create_crate(&mut crate_graph, oort_api_id);
        let core_dep = Dependency::new(CrateName::new("core").unwrap(), core_crate);
        let alloc_dep = Dependency::new(CrateName::new("alloc").unwrap(), alloc_crate);
        let std_dep = Dependency::new(CrateName::new("std").unwrap(), std_crate);
        let oort_api_dep = Dependency::new(CrateName::new("oort_api").unwrap(), oort_api_crate);

        crate_graph.add_dep(std_crate, core_dep.clone()).unwrap();
        crate_graph.add_dep(std_crate, alloc_dep.clone()).unwrap();
        crate_graph.add_dep(alloc_crate, core_dep.clone()).unwrap();

        crate_graph.add_dep(my_crate, core_dep).unwrap();
        crate_graph.add_dep(my_crate, alloc_dep).unwrap();
        crate_graph.add_dep(my_crate, std_dep).unwrap();
        crate_graph.add_dep(my_crate, oort_api_dep).unwrap();

        change.change_file(file_id, Some(Arc::new(text)));
        change.change_file(std_id, Some(Arc::new(fake_std)));
        change.change_file(core_id, Some(Arc::new(fake_core)));
        change.change_file(alloc_id, Some(Arc::new(fake_alloc)));
        change.change_file(oort_api_id, Some(Arc::new(fake_oort_api)));
        change.set_crate_graph(crate_graph);
        host.apply_change(change);

        Self {
            link,
            analysis_host: host,
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, request: Self::Input, who: HandlerId) {
        log::info!("AnalyzerAgent got message: {:?}", request);
        match request {
            Request::Analyze(text) => {
                let file_id = ide::FileId(0);
                let mut change = ide::Change::new();
                change.change_file(file_id, Some(Arc::new(text)));
                self.analysis_host.apply_change(change);
                let analysis = self.analysis_host.analysis();
                let line_index = analysis.file_line_index(file_id).unwrap();
                let diagnostics = analysis.diagnostics(
                    &DiagnosticsConfig::test_sample(),
                    ide::AssistResolveStrategy::None,
                    ide::FileId(0),
                );
                match diagnostics {
                    Ok(diags) => {
                        log::info!("Diagnostics: {:?}", diags);
                        let diags = diags
                            .iter()
                            .map(|diag| translate_diagnostic(diag, line_index.as_ref()))
                            .collect();
                        log::info!("Translated diagnostics: {:?}", diags);
                        self.link.respond(who, Response::AnalyzeFinished(diags));
                    }
                    Err(e) => {
                        log::error!("Error getting diagnostics: {:?}", e);
                    }
                }
            }
        }
    }

    fn name_of_resource() -> &'static str {
        "oort_analyzer.js"
    }
}

fn translate_diagnostic(diag: &ide::Diagnostic, line_index: &LineIndex) -> Diagnostic {
    let start = line_index.line_col(diag.range.start());
    let end = line_index.line_col(diag.range.end());
    Diagnostic {
        message: diag.message.clone(),
        start_line: start.line,
        start_column: start.col,
        end_line: end.line,
        end_column: end.col,
    }
}
