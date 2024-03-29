use ide_db::SnippetCap;
pub use oort_proto::analyzer::*;
use std::default::Default;
use triomphe::Arc;
use yew_agent::{HandlerId, Private, WorkerLink};

use cfg::CfgOptions;
use ide::{
    AnalysisHost, Change, CompletionConfig, CrateGraph, CrateId, DiagnosticsConfig, Edition,
    FileId, LineIndex, SourceRoot, TextSize,
};
use ide_db::base_db::{
    CrateDisplayName, CrateName, CrateOrigin, Dependency, Env, FileSet, LangCrateOrigin, VfsPath,
};
use ide_db::imports::insert_use::{ImportGranularity, InsertUseConfig, PrefixKind};

pub fn create_source_root(name: &str, f: FileId) -> SourceRoot {
    let mut file_set = FileSet::default();
    file_set.insert(f, VfsPath::new_virtual_path(format!("/{name}/src/lib.rs")));
    SourceRoot::new_library(file_set)
}

pub fn create_crate(
    crate_graph: &mut CrateGraph,
    name: &str,
    f: FileId,
    origin: CrateOrigin,
) -> CrateId {
    let mut cfg = CfgOptions::default();
    cfg.insert_atom("unix".into());
    cfg.insert_key_value("target_arch".into(), "x86_64".into());
    cfg.insert_key_value("target_pointer_width".into(), "64".into());
    crate_graph.add_crate_root(
        f,
        Edition::Edition2021,
        Some(CrateDisplayName::from_canonical_name(name.to_string())),
        None,
        cfg,
        Default::default(),
        Env::default(),
        false,
        origin,
        Err("no target layout".into()),
        None,
    )
}

pub struct AnalyzerAgent {
    link: WorkerLink<AnalyzerAgent>,
    analysis_host: ide::AnalysisHost,
}

impl yew_agent::Worker for AnalyzerAgent {
    type Reach = Private<Self>;
    type Message = ();
    type Input = Request;
    type Output = Response;

    fn create(link: WorkerLink<Self>) -> Self {
        let oort_api_src = include_str!("../../../shared/api/src/lib.rs");
        let vec_src = include_str!("../../../shared/api/src/vec.rs");
        let fake_core_src = include_str!("../stdlib/mini_core.rs");
        let fake_std_src = include_str!("../stdlib/mini_std.rs");

        let mut host = AnalysisHost::default();
        let file_id = FileId(0);
        let std_id = FileId(1);
        let core_id = FileId(2);
        let alloc_id = FileId(3);
        let oort_api_id = FileId(4);
        let vec_id = FileId(5);

        let mut file_set = FileSet::default();
        file_set.insert(
            file_id,
            VfsPath::new_virtual_path("/my_crate/main.rs".to_string()),
        );
        let source_root = SourceRoot::new_local(file_set);

        let local_origin = CrateOrigin::Local {
            repo: None,
            name: None,
        };

        let oort_api_source_root = {
            let mut file_set = FileSet::default();
            file_set.insert(
                oort_api_id,
                VfsPath::new_virtual_path("/oort_api/src/lib.rs".into()),
            );
            file_set.insert(
                vec_id,
                VfsPath::new_virtual_path("/oort_api/src/vec.rs".into()),
            );
            SourceRoot::new_library(file_set)
        };

        let mut change = Change::new();
        change.set_roots(vec![
            source_root,
            create_source_root("std", std_id),
            create_source_root("core", core_id),
            create_source_root("alloc", alloc_id),
            oort_api_source_root,
        ]);
        let mut crate_graph = CrateGraph::default();
        let my_crate = create_crate(&mut crate_graph, "user", file_id, local_origin.clone());
        let std_crate = create_crate(
            &mut crate_graph,
            "std",
            std_id,
            CrateOrigin::Lang(LangCrateOrigin::Std),
        );
        let core_crate = create_crate(
            &mut crate_graph,
            "core",
            core_id,
            CrateOrigin::Lang(LangCrateOrigin::Core),
        );
        let alloc_crate = create_crate(
            &mut crate_graph,
            "alloc",
            alloc_id,
            CrateOrigin::Lang(LangCrateOrigin::Alloc),
        );
        let oort_api_crate = create_crate(
            &mut crate_graph,
            "oort_api",
            oort_api_id,
            local_origin.clone(),
        );
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

        change.change_file(file_id, Some(Arc::from("")));
        change.change_file(std_id, Some(Arc::from(fake_std_src)));
        change.change_file(core_id, Some(Arc::from(fake_core_src)));
        change.change_file(alloc_id, Some(Arc::from("")));
        change.change_file(oort_api_id, Some(Arc::from(oort_api_src)));
        change.change_file(vec_id, Some(Arc::from(vec_src)));
        change.set_crate_graph(crate_graph);
        host.apply_change(change);

        let analysis = host.analysis();
        for (name, id) in [
            ("std", std_id),
            ("core", core_id),
            ("alloc", alloc_id),
            /* TODO
            ("oort_api", oort_api_id),
            ("vec", vec_id),
            */
        ] {
            let diagnostics = analysis
                .diagnostics(
                    &DiagnosticsConfig::test_sample(),
                    ide::AssistResolveStrategy::None,
                    id,
                )
                .unwrap();
            if !diagnostics.is_empty() {
                log::error!("{} diagnostics: {:#?}", name, diagnostics);
            }
        }

        Self {
            link,
            analysis_host: host,
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, request: Self::Input, who: HandlerId) {
        log::info!("AnalyzerAgent got message: {:?}", request);
        let response = match request {
            Request::Diagnostics(text) => self.diagnostics(text),
            Request::Completion(line, col) => self.completion(line, col),
        };
        if let Some(msg) = response {
            self.link.respond(who, msg);
        }
    }

    fn name_of_resource() -> &'static str {
        "oort_analyzer_worker.js"
    }
}

