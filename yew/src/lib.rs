pub mod code_size;
pub mod codestorage;
pub mod js;
pub mod leaderboard;
pub mod telemetry;
pub mod ui;
pub mod userid;

use chrono::NaiveDateTime;
use gloo_render::{request_animation_frame, AnimationFrame};
use leaderboard::Leaderboard;
use monaco::{
    api::CodeEditorOptions, sys::editor::BuiltinTheme, yew::CodeEditor, yew::CodeEditorLink,
};
use oort_sim_agent::SimAgent;
use oort_simulator::scenario::{self, Status};
use oort_simulator::{script, simulation};
use rand::Rng;
use rbtag::{BuildDateTime, BuildInfo};
use std::rc::Rc;
use telemetry::Telemetry;
use ui::UI;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{EventTarget, HtmlInputElement};
use yew::events::Event;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

#[derive(BuildDateTime, BuildInfo)]
struct BuildTag;

pub fn version() -> String {
    let build_time = NaiveDateTime::from_timestamp(
        BuildTag {}
            .get_build_timestamp()
            .parse::<i64>()
            .unwrap_or(0),
        0,
    );

    let commit = BuildTag {}.get_build_commit();

    if commit.contains("dirty") {
        commit.to_string()
    } else {
        format!("{} {}", build_time.format("%Y%m%d.%H%M%S"), commit)
    }
}

fn make_monaco_options() -> CodeEditorOptions {
    CodeEditorOptions::default()
        .with_language("rust".to_owned())
        .with_value("foo".to_owned())
        .with_builtin_theme(BuiltinTheme::VsDark)
}

fn empty() -> JsValue {
    js_sys::Object::new().into()
}

pub enum Msg {
    Render,
    SelectScenario(String),
    KeyEvent(web_sys::KeyboardEvent),
    WheelEvent(web_sys::WheelEvent),
    ReceivedSimAgentResponse(oort_sim_agent::Response),
    ReceivedBackgroundSimAgentResponse(oort_sim_agent::Response),
    RequestSnapshot,
    EditorAction(String),
    ShowDocumentation,
    DismissOverlay,
}

enum Overlay {
    Documentation,
    #[allow(dead_code)]
    MissionComplete,
}

pub struct Model {
    render_handle: Option<AnimationFrame>,
    scenario_name: String,
    sim_agent: Box<dyn Bridge<SimAgent>>,
    background_agents: Vec<Box<dyn Bridge<SimAgent>>>,
    background_statuses: Vec<Status>,
    editor_link: CodeEditorLink,
    overlay: Option<Overlay>,
    overlay_ref: NodeRef,
    ui: Option<Box<UI>>,
    status_ref: NodeRef,
    last_status: Status,
    running_code: String,
    current_decorations: js_sys::Array,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(context: &yew::Context<Self>) -> Self {
        context
            .link()
            .send_message(Msg::SelectScenario("welcome".to_string()));
        let link2 = context.link().clone();
        let render_handle = Some(request_animation_frame(move |_ts| {
            link2.send_message(Msg::Render)
        }));
        let sim_agent = SimAgent::bridge(context.link().callback(Msg::ReceivedSimAgentResponse));
        Self {
            render_handle,
            scenario_name: String::new(),
            sim_agent,
            background_agents: Vec::new(),
            background_statuses: Vec::new(),
            editor_link: CodeEditorLink::default(),
            overlay: None,
            overlay_ref: NodeRef::default(),
            ui: None,
            status_ref: NodeRef::default(),
            last_status: Status::Running,
            running_code: String::new(),
            current_decorations: js_sys::Array::new(),
        }
    }

