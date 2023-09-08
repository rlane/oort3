use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Debug)]
pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct SeedWindowProps {
    pub host: web_sys::Element,
    pub current_seed: u32,
    pub change_cb: Callback<Option<u32>>,
}

pub struct SeedWindow {}

impl Component for SeedWindow {
    type Message = Msg;
    type Properties = SeedWindowProps;

    fn create(_context: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _context: &yew::Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn changed(&mut self, _context: &Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let input_ref = NodeRef::default();
        let change_seed_cb = {
            let input_ref = input_ref.clone();
            let change_cb = context.props().change_cb.clone();
            change_cb.reform(move |e: MouseEvent| {
                e.prevent_default();
                let v = input_ref.cast::<HtmlInputElement>().unwrap().value();
                v.parse::<u32>().ok()
            })
        };

        let randomize_seed_cb = {
            let seed = rand::random::<u32>();
            let change_cb = context.props().change_cb.clone();
            change_cb.reform(move |e: MouseEvent| {
                e.prevent_default();
                Some(seed)
            })
        };

        let clear_seed_cb = {
            let change_cb = context.props().change_cb.clone();
            change_cb.reform(|e: MouseEvent| {
                e.prevent_default();
                None
            })
        };

        create_portal(
            html! {
                <div>
                    <h1>{ "Seed" }</h1>
                    <p>{ "The random number seed determines the initial state of the simulation and all subsequent random number generation." }</p>
                    <p>{ "Seeds 0 through 9 are used in the \"Mission Complete\" screen to get an average time. Seeds 0 through 99 are used in the tournament." }</p>
                    <form >
                        <input type="text" ref={input_ref} value={context.props().current_seed.to_string()} />
                        <button type="submit" onclick={change_seed_cb}>{ "Submit" }</button>
                        <button type="submit" onclick={randomize_seed_cb}>{ "Randomize" }</button>
                        <button type="submit" onclick={clear_seed_cb}>{ "Clear" }</button>
                    </form>
                </div>
            },
            context.props().host.clone(),
        )
    }
}
