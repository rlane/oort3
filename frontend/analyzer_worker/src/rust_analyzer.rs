use ide_db::SnippetCap;
use intern::Symbol;
use std::default::Default;
use triomphe::Arc;
use std::path::PathBuf;
use camino::Utf8PathBuf;

use cfg::CfgOptions;
use ide::{
    AnalysisHost, CompletionConfig, DiagnosticsConfig, Edition,
    FileId, LineIndex, SourceRoot, TextSize,
};
use ide_db::base_db::{
    CrateDisplayName, CrateName, CrateOrigin, DependencyBuilder, Env, FileSet, LangCrateOrigin, VfsPath,
    CrateGraphBuilder, CrateWorkspaceData, FileChange as Change, CrateBuilderId,
};
use vfs::AbsPathBuf;
use ide_db::ChangeWithProcMacros;
use ide_db::imports::insert_use::{ImportGranularity, InsertUseConfig, PrefixKind};

use crate::{CodeAnalyzer, CompletionItem, Diagnostic};

pub fn create_source_root(name: &str, f: FileId) -> SourceRoot {
    let mut file_set = FileSet::default();
    file_set.insert(f, VfsPath::new_virtual_path(format!("/{name}/src/lib.rs")));
    SourceRoot::new_library(file_set)
}

pub fn create_crate(
    crate_graph: &mut CrateGraphBuilder,
    name: &str,
    f: FileId,
    origin: CrateOrigin,
) -> CrateBuilderId {
    let mut cfg = CfgOptions::default();
    cfg.insert_atom(Symbol::intern("unix"));
    cfg.insert_key_value(Symbol::intern("target_arch"), Symbol::intern("x86_64"));
    cfg.insert_key_value(Symbol::intern("target_pointer_width"), Symbol::intern("64"));
    let ws_data = Arc::new(CrateWorkspaceData {
        target: Err("no target layout".into()),
        toolchain: None,
    });
    crate_graph.add_crate_root(
        f,
        Edition::Edition2021,
        Some(CrateDisplayName::from_canonical_name(name)),
        None,
        cfg,
        Default::default(),
        Env::default(),
        origin,
        Vec::new(),
        false,
        Arc::new({
            let path = PathBuf::from("/fake/path");
            let utf8_path = Utf8PathBuf::from_path_buf(path).unwrap();
            let abs_path_buf: AbsPathBuf = unsafe { std::mem::transmute(utf8_path) };
            abs_path_buf
        }),
        ws_data,
    )
}

pub struct RustAnalyzerBackend {
    analysis_host: ide::AnalysisHost,
}

impl Default for RustAnalyzerBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RustAnalyzerBackend {
    pub fn new() -> Self {
        let oort_api_src = include_str!("../../../shared/api/src/lib.rs");
        let vec_src = include_str!("../../../shared/api/src/vec.rs");
        let fake_core_src = include_str!("../stdlib/mini_core.rs");
        let fake_std_src = include_str!("../stdlib/mini_std.rs");

        let mut host = AnalysisHost::default();
        let file_id = FileId::from_raw(0);
        let std_id = FileId::from_raw(1);
        let core_id = FileId::from_raw(2);
        let alloc_id = FileId::from_raw(3);
        let oort_api_id = FileId::from_raw(4);
        let vec_id = FileId::from_raw(5);

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

        let mut change = Change::default();
        change.set_roots(vec![
            source_root,
            create_source_root("std", std_id),
            create_source_root("core", core_id),
            create_source_root("alloc", alloc_id),
            oort_api_source_root,
        ]);
        let mut crate_graph = CrateGraphBuilder::default();
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
        let core_dep = DependencyBuilder::new(CrateName::new("core").unwrap(), core_crate);
        let alloc_dep = DependencyBuilder::new(CrateName::new("alloc").unwrap(), alloc_crate);
        let std_dep = DependencyBuilder::new(CrateName::new("std").unwrap(), std_crate);
        let oort_api_dep = DependencyBuilder::new(CrateName::new("oort_api").unwrap(), oort_api_crate);

        crate_graph.add_dep(std_crate, core_dep.clone()).unwrap();
        crate_graph.add_dep(std_crate, alloc_dep.clone()).unwrap();
        crate_graph.add_dep(alloc_crate, core_dep.clone()).unwrap();

        crate_graph.add_dep(my_crate, core_dep).unwrap();
        crate_graph.add_dep(my_crate, alloc_dep).unwrap();
        crate_graph.add_dep(my_crate, std_dep).unwrap();
        crate_graph.add_dep(my_crate, oort_api_dep).unwrap();

        change.change_file(file_id, Some("".to_string()));
        change.change_file(std_id, Some(fake_std_src.to_string()));
        change.change_file(core_id, Some(fake_core_src.to_string()));
        change.change_file(alloc_id, Some("".to_string()));
        change.change_file(oort_api_id, Some(oort_api_src.to_string()));
        change.change_file(vec_id, Some(vec_src.to_string()));
        change.set_crate_graph(crate_graph);
        host.apply_change(ChangeWithProcMacros {
            source_change: change,
            proc_macros: Default::default(),
        });

        Self {
            analysis_host: host,
        }
    }
}