    fn update(&mut self, context: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Render => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.render();
                }
                let link2 = context.link().clone();
                self.render_handle = Some(request_animation_frame(move |_ts| {
                    link2.send_message(Msg::Render)
                }));
                self.check_status(context)
            }
            Msg::SelectScenario(scenario_name) => {
                self.scenario_name = scenario_name;
                let code = codestorage::load(&self.scenario_name);
                self.editor_link.with_editor(|editor| {
                    editor.get_model().unwrap().set_value(&code);
                });
                self.running_code = String::new();
                let seed = rand::thread_rng().gen();
                self.ui = Some(Box::new(UI::new(
                    context.link().callback(|_| Msg::RequestSnapshot),
                )));
                self.sim_agent.send(oort_sim_agent::Request::StartScenario {
                    scenario_name: self.scenario_name.to_owned(),
                    seed,
                    code: String::new(),
                });
                self.background_agents.clear();
                true
            }
            Msg::EditorAction(ref action) if action == "oort-execute" => {
                let code = self
                    .editor_link
                    .with_editor(|editor| editor.get_model().unwrap().get_value())
                    .unwrap();
                codestorage::save(&self.scenario_name, &code);
                self.running_code = code.clone();
                let seed = rand::thread_rng().gen();
                self.ui = Some(Box::new(UI::new(
                    context.link().callback(|_| Msg::RequestSnapshot),
                )));
                self.sim_agent.send(oort_sim_agent::Request::StartScenario {
                    scenario_name: self.scenario_name.to_owned(),
                    seed,
                    code,
                });
                self.background_agents.clear();
                false
            }
            Msg::EditorAction(ref action) if action == "oort-restore-initial-code" => {
                let code = scenario::load(&self.scenario_name).initial_code();
                self.editor_link.with_editor(|editor| {
                    editor.get_model().unwrap().set_value(&code);
                });
                false
            }
            Msg::EditorAction(ref action) if action == "oort-load-solution" => {
                let code = scenario::load(&self.scenario_name).solution();
                self.editor_link.with_editor(|editor| {
                    editor.get_model().unwrap().set_value(&code);
                });
                false
            }
            Msg::EditorAction(action) => {
                log::info!("Got unexpected editor action {}", action);
                false
            }
            Msg::KeyEvent(e) => {
                self.ui.as_mut().unwrap().on_key_event(e);
                false
            }
            Msg::WheelEvent(e) => {
                self.ui.as_mut().unwrap().on_wheel_event(e);
                false
            }
            Msg::ReceivedSimAgentResponse(oort_sim_agent::Response::Snapshot { snapshot }) => {
                self.display_errors(&snapshot.errors);
                self.ui.as_mut().unwrap().on_snapshot(snapshot);
                false
            }
            Msg::ReceivedBackgroundSimAgentResponse(oort_sim_agent::Response::Snapshot {
                snapshot,
            }) => {
                self.background_statuses.push(snapshot.status);
                true
            }
            Msg::RequestSnapshot => {
                self.sim_agent
                    .send(oort_sim_agent::Request::Snapshot { nonce: 0 });
                false
            }
            Msg::ShowDocumentation => {
                self.overlay = Some(Overlay::Documentation);
                true
            }
            Msg::DismissOverlay => {
                self.overlay = None;
                self.background_agents.clear();
                true
            }
        }
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let render_option = |name: String| {
            let selected = name == self.scenario_name;
            html! { <option name={name.clone()} selected={selected}>{name}</option> }
        };

        let select_scenario_cb = context.link().callback(|e: Event| {
            let target: EventTarget = e
                .target()
                .expect("Event should have a target when dispatched");
            let data = target.unchecked_into::<HtmlInputElement>().value();
            Msg::SelectScenario(data)
        });

        let key_event_cb = context.link().callback(Msg::KeyEvent);
        let wheel_event_cb = context.link().callback(Msg::WheelEvent);
        let show_documentation_cb = context.link().callback(|_| Msg::ShowDocumentation);

        let username = userid::get_username(&userid::get_userid());

        let monaco_options: Rc<CodeEditorOptions> = Rc::new(make_monaco_options());

        html! {
        <>
            <canvas id="glcanvas"
                tabindex="1"
                onkeydown={key_event_cb.clone()}
                onkeyup={key_event_cb}
                onwheel={wheel_event_cb} />
            <div id="editor">
                <CodeEditor options={monaco_options} link={self.editor_link.clone()} />
            </div>
            <div id="status" ref={self.status_ref.clone()} />
            <div id="toolbar">
                <div class="toolbar-elem title">{ "Oort" }</div>
                <div class="toolbar-elem right">
                    <select onchange={select_scenario_cb}>
                        { for scenario::list().iter().cloned().map(render_option) }
                    </select>
                </div>
                <div class="toolbar-elem right"><a href="#" onclick={show_documentation_cb}>{ "Documentation" }</a></div>
                <div class="toolbar-elem right"><a href="http://github.com/rlane/oort3" target="_none">{ "GitHub" }</a></div>
                <div class="toolbar-elem right"><a href="https://trello.com/b/PLQYouu8" target="_none">{ "Trello" }</a></div>
                <div class="toolbar-elem right"><a href="https://discord.gg/vYyu9EhkKH" target="_none">{ "Discord" }</a></div>
                <div id="username" class="toolbar-elem right" title="Your username">{ username }</div>
            </div>
            { self.render_overlay(context) }
        </>
        }
    }

    fn rendered(&mut self, context: &yew::Context<Self>, first_render: bool) {
        if first_render {
            self.editor_link.with_editor(|editor| {
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
            });
        }

        if self.overlay.is_some() {
            self.focus_overlay();
        }
    }
}

