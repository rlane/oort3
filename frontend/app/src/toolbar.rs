use oort_simulator::scenario;
use regex::Regex;
use wasm_bindgen::JsCast;
use yew::events::Event;
use yew::prelude::*;

#[derive(Debug)]
pub enum Msg {
    ChangeUsername(String),
}

#[derive(Properties, Clone, PartialEq)]
pub struct ToolbarProps {
    pub select_scenario_cb: Callback<Event>,
    pub show_feedback_cb: Callback<web_sys::MouseEvent>,
    pub scenario_name: String,
}

pub struct Toolbar {}

impl Component for Toolbar {
    type Message = Msg;
    type Properties = ToolbarProps;

    fn create(_context: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _context: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ChangeUsername(username) => {
                let re = Regex::new(r"^[a-zA-A0-9_-]+").unwrap();
                if !re.is_match(&username) || censor::Censor::Standard.check(&username) {
                    return true;
                }
                let window = web_sys::window().expect("no global `window` exists");
                let storage = window
                    .local_storage()
                    .expect("failed to get local storage")
                    .unwrap();
                if let Err(msg) = storage.set_item("/user/name", &username) {
                    log::error!("Failed to save username: {:?}", msg);
                }
                log::info!("Changed username to {:?}", username);
            }
        }
        true
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let host = gloo_utils::document()
            .get_element_by_id("toolbar")
            .expect("a #toolbar element");

        let render_option = |name: String| {
            let scenario = scenario::load(&name);
            let selected = name == context.props().scenario_name;
            html! { <option value={name.clone()} selected={selected}>{scenario.human_name()}</option> }
        };

        let username = crate::userid::get_username();
        let select_scenario_cb = context.props().select_scenario_cb.clone();
        let show_feedback_cb = context.props().show_feedback_cb.clone();

        let username_keydown_cb = context
            .link()
            .batch_callback(|event: web_sys::KeyboardEvent| {
                let input_box: web_sys::HtmlInputElement =
                    event.target().unwrap().dyn_into().unwrap();
                if event.key() == "Enter" {
                    let _ = input_box.blur();
                }
                vec![]
            });
        let username_blur_cb = context.link().callback(|event: web_sys::FocusEvent| {
            let input_box: web_sys::HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
            Msg::ChangeUsername(input_box.value())
        });

        create_portal(
            html! {
                <>
                    <div class="toolbar-elem title">{ "Oort" }</div>
                    <div class="toolbar-elem right">
                        <select onchange={select_scenario_cb}>
                            { for scenario::list().iter().cloned().map(render_option) }
                        </select>
                    </div>
                    <div class="toolbar-elem right"><a href="#" onclick={show_feedback_cb}>{ "Feedback" }</a></div>
                    <div class="toolbar-elem right"><a href="https://docs.rs/oort_api" target="_blank">{ "API Reference" }</a></div>
                    <div class="toolbar-elem right"><a href="http://github.com/rlane/oort3/wiki" target="_blank">{ "Wiki" }</a></div>
                    <div class="toolbar-elem right"><a href="http://github.com/rlane/oort3" target="_blank">{ "GitHub" }</a></div>
                    <div class="toolbar-elem right"><a href="https://trello.com/b/PLQYouu8" target="_blank">{ "Trello" }</a></div>
                    <div class="toolbar-elem right"><a href="https://discord.gg/vYyu9EhkKH" target="_blank">{ "Discord" }</a></div>
                    <div id="username" class="toolbar-elem right" title="Your username">
                        <input type="text"
                            value={username}
                            spellcheck="false"
                            onblur={username_blur_cb}
                            onkeydown={username_keydown_cb} />
                    </div>
                </>
            },
            host,
        )
    }
}
