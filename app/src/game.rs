use crate::editor_window::EditorWindow;
use crate::filesystem;
use crate::leaderboard::Leaderboard;
use crate::simulation_window::SimulationWindow;
use crate::telemetry::Telemetry;
use crate::toolbar::Toolbar;
use gloo_render::{request_animation_frame, AnimationFrame};
use monaco::yew::CodeEditorLink;
use oort_analyzer::AnalyzerAgent;
use oort_simulator::scenario::{self, Status};
use oort_simulator::simulation::Code;
use oort_simulator::snapshot::Snapshot;
use oort_simulator::{simulation, vm};
use oort_worker::SimAgent;
use qstring::QString;
use rand::Rng;
use regex::Regex;
use reqwasm::http::Request;
use simulation::PHYSICS_TICK_LENGTH;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{EventTarget, HtmlInputElement};
use yew::events::Event;
use yew::html::Scope;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;

fn empty() -> JsValue {
    js_sys::Object::new().into()
}

pub enum Msg {
    Render,
    RegisterSimulationWindowLink(Scope<SimulationWindow>),
    SelectScenario(String),
    SimulationFinished(Snapshot),
    ReceivedBackgroundSimAgentResponse(oort_worker::Response, u32),
    ReceivedAnalyzerAgentResponse(oort_analyzer::Response),
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
    analyzer_agent: Box<dyn Bridge<AnalyzerAgent>>,
    background_agents: Vec<Box<dyn Bridge<SimAgent>>>,
    background_snapshots: Vec<(u32, Snapshot)>,
    editor_link: CodeEditorLink,
    overlay: Option<Overlay>,
    overlay_ref: NodeRef,
    running_source_code: Code,
    running_codes: Vec<Code>,
    current_compiler_decorations: js_sys::Array,
    current_analyzer_decorations: js_sys::Array,
    last_analyzed_text: String,
    saw_slow_compile: bool,
    local_compiler: bool,
    compiler_errors: Option<String>,
    frame: u64,
    last_window_size: (i32, i32),
    last_snapshot: Option<Snapshot>,
    simulation_window_link: Option<Scope<SimulationWindow>>,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Props {
    pub scenario: String,
    pub demo: bool,
    pub version: String,
}

impl Component for Game {
    type Message = Msg;
    type Properties = Props;

    fn create(context: &yew::Context<Self>) -> Self {
        let link2 = context.link().clone();
        let render_handle = Some(request_animation_frame(move |_ts| {
            link2.send_message(Msg::Render)
        }));
        let analyzer_agent =
            AnalyzerAgent::bridge(context.link().callback(Msg::ReceivedAnalyzerAgentResponse));
        let local_compiler =
            QString::from(context.link().location().unwrap().search().as_str()).has("local");
        if local_compiler {
            log::info!("Using local compiler");
        }
        Self {
            render_handle,
            scenario_name: String::new(),
            analyzer_agent,
            background_agents: Vec::new(),
            background_snapshots: Vec::new(),
            editor_link: CodeEditorLink::default(),
            overlay: None,
            overlay_ref: NodeRef::default(),
            running_source_code: Code::None,
            running_codes: Vec::new(),
            current_compiler_decorations: js_sys::Array::new(),
            current_analyzer_decorations: js_sys::Array::new(),
            last_analyzed_text: "".to_string(),
            saw_slow_compile: false,
            local_compiler,
            compiler_errors: None,
            frame: 0,
            last_window_size: (0, 0),
            last_snapshot: None,
            simulation_window_link: None,
        }
    }

