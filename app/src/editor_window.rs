use monaco::{
    api::CodeEditorOptions, sys::editor::BuiltinTheme, yew::CodeEditor, yew::CodeEditorLink,
};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use yew::prelude::*;

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
pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct EditorWindowProps {
    pub host: web_sys::Element,
    pub editor_link: CodeEditorLink,
    pub on_editor_action: Callback<String>,
}

pub struct EditorWindow {}

impl Component for EditorWindow {
    type Message = Msg;
    type Properties = EditorWindowProps;

    fn create(_context: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _context: &yew::Context<Self>, _msg: Self::Message) -> bool {
        false
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
