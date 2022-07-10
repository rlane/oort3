use crate::filesystem;
use crate::leaderboard::Leaderboard;
use crate::telemetry::Telemetry;
use crate::ui::UI;
use gloo_render::{request_animation_frame, AnimationFrame};
use monaco::{
    api::CodeEditorOptions, sys::editor::BuiltinTheme, yew::CodeEditor, yew::CodeEditorLink,
};
use oort_simulator::scenario::{self, Status};
use oort_simulator::simulation::Code;
use oort_simulator::{script, simulation};
use oort_worker::SimAgent;
use qstring::QString;
use rand::Rng;
use regex::Regex;
use reqwasm::http::Request;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{EventTarget, HtmlInputElement};
use yew::events::Event;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;

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
    MouseEvent(web_sys::MouseEvent),
    ReceivedSimAgentResponse(oort_worker::Response),
    ReceivedBackgroundSimAgentResponse(oort_worker::Response),
    RequestSnapshot,
    EditorAction(String),
    ShowDocumentation,
    DismissOverlay,
    CompileSucceeded(Code),
    CompileFailed(String),
    CompileSlow,
    LoadedCodeFromDisk(String),
    SubmitToTournament,
}

enum Overlay {
    Documentation,
    #[allow(dead_code)]
    MissionComplete,
    Compiling,
}

pub struct Game {
    render_handle: Option<AnimationFrame>,
    scenario_name: String,
    sim_agent: Box<dyn Bridge<SimAgent>>,
    background_agents: Vec<Box<dyn Bridge<SimAgent>>>,
    background_statuses: Vec<Status>,
    editor_link: CodeEditorLink,
    overlay: Option<Overlay>,
    overlay_ref: NodeRef,
    ui: Option<Box<UI>>,
    nonce: u32,
    status_ref: NodeRef,
    last_status: Status,
    running_source_code: Code,
    running_codes: Vec<Code>,
    current_decorations: js_sys::Array,
    saw_slow_compile: bool,
    local_compiler: bool,
    compiler_errors: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub scenario: String,
    pub demo: bool,
}

impl Component for Game {
    type Message = Msg;
    type Properties = Props;

