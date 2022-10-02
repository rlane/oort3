use oort_simulator::scenario;
use yew::events::Event;
use yew::prelude::*;

#[derive(Debug)]
pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct ToolbarProps {
    pub select_scenario_cb: Callback<Event>,
    pub show_documentation_cb: Callback<web_sys::MouseEvent>,
    pub scenario_name: String,
}

pub struct Toolbar {}

impl Component for Toolbar {
    type Message = Msg;
    type Properties = ToolbarProps;

    fn create(_context: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _context: &yew::Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let host = gloo_utils::document()
            .get_element_by_id("toolbar")
            .expect("a #toolbar element");

        let render_option = |name: String| {
            let selected = name == context.props().scenario_name;
            html! { <option name={name.clone()} selected={selected}>{name}</option> }
        };

        let username = crate::userid::get_username();
        let select_scenario_cb = context.props().select_scenario_cb.clone();
        let show_documentation_cb = context.props().show_documentation_cb.clone();

        create_portal(
            html! {
                <>
                    <div class="toolbar-elem title">{ "Oort" }</div>
                    <div class="toolbar-elem right">
                        <select onchange={select_scenario_cb}>
                            { for scenario::list().iter().cloned().map(render_option) }
                        </select>
                    </div>
                    <div class="toolbar-elem right"><a href="#" onclick={show_documentation_cb}>{ "Documentation" }</a></div>
                    <div class="toolbar-elem right"><a href="http://github.com/rlane/oort3/wiki" target="_none">{ "Wiki" }</a></div>
                    <div class="toolbar-elem right"><a href="http://github.com/rlane/oort3" target="_none">{ "GitHub" }</a></div>
                    <div class="toolbar-elem right"><a href="https://trello.com/b/PLQYouu8" target="_none">{ "Trello" }</a></div>
                    <div class="toolbar-elem right"><a href="https://discord.gg/vYyu9EhkKH" target="_none">{ "Discord" }</a></div>
                    <div id="username" class="toolbar-elem right" title="Your username">{ username }</div>
                </>
            },
            host,
        )
    }
}
