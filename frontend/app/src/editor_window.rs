#![allow(clippy::drop_non_drop)]
use crate::analyzer_stub::{self, AnalyzerAgent, CompletionItem};
use crate::js;
use crate::js::filesystem::{
    DirectoryHandle, DirectoryValidateResponseEntry, FileHandle, PickEntry,
};
use gloo_timers::callback::Interval;
use gloo_timers::callback::Timeout;
use js_sys::Function;
use monaco::sys::Position;
use monaco::{
    api::CodeEditorOptions, sys::editor::BuiltinTheme, yew::CodeEditor, yew::CodeEditorLink,
};
use oort_simulator::simulation::Code;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{DragEvent, Element, FileSystemFileEntry};
use yew::html::Scope;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

fn empty() -> JsValue {
    js_sys::Object::new().into()
}

fn make_monaco_options() -> CodeEditorOptions {
    CodeEditorOptions::default()
        .with_language("rust".to_owned())
        .with_value("".to_owned())
        .with_builtin_theme(BuiltinTheme::VsDark)
}

#[derive(Debug)]
pub enum Msg {
    EditorAction(String),
    RequestAnalyzer,
    AnalyzerResponse(analyzer_stub::Response),
    RequestCompletion {
        line: u32,
        col: u32,
        resolve: Function,
        reject: Function,
    },
    LoadedCodeFromDisk(String),
    OpenedFile(FileHandle),
    LinkedFile(FileHandle),
    LinkedDirectory(DirectoryHandle),
    UnlinkedFile,
    CheckLinkedFile,
    CheckLinkedDirectory,
    LinkedDirectoryUpdate(u64, Vec<DirectoryValidateResponseEntry>),
    LinkedDirectorySetEntryPoint(DirectoryLinkEntryPoint),
    Drop(DragEvent),
    CancelDrop(MouseEvent),
}

#[derive(Properties, Clone, PartialEq)]
pub struct EditorWindowProps {
    pub host: web_sys::Element,
    pub editor_link: CodeEditorLink,
    pub on_editor_action: Callback<String>,
    pub team: usize,
    pub scenario_name: String,
}

pub struct EditorWindow {
    editor_link: CodeEditorLink,
    current_analyzer_decorations: js_sys::Array,
    last_analyzed_text: String,
    analyzer_agent: Box<dyn Bridge<AnalyzerAgent>>,
    #[allow(dead_code)]
    analyzer_interval: Interval,
    current_completion: Option<(Function, Function)>,
    folded: bool,
    file_handle: Option<FileHandle>,
    directory_link: Option<DirectoryLink>,
    linked: bool,
    drop_target_ref: NodeRef,
}

#[derive(Debug)]
struct DirectoryLink {
    handle: DirectoryHandle,
    last_modified: u64,
    files: Vec<String>,
    entry_point: DirectoryLinkEntryPoint,
}

#[derive(Debug, Clone)]
pub enum DirectoryLinkEntryPoint {
    None,
    LibRs,
    ScenarioOrDefault,
    SelectedFile(String),
}

impl Component for EditorWindow {
    type Message = Msg;
    type Properties = EditorWindowProps;

