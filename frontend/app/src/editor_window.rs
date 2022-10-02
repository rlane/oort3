use gloo_timers::callback::Interval;
use monaco::{
    api::CodeEditorOptions, sys::editor::BuiltinTheme, yew::CodeEditor, yew::CodeEditorLink,
};
use oort_analyzer_worker::AnalyzerAgent;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

fn empty() -> JsValue {
    js_sys::Object::new().into()
}

fn make_monaco_options() -> CodeEditorOptions {
    CodeEditorOptions::default()
        .with_language("rust".to_owned())
        .with_value("foo".to_owned())
        .with_builtin_theme(BuiltinTheme::VsDark)
}

#[derive(Debug)]
pub enum Msg {
    RequestAnalyzer,
    AnalyzerResponse(oort_analyzer_worker::Response),
}

#[derive(Properties, Clone, PartialEq)]
pub struct EditorWindowProps {
    pub host: web_sys::Element,
    pub editor_link: CodeEditorLink,
    pub on_editor_action: Callback<String>,
}

pub struct EditorWindow {
    editor_link: CodeEditorLink,
    current_analyzer_decorations: js_sys::Array,
    last_analyzed_text: String,
    analyzer_agent: Box<dyn Bridge<AnalyzerAgent>>,
    #[allow(dead_code)]
    analyzer_interval: Interval,
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
        }
    }

    fn update(&mut self, context: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RequestAnalyzer => {
                let text = self
                    .editor_link
                    .with_editor(|editor| editor.get_model().unwrap().get_value())
                    .unwrap();
                if text != self.last_analyzed_text {
                    self.analyzer_agent
                        .send(oort_analyzer_worker::Request::Analyze(text.clone()));
                    self.last_analyzed_text = text;
                }
                false
            }
            Msg::AnalyzerResponse(oort_analyzer_worker::Response::AnalyzeFinished(diags)) => {
                self.display_analyzer_diagnostics(context, &diags);
                false
            }
        }
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let monaco_options: Rc<CodeEditorOptions> = Rc::new(make_monaco_options());
        let editor_link = context.props().editor_link.clone();

        create_portal(
            html! {
                <div class="editor">
                    <CodeEditor options={monaco_options} link={editor_link} />
                </div>
            },
            context.props().host.clone(),
        )
    }

    fn rendered(&mut self, context: &yew::Context<Self>, first_render: bool) {
        let editor_link = context.props().editor_link.clone();
        if first_render {
            editor_link.with_editor(|editor| {
                let add_action = |id: &'static str, label, key: Option<u32>| {
                    let cb = context.props().on_editor_action.clone();
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
                            &js_sys::JSON::parse(&format!("[{}]", key)).unwrap(),
                        )
                        .unwrap();
                    }
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

                add_action(
                    "oort-format",
                    "Format code",
                    Some(
                        monaco::sys::KeyMod::ctrl_cmd() as u32 | monaco::sys::KeyCode::KeyE as u32,
                    ),
                );

                {
                    let ed: &monaco::sys::editor::IStandaloneCodeEditor = editor.as_ref();
                    let options = monaco::sys::editor::IEditorOptions::from(empty());
                    let minimap_options = monaco::sys::editor::IEditorMinimapOptions::from(empty());
                    minimap_options.set_enabled(Some(false));
                    options.set_minimap(Some(&minimap_options));
                    ed.update_options(&options);
                }

                js_sys::Reflect::set(
                    &web_sys::window().unwrap(),
                    &JsValue::from_str("editor"),
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
        diags: &[oort_analyzer_worker::Diagnostic],
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
}
