use crate::editor_window::EditorWindow;
use crate::filesystem;
use crate::leaderboard::Leaderboard;
use crate::simulation_window::SimulationWindow;
use crate::telemetry::Telemetry;
use crate::toolbar::Toolbar;
use gloo_render::{request_animation_frame, AnimationFrame};
use monaco::yew::CodeEditorLink;
use oort_simulator::scenario::{self, Status};
use oort_simulator::simulation::Code;
use oort_simulator::snapshot::Snapshot;
use oort_simulator::{simulation, vm};
use oort_worker::SimAgent;
use rand::Rng;
use regex::Regex;
use reqwasm::http::Request;
use serde::Deserialize;
use simulation::PHYSICS_TICK_LENGTH;
use std::collections::HashMap;
use std::rc::Rc;
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
    EditorAction { team: usize, action: String },
    ShowDocumentation,
    DismissOverlay,
    CompileFinished(Vec<Result<Code, String>>),
    CompileSlow,
    LoadedCodeFromDisk { team: usize, text: String },
    SubmitToTournament,
}

enum Overlay {
    Documentation,
    #[allow(dead_code)]
    MissionComplete,
    Compiling,
}

#[derive(Deserialize, Debug, Default)]
struct QueryParams {
    pub seed: Option<u32>,
    #[serde(default)]
    pub local: bool,
}

pub struct Game {
    render_handle: Option<AnimationFrame>,
    scenario_name: String,
    background_agents: Vec<Box<dyn Bridge<SimAgent>>>,
    background_snapshots: Vec<(u32, Snapshot)>,
    overlay: Option<Overlay>,
    overlay_ref: NodeRef,
    saw_slow_compile: bool,
    local_compiler: bool,
    compiler_errors: Option<String>,
    frame: u64,
    last_window_size: (i32, i32),
    last_snapshot: Option<Snapshot>,
    simulation_window_link: Option<Scope<SimulationWindow>>,
    teams: Vec<Team>,
    editor_links: Vec<CodeEditorLink>,
    compilation_cache: HashMap<Code, Code>,
}

pub struct Team {
    editor_link: CodeEditorLink,
    initial_source_code: Code,
    initial_compiled_code: Code,
    running_source_code: Code,
    running_compiled_code: Code,
    current_compiler_decorations: js_sys::Array,
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
        let q = parse_query_params(context);
        let local_compiler = q.local;
        if local_compiler {
            log::info!("Using local compiler");
        }

        let compilation_cache = HashMap::new();

