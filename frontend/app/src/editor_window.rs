#![allow(clippy::drop_non_drop)]
use crate::analyzer_stub::{self, AnalyzerAgent, CompletionItem};
use crate::js;
use crate::js::filesystem::{DirectoryHandle, FileHandle};
use gloo_timers::callback::Interval;
use gloo_timers::callback::Timeout;
use js_sys::Function;
use monaco::sys::Position;
use monaco::{
    api::CodeEditorOptions, sys::editor::BuiltinTheme, yew::CodeEditor, yew::CodeEditorLink,
};
use oort_multifile::Multifile;
use oort_simulator::simulation::Code;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{DragEvent, Element, EventTarget, FileSystemFileEntry, HtmlInputElement};
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
    LoadedCodeFromDisk(Multifile),
    SelectedMain(String),
    LinkedFiles(Vec<FileHandle>),
    LinkedDirectory(DirectoryHandle),
    UnlinkedFiles,
    CheckLinkedFile,
    Drop(DragEvent),
    CancelDrop(MouseEvent),
}

pub enum EditorAction {
    Execute,
    Replay,
    ReplayPaused,
}

pub struct LinkedFiles {
    file_handles: Vec<FileHandle>,
    directory_handle: Option<DirectoryHandle>,
    multifile: Multifile,
    main_filename: Option<String>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct EditorWindowProps {
    pub host: web_sys::Element,
    pub editor_link: CodeEditorLink,
    pub on_editor_action: Callback<String>,
    pub team: usize,
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
    drop_target_ref: NodeRef,
    linked_files: Option<LinkedFiles>,
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
            drop_target_ref: NodeRef::default(),
            linked_files: None,
        }
    }

    fn update(&mut self, context: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::EditorAction(ref action) if action == "oort-link-file" => {
                if has_open_file_picker() {
                    let cb = context.link().callback(Msg::LinkedFiles);
                    wasm_bindgen_futures::spawn_local(async move {
                        match js::filesystem::open().await {
                            Ok(file_handles) => {
                                let file_handles =
                                    file_handles.dyn_into::<js_sys::Array>().unwrap();
                                let file_handles = file_handles
                                    .iter()
                                    .map(|file_handle| {
                                        file_handle.dyn_into::<FileHandle>().unwrap()
                                    })
                                    .collect::<Vec<_>>();
                                cb.emit(file_handles);
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
                if has_open_file_picker() {
                    let cb = context.link().callback(Msg::LinkedDirectory);
                    wasm_bindgen_futures::spawn_local(async move {
                        match js::filesystem::openDirectory().await {
                            Ok(directory_handle) => {
                                let directory_handle =
                                    directory_handle.dyn_into::<DirectoryHandle>().unwrap();
                                cb.emit(directory_handle);
                            }
                            Err(e) => log::error!("open failed: {:?}", e),
                        };
                    });
                }
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
            Msg::LoadedCodeFromDisk(multifile) => {
                if let Some(linked_files) = self.linked_files.as_mut() {
                    let reset_main = !linked_files
                        .main_filename
                        .as_ref()
                        .is_some_and(|x| multifile.filenames.contains(x));
                    if reset_main {
                        linked_files.main_filename = multifile.filenames.first().cloned();
                    }
                    assert!(linked_files.main_filename.is_some());
                    let text = multifile
                        .finalize(linked_files.main_filename.as_ref().unwrap())
                        .unwrap();
                    linked_files.multifile = multifile;
                    let editor_link = context.props().editor_link.clone();
                    editor_link.with_editor(|editor| {
                        if editor.get_model().unwrap().get_value() != text {
                            editor.get_model().unwrap().set_value(&text);
                        }
                    });
                }
                true
            }
            Msg::SelectedMain(main_filename) => {
                if let Some(linked_files) = self.linked_files.as_mut() {
                    linked_files.main_filename = Some(main_filename);
                    context.link().send_message(Msg::CheckLinkedFile);
                }
                false
            }
            Msg::LinkedFiles(file_handles) => {
                self.linked_files = Some(LinkedFiles {
                    file_handles,
                    directory_handle: None,
                    multifile: Multifile::empty(),
                    main_filename: None,
                });
                context.link().send_message(Msg::CheckLinkedFile);
                false
            }
            Msg::LinkedDirectory(directory_handle) => {
                self.linked_files = Some(LinkedFiles {
                    file_handles: vec![],
                    directory_handle: Some(directory_handle),
                    multifile: Multifile::empty(),
                    main_filename: None,
                });
                context.link().send_message(Msg::CheckLinkedFile);
                false
            }
            Msg::UnlinkedFiles => {
                self.linked_files = None;
                true
            }
            Msg::CheckLinkedFile => {
                if let Some(linked_files) = self.linked_files.as_ref() {
                    let link = context.link().clone();
                    let directory_handle = linked_files.directory_handle.clone();
                    let file_handles = linked_files.file_handles.clone();

                    wasm_bindgen_futures::spawn_local(async move {
                        let file_handles = if let Some(directory_handle) = directory_handle.as_ref()
                        {
                            let file_handles = directory_handle.getFiles().await;
                            let file_handles = file_handles.dyn_into::<js_sys::Array>().unwrap();
                            let file_handles = file_handles
                                .iter()
                                .map(|file_handle| file_handle.dyn_into::<FileHandle>().unwrap())
                                .collect::<Vec<_>>();
                            file_handles
                        } else {
                            file_handles
                        };

                        match read_multifile(&file_handles).await {
                            Ok(multifile) => {
                                link.send_message(Msg::LoadedCodeFromDisk(multifile));
                                let timeout = Timeout::new(1_000, move || {
                                    link.send_message(Msg::CheckLinkedFile);
                                });
                                timeout.forget();
                            }
                            Err(e) => {
                                log::error!("reload failed: {:?}", e);
                                link.send_message(Msg::UnlinkedFiles);
                            }
                        }
                    });
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
                // TODO support multiple files
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
                                .send_message(Msg::LinkedFiles(vec![FileHandle::new(file_entry)]));
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

        let cmd_or_ctrl = cmd_or_ctrl();

        let select_main_cb = context.link().callback(|e: Event| {
            let target: EventTarget = e
                .target()
                .expect("Event should have a target when dispatched");
            let data = target.unchecked_into::<HtmlInputElement>().value();
            Msg::SelectedMain(data)
        });
        let multifile_select = if let Some(linked_files) = self.linked_files.as_ref() {
            let render_main_option = |name: &str| {
                let selected = linked_files
                    .main_filename
                    .as_ref()
                    .map(|x| x == name)
                    .unwrap_or_default();
                html! { <option value={name.to_string()} selected={selected}>{name.to_string()}</option> }
            };
            html! {
                <select onchange={select_main_cb}>
                    { for linked_files.multifile.filenames.iter().map(|x| render_main_option(x)) }
                </select>
            }
        } else {
            html! {}
        };
        let show_multifile_selector = self.linked_files.is_some();
        let relink_cb = context
            .link()
            .callback(|_| Msg::EditorAction("oort-link-file".to_string()));
        let unlink_cb = context.link().callback(|_| Msg::UnlinkedFiles);

        create_portal(
            html! {
                <>
                    <div class="multifile_select" hidden={!show_multifile_selector}>
                        <p>
                            { "This editor is linked to files on disk. Select the Ship implementation below, " }
                            <a href="#" onclick={relink_cb}>{ "link to different files" }</a>{ ", or " }
                            <a href="#" onclick={unlink_cb}>{ "unlink" }</a>{ "." }
                        </p>
                        {multifile_select}
                    </div>
                    <div class="editor" hidden={show_multifile_selector}>
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
                        title={format!("Replay paused ({cmd_or_ctrl}-Alt-Enter)")}
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

                add_action(
                    "oort-replay-paused",
                    "Replay paused",
                    Some(
                        monaco::sys::KeyMod::ctrl_cmd() as u32
                            | monaco::sys::KeyMod::alt() as u32
                            | monaco::sys::KeyCode::Enter as u32,
                    ),
                );

                add_action("oort-restore-initial-code", "Restore initial code", None);

                add_action("oort-load-solution", "Load solution", None);

                add_action(
                    "oort-link-file",
                    "Link to file(s) on disk",
                    Some(
                        monaco::sys::KeyMod::ctrl_cmd() as u32 | monaco::sys::KeyCode::KeyI as u32,
                    ),
                );

                add_action(
                    "oort-link-directory",
                    "Link to a directory on disk",
                    Some(
                        monaco::sys::KeyMod::ctrl_cmd() as u32
                            | monaco::sys::KeyMod::shift() as u32
                            | monaco::sys::KeyCode::KeyI as u32,
                    ),
                );

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

pub fn cmd_or_ctrl() -> String {
    if is_mac() {
        "Cmd".to_string()
    } else {
        "Ctrl".to_string()
    }
}

pub fn is_mac() -> bool {
    gloo_utils::window()
        .navigator()
        .app_version()
        .unwrap()
        .contains("Mac")
}

fn has_open_file_picker() -> bool {
    gloo_utils::window().has_own_property(&"showOpenFilePicker".into())
}

async fn read_multifile(file_handles: &[FileHandle]) -> anyhow::Result<Multifile> {
    let mut files = HashMap::new();
    for file_handle in file_handles.iter() {
        let file = match file_handle.read().await {
            Ok(text) => text.as_string().unwrap(),
            Err(e) => {
                log::error!("load failed: {:?}", e);
                continue;
            }
        };
        files.insert(file_handle.name().await.as_string().unwrap(), file);
    }

    oort_multifile::join(files)
}