    fn create(context: &yew::Context<Self>) -> Self {
        context
            .link()
            .send_message(Msg::SelectScenario(context.props().scenario.clone()));
        let link2 = context.link().clone();
        let render_handle = Some(request_animation_frame(move |_ts| {
            link2.send_message(Msg::Render)
        }));
        let sim_agent = SimAgent::bridge(context.link().callback(Msg::ReceivedSimAgentResponse));
        let local_compiler =
            QString::from(context.link().location().unwrap().search().as_str()).has("local");
        if local_compiler {
            log::info!("Using local compiler");
        }
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
            nonce: 0,
            status_ref: NodeRef::default(),
            last_status: Status::Running,
            running_source_code: Code::None,
            running_codes: Vec::new(),
            current_decorations: js_sys::Array::new(),
            saw_slow_compile: false,
            local_compiler,
            compiler_errors: None,
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
                crate::codestorage::save(
                    &self.scenario_name,
                    &str_to_code(
                        &self
                            .editor_link
                            .with_editor(|editor| editor.get_model().unwrap().get_value())
                            .unwrap(),
                    ),
                );
                self.scenario_name = scenario_name;
                let mut codes = crate::codestorage::load(&self.scenario_name);
                let displayed_code = if self.scenario_name == "welcome" {
                    Code::Rust(
                        "\
// Welcome to Oort.
// Select a scenario from the list in the top-right of the page.
// If you're new, start with \"tutorial01\"."
                            .to_string(),
                    )
                } else if let Code::Builtin(name) = &codes[0] {
                    oort_simulator::script::builtin::load_source(name).unwrap()
                } else {
                    codes[0].clone()
                };
                self.editor_link.with_editor(|editor| {
                    editor
                        .get_model()
                        .unwrap()
                        .set_value(&code_to_string(&displayed_code));
                });
                codes[0] = if context.props().demo || self.scenario_name == "welcome" {
                    oort_simulator::scenario::load(&self.scenario_name).solution()
                } else {
                    Code::None
                };
                self.run(context, &codes);
                true
            }
            Msg::EditorAction(ref action) if action == "oort-execute" => {
                let code = self
                    .editor_link
                    .with_editor(|editor| editor.get_model().unwrap().get_value())
                    .unwrap();
                let code = str_to_code(&code);
                crate::codestorage::save(&self.scenario_name, &code);
                self.start_compile(context, code);
                true
            }
            Msg::EditorAction(ref action) if action == "oort-restore-initial-code" => {
                let mut code = scenario::load(&self.scenario_name).initial_code()[0].clone();
                if let Code::Builtin(name) = code {
                    code = oort_simulator::script::builtin::load_source(&name).unwrap()
                }
                self.editor_link.with_editor(|editor| {
                    editor
                        .get_model()
                        .unwrap()
                        .set_value(&code_to_string(&code));
                });
                false
            }
            Msg::EditorAction(ref action) if action == "oort-load-solution" => {
                let mut code = scenario::load(&self.scenario_name).solution();
                if let Code::Builtin(name) = code {
                    code = oort_simulator::script::builtin::load_source(&name).unwrap()
                }
                self.editor_link.with_editor(|editor| {
                    editor
                        .get_model()
                        .unwrap()
                        .set_value(&code_to_string(&code));
                });
                false
            }
            Msg::EditorAction(ref action) if action == "oort-load-file" => {
                let cb = context.link().callback(Msg::LoadedCodeFromDisk);
                filesystem::load(Box::new(move |text| cb.emit(text)));
                false
            }
            Msg::EditorAction(ref action) if action == "oort-reload-file" => {
                let cb = context.link().callback(Msg::LoadedCodeFromDisk);
                filesystem::reload(Box::new(move |text| cb.emit(text)));
                false
            }
            Msg::EditorAction(action) => {
                log::info!("Got unexpected editor action {}", action);
                false
            }
            Msg::KeyEvent(e) => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.on_key_event(e);
                }
                false
            }
            Msg::WheelEvent(e) => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.on_wheel_event(e);
                }
                false
            }
            Msg::MouseEvent(e) => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.on_mouse_event(e);
                }
                false
            }
            Msg::ReceivedSimAgentResponse(oort_worker::Response::Snapshot { snapshot }) => {
                self.display_errors(&snapshot.errors);
                if let Some(ui) = self.ui.as_mut() {
                    ui.on_snapshot(snapshot);
                }
                false
            }
            Msg::ReceivedBackgroundSimAgentResponse(oort_worker::Response::Snapshot {
                snapshot,
            }) => {
                self.background_statuses.push(snapshot.status);
                true
            }
            Msg::RequestSnapshot => {
                self.sim_agent
                    .send(oort_worker::Request::Snapshot { nonce: self.nonce });
                false
            }
            Msg::ShowDocumentation => {
                self.overlay = Some(Overlay::Documentation);
                true
            }
            Msg::DismissOverlay => {
                self.overlay = None;
                self.background_agents.clear();
                self.background_statuses.clear();
                true
            }
            Msg::CompileSucceeded(code) => {
                if matches!(self.overlay, Some(Overlay::Compiling)) {
                    self.overlay = None;
                }
                self.display_errors(&[]);
                let mut codes = crate::codestorage::load(&self.scenario_name);
                codes[0] = code;
                self.run(context, &codes);
                true
            }
            Msg::CompileFailed(error) => {
                if matches!(self.overlay, Some(Overlay::Compiling)) {
                    self.overlay = None;
                }
                self.display_errors(&Self::make_editor_errors(&error));
                self.compiler_errors = Some(error);
                true
            }
            Msg::CompileSlow => {
                self.saw_slow_compile = true;
                false
            }
            Msg::LoadedCodeFromDisk(text) => {
                self.editor_link.with_editor(|editor| {
                    editor.get_model().unwrap().set_value(&text);
                });
                false
            }
            Msg::SubmitToTournament => {
                crate::telemetry::send(Telemetry::SubmitToTournament {
                    scenario_name: self.scenario_name.clone(),
                    code: code_to_string(&self.running_source_code),
                });
                false
            }
        }
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let render_option = |name: String| {
            let selected = name == self.scenario_name;
            html! { <option name={name.clone()} selected={selected}>{name}</option> }
        };

        let history = context.link().history().unwrap();
        let select_scenario_cb = context.link().callback(move |e: Event| {
            let target: EventTarget = e
                .target()
                .expect("Event should have a target when dispatched");
            let data = target.unchecked_into::<HtmlInputElement>().value();
            history.push(crate::Route::Scenario { name: data.clone() });
            Msg::SelectScenario(data)
        });

        let key_event_cb = context.link().callback(Msg::KeyEvent);
        let wheel_event_cb = context.link().callback(Msg::WheelEvent);
        let mouse_event_cb = context.link().callback(Msg::MouseEvent);
        let show_documentation_cb = context.link().callback(|_| Msg::ShowDocumentation);

        let username = crate::userid::get_username(&crate::userid::get_userid());

        let monaco_options: Rc<CodeEditorOptions> = Rc::new(make_monaco_options());

        html! {
        <>
            <canvas id="glcanvas"
                tabindex="1"
                onkeydown={key_event_cb.clone()}
                onkeyup={key_event_cb}
                onwheel={wheel_event_cb}
                onclick={mouse_event_cb} />
            <div id="editor">
                <CodeEditor options={monaco_options} link={self.editor_link.clone()} />
            </div>
            <div id="status" ref={self.status_ref.clone()} />
            <div id="picked"><pre id="picked_text"></pre></div>
            <div id="toolbar">
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
            </div>
            { self.render_overlay(context) }
            { self.render_compiler_errors(context) }
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

                add_action("oort-load-file", "Load from a file", None);

                add_action(
                    "oort-reload-file",
                    "Reload from file",
                    Some(
                        monaco::sys::KeyMod::ctrl_cmd() as u32 | monaco::sys::KeyCode::KeyY as u32,
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
            });
        }

        if self.overlay.is_some() {
            self.focus_overlay();
        }
    }
}

