use monaco::{
    api::CodeEditorOptions, sys::editor::BuiltinTheme, yew::CodeEditor, yew::CodeEditorLink,
};
use std::rc::Rc;
use yew::prelude::*;

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
    pub editor_link: CodeEditorLink,
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
        let host = gloo_utils::document()
            .get_element_by_id("editor-window")
            .expect("a #editor-window element");

        create_portal(
            html! {
                <div id="editor">
                    <CodeEditor options={monaco_options} link={editor_link} />
                </div>
            },
            host,
        )
    }
}