    fn create(context: &yew::Context<Self>) -> Self {
        let cb = {
            let link = context.link().clone();
            move |e| link.send_message(Msg::AnalyzerResponse(e))
        };
        let analyzer_agent = AnalyzerAgent::bridge(Rc::new(cb));
        let analyzer_interval = Interval::new(100, {
            let link = context.link().clone();
            move || link.send_message(Msg::RequestAnalyzer)
        });
        Self {
            editor_link: context.props().editor_link.clone(),
            current_analyzer_decorations: js_sys::Array::new(),
            last_analyzed_text: "".to_string(),
            analyzer_agent,
            analyzer_interval,
            current_completion: None,
            folded: false,
            file_handle: None,
            directory_link: None,
            linked: false,
            drop_target_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, context: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::EditorAction(ref action) if action == "oort-load-file" => {
                let text_cb = context.link().callback(Msg::LoadedCodeFromDisk);
                let file_handle_cb = context.link().callback(Msg::OpenedFile);
                wasm_bindgen_futures::spawn_local(async move {
                    let text = match js::filesystem::open().await {
                        Ok(file_handle) => {
                            let file_handle = file_handle.dyn_into::<FileHandle>().unwrap();
                            file_handle_cb.emit(file_handle.clone());
                            file_handle.read().await
                        }
                        Err(e) => Err(e),
                    };
                    match text {
                        Ok(text) => text_cb.emit(text.as_string().unwrap()),
                        Err(e) => log::error!("load failed: {:?}", e),
                    }
                });
                self.linked = false;
                self.set_read_only(false);
                false
            }
            Msg::EditorAction(ref action) if action == "oort-reload-file" => {
                if let Some(file_handle) = self.file_handle.clone() {
                    let cb = context.link().callback(Msg::LoadedCodeFromDisk);
                    wasm_bindgen_futures::spawn_local(async move {
                        match file_handle.read().await {
                            Ok(text) => cb.emit(text.as_string().unwrap()),
                            Err(e) => log::error!("reload failed: {:?}", e),
                        }
                    });
                } else {
                    context
                        .link()
                        .send_message(Msg::EditorAction("oort-load-file".to_string()));
                }
                false
            }
            Msg::EditorAction(ref action) if action == "oort-link-file" => {
                if has_open_file_picker() {
                    let cb = context.link().callback(Msg::LinkedFile);
                    wasm_bindgen_futures::spawn_local(async move {
                        match js::filesystem::open().await {
                            Ok(file_handle) => {
                                let file_handle = file_handle.dyn_into::<FileHandle>().unwrap();
                                cb.emit(file_handle);
                            }
                            Err(e) => log::error!("open failed: {:?}", e),
                        };
                    });
                } else {
                    self.drop_target_ref
                        .cast::<Element>()
                        .unwrap()
                        .set_class_name("drop_target");
                }
                false
            }
            Msg::EditorAction(ref action) if action == "oort-link-directory" => {
                if has_directory_picker() {
                    let cb = context.link().callback(Msg::LinkedDirectory);
                    wasm_bindgen_futures::spawn_local(async move {
                        match js::filesystem::open_directory().await {
                            Ok(directory_handle) => {
                                let directory_handle =
                                    directory_handle.dyn_into::<DirectoryHandle>().unwrap();
                                cb.emit(directory_handle);
                            }
                            Err(e) => log::error!("open directory failed: {:?}", e),
                        };
                    });
                } else {
                    log::warn!("Linked directories not supported on this platform");
                }
                false
            }
            Msg::EditorAction(ref action) if action == "oort-link-directory-select" => {
                // Used to select a new entry point for linked directory mode
                let Some(directory_link) = &self.directory_link else {
                    log::warn!("No directory linked");
                    return false;
                };

                let link = context.link().clone();
                let mut items = Vec::<PickEntry>::new();
                items.push(PickEntry::new("__lib__", "Use provided lib.rs"));
                items.push(PickEntry::new(
                    "__scenario__",
                    "Use ai_[scenario].rs or ai_default.rs",
                ));

                items.extend(
                    directory_link
                        .files
                        .iter()
                        .filter(|f| f.starts_with("ai_"))
                        .map(|v| PickEntry::new(v, v)),
                );

                let editor_link = self.editor_link.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let js_editor = editor_link
                        .with_editor(|ed| JsValue::from(ed.as_ref()))
                        .unwrap();

                    let items = items
                        .iter()
                        .map(|e| serde_wasm_bindgen::to_value(e).unwrap())
                        .collect();
                    let selection =
                        js::filesystem::pick(js_editor, "Select an entry point".into(), items)
                            .await;

                    match selection {
                        Ok(result) => {
                            let file_name = result.as_string().unwrap();

                            let entry_point = match file_name.as_str() {
                                "__lib__" => DirectoryLinkEntryPoint::LibRs,
                                "__scenario__" => DirectoryLinkEntryPoint::ScenarioOrDefault,
                                f if f.starts_with("ai_") => {
                                    DirectoryLinkEntryPoint::SelectedFile(String::from(f))
                                }
                                _ => {
                                    panic!("Invalid file selected: {:?}", file_name);
                                }
                            };

                            link.send_message(Msg::LinkedDirectorySetEntryPoint(entry_point));
                        }
                        Err(e) => {
                            log::error!("Failed selecting directory link entry point: {:?}", e)
                        }
                    }
                });

                false
            }
            Msg::EditorAction(ref action) if action == "oort-unlink-file" => {
                context.link().send_message(Msg::UnlinkedFile);
                false
            }
            Msg::EditorAction(ref action) if action == "oort-toggle-fold" => {
                self.toggle_fold();
                false
            }
            Msg::EditorAction(action) => {
                context.props().on_editor_action.emit(action);
                false
            }
            Msg::RequestAnalyzer => {
                let text = self
                    .editor_link
                    .with_editor(|editor| editor.get_model().unwrap().get_value())
                    .unwrap();
                if text != self.last_analyzed_text {
                    if crate::game::is_encrypted(&Code::Rust(text.clone())) {
                        self.display_analyzer_diagnostics(context, &[]);
                    } else {
                        self.analyzer_agent
                            .send(analyzer_stub::Request::Diagnostics(text.clone()));
                    }
                    self.last_analyzed_text = text;
                }
                false
            }
            Msg::AnalyzerResponse(analyzer_stub::Response::Diagnostics(diags)) => {
                self.display_analyzer_diagnostics(context, &diags);
                false
            }
            Msg::RequestCompletion {
                line,
                col,
                resolve,
                reject,
            } => {
                self.current_completion = Some((resolve, reject));
                self.analyzer_agent
                    .send(analyzer_stub::Request::Completion(line, col));
                false
            }
            Msg::AnalyzerResponse(analyzer_stub::Response::Completion(completions)) => {
                if let Some((resolve, _)) = self.current_completion.clone() {
                    let this = JsValue::null();
                    let results = CompletionList {
                        suggestions: completions,
                    };
                    resolve
                        .call1(&this, &serde_wasm_bindgen::to_value(&results).unwrap())
                        .unwrap();
                    self.current_completion = None;
                }
                false
            }
            Msg::LoadedCodeFromDisk(text) => {
                let editor_link = context.props().editor_link.clone();
                editor_link.with_editor(|editor| {
                    if editor.get_model().unwrap().get_value() != text {
                        editor.get_model().unwrap().set_value(&text);
                        // TODO trigger analyzer run
                    }
                });
                false
            }
            Msg::OpenedFile(file_handle) => {
                self.file_handle = Some(file_handle);
                self.directory_link = None;
                self.linked = false;
                false
            }
            Msg::LinkedFile(file_handle) => {
                self.file_handle = Some(file_handle);
                self.directory_link = None;
                self.linked = true;
                context.link().send_message(Msg::CheckLinkedFile);
                self.set_read_only(true);
                false
            }
            Msg::LinkedDirectory(handle) => {
                self.file_handle = None;
                self.directory_link = Some(DirectoryLink {
                    handle,
                    last_modified: 0,
                    files: Vec::new(),
                    entry_point: DirectoryLinkEntryPoint::None,
                });
                self.linked = true;
                context.link().send_message(Msg::CheckLinkedDirectory);
                self.set_read_only(true);
                false
            }
            Msg::UnlinkedFile => {
                self.linked = false;
                self.file_handle = None;
                self.directory_link = None;
                self.set_read_only(false);
                false
            }
            Msg::CheckLinkedFile => {
                if self.linked && self.get_read_only() {
                    if let Some(file_handle) = self.file_handle.clone() {
                        let link = context.link().clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            match file_handle.read().await {
                                Ok(text) => {
                                    link.send_message(Msg::LoadedCodeFromDisk(
                                        text.as_string().unwrap(),
                                    ));
                                    let timeout = Timeout::new(1_000, move || {
                                        link.send_message(Msg::CheckLinkedFile);
                                    });
                                    timeout.forget();
                                }
                                Err(e) => {
                                    log::error!("reload failed: {:?}", e);
                                    link.send_message(Msg::UnlinkedFile);
                                }
                            }
                        });
                    }
                }
                false
            }
            Msg::LinkedDirectoryUpdate(last_modified, files) => {
                let Some(directory_link) = &mut self.directory_link else {
                    log::warn!("No directory linked");
                    return false;
                };

                // Re-bundle a linked directory
                let link = context.link().clone();

                let had_files_before = !directory_link.files.is_empty();

                directory_link.last_modified = last_modified;
                directory_link.files = files.iter().map(|f| f.name.clone()).collect();
                let entry_point = directory_link.entry_point.clone();

                if !had_files_before && !directory_link.files.is_empty() {
                    // If this is the first time we've seen files, prompt to select an entry point
                    link.send_message(Msg::EditorAction("oort-link-directory-select".to_string()));
                }

                let mut files = files
                    .into_iter()
                    .map(|f| (f.name, f.contents))
                    .collect::<std::collections::HashMap<_, _>>();

                // Generate our lib.rs
                let mut lib_rs = Vec::new();
                lib_rs.push(String::from("// Linked to directory"));

                let scenario_name = context.props().scenario_name.clone();

                let mut load_file_entry = |file_name: &String| {
                    lib_rs.push(format!("// Entry point: {}", file_name));

                    // Add in our entry point
                    let mod_name = file_name.trim_end_matches(".rs");
                    lib_rs.push(format!("pub use {}::Ship;", mod_name));
                    lib_rs.push(format!("pub mod {};", mod_name));
                    lib_rs.push("".into());
                    lib_rs.push("// Modules".into());

                    // Add in our non-ai and non-lib.rs files
                    // TODO trim down to the modules we actually use
                    for (name, _contents) in &files {
                        if !name.starts_with("ai_") && name != "lib.rs" {
                            let mod_name = name.trim_end_matches(".rs");
                            lib_rs.push(format!("pub mod {};", mod_name));
                        }
                    }
                };

                match &entry_point {
                    DirectoryLinkEntryPoint::None => {
                        lib_rs.push(String::from("// No entry point selected"));
                    }
                    DirectoryLinkEntryPoint::LibRs => {
                        if !files.contains_key("lib.rs") {
                            files.insert("lib.rs".into(), String::from("// No lib.rs found"));
                        }
                    }
                    DirectoryLinkEntryPoint::ScenarioOrDefault => {
                        let scenario_file_name = format!("ai_{}.rs", scenario_name);
                        if files.contains_key(&scenario_file_name) {
                            load_file_entry(&scenario_file_name);
                        } else if files.contains_key("ai_default.rs") {
                            load_file_entry(&"ai_default.rs".into());
                        } else {
                            lib_rs
                                .push(format!("// No {}.rs or ai_default.rs found", scenario_name));
                        }
                    }
                    DirectoryLinkEntryPoint::SelectedFile(file_name) => {
                        load_file_entry(&file_name);
                    }
                }

                // Overwrite existing lib.rs unless we are using an existing one
                match &entry_point {
                    DirectoryLinkEntryPoint::LibRs => {}
                    _ => {
                        files.insert("lib.rs".into(), lib_rs.join("\n"));
                    }
                };

                match oort_multifile::join(files) {
                    Ok(bundled) => {
                        let mut lines = Vec::new();

                        lines.push(bundled);

                        link.send_message(Msg::LoadedCodeFromDisk(lines.join("\n")));
                    }
                    Err(e) => {
                        log::error!("Failed to join files: {:?}", e);
                        link.send_message(Msg::UnlinkedFile);
                    }
                }

                false
            }
            Msg::CheckLinkedDirectory => {
                // Periodically scan a linked directory for modifications to re-bundle
                if self.linked && self.get_read_only() {
                    if let Some(directory_link) = &self.directory_link {
                        let link = context.link().clone();
                        let old_last_modified = directory_link.last_modified;
                        let directory_handle = directory_link.handle.clone();

                        wasm_bindgen_futures::spawn_local(async move {
                            match directory_handle.load_files().await {
                                Ok(directory_result) => {
                                    let files =
                                        serde_wasm_bindgen::from_value::<
                                            Vec<DirectoryValidateResponseEntry>,
                                        >(directory_result)
                                        .expect("Invalid data from directory handle");

                                    let last_modified =
                                        files.iter().map(|f| f.last_modified).max().unwrap_or(0);

                                    // Avoid re-bundling if nothing changed
                                    if last_modified > old_last_modified {
                                        link.send_message(Msg::LinkedDirectoryUpdate(
                                            last_modified,
                                            files,
                                        ));
                                    }

                                    let timeout = Timeout::new(1_000, move || {
                                        link.send_message(Msg::CheckLinkedDirectory);
                                    });
                                    timeout.forget();
                                }
                                Err(e) => {
                                    log::error!("reload failed: {:?}", e);
                                    link.send_message(Msg::UnlinkedFile);
                                }
                            }
                        });
                    }
                }
                false
            }
            Msg::LinkedDirectorySetEntryPoint(entry_mode) => {
                if let Some(directory_link) = &mut self.directory_link {
                    directory_link.entry_point = entry_mode;
                    // XXX Force a re-bundle with the new options
                    directory_link.last_modified = 0;
                } else {
                    log::warn!("No directory linked");
                }

                false
            }
            Msg::CancelDrop(e) => {
                self.drop_target_ref
                    .cast::<Element>()
                    .unwrap()
                    .set_class_name("drop_target display_none");
                e.prevent_default();
                false
            }
            Msg::Drop(e) => {
                self.drop_target_ref
                    .cast::<Element>()
                    .unwrap()
                    .set_class_name("drop_target display_none");
                e.prevent_default();
                if let Some(data_transfer) = e.data_transfer() {
                    if let Some(entry) = data_transfer.items().get(0) {
                        if let Ok(Some(file_entry)) = entry.webkit_get_as_entry() {
                            let file_entry = file_entry.unchecked_into::<FileSystemFileEntry>();
                            context
                                .link()
                                .send_message(Msg::LinkedFile(FileHandle::new(file_entry)));
                        }
                    }
                }
                false
            }
        }
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let monaco_options: Rc<CodeEditorOptions> = Rc::new(make_monaco_options());
        let editor_link = context.props().editor_link.clone();
        let run_cb = context
            .props()
            .on_editor_action
            .reform(|_| "oort-execute".to_string());
        let replay_cb = context
            .props()
            .on_editor_action
            .reform(|_| "oort-replay".to_string());
        let replay_paused_cb = context
            .props()
            .on_editor_action
            .reform(|_| "oort-replay-paused".to_string());
        let cmd_or_ctrl = if is_mac() { "Cmd" } else { "Ctrl" };

        create_portal(
            html! {
                <>
                    <div class="editor">
                        <CodeEditor options={monaco_options} link={editor_link} />
                    </div>
                    <div class="run_button"><span
                        onclick={run_cb}
                        class="material-symbols-outlined"
                        title={format!("Execute ({cmd_or_ctrl}-Enter)")}
                    >{ "play_circle" }</span></div>
                    <div class="replay_button"><span
                        onclick={replay_cb}
                        class="material-symbols-outlined"
                        title={format!("Replay ({cmd_or_ctrl}-Shift-Enter)")}
                    >{ "replay" }</span></div>
                    <div class="replay_button_paused"><span
                        onclick={replay_paused_cb}
                        class="material-symbols-outlined"
                        title={"Replay paused"}
                    >{ "autopause" }</span></div>
                    <form>
                        <div class="drop_target display_none" ref={self.drop_target_ref.clone()}>
                            <span for="file" ondrop={context.link().callback(Msg::Drop)}>
                                { "Drop file here " }
                            </span>
                            <input type="file" ondrop={context.link().callback(Msg::Drop)} />
                            <button onclick={context.link().callback(Msg::CancelDrop)}>{ "cancel" }</button>
                        </div>
                    </form>
                </>
            },
            context.props().host.clone(),
        )
    }

    fn rendered(&mut self, context: &yew::Context<Self>, first_render: bool) {
        let editor_link = context.props().editor_link.clone();
        if first_render {
            editor_link.with_editor(|editor| {
                let add_action = |id: &'static str, label, key: Option<u32>| {
                    let cb = context.link().callback(Msg::EditorAction);
                    let closure =
                        Closure::wrap(Box::new(move |_: JsValue| cb.emit(id.to_string()))
                            as Box<dyn FnMut(_)>);

                    let ed: &monaco::sys::editor::IStandaloneCodeEditor = editor.as_ref();
                    let action = monaco::sys::editor::IActionDescriptor::from(empty());
                    action.set_id(id);
                    action.set_label(label);
                    action.set_context_menu_group_id(Some("navigation"));
                    action.set_context_menu_order(Some(1.5));
                    js_sys::Reflect::set(
                        &action,
                        &JsValue::from_str("run"),
                        &closure.into_js_value(),
                    )
                    .unwrap();
                    if let Some(key) = key {
                        js_sys::Reflect::set(
                            &action,
                            &JsValue::from_str("keybindings"),
                            &js_sys::JSON::parse(&format!("[{key}]")).unwrap(),
                        )
                        .unwrap();
                    }
                    ed.add_action(&action);
                };

                let add_action_without_context_menu = |id: &'static str, label| {
                    let cb = context.link().callback(Msg::EditorAction);
                    let closure =
                        Closure::wrap(Box::new(move |_: JsValue| cb.emit(id.to_string()))
                            as Box<dyn FnMut(_)>);

                    let ed: &monaco::sys::editor::IStandaloneCodeEditor = editor.as_ref();
                    let action = monaco::sys::editor::IActionDescriptor::from(empty());
                    action.set_id(id);
                    action.set_label(label);
                    js_sys::Reflect::set(
                        &action,
                        &JsValue::from_str("run"),
                        &closure.into_js_value(),
                    )
                    .unwrap();
                    ed.add_action(&action);
                };

                add_action(
                    "oort-execute",
                    "Execute",
                    Some(
                        monaco::sys::KeyMod::ctrl_cmd() as u32 | monaco::sys::KeyCode::Enter as u32,
                    ),
                );

                add_action(
                    "oort-replay",
                    "Replay",
                    Some(
                        monaco::sys::KeyMod::ctrl_cmd() as u32
                            | monaco::sys::KeyMod::shift() as u32
                            | monaco::sys::KeyCode::Enter as u32,
                    ),
                );

                add_action("oort-replay-paused", "Replay paused", None);

                add_action("oort-restore-initial-code", "Restore initial code", None);

                add_action("oort-load-solution", "Load solution", None);

                add_action("oort-load-file", "Load from a file", None);

                add_action(
                    "oort-reload-file",
                    "Reload from file",
                    Some(
                        monaco::sys::KeyMod::ctrl_cmd() as u32 | monaco::sys::KeyCode::KeyY as u32,
                    ),
                );

                add_action("oort-link-file", "Link to file on disk", None);
                if has_directory_picker() {
                    add_action("oort-link-directory", "Link to directory on disk", None);
                    add_action_without_context_menu(
                        "oort-link-directory-select",
                        "Select linked directory entry point",
                    );
                }
                add_action("oort-unlink-file", "Unlink from a file on disk", None);

                add_action(
                    "oort-format",
                    "Format code",
                    Some(
                        monaco::sys::KeyMod::ctrl_cmd() as u32 | monaco::sys::KeyCode::KeyE as u32,
                    ),
                );

                add_action(
                    "oort-toggle-fold",
                    "Toggle code folding",
                    Some(
                        monaco::sys::KeyMod::ctrl_cmd() as u32 | monaco::sys::KeyCode::KeyK as u32,
                    ),
                );

                add_action_without_context_menu(
                    "oort-submit-to-tournament",
                    "Submit to tournament",
                );

                {
                    let ed: &monaco::sys::editor::IStandaloneCodeEditor = editor.as_ref();
                    let options = monaco::sys::editor::IEditorOptions::from(empty());
                    let minimap_options = monaco::sys::editor::IEditorMinimapOptions::from(empty());
                    minimap_options.set_enabled(Some(false));
                    options.set_minimap(Some(&minimap_options));
                    ed.update_options(&options);
                }

                {
                    let ed: &monaco::sys::editor::IStandaloneCodeEditor = editor.as_ref();
                    let completer = Completer::new(context.link().clone());
                    js_sys::Reflect::set(
                        &ed.get_model().unwrap(),
                        &JsValue::from_str("completer"),
                        &completer.into(),
                    )
                    .unwrap();
                }

                js_sys::Reflect::set(
                    &web_sys::window().unwrap(),
                    &format!("editor{}", context.props().team).into(),
                    editor.as_ref(),
                )
                .unwrap();
            });
        }
    }
}