impl Model {
    fn check_status(&mut self, context: &Context<Self>) -> bool {
        if let Some(ui) = self.ui.as_ref() {
            if self.last_status == ui.status() {
                return false;
            }
            self.last_status = ui.status();

            if let Status::Victory { team: 0 } = ui.status() {
                let snapshot = ui.snapshot().unwrap();
                let code = &self.running_code;
                if !snapshot.cheats {
                    telemetry::send(Telemetry::FinishScenario {
                        scenario_name: self.scenario_name.clone(),
                        code: code.to_string(),
                        ticks: (snapshot.time / simulation::PHYSICS_TICK_LENGTH) as u32,
                        code_size: code_size::calculate(code),
                    });
                }

                self.background_agents.clear();
                for i in 0..10 {
                    let mut sim_agent = SimAgent::bridge(
                        context
                            .link()
                            .callback(Msg::ReceivedBackgroundSimAgentResponse),
                    );
                    sim_agent.send(oort_sim_agent::Request::RunScenario {
                        scenario_name: self.scenario_name.to_owned(),
                        seed: i,
                        code: self.running_code.clone(),
                    });
                    self.background_agents.push(sim_agent);
                }

                self.overlay = Some(Overlay::MissionComplete);
                return true;
            }
        }

        false
    }

    fn render_overlay(&self, context: &yew::Context<Self>) -> Html {
        let outer_click_cb = context.link().callback(|_| Msg::DismissOverlay);
        let inner_click_cb = context.link().batch_callback(|e: web_sys::MouseEvent| {
            e.stop_propagation();
            None
        });
        let key_cb = context.link().batch_callback(|e: web_sys::KeyboardEvent| {
            if e.key() == "Escape" {
                Some(Msg::DismissOverlay)
            } else {
                None
            }
        });
        if self.overlay.is_none() {
            return html! {};
        }
        html! {
            <div ref={self.overlay_ref.clone()} id="overlay"
                onkeydown={key_cb} onclick={outer_click_cb} tabindex="-1">
                <div class="inner-overlay" onclick={inner_click_cb}>{
                    match self.overlay {
                        Some(Overlay::Documentation) => self.render_documentation_overlay(),
                        Some(Overlay::MissionComplete) => self.render_mission_complete_overlay(context),
                        None => unreachable!(),
                    }
                }</div>
            </div>
        }
    }

