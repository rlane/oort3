use yew::prelude::*;

#[derive(Debug)]
pub enum Msg {}

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct CompilerOutputWindowProps {
    pub host: web_sys::Element,
    pub compiler_errors: Option<String>,
}

pub struct CompilerOutputWindow {}

impl Component for CompilerOutputWindow {
    type Message = Msg;
    type Properties = CompilerOutputWindowProps;

    fn create(_context: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _context: &yew::Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let compile_errors = context.props().compiler_errors.clone();
        create_portal(
            html! {
                <>
                    <div class="compiler-output">
                        <h1>{ "Compiler Output" }</h1>
                        <pre>
                            { compile_errors.unwrap_or_default() }
                        </pre>
                    </div>
                </>
            },
            context.props().host.clone(),
        )
    }
}