impl EditorWindow {
    pub fn display_analyzer_diagnostics(
        &mut self,
        _context: &Context<Self>,
        diags: &[analyzer_stub::Diagnostic],
    ) {
        use monaco::sys::{
            editor::IModelDecorationOptions, editor::IModelDeltaDecoration, IMarkdownString, Range,
        };
        let decorations: Vec<IModelDeltaDecoration> = diags
            .iter()
            .map(|diag| {
                let decoration: IModelDeltaDecoration = empty().into();
                decoration.set_range(
                    &Range::new(
                        diag.start_line as f64 + 1.0,
                        diag.start_column as f64 + 1.0,
                        diag.end_line as f64 + 1.0,
                        diag.end_column as f64
                            + 1.0
                            + if diag.start_column == diag.end_column {
                                1.0
                            } else {
                                0.0
                            },
                    )
                    .unchecked_into(),
                );
                let options: IModelDecorationOptions = empty().into();
                options.set_class_name("errorDecoration".into());
                let hover_message: IMarkdownString = empty().into();
                js_sys::Reflect::set(
                    &hover_message,
                    &JsValue::from_str("value"),
                    &JsValue::from_str(&diag.message),
                )
                .unwrap();
                options.set_hover_message(&hover_message);
                decoration.set_options(&options);
                decoration
            })
            .collect();
        let decorations_jsarray = js_sys::Array::new();
        for decoration in decorations {
            decorations_jsarray.push(&decoration);
        }
        self.current_analyzer_decorations = self
            .editor_link
            .with_editor(|editor| {
                editor
                    .as_ref()
                    .delta_decorations(&self.current_analyzer_decorations, &decorations_jsarray)
            })
            .unwrap();
    }