        Self {
            render_handle,
            scenario_name: String::new(),
            background_agents: Vec::new(),
            background_snapshots: Vec::new(),
            overlay: None,
            overlay_ref: NodeRef::default(),
            saw_slow_compile: false,
            local_compiler,
            compiler_errors: None,
            frame: 0,
            last_window_size: (0, 0),
            last_snapshot: None,
            simulation_window_link: None,
            teams: Vec::new(),
            editor_links: vec![CodeEditorLink::default(), CodeEditorLink::default()],
            compilation_cache,
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
                    for editor_link in &self.editor_links {
                        editor_link.with_editor(|editor| {
                            let ed: &monaco::sys::editor::IStandaloneCodeEditor = editor.as_ref();
                            ed.layout(None);
                        });
                    }
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
                self.change_scenario(context, &scenario_name);
                true
            }
            Msg::SimulationFinished(snapshot) => self.on_simulation_finished(context, snapshot),
            Msg::EditorAction {
                team: _,
                ref action,
            } if action == "oort-execute" => {
                crate::codestorage::save(
                    &self.scenario_name,
                    &self.player_team().get_editor_code(),
                );
                for team in self.teams.iter_mut() {
                    team.running_source_code = team.get_editor_code();
                }
                self.start_compile(context);
                true
            }
            Msg::EditorAction { team, ref action } if action == "oort-restore-initial-code" => {
                let mut code = scenario::load(&self.scenario_name).initial_code()[team].clone();
                if let Code::Builtin(name) = code {
                    code = oort_simulator::vm::builtin::load_source(&name).unwrap()
                }
                self.team(team).set_editor_text(&code_to_string(&code));
                false
            }
            Msg::EditorAction { team, ref action } if action == "oort-load-solution" => {
                let mut code = scenario::load(&self.scenario_name).solution();
                if let Code::Builtin(name) = code {
                    code = oort_simulator::vm::builtin::load_source(&name).unwrap()
                }
                self.team(team).set_editor_text(&code_to_string(&code));
                false
            }
            Msg::EditorAction { team, ref action } if action == "oort-load-file" => {
                let cb = context
                    .link()
                    .callback(move |text| Msg::LoadedCodeFromDisk { team, text });
                filesystem::load(Box::new(move |text| cb.emit(text)));
                false
            }
            Msg::EditorAction { team, ref action } if action == "oort-reload-file" => {
                let cb = context
                    .link()
                    .callback(move |text| Msg::LoadedCodeFromDisk { team, text });
                filesystem::reload(Box::new(move |text| cb.emit(text)));
                false
            }
            Msg::EditorAction { team, ref action } if action == "oort-format" => {
                let text = self.team(team).get_editor_text();
                let text = crate::format::format(&text);
                self.team(team).set_editor_text(&text);
                false
            }
            Msg::EditorAction { team: _, action } => {
                log::info!("Got unexpected editor action {}", action);
                false
            }
            Msg::ReceivedBackgroundSimAgentResponse(
                oort_worker::Response::Snapshot { snapshot },
                seed,
            ) => {
                self.background_snapshots.push((seed, snapshot));
                if let Some(summary) = self.summarize_background_simulations() {
                    let code = self.player_team().running_source_code.clone();
                    crate::telemetry::send(Telemetry::FinishScenario {
                        scenario_name: self.scenario_name.clone(),
                        code: code_to_string(&code),
                        ticks: (summary.average_time.unwrap_or(0.0)
                            / simulation::PHYSICS_TICK_LENGTH)
                            as u32,
                        code_size: crate::code_size::calculate(&code_to_string(&code)),
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
            Msg::CompileFinished(results) => {
                if matches!(self.overlay, Some(Overlay::Compiling)) {
                    self.overlay = None;
                }
                if self.compilation_cache.len() > 10 {
                    self.compilation_cache.clear();
                }
                for (team, result) in results.iter().enumerate() {
                    match result {
                        Ok(code) => {
                            self.team_mut(team).display_compiler_errors(&[]);
                            self.team_mut(team).running_compiled_code = code.clone();
                            self.compilation_cache
                                .insert(self.team(team).running_source_code.clone(), code.clone());
                        }
                        Err(error) => {
                            self.team_mut(team)
                                .display_compiler_errors(&make_editor_errors(error));
                            self.team_mut(team).running_compiled_code = Code::None;
                        }
                    }
                }
                let errors: Vec<_> = results
                    .iter()
                    .filter_map(|x| x.as_ref().err())
                    .cloned()
                    .collect();
                if errors.is_empty() {
                    crate::telemetry::send(Telemetry::StartScenario {
                        scenario_name: self.scenario_name.clone(),
                        code: code_to_string(&self.player_team().running_source_code),
                    });
                    self.run(context);
                } else {
                    self.compiler_errors = Some(errors.join("\n"));
                }
                true
            }
            Msg::CompileSlow => {
                self.saw_slow_compile = true;
                false
            }
            Msg::LoadedCodeFromDisk { team, text } => {
                self.team(team).set_editor_text(&text);
                false
            }
            Msg::SubmitToTournament => {
                crate::telemetry::send(Telemetry::SubmitToTournament {
                    scenario_name: self.scenario_name.clone(),
                    code: code_to_string(&self.player_team().running_source_code),
                });
                false
            }
        }
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        // For Toolbar
        let navigator = context.link().navigator().unwrap();
        let select_scenario_cb = context.link().callback(move |e: Event| {
            let target: EventTarget = e
                .target()
                .expect("Event should have a target when dispatched");
            let data = target.unchecked_into::<HtmlInputElement>().value();
            navigator.push(&crate::Route::Scenario {
                scenario: data.clone(),
            });
            Msg::SelectScenario(data)
        });
        let show_documentation_cb = context.link().callback(|_| Msg::ShowDocumentation);

        // For EditorWindow 0
        let editor_window0_host = gloo_utils::document()
            .get_element_by_id("editor-window-0")
            .expect("a #editor-window element");
        let editor0_link = self.editor_links[0].clone();
        let on_editor0_action = context
            .link()
            .callback(|action| Msg::EditorAction { team: 0, action });

        // For EditorWindow 1
        let editor_window1_host = gloo_utils::document()
            .get_element_by_id("editor-window-1")
            .expect("a #editor-window element");
        let editor1_link = self.editor_links[1].clone();
        let on_editor1_action = context
            .link()
            .callback(|action| Msg::EditorAction { team: 1, action });

        // For SimulationWindow
        let simulation_window_host = gloo_utils::document()
            .get_element_by_id("simulation-window")
            .expect("a #simulation-window element");
        let on_simulation_finished = context.link().callback(Msg::SimulationFinished);
        let register_link = context.link().callback(Msg::RegisterSimulationWindowLink);
        let version = context.props().version.clone();
        let compiler_errors = self.compiler_errors.clone();

        html! {
        <>
            <Toolbar scenario_name={self.scenario_name.clone()} {select_scenario_cb} {show_documentation_cb} />
            <EditorWindow host={editor_window0_host} editor_link={editor0_link} on_editor_action={on_editor0_action} />
            <EditorWindow host={editor_window1_host} editor_link={editor1_link} on_editor_action={on_editor1_action} />
            <SimulationWindow host={simulation_window_host} {on_simulation_finished} {register_link} {version} {compiler_errors} />
            { self.render_overlay(context) }
        </>
        }
    }

    fn rendered(&mut self, _context: &yew::Context<Self>, _first_render: bool) {
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
    fn on_simulation_finished(&mut self, context: &yew::Context<Self>, snapshot: Snapshot) -> bool {
        let status = snapshot.status;

        if !snapshot.errors.is_empty() {
            self.compiler_errors = Some(format!("Simulation errors: {:?}", snapshot.errors));
            return true;
        }

        if context.props().demo && status != Status::Running {
            context
                .link()
                .send_message(Msg::SelectScenario(context.props().scenario.clone()));
            return false;
        }

        if self.leaderboard_eligible() {
            if let Status::Victory { team: 0 } = status {
                self.background_agents.clear();
                self.background_snapshots.clear();
                let codes: Vec<_> = self
                    .teams
                    .iter()
                    .map(|x| x.running_compiled_code.clone())
                    .collect();
                for seed in 0..10 {
                    let cb = {
                        let link = context.link().clone();
                        move |e| link.send_message(Msg::ReceivedBackgroundSimAgentResponse(e, seed))
                    };
                    let mut sim_agent = SimAgent::bridge(Rc::new(cb));
                    sim_agent.send(oort_worker::Request::RunScenario {
                        scenario_name: self.scenario_name.to_owned(),
                        seed,
                        codes: codes.clone(),
                    });
                    self.background_agents.push(sim_agent);
                }

                self.overlay = Some(Overlay::MissionComplete);
            }
        }

        self.last_snapshot = Some(snapshot);
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
        let code_size =
            crate::code_size::calculate(&code_to_string(&self.player_team().running_source_code));

        let next_scenario = scenario::load(&self.scenario_name).next_scenario();

        let make_seed_link_cb = |seed: u32| {
            let navigator = context.link().navigator().unwrap();
            let scenario_name = self.scenario_name.clone();
            context.link().batch_callback(move |_| {
                let mut query = std::collections::HashMap::<&str, String>::new();
                query.insert("seed", seed.to_string());
                navigator
                    .push_with_query(
                        &crate::Route::Scenario {
                            scenario: scenario_name.clone(),
                        },
                        &query,
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

    pub fn start_compile(&mut self, context: &Context<Self>) {
        self.compiler_errors = None;
        self.overlay = Some(Overlay::Compiling);

        let finished_callback = context.link().callback(Msg::CompileFinished);
        let slow_compile_callback = context.link().callback(|_| Msg::CompileSlow);

        let url = if self.local_compiler {
            "http://localhost:8081/compile"
        } else if self.saw_slow_compile {
            "https://api.oort.rs/compile"
        } else {
            "https://api-vm.oort.rs/compile"
        };

        async fn compile(
            url: &str,
            text: String,
            slow_compile_cb: Callback<()>,
        ) -> Result<Code, String> {
            let start_time = instant::Instant::now();
            let check_compile_time = || {
                let elapsed = instant::Instant::now() - start_time;
                if elapsed > std::time::Duration::from_millis(3000) {
                    log::info!("Compilation was slow, switching backend to serverless");
                    slow_compile_cb.emit(());
                }
            };

            let result = Request::post(url).body(text).send().await;
            if let Err(e) = result {
                log::error!("Compile error: {}", e);
                check_compile_time();
                return Err(e.to_string());
            }

            let response = result.unwrap();
            if !response.ok() {
                let error = response.text().await.unwrap();
                log::error!("Compile error: {}", error);
                check_compile_time();
                return Err(error);
            }

            let wasm = response.binary().await;
            if let Err(e) = wasm {
                log::error!("Compile error: {}", e);
                check_compile_time();
                return Err(e.to_string());
            }

            let elapsed = instant::Instant::now() - start_time;
            log::info!("Compile succeeded in {:?}", elapsed);
            check_compile_time();
            Ok(Code::Wasm(wasm.unwrap()))
        }

        let source_codes: Vec<_> = self
            .teams
            .iter()
            .map(|team| {
                if team.running_source_code == team.initial_source_code
                    && team.initial_compiled_code != Code::None
                {
                    team.initial_compiled_code.clone()
                } else if let Some(compiled_code) =
                    self.compilation_cache.get(&team.running_source_code)
                {
                    compiled_code.clone()
                } else {
                    team.running_source_code.clone()
                }
            })
            .collect();

        wasm_bindgen_futures::spawn_local(async move {
            let mut results = vec![];
            for source_code in source_codes {
                let result = match source_code {
                    Code::Rust(text) => compile(url, text, slow_compile_callback.clone()).await,
                    Code::Builtin(name) => oort_simulator::vm::builtin::load_compiled(&name),
                    other => Ok(other),
                };
                results.push(result);
            }
            finished_callback.emit(results);
        });
    }

    pub fn run(&mut self, context: &Context<Self>) {
        self.compiler_errors = None;

        let codes: Vec<_> = self
            .teams
            .iter()
            .map(|x| x.running_compiled_code.clone())
            .collect();

        let q = parse_query_params(context);
        let seed = q.seed.unwrap_or_else(|| rand::thread_rng().gen());

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

    pub fn change_scenario(&mut self, context: &Context<Self>, scenario_name: &str) {
        if !self.teams.is_empty() {
            crate::codestorage::save(&self.scenario_name, &self.player_team().get_editor_code());
        }
        self.scenario_name = scenario_name.to_string();
        let codes = crate::codestorage::load(&self.scenario_name);
        let scenario = oort_simulator::scenario::load(&self.scenario_name);

        let to_source_code = |code: &Code| match code {
            Code::Builtin(name) => oort_simulator::vm::builtin::load_source(name).unwrap(),
            _ => code.clone(),
        };

        let mut player_team = Team::new(self.editor_links[0].clone());
        player_team.initial_source_code = to_source_code(&codes[0]);

        let mut enemy_team = Team::new(self.editor_links[1].clone());
        enemy_team.initial_source_code = to_source_code(&codes[1]);
        enemy_team.running_source_code = to_source_code(&codes[1]);
        enemy_team.initial_compiled_code = codes[1].clone();
        enemy_team.running_compiled_code = codes[1].clone();

        if context.props().demo || self.scenario_name == "welcome" {
            let solution = scenario.solution();
            player_team.initial_source_code = to_source_code(&solution);
            player_team.running_source_code = player_team.initial_source_code.clone();
            player_team.running_compiled_code = solution;
        };

        if self.scenario_name == "welcome" {
            player_team.initial_source_code = Code::Rust(
                "\
// Welcome to Oort.
// Select a scenario from the list in the top-right of the page.
// If you're new, start with \"tutorial01\"."
                    .to_string(),
            );
        }

        player_team.set_editor_text(&code_to_string(&player_team.initial_source_code));
        enemy_team.set_editor_text(&code_to_string(&enemy_team.initial_source_code));
        self.teams = vec![player_team, enemy_team];
        self.run(context);
    }

    pub fn team(&self, index: usize) -> &Team {
        self.teams.get(index).expect("Invalid team")
    }

    pub fn team_mut(&mut self, index: usize) -> &mut Team {
        self.teams.get_mut(index).expect("Invalid team")
    }

    pub fn player_team(&self) -> &Team {
        self.team(0)
    }

    pub fn leaderboard_eligible(&self) -> bool {
        for team in &self.teams.as_slice()[1..] {
            if team.running_source_code != team.initial_source_code {
                log::info!("Not eligible for leaderboard due to modified opponent code");
                return false;
            }
        }
        true
    }
}

impl Team {
    pub fn new(editor_link: CodeEditorLink) -> Self {
        Self {
            editor_link,
            initial_source_code: Code::None,
            running_source_code: Code::None,
            initial_compiled_code: Code::None,
            running_compiled_code: Code::None,
            current_compiler_decorations: js_sys::Array::new(),
        }
    }

    pub fn get_editor_text(&self) -> String {
        self.editor_link
            .with_editor(|editor| editor.get_model().unwrap().get_value())
            .unwrap()
    }

    pub fn get_editor_code(&self) -> Code {
        str_to_code(&self.get_editor_text())
    }

    pub fn set_editor_text(&self, text: &str) {
        self.editor_link.with_editor(|editor| {
            editor.get_model().unwrap().set_value(text);
        });
        // TODO trigger analyzer run
    }

    pub fn display_compiler_errors(&mut self, errors: &[vm::Error]) {
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

fn parse_query_params(context: &Context<Game>) -> QueryParams {
    let location = context.link().location().unwrap();
    match location.query::<QueryParams>() {
        Ok(q) => q,
        Err(e) => {
            log::info!("Failed to parse query params: {:?}", e);
            Default::default()
        }
    }
}

fn make_editor_errors(error: &str) -> Vec<vm::Error> {
    let re = Regex::new(r"(?m)error.*?: (.*?)$\n.*?ai/src/user.rs:(\d+):").unwrap();
    re.captures_iter(error)
        .map(|m| vm::Error {
            line: m[2].parse().unwrap(),
            msg: m[1].to_string(),
        })
        .collect()
}