impl Game {
    fn check_status(&mut self, context: &Context<Self>) -> bool {
        if let Some(ui) = self.ui.as_ref() {
            if self.last_status == ui.status() {
                return false;
            }
            self.last_status = ui.status();
            if context.props().demo && ui.status() != Status::Running {
                context
                    .link()
                    .send_message(Msg::SelectScenario(context.props().scenario.clone()));
                return true;
            }

            if let Status::Victory { team: 0 } = ui.status() {
                let snapshot = ui.snapshot().unwrap();
                let code = &self.running_source_code;
                if !snapshot.cheats {
                    crate::telemetry::send(Telemetry::FinishScenario {
                        scenario_name: self.scenario_name.clone(),
                        code: code_to_string(code),
                        ticks: (snapshot.time / simulation::PHYSICS_TICK_LENGTH) as u32,
                        code_size: crate::code_size::calculate(&code_to_string(code)),
                    });
                }

                self.background_agents.clear();
                self.background_statuses.clear();
                for i in 0..10 {
                    let mut sim_agent = SimAgent::bridge(
                        context
                            .link()
                            .callback(Msg::ReceivedBackgroundSimAgentResponse),
                    );
                    sim_agent.send(oort_worker::Request::RunScenario {
                        scenario_name: self.scenario_name.to_owned(),
                        seed: i,
                        codes: self.running_codes.clone(),
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
        let inner_class = match &self.overlay {
            Some(Overlay::Compiling) => "inner-overlay small-overlay",
            _ => "inner-overlay",
        };

        html! {
            <div ref={self.overlay_ref.clone()} id="overlay"
                onkeydown={key_cb} onclick={outer_click_cb} tabindex="-1">
                <div class={inner_class} onclick={inner_click_cb}>{
                    match &self.overlay {
                        Some(Overlay::Documentation) => html! { <crate::documentation::Documentation /> },
                        Some(Overlay::MissionComplete) => self.render_mission_complete_overlay(context),
                        Some(Overlay::Compiling) => html! { <h1 class="compiling">{ "Compiling..." }</h1> },
                        None => unreachable!(),
                    }
                }</div>
            </div>
        }
    }

    fn render_compiler_errors(&self, _context: &yew::Context<Self>) -> Html {
        if let Some(e) = self.compiler_errors.as_ref() {
            html! {
                <div id="compiler-errors">
                    <pre>
                        <h1>{ "Compile error" }</h1>
                        { e }
                    </pre>
                </div>
            }
        } else {
            html! {}
        }
    }

    fn focus_overlay(&self) {
        if let Some(element) = self.overlay_ref.cast::<web_sys::HtmlElement>() {
            element.focus().expect("focusing overlay");
        }
    }

    fn render_mission_complete_overlay(&self, context: &yew::Context<Self>) -> Html {
        let time = if let Some(ui) = self.ui.as_ref() {
            ui.snapshot().unwrap().time
        } else {
            0.0
        };
        let code_size = crate::code_size::calculate(&code_to_string(&self.running_source_code));

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
            let submit_button =
                if scenario::load(&self.scenario_name).is_tournament() && victory_count >= 8 {
                    let cb = context.link().batch_callback(move |_| {
                        vec![Msg::SubmitToTournament, Msg::DismissOverlay]
                    });
                    html! {
                        <>
                            <br />
                            <button onclick={cb}>{ "Submit to tournament" }</button>
                        </>
                    }
                } else {
                    html! {}
                };
            html! {
                <>
                    <span>{ "Simulations complete: " }{ victory_count }{"/"}{ self.background_agents.len() }{ " successful" }</span>
                    { submit_button }
                </>
            }
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

    pub fn make_editor_errors(error: &str) -> Vec<script::Error> {
        let re = Regex::new(r"(?m)error.*?: (.*?)$\n.*?ai/src/user.rs:(\d+):").unwrap();
        re.captures_iter(error)
            .map(|m| script::Error {
                line: m[2].parse().unwrap(),
                msg: m[1].to_string(),
            })
            .collect()
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

    pub fn start_compile(&mut self, context: &Context<Self>, code: Code) {
        self.ui = None;
        self.compiler_errors = None;
        self.nonce = rand::thread_rng().gen();
        self.running_source_code = code.clone();
        let success_callback = context.link().callback(Msg::CompileSucceeded);
        let failure_callback = context.link().callback(Msg::CompileFailed);
        let compile_slow_callback = context.link().callback(|_| Msg::CompileSlow);
        match code {
            Code::Rust(src_code) => {
                let saw_slow_compile = self.saw_slow_compile;
                let url = if self.local_compiler {
                    "http://localhost:8081/compile"
                } else if saw_slow_compile {
                    "https://api.oort.rs/compile"
                } else {
                    "https://api-vm.oort.rs/compile"
                };

                wasm_bindgen_futures::spawn_local(async move {
                    let start_time = instant::Instant::now();
                    let check_compile_time = || {
                        let elapsed = instant::Instant::now() - start_time;
                        if !saw_slow_compile && elapsed > std::time::Duration::from_millis(3000) {
                            log::info!("Compilation was slow, switching backend to serverless");
                            compile_slow_callback.emit(());
                        }
                    };

                    let result = Request::post(url).body(src_code).send().await;
                    if let Err(e) = result {
                        log::error!("Compile error: {}", e);
                        failure_callback.emit(e.to_string());
                        check_compile_time();
                        return;
                    }

                    let response = result.unwrap();
                    if !response.ok() {
                        let error = response.text().await.unwrap();
                        log::error!("Compile error: {}", error);
                        failure_callback.emit(error);
                        check_compile_time();
                        return;
                    }

                    let wasm = response.binary().await;
                    if let Err(e) = wasm {
                        log::error!("Compile error: {}", e);
                        failure_callback.emit(e.to_string());
                        check_compile_time();
                        return;
                    }

                    let elapsed = instant::Instant::now() - start_time;
                    log::info!("Compile succeeded in {:?}", elapsed);
                    check_compile_time();
                    success_callback.emit(Code::Wasm(wasm.unwrap()));
                });

                self.overlay = Some(Overlay::Compiling);
            }
            Code::Builtin(name) => match oort_simulator::script::builtin::load_compiled(&name) {
                Ok(code) => success_callback.emit(code),
                Err(e) => failure_callback.emit(e),
            },
            _ => unreachable!(),
        }
    }

    pub fn run(&mut self, context: &Context<Self>, codes: &[Code]) {
        self.compiler_errors = None;
        self.running_codes = codes.to_vec();
        let seed = rand::thread_rng().gen();
        self.ui = Some(Box::new(UI::new(
            context.link().callback(|_| Msg::RequestSnapshot),
            self.nonce,
        )));
        self.sim_agent.send(oort_worker::Request::StartScenario {
            scenario_name: self.scenario_name.to_owned(),
            seed,
            codes: codes.to_vec(),
            nonce: self.nonce,
        });
        self.background_agents.clear();
        self.background_statuses.clear();
    }
}

pub fn code_to_string(code: &Code) -> String {
    match code {
        Code::None => "".to_string(),
        Code::Rust(s) => s.clone(),
        Code::Wasm(_) => "// wasm".to_string(),
        Code::Builtin(name) => format!("#builtin:{}", name),
    }
}

pub fn str_to_code(s: &str) -> Code {
    let re = Regex::new(r"#builtin:(.*)").unwrap();
    if let Some(m) = re.captures(s) {
        Code::Builtin(m[1].to_string())
    } else {
        Code::Rust(s.to_string())
    }
}