impl AnalyzerAgent {
    fn diagnostics(&mut self, text: String) -> Option<Response> {
        let file_id = ide::FileId(0);
        let mut change = ide::Change::new();
        change.change_file(file_id, Some(Arc::from(text)));
        self.analysis_host.apply_change(change);
        let analysis = self.analysis_host.analysis();
        let line_index = analysis.file_line_index(file_id).unwrap();
        let diagnostics = analysis.diagnostics(
            &DiagnosticsConfig::test_sample(),
            ide::AssistResolveStrategy::None,
            file_id,
        );
        match diagnostics {
            Ok(diags) => {
                log::info!("Diagnostics: {:?}", diags);
                let diags = diags
                    .iter()
                    .map(|diag| translate_diagnostic(diag, line_index.as_ref()))
                    .collect();
                log::info!("Translated diagnostics: {:?}", diags);
                Some(Response::Diagnostics(diags))
            }
            Err(e) => {
                log::error!("Error getting diagnostics: {:?}", e);
                None
            }
        }
    }

    fn completion(&mut self, line: u32, col: u32) -> Option<Response> {
        let file_id = ide::FileId(0);

        const COMPLETION_CONFIG: CompletionConfig = CompletionConfig {
            enable_postfix_completions: true,
            enable_imports_on_the_fly: true,
            enable_self_on_the_fly: true,
            enable_private_editable: false,
            callable: Some(ide::CallableSnippets::FillArguments),
            snippet_cap: SnippetCap::new(true),
            insert_use: InsertUseConfig {
                granularity: ImportGranularity::Module,
                enforce_granularity: false,
                prefix_kind: PrefixKind::Plain,
                group: true,
                skip_glob_imports: false,
            },
            prefer_no_std: false,
            snippets: Vec::new(),
            limit: Some(10),
        };

        let analysis = self.analysis_host.analysis();
        let line_index = analysis.file_line_index(file_id).unwrap();

        let line_col = ide::LineCol {
            line: line - 1,
            col: col - 1,
        };
        let file_length = analysis.file_text(file_id).unwrap().len();
        let offset = line_index
            .offset(line_col)
            .unwrap_or_default()
            .min(TextSize::from(file_length as u32));
        let pos = ide::FilePosition { file_id, offset };

        let items = match analysis.completions(&COMPLETION_CONFIG, pos, None).unwrap() {
            Some(items) => items,
            None => return None,
        };

        let results = items
            .iter()
            .map(|item| {
                let inserts: Vec<_> = item.text_edit.iter().map(|x| x.insert.clone()).collect();
                let text = inserts.join("");
                CompletionItem {
                    label: item.label.to_string(),
                    kind: 1,
                    detail: item
                        .detail
                        .as_ref()
                        .map(|it| it.to_string())
                        .unwrap_or_default(),
                    insertText: text,
                    insertTextRules: if item.is_snippet { 4 } else { 0 },
                    filterText: item.lookup().to_string(),
                }
            })
            .collect();

        Some(Response::Completion(results))
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
