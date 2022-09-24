use yew::prelude::*;

#[derive(Debug)]
pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct SimulationWindowProps {
    pub on_key_event: Callback<web_sys::KeyboardEvent>,
    pub on_wheel_event: Callback<web_sys::WheelEvent>,
    pub on_mouse_event: Callback<web_sys::MouseEvent>,
    pub status_ref: NodeRef,
}

pub struct SimulationWindow {}

impl Component for SimulationWindow {
    type Message = Msg;
    type Properties = SimulationWindowProps;

    fn create(_context: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _context: &yew::Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let key_event_cb = context.props().on_key_event.clone();
        let wheel_event_cb = context.props().on_wheel_event.clone();
        let mouse_event_cb = context.props().on_mouse_event.clone();
        let status_ref = context.props().status_ref.clone();
        let host = gloo_utils::document()
            .get_element_by_id("simulation-window")
            .expect("a #simulation-window element");

        create_portal(
            html! {
                <>
                    <canvas id="glcanvas"
                        tabindex="1"
                        onkeydown={key_event_cb.clone()}
                        onkeyup={key_event_cb}
                        onwheel={wheel_event_cb}
                        onclick={mouse_event_cb} />
                    <div id="status" ref={status_ref} />
                    <div id="picked"><pre id="picked_text"></pre></div>
                </>
            },
            host,
        )
    }
}