    fn update(&mut self, context: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Render => {
                if self.frame % 6 == 0 {
                    // TODO: Use ResizeObserver when stable.
                    let root = gloo_utils::document().document_element().unwrap();
                    let new_size = (root.client_width(), root.client_height());
                    if new_size != self.last_window_size {
                        crate::js::golden_layout::update_size();
                        self.last_window_size = new_size;
                    }
                    self.editor_link.with_editor(|editor| {
                        let ed: &monaco::sys::editor::IStandaloneCodeEditor = editor.as_ref();
                        ed.layout(None);
                        let text = editor.get_model().unwrap().get_value();
                        if text != self.last_analyzed_text {
                            self.analyzer_agent
                                .send(oort_analyzer::Request::Analyze(text.clone()));
                            self.last_analyzed_text = text;
                        }
                    });
                }
                self.frame += 1;

                if let Some(link) = self.simulation_window_link.as_ref() {
                    link.send_message(crate::simulation_window::Msg::Render);
                }

                let link2 = context.link().clone();
                self.render_handle = Some(request_animation_frame(move |_ts| {
                    link2.send_message(Msg::Render)
                }));

                false
            }
            Msg::RegisterSimulationWindowLink(link) => {
                self.simulation_window_link = Some(link);
                context
                    .link()
                    .send_message(Msg::SelectScenario(context.props().scenario.clone()));
                false
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
                    oort_simulator::vm::builtin::load_source(name).unwrap()
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
            Msg::SimulationFinished(snapshot) => {
                self.on_simulation_finished(context, snapshot);
                false
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
                    code = oort_simulator::vm::builtin::load_source(&name).unwrap()
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
                    code = oort_simulator::vm::builtin::load_source(&name).unwrap()
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
            Msg::EditorAction(ref action) if action == "oort-format" => {
                self.editor_link.with_editor(|editor| {
                    let model = editor.get_model().unwrap();
                    model.set_value(&crate::format::format(&model.get_value()));
                    self.analyzer_agent
                        .send(oort_analyzer::Request::Analyze(model.get_value()));
                });
                false
            }
            Msg::EditorAction(action) => {
                log::info!("Got unexpected editor action {}", action);
                false
            }
            Msg::ReceivedBackgroundSimAgentResponse(
                oort_worker::Response::Snapshot { snapshot },
                seed,
            ) => {
                self.background_snapshots.push((seed, snapshot));
                if let Some(summary) = self.summarize_background_simulations() {
                    let code = &self.running_source_code;
                    crate::telemetry::send(Telemetry::FinishScenario {
                        scenario_name: self.scenario_name.clone(),
                        code: code_to_string(code),
                        ticks: (summary.average_time.unwrap_or(0.0)
                            / simulation::PHYSICS_TICK_LENGTH)
                            as u32,
                        code_size: crate::code_size::calculate(&code_to_string(code)),
                        success: summary.failed_seeds.is_empty(),
                    });
                }
                true
            }
            Msg::ShowDocumentation => {
                self.overlay = Some(Overlay::Documentation);
                true
            }
            Msg::DismissOverlay => {
                self.overlay = None;
                self.background_agents.clear();
                self.background_snapshots.clear();
                true
            }
            Msg::CompileSucceeded(code) => {
                if matches!(self.overlay, Some(Overlay::Compiling)) {
                    self.overlay = None;
                }
                self.display_errors(&[]);
                crate::telemetry::send(Telemetry::StartScenario {
                    scenario_name: self.scenario_name.clone(),
                    code: code_to_string(&self.running_source_code),
                });
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
            Msg::ReceivedAnalyzerAgentResponse(oort_analyzer::Response::AnalyzeFinished(diags)) => {
                self.display_analyzer_diagnostics(&diags);
                false
            }
        }
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        // For Toolbar
        let history = context.link().history().unwrap();
        let select_scenario_cb = context.link().callback(move |e: Event| {
            let target: EventTarget = e
                .target()
                .expect("Event should have a target when dispatched");
            let data = target.unchecked_into::<HtmlInputElement>().value();
            history.push(crate::Route::Scenario { name: data.clone() });
            Msg::SelectScenario(data)
        });
        let show_documentation_cb = context.link().callback(|_| Msg::ShowDocumentation);

        // For EditorWindow
        let editor_window_host = gloo_utils::document()
            .get_element_by_id("editor-window")
            .expect("a #editor-window element");
        let editor_link = self.editor_link.clone();

        // For SimulationWindow
        let simulation_window_host = gloo_utils::document()
            .get_element_by_id("simulation-window")
            .expect("a #simulation-window element");
        let on_simulation_finished = context.link().callback(Msg::SimulationFinished);
        let register_link = context.link().callback(Msg::RegisterSimulationWindowLink);
        let version = context.props().version.clone();

        html! {
        <>
            <Toolbar scenario_name={self.scenario_name.clone()} {select_scenario_cb} {show_documentation_cb} />
            <EditorWindow host={editor_window_host} {editor_link} />
            <SimulationWindow host={simulation_window_host} {on_simulation_finished} {register_link} {version} />
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

        if self.overlay.is_some() {
            self.focus_overlay();
        }
    }
}

struct BackgroundSimSummary {
    count: usize,
    victory_count: usize,
    failed_seeds: Vec<u32>,
    average_time: Option<f64>,
    best_seed: Option<u32>,
    worst_seed: Option<u32>,
}

impl Game {
    fn on_simulation_finished(&mut self, context: &yew::Context<Self>, snapshot: Snapshot) {
        let status = snapshot.status;

        if context.props().demo && status != Status::Running {
            context
                .link()
                .send_message(Msg::SelectScenario(context.props().scenario.clone()));
            return;
        }

        self.display_errors(&snapshot.errors);

        if let Status::Victory { team: 0 } = status {
            self.background_agents.clear();
            self.background_snapshots.clear();
            for seed in 0..10 {
                let mut sim_agent = SimAgent::bridge(
                    context
                        .link()
                        .callback(move |msg| Msg::ReceivedBackgroundSimAgentResponse(msg, seed)),
                );
                sim_agent.send(oort_worker::Request::RunScenario {
                    scenario_name: self.scenario_name.to_owned(),
                    seed,
                    codes: self.running_codes.clone(),
                });
                self.background_agents.push(sim_agent);
            }

            self.overlay = Some(Overlay::MissionComplete);
        }

        self.last_snapshot = Some(snapshot);
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

    fn summarize_background_simulations(&self) -> Option<BackgroundSimSummary> {
        if self.background_snapshots.len() != self.background_agents.len() {
            return None;
        }

        let is_victory = |status: &scenario::Status| matches!(*status, Status::Victory { team: 0 });
        let mut failed_seeds: Vec<u32> = self
            .background_snapshots
            .iter()
            .filter(|(_, snapshot)| !is_victory(&snapshot.status))
            .map(|(seed, _)| *seed)
            .collect();
        failed_seeds.sort();
        let victory_count = self.background_snapshots.len() - failed_seeds.len();
        let average_time: Option<f64> = if victory_count > 0 {
            Some(
                self.background_snapshots
                    .iter()
                    .filter(|(_, snapshot)| is_victory(&snapshot.status))
                    .map(|(_, snapshot)| snapshot.time)
                    .sum::<f64>()
                    / victory_count as f64,
            )
        } else {
            None
        };

        let mut victory_seeds_by_time: Vec<_> = self
            .background_snapshots
            .iter()
            .filter(|(_, snapshot)| is_victory(&snapshot.status))
            .map(|(seed, snapshot)| (*seed, snapshot.time))
            .collect();
        victory_seeds_by_time.sort_by_key(|(_, time)| (time / PHYSICS_TICK_LENGTH) as i64);
        let best_seed = victory_seeds_by_time.first().map(|(seed, _)| *seed);
        let mut worst_seed = victory_seeds_by_time.last().map(|(seed, _)| *seed);
        if worst_seed == best_seed {
            worst_seed = None;
        }

        Some(BackgroundSimSummary {
            count: self.background_agents.len(),
            victory_count,
            failed_seeds,
            average_time,
            best_seed,
            worst_seed,
        })
    }

    fn render_mission_complete_overlay(&self, context: &yew::Context<Self>) -> Html {
        let time = if let Some(snapshot) = self.last_snapshot.as_ref() {
            snapshot.time
        } else {
            0.0
        };
        let code_size = crate::code_size::calculate(&code_to_string(&self.running_source_code));

        let next_scenario = scenario::load(&self.scenario_name).next_scenario();

        let make_seed_link_cb = |seed: u32| {
            let history = context.link().history().unwrap();
            let scenario_name = self.scenario_name.clone();
            context.link().batch_callback(move |_| {
                let mut query = std::collections::HashMap::<&str, String>::new();
                query.insert("seed", seed.to_string());
                history
                    .push_with_query(
                        crate::Route::Scenario {
                            name: scenario_name.clone(),
                        },
                        query,
                    )
                    .unwrap();
                vec![
                    Msg::DismissOverlay,
                    Msg::SelectScenario(scenario_name.clone()),
                ]
            })
        };
        let make_seed_link = |seed| html! { <a onclick={make_seed_link_cb(seed)}>{ seed }</a> };

        let background_status = if let Some(summary) = self.summarize_background_simulations() {
            let next_scenario_link = if summary.failed_seeds.is_empty() {
                match next_scenario {
                    Some(scenario_name) => {
                        let next_scenario_cb = context.link().batch_callback(move |_| {
                            vec![
                                Msg::SelectScenario(scenario_name.clone()),
                                Msg::DismissOverlay,
                            ]
                        });
                        html! { <><br /><a href="#" onclick={next_scenario_cb}>{ "Next mission" }</a></> }
                    }
                    None => {
                        html! {}
                    }
                }
            } else {
                html! {}
            };
            let failures = if summary.failed_seeds.is_empty() {
                html! {}
            } else {
                html! {
                    <>
                    <br />
                    <span>
                        <><b class="error">{ "Your solution did not pass all simulations." }</b><br />{ "Failed seeds: " }</>
                        { summary.failed_seeds.iter().cloned().map(|seed: u32| html! {
                            <>{ make_seed_link(seed) }{ "\u{00a0}" }</>  }).collect::<Html>() }
                    </span>
                    </>
                }
            };

            let best_and_worst_seeds = match (summary.best_seed, summary.worst_seed) {
                (Some(best), Some(worst)) => html! {
                    <><br /><span>{ "Best seed: " }{ make_seed_link(best) }{ " Worst: " }{ make_seed_link(worst) }</span></>
                },
                (Some(best), None) => {
                    html! { <><br /><span>{ "Best seed: " }{ make_seed_link(best) }</span></> }
                }
                _ => html! {},
            };
            let submit_button = if scenario::load(&self.scenario_name).is_tournament()
                && summary.victory_count >= (summary.count as f64 * 0.8) as usize
            {
                let cb = context
                    .link()
                    .batch_callback(move |_| vec![Msg::SubmitToTournament, Msg::DismissOverlay]);
                html! {
                    <>
                        <br /><button onclick={cb}>{ "Submit to tournament" }</button><br/>
                    </>
                }
            } else {
                html! {}
            };
            html! {
                <>
                    <span>{ "Simulations complete: " }{ summary.victory_count }{"/"}{ summary.count }{ " successful" }</span><br />
                    <span>
                        { "Average time: " }
                        {
                            if let Some(average_time) = summary.average_time {
                                format!("{:.2} seconds", average_time)
                            } else {
                                "none".to_string()
                            }
                        }
                    </span>
                    { failures }
                    { best_and_worst_seeds }
                    { submit_button }
                    { next_scenario_link }
                    <br />
                    <Leaderboard scenario_name={ self.scenario_name.clone() }/>
                </>
            }
        } else {
            html! { <span>{ "Waiting for simulations (" }{ self.background_snapshots.len() }{ "/" }{ self.background_agents.len() }{ " complete)" }</span> }
        };

        html! {
            <div class="centered">
                <h1>{ "Mission Complete" }</h1>
                { "Time: " }{ format!("{:.2}", time) }{ " seconds" }<br/>
                { "Code size: " }{ code_size }{ " bytes" }<br/><br/>
                { background_status }<br/><br/>
                <br/><br/>
            </div>
        }
    }

    pub fn make_editor_errors(error: &str) -> Vec<vm::Error> {
        let re = Regex::new(r"(?m)error.*?: (.*?)$\n.*?ai/src/user.rs:(\d+):").unwrap();
        re.captures_iter(error)
            .map(|m| vm::Error {
                line: m[2].parse().unwrap(),
                msg: m[1].to_string(),
            })
            .collect()
    }

    pub fn display_errors(&mut self, errors: &[vm::Error]) {
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
        self.current_compiler_decorations = self
            .editor_link
            .with_editor(|editor| {
                editor
                    .as_ref()
                    .delta_decorations(&self.current_compiler_decorations, &decorations_jsarray)
            })
            .unwrap();
    }

    pub fn display_analyzer_diagnostics(&mut self, diags: &[oort_analyzer::Diagnostic]) {
        use monaco::sys::{
            editor::IModelDecorationOptions, editor::IModelDeltaDecoration, IMarkdownString, Range,
        };
        let decorations: Vec<IModelDeltaDecoration> = diags
            .iter()
            .map(|diag| {
                let decoration: IModelDeltaDecoration = empty().into();
                decoration.set_range(
                    &Range::new(
                        diag.start_line as f64 + 1.0,
                        diag.start_column as f64 + 1.0,
                        diag.end_line as f64 + 1.0,
                        diag.end_column as f64
                            + 1.0
                            + if diag.start_column == diag.end_column {
                                1.0
                            } else {
                                0.0
                            },
                    )
                    .unchecked_into(),
                );
                let options: IModelDecorationOptions = empty().into();
                options.set_class_name("errorDecoration".into());
                let hover_message: IMarkdownString = empty().into();
                js_sys::Reflect::set(
                    &hover_message,
                    &JsValue::from_str("value"),
                    &JsValue::from_str(&diag.message),
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
        self.current_analyzer_decorations = self
            .editor_link
            .with_editor(|editor| {
                editor
                    .as_ref()
                    .delta_decorations(&self.current_analyzer_decorations, &decorations_jsarray)
            })
            .unwrap();
    }

    pub fn start_compile(&mut self, context: &Context<Self>, code: Code) {
        self.compiler_errors = None;
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
            Code::Builtin(name) => match oort_simulator::vm::builtin::load_compiled(&name) {
                Ok(code) => success_callback.emit(code),
                Err(e) => failure_callback.emit(e),
            },
            _ => unreachable!(),
        }
    }

    pub fn run(&mut self, context: &Context<Self>, codes: &[Code]) {
        self.compiler_errors = None;
        self.running_codes = codes.to_vec();
        let seed: u32 = QString::from(context.link().location().unwrap().search().as_str())
            .get("seed")
            .and_then(|x| x.parse().ok())
            .unwrap_or_else(|| rand::thread_rng().gen());
        if let Some(link) = self.simulation_window_link.as_ref() {
            link.send_message(crate::simulation_window::Msg::StartSimulation {
                scenario_name: self.scenario_name.clone(),
                seed,
                codes: codes.to_vec(),
            });
        } else {
            log::error!("Missing SimulationWindow");
        }
        self.background_agents.clear();
        self.background_snapshots.clear();
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