    fn toggle_fold(&mut self) {
        if self.folded {
            self.editor_link.with_editor(|ed| {
                ed.as_ref()
                    .trigger("fold", "editor.unfoldAll", &JsValue::NULL);
            });
        } else {
            self.editor_link.with_editor(|ed| {
                ed.as_ref()
                    .trigger("fold", "editor.foldAll", &JsValue::NULL);
            });
        }
        self.folded = !self.folded;
    }

    fn set_read_only(&mut self, read_only: bool) {
        self.editor_link.with_editor(|editor| {
            let ed: &monaco::sys::editor::IStandaloneCodeEditor = editor.as_ref();
            let options = monaco::sys::editor::IEditorOptions::from(empty());
            options.set_read_only(Some(read_only));
            ed.update_options(&options);
        });
    }

    fn get_read_only(&self) -> bool {
        self.editor_link
            .with_editor(|editor| {
                let ed: &monaco::sys::editor::IStandaloneCodeEditor = editor.as_ref();
                ed.get_raw_options().read_only().unwrap_or(false)
            })
            .unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize)]
struct CompletionList {
    pub suggestions: Vec<CompletionItem>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Completer {
    link: Scope<EditorWindow>,
}

#[wasm_bindgen]
impl Completer {
    fn new(link: Scope<EditorWindow>) -> Self {
        Self { link }
    }

    pub fn complete(&mut self, position: Position) -> js_sys::Promise {
        js_sys::Promise::new(&mut |resolve, reject| {
            self.link.send_message(Msg::RequestCompletion {
                line: position.line_number() as u32,
                col: position.column() as u32,
                resolve,
                reject,
            })
        })
    }
}

fn is_mac() -> bool {
    gloo_utils::window()
        .navigator()
        .app_version()
        .unwrap()
        .contains("Mac")
}

fn has_open_file_picker() -> bool {
    gloo_utils::window().has_own_property(&"showOpenFilePicker".into())
}

fn has_directory_picker() -> bool {
    gloo_utils::window().has_own_property(&"showDirectoryPicker".into())
}
