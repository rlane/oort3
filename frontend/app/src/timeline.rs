use yew::prelude::*;

#[derive(Debug)]
pub enum Msg {}

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct TimelineProps {
    pub host: web_sys::Element,
    pub frame_count: u32,
    pub change_cb: Callback<Option<u32>>,
}

pub struct Timeline {}

impl Component for Timeline {
    type Message = Msg;
    type Properties = TimelineProps;

    fn create(_context: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _context: &yew::Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let change_seed_cb = {
            let input_ref = input_ref.clone();
            let change_cb = context.props().change_cb.clone();
            change_cb.reform(move |e: MouseEvent| {
                e.prevent_default();
                let v = input_ref.cast::<HtmlInputElement>().unwrap().value();
                v.parse::<u32>().ok()
            })
        };

        html! {
            <>
                <div class="slidecontainer">
                    <input type="range" min=0 max={self.frame_count} value={self.frame_count} oninput={} class="slider" id="myRange">
                    <p>Value: <span id="demo"></span></p>
                </div>
            </>
        }
    }
}