impl CodeAnalyzer for RustAnalyzerBackend {
    fn update_file(&mut self, text: String) -> Vec<Diagnostic> {
        let file_id = FileId::from_raw(0);
        let mut change = Change::default();
        change.change_file(file_id, Some(text));
        self.analysis_host.apply_change(ChangeWithProcMacros {
            source_change: change,
            proc_macros: Default::default(),
        });
        let analysis = self.analysis_host.analysis();
        let line_index = analysis.file_line_index(file_id).unwrap();
        let diagnostics = analysis.full_diagnostics(
            &DiagnosticsConfig::test_sample(),
            ide::AssistResolveStrategy::None,
            file_id,
        );
        match diagnostics {
            Ok(diags) => {
                let diags = diags
                    .iter()
                    .map(|diag| translate_diagnostic(diag, line_index.as_ref()))
                    .collect();
                diags
            }
            Err(e) => {
                log::error!("Error getting diagnostics: {:?}", e);
                Vec::new()
            }
        }
    }

    fn completions(&self, line: u32, col: u32) -> Vec<CompletionItem> {
        let file_id = FileId::from_raw(0);

        let completion_config = CompletionConfig {
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
            fields_to_resolve: ide::CompletionFieldsToResolve::empty(),
            enable_auto_iter: true,
            enable_auto_await: true,
            enable_term_search: false,
            term_search_fuel: 0,
            full_function_signatures: false,
            add_semicolon_to_unit: true,
            prefer_prelude: true,
            prefer_absolute: false,
            exclude_flyimport: Vec::new(),
            exclude_traits: &[],
            add_colons_to_module: true,
            ra_fixture: ide::RaFixtureConfig {
                disable_ra_fixture: true,
                minicore: ide_db::MiniCore::default(),
            },
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

        let items = match analysis.completions(&completion_config, pos, None).unwrap() {
            Some(items) => items,
            None => return Vec::new(),
        };

        items
            .iter()
            .map(|item| {
                let inserts: Vec<_> = item.text_edit.iter().map(|x| x.insert.clone()).collect();
                let text = inserts.join("");
                CompletionItem {
                    label: item.label.primary.to_string(),
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
            .collect()
    }
}

fn translate_diagnostic(diag: &ide::Diagnostic, line_index: &LineIndex) -> Diagnostic {
    let start = line_index.line_col(diag.range.range.start());
    let end = line_index.line_col(diag.range.range.end());
    Diagnostic {
        message: diag.message.clone(),
        start_line: start.line,
        start_column: start.col,
        end_line: end.line,
        end_column: end.col,
    }
}
