#![allow(clippy::drop_non_drop)]
use crate::analyzer_stub::{self, AnalyzerAgent, CompletionItem};
use crate::js;
use crate::js::filesystem::FileHandle;
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
    UnlinkedFile,
    CheckLinkedFile,
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
    file_handle: Option<FileHandle>,
    linked: bool,
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
            linked: false,
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
                self.linked = false;
                false
            }
            Msg::LinkedFile(file_handle) => {
                self.file_handle = Some(file_handle);
                self.linked = true;
                context.link().send_message(Msg::CheckLinkedFile);
                false
            }
            Msg::UnlinkedFile => {
                self.linked = false;
                self.file_handle = None;
                false
            }
            Msg::CheckLinkedFile => {
                if self.linked {
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
        }
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let monaco_options: Rc<CodeEditorOptions> = Rc::new(make_monaco_options());
        let editor_link = context.props().editor_link.clone();
        let run_cb = context
            .props()
            .on_editor_action
            .reform(|_| "oort-execute".to_string());
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