    fn focus_overlay(&self) {
        if let Some(element) = self.overlay_ref.cast::<web_sys::HtmlElement>() {
            element.focus().expect("focusing overlay");
        }
    }

    fn render_documentation_overlay(&self) -> Html {
        html! {
            <>
                <h1>{ "Quick Reference" }</h1>
                { "Press Escape to close. File bugs on " }<a href="http://github.com/rlane/oort3/issues" target="_none">{ "GitHub" }</a>{ "." }<br />

                <h2>{ "Basics" }</h2>
                { "Select a scenario from the list in the top-right of the page." }<br/>
                { "Press Ctrl-Enter in the editor to run the scenario with a new version of your code." }<br/>
                { "The game calls your " }<code>{ "tick()" }</code>{ " function 60 times per second." }
            </>
        }
    }

    fn render_mission_complete_overlay(&self, context: &yew::Context<Self>) -> Html {
        let time = self.ui.as_ref().unwrap().snapshot().unwrap().time;
        let code_size = code_size::calculate(&self.running_code);

        let next_scenario = scenario::load(&self.scenario_name).next_scenario();
        let next_scenario_link = match next_scenario {
            Some(scenario_name) => {
                let next_scenario_cb = context.link().batch_callback(move |_| {
                    vec![
                        Msg::SelectScenario(scenario_name.clone()),
                        Msg::DismissOverlay,
                    ]
                });
                html! { <a href="#" onclick={next_scenario_cb}>{ "Next mission" }</a> }
            }
            None => {
                html! { <span>{ "Use the scenario list in the top-right of the page to choose your next mission." }</span> }
            }
        };

        let background_status = if self.background_statuses.len() == self.background_agents.len() {
            let victory_count = self
                .background_statuses
                .iter()
                .filter(|s| matches!(s, Status::Victory { team: 0 }))
                .count();
            html! { <span>{ "Simulations complete: " }{ victory_count }{"/"}{ self.background_agents.len() }{ " successful" }</span> }
        } else {
            html! { <span>{ "Waiting for simulations (" }{ self.background_statuses.len() }{ "/" }{ self.background_agents.len() }{ " complete)" }</span> }
        };

        html! {
            <div class="centered">
                <h1>{ "Mission Complete" }</h1>
                { "Time: " }{ format!("{:.2}", time) }{ " seconds" }<br/><br/>
                { "Code size: " }{ code_size }{ " bytes" }<br/><br/>
                { background_status }<br/><br/>
                { next_scenario_link }
                <br/><br/>
                <Leaderboard scenario_name={ self.scenario_name.clone() }/>
            </div>
        }
    }

    pub fn display_errors(&mut self, errors: &[script::Error]) {
        use monaco::sys::{
            editor::IModelDecorationOptions, editor::IModelDeltaDecoration, IMarkdownString, Range,
        };
        let decorations: Vec<IModelDeltaDecoration> = errors
            .iter()
            .map(|error| {
                let decoration: IModelDeltaDecoration = empty().into();
                decoration.set_range(
                    &Range::new(error.line as f64, 1.0, error.line as f64, 1.0).unchecked_into(),
                );
                let options: IModelDecorationOptions = empty().into();
                options.set_is_whole_line(Some(true));
                options.set_class_name("errorDecoration".into());
                let hover_message: IMarkdownString = empty().into();
                js_sys::Reflect::set(
                    &hover_message,
                    &JsValue::from_str("value"),
                    &JsValue::from_str(&error.msg),
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
        self.current_decorations = self
            .editor_link
            .with_editor(|editor| {
                editor
                    .as_ref()
                    .delta_decorations(&self.current_decorations, &decorations_jsarray)
            })
            .unwrap();
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    log::info!("Version {}", &version());
    let userid = userid::get_userid();
    log::info!("userid {}", &userid);
    log::info!("username {}", &userid::get_username(&userid));
    yew::start_app::<Model>();
    Ok(())
}
