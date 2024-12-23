use crate::codestorage;
use crate::compiler_output_window::CompilerOutputWindow;
use crate::documentation::Documentation;
use crate::editor_window::{EditorAction, EditorWindow};
use crate::gtag;
use crate::js;
use crate::leaderboard::Leaderboard;
use crate::leaderboard_window::LeaderboardWindow;
use crate::query_params;
use crate::seed_window::SeedWindow;
use crate::services;
use crate::simulation_window::SimulationWindow;
use crate::toolbar::Toolbar;
use crate::userid;
use crate::versions_window::VersionsWindow;
use crate::welcome::Welcome;
use monaco::yew::CodeEditorLink;
use oort_proto::{LeaderboardSubmission, Telemetry};
use oort_simulation_worker::SimAgent;
use oort_simulator::scenario::{self, Status, MAX_TICKS};
use oort_simulator::simulation;
use oort_simulator::simulation::Code;
use oort_simulator::snapshot::Snapshot;
use rand::Rng;
use regex::Regex;
use reqwasm::http::Request;
use simulation::PHYSICS_TICK_LENGTH;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{EventTarget, HtmlInputElement};
use yew::events::Event;
use yew::html::Scope;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;

const NUM_BACKGROUND_SIMULATIONS: u32 = 10;

fn empty() -> JsValue {
    js_sys::Object::new().into()
}

#[derive(Debug)]
pub enum Msg {
    RegisterSimulationWindowLink(Scope<SimulationWindow>),
    Start,
    SimulationFinished(Snapshot),
    ReceivedBackgroundSimAgentResponse(oort_simulation_worker::Response, u32),
    EditorAction { team: usize, action: String },
    ShowFeedback,
    DismissOverlay,
    CompileFinished(Vec<Result<Code, String>>, ExecutionMode),
    SubmitToTournament,
    UploadShortcode,
    FormattedCode { team: usize, text: String },
    ReplaceCode { team: usize, text: String },
    ShowError(String),
    Resized,
    LoadVersion(String),
    SaveVersion(String),
    RefreshVersions,
    Nop,
}

enum Overlay {
    #[allow(dead_code)]
    MissionComplete,
    Compiling,
    Feedback,
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutionMode {
    Initial,
    Run,
    Replay { paused: bool },
}

pub struct Game {
    background_agents: Vec<Box<dyn Bridge<SimAgent>>>,
    background_snapshots: Vec<(u32, Snapshot)>,
    background_nonce: u32,
    overlay: Option<Overlay>,
    overlay_ref: NodeRef,
    simulation_canvas_ref: NodeRef,
    compiler_errors: Option<String>,
    last_window_size: (i32, i32),
    last_snapshot: Option<Snapshot>,
    simulation_window_link: Option<Scope<SimulationWindow>>,
    teams: Vec<Team>,
    editor_links: Vec<CodeEditorLink>,
    compilation_cache: HashMap<Code, Code>,
    previous_seed: Option<u32>,
    versions_update_timestamp: chrono::DateTime<chrono::Utc>,
    execution_mode: ExecutionMode,
}

pub struct Team {
    editor_link: CodeEditorLink,
    initial_source_code: Code,
    initial_compiled_code: Code,
    running_source_code: Code,
    running_compiled_code: Code,
    current_compiler_decorations: js_sys::Array,
}

#[derive(Properties, PartialEq, Eq, Debug)]
pub struct Props {
    pub scenario: String,
    #[prop_or_default]
    pub demo: bool,
    pub version: String,
    pub seed: Option<u32>,
    pub player0: Option<String>,
    pub player1: Option<String>,
}

impl Component for Game {
    type Message = Msg;
    type Properties = Props;

    fn create(context: &yew::Context<Self>) -> Self {
        js::golden_layout::init();

        {
            let link = context.link().clone();
            let closure = Closure::new(move || link.send_message(Msg::Resized));
            crate::js::resize::start(&closure);
            closure.forget();
        }

        let compilation_cache = HashMap::new();

        Self {
            background_agents: Vec::new(),
            background_snapshots: Vec::new(),
            background_nonce: 0,
            overlay: None,
            overlay_ref: NodeRef::default(),
            simulation_canvas_ref: NodeRef::default(),
            compiler_errors: None,
            last_window_size: (0, 0),
            last_snapshot: None,
            simulation_window_link: None,
            teams: Vec::new(),
            editor_links: vec![CodeEditorLink::default(), CodeEditorLink::default()],
            compilation_cache,
            previous_seed: None,
            versions_update_timestamp: chrono::Utc::now(),
            execution_mode: ExecutionMode::Initial,
        }
    }

    fn update(&mut self, context: &yew::Context<Self>, msg: Self::Message) -> bool {
        log::debug!("Received update {:?}", msg);
        match msg {
            Msg::RegisterSimulationWindowLink(link) => {
                self.simulation_window_link = Some(link);
                context.link().send_message(Msg::Start);
                false
            }
            Msg::Start => {
                let shortcodes = [
                    context.props().player0.clone(),
                    context.props().player1.clone(),
                ];
                let has_shortcodes = !shortcodes.iter().all(Option::is_none);
                log::info!("has shortcodes {}", has_shortcodes);
                self.change_scenario(context, &context.props().scenario, !has_shortcodes);
                if has_shortcodes {
                    context.link().send_future_batch(async move {
                        let mut msgs = vec![];
                        for (team, shortcode) in shortcodes.iter().enumerate() {
                            if let Some(shortcode) = shortcode {
                                match services::get_shortcode(shortcode).await {
                                    Ok(text) => msgs.push(Msg::ReplaceCode { team, text }),
                                    Err(e) => {
                                        msgs.push(Msg::ShowError(format!(
                                            "Failed to get shortcode: {e:?}"
                                        )));
                                        return msgs;
                                    }
                                }
                            }
                        }
                        msgs.push(Msg::EditorAction {
                            team: 0,
                            action: "oort-execute".to_string(),
                        }); // TODO
                        msgs
                    });
                }
                true
            }
            Msg::SimulationFinished(snapshot) => self.on_simulation_finished(context, snapshot),
            Msg::EditorAction {
                team: _,
                ref action,
            } if action == "oort-execute" => {
                self.save_current_code(context, &context.props().scenario, None);
                for team in self.teams.iter_mut() {
                    team.running_source_code = team.get_editor_code();
                }
                self.start_compile(context, ExecutionMode::Run);
                true
            }
            Msg::EditorAction {
                team: _,
                ref action,
            } if action == "oort-replay" => {
                self.save_current_code(context, &context.props().scenario, None);
                for team in self.teams.iter_mut() {
                    team.running_source_code = team.get_editor_code();
                }
                self.start_compile(context, ExecutionMode::Replay { paused: false });
                true
            }
            Msg::EditorAction {
                team: _,
                ref action,
            } if action == "oort-replay-paused" => {
                self.save_current_code(context, &context.props().scenario, None);
                for team in self.teams.iter_mut() {
                    team.running_source_code = team.get_editor_code();
                }
                self.start_compile(context, ExecutionMode::Replay { paused: true });
                true
            }
            Msg::EditorAction { team, ref action } if action == "oort-restore-initial-code" => {
                let mut code = scenario::load(&context.props().scenario)
                    .initial_code()
                    .get(team)
                    .unwrap_or(&Code::None)
                    .clone();
                if let Code::Builtin(name) = code {
                    code = oort_simulator::vm::builtin::load_source(&name).unwrap()
                }
                self.team(team).set_editor_text(&code_to_string(&code));
                false
            }
            Msg::EditorAction { team, ref action } if action == "oort-load-solution" => {
                let mut code = scenario::load(&context.props().scenario).solution();
                if let Code::Builtin(name) = code {
                    code = oort_simulator::vm::builtin::load_source(&name).unwrap()
                }
                self.team(team).set_editor_text(&code_to_string(&code));
                false
            }
            Msg::EditorAction { team, ref action } if action == "oort-format" => {
                let text = self.team(team).get_editor_text();
                let cb = context
                    .link()
                    .callback(move |text| Msg::FormattedCode { team, text });
                services::format(text, cb);
                false
            }
            Msg::EditorAction {
                team: _,
                ref action,
            } if action == "oort-submit-to-tournament" => {
                let scenario_name = context.props().scenario.clone();
                let source_code = self.player_team().get_editor_text();
                services::send_telemetry(Telemetry::SubmitToTournament {
                    scenario_name: scenario_name.clone(),
                    code: source_code.clone(),
                });
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) =
                        services::submit_to_tournament(&scenario_name, &source_code).await
                    {
                        log::error!("Error submitting to tournament: {}", e);
                    }
                });
                false
            }
            Msg::EditorAction { team: _, action } => {
                log::info!("Got unexpected editor action {}", action);
                false
            }
            Msg::ReceivedBackgroundSimAgentResponse(
                oort_simulation_worker::Response::Snapshot { snapshot },
                seed,
            ) => {
                if snapshot.nonce == self.background_nonce {
                    if snapshot.status == Status::Running
                        && snapshot.time < (MAX_TICKS as f64 * PHYSICS_TICK_LENGTH)
                    {
                        if !self.background_agents.is_empty() {
                            self.background_agents[seed as usize].send(
                                oort_simulation_worker::Request::Snapshot {
                                    ticks: 100,
                                    nonce: self.background_nonce,
                                },
                            );
                        }
                        false
                    } else {
                        self.background_snapshots.push((seed, snapshot));
                        if let Some(summary) =
                            self.summarize_background_simulations(&context.props().scenario)
                        {
                            let code = self.player_team().running_source_code.clone();
                            services::send_telemetry(Telemetry::FinishScenario {
                                scenario_name: context.props().scenario.clone(),
                                code: code_to_string(&code),
                                ticks: (summary.average_time.unwrap_or(0.0)
                                    / simulation::PHYSICS_TICK_LENGTH)
                                    as u32,
                                code_size: crate::code_size::calculate(&code_to_string(&code)),
                                success: summary.failed_seeds.is_empty(),
                                time: summary.average_time,
                            });
                            if summary.failed_seeds.is_empty() {
                                if let Some(average_time) = summary.average_time {
                                    self.save_current_code(
                                        context,
                                        context.props().scenario.as_str(),
                                        Some(format!("{:.3} seconds", average_time)),
                                    );
                                }
                            }
                        }
                        true
                    }
                } else {
                    false
                }
            }
            Msg::ShowFeedback => {
                self.overlay = Some(Overlay::Feedback);
                true
            }
            Msg::ShowError(e) => {
                self.overlay = Some(Overlay::Error(e));
                true
            }
            Msg::DismissOverlay => {
                self.overlay = None;
                self.background_agents.clear();
                self.background_snapshots.clear();
                self.background_nonce = 0;
                self.focus_editor(0);
                true
            }
            Msg::CompileFinished(results, execution_mode) => {
                if matches!(self.overlay, Some(Overlay::Compiling)) {
                    self.overlay = None;
                }

                // TODO: Smarter cache eviction policy
                if self.compilation_cache.len() > 10 {
                    self.compilation_cache.clear();
                }
                let mut teams_with_errors = vec![];

                // Display compiler errors or cache compilation results
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
                            teams_with_errors.push(team);
                        }
                    }
                }
                let errors: Vec<_> = results
                    .iter()
                    .filter_map(|x| x.as_ref().err())
                    .cloned()
                    .collect();
                if errors.is_empty() {
                    // If no errors, start running simulation
                    services::send_telemetry(Telemetry::StartScenario {
                        scenario_name: context.props().scenario.clone(),
                        code: code_to_string(&self.player_team().running_source_code),
                    });
                    self.run(context, execution_mode);
                    self.focus_simulation();
                } else {
                    // Populate compiler output with compilation errors and focus compiler output tab
                    self.compiler_errors = Some(errors.join("\n"));
                    self.focus_editor(teams_with_errors[0]);
                    js::golden_layout::select_tab("compiler_output");
                }
                true
            }
            Msg::FormattedCode { team, text } => {
                self.team(team).set_editor_text_preserving_cursor(&text);
                false
            }
            Msg::ReplaceCode { team, text } => {
                self.team(team).set_editor_text(&text);
                false
            }
            Msg::LoadVersion(id) => {
                self.save_current_code(context, &context.props().scenario, None);
                self.focus_editor(0);
                try_send_future(context.link(), async move {
                    let version_control = oort_version_control::VersionControl::new().await?;
                    let version = version_control.get_version(&id).await?;
                    let code = version_control.get_code(&version.digest).await?;
                    Ok::<_, oort_version_control::Error>(Msg::ReplaceCode {
                        team: 0,
                        text: code,
                    })
                });
                false
            }
            Msg::SaveVersion(label) => {
                self.save_current_code(context, &context.props().scenario, Some(label));
                false
            }
            Msg::RefreshVersions => {
                self.versions_update_timestamp = chrono::Utc::now();
                true
            }
            Msg::SubmitToTournament => {
                services::send_telemetry(Telemetry::SubmitToTournament {
                    scenario_name: context.props().scenario.clone(),
                    code: code_to_string(&self.player_team().running_source_code),
                });
                let scenario_name = context.props().scenario.clone();
                let code = code_to_string(&self.player_team().running_source_code);
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) = services::submit_to_tournament(&scenario_name, &code).await {
                        log::error!("Error submitting to tournament: {}", e);
                    }
                });
                false
            }
            Msg::UploadShortcode => {
                let code = code_to_string(&self.player_team().running_source_code);
                wasm_bindgen_futures::spawn_local(async move {
                    match services::upload_shortcode(&code).await {
                        Ok(shortcode) => {
                            log::info!("Got shortcode {}", shortcode);
                            crate::js::clipboard::write(&shortcode);
                        }
                        Err(e) => {
                            log::error!("Error uploading to shortcode: {}", e);
                        }
                    }
                });
                false
            }
            Msg::Resized => {
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
                false
            }
            Msg::Nop => false,
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
            Msg::Nop
        });
        let show_feedback_cb = context.link().callback(|_| Msg::ShowFeedback);

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

        // For Welcome
        let welcome_window_host = gloo_utils::document()
            .get_element_by_id("welcome-window")
            .expect("a #welcome-window element");
        let navigator = context.link().navigator().unwrap();
        let select_scenario_cb2 = context.link().batch_callback(move |name: String| {
            navigator.push(&crate::Route::Scenario {
                scenario: name.clone(),
            });
            vec![Msg::DismissOverlay]
        });

        // For Documentation.
        let documentation_window_host = gloo_utils::document()
            .get_element_by_id("documentation-window")
            .expect("a #documentation-window element");

        // For CompilerOutput.
        let compiler_output_window_host = gloo_utils::document()
            .get_element_by_id("compiler-output-window")
            .expect("a #compiler-output-window element");
        let compiler_errors = self.compiler_errors.clone();

        // For LeaderboardWindow.
        let leaderboard_window_host = gloo_utils::document()
            .get_element_by_id("leaderboard-window")
            .expect("a #leaderboard-window element");
        let play_cb = {
            let link = context.link().clone();
            let navigator = context.link().navigator().unwrap();
            let scenario_name = context.props().scenario.clone();
            context.link().batch_callback(move |args: (i32, String)| {
                let (team, shortcode) = args;
                let location = link.location().expect("location");
                let mut query = query_params(&location);
                if team == 0 {
                    query.player0 = Some(shortcode);
                } else if team == 1 {
                    query.player1 = Some(shortcode);
                }
                navigator
                    .push_with_query(
                        &crate::Route::Scenario {
                            scenario: scenario_name.clone(),
                        },
                        &query,
                    )
                    .unwrap();
                vec![]
            })
        };

        // For VersionsWindow.
        let versions_window_host = gloo_utils::document()
            .get_element_by_id("versions-window")
            .expect("a #versions-window element");
        let load_cb = context.link().callback(Msg::LoadVersion);
        let save_cb = context.link().callback(Msg::SaveVersion);

        // For SeedWindow.
        let seed_window_host = gloo_utils::document()
            .get_element_by_id("seed-window")
            .expect("a #seed-window element");
        let current_seed = self
            .configured_seed(context)
            .unwrap_or(self.previous_seed.unwrap_or(0));
        let change_seed_cb = {
            let scenario_name = context.props().scenario.clone();
            let navigator = context.link().navigator().unwrap();
            let link = context.link().clone();
            context.link().callback(move |seed: Option<u32>| {
                let location = link.location().expect("location");
                let mut query = query_params(&location);
                query.seed = seed;
                navigator
                    .push_with_query(
                        &crate::Route::Scenario {
                            scenario: scenario_name.clone(),
                        },
                        &query,
                    )
                    .unwrap();
                Msg::DismissOverlay
            })
        };

        let on_simulation_editor_action_cb = context.link().callback(|action| Msg::EditorAction {
            team: 0,
            action: match action {
                EditorAction::Execute => "oort-execute".to_string(),
                EditorAction::Replay => "oort-replay".to_string(),
                EditorAction::ReplayPaused => "oort-replay-paused".to_string(),
            },
        });

        html! {
        <>
            <Toolbar scenario_name={context.props().scenario.clone()} {select_scenario_cb} show_feedback_cb={show_feedback_cb.clone()} />
            <Welcome host={welcome_window_host} show_feedback_cb={show_feedback_cb.clone()} select_scenario_cb={select_scenario_cb2} />
            <EditorWindow host={editor_window0_host} editor_link={editor0_link} on_editor_action={on_editor0_action} team=0 scenario={context.props().scenario.clone()} />
            <EditorWindow host={editor_window1_host} editor_link={editor1_link} on_editor_action={on_editor1_action} team=1 scenario={context.props().scenario.clone()} />
            <SimulationWindow host={simulation_window_host} {on_simulation_finished} {register_link} on_editor_action={on_simulation_editor_action_cb} {version} canvas_ref={self.simulation_canvas_ref.clone()} />
            <Documentation host={documentation_window_host} {show_feedback_cb} />
            <CompilerOutputWindow host={compiler_output_window_host} {compiler_errors} />
            <LeaderboardWindow host={leaderboard_window_host} scenario_name={context.props().scenario.clone()} {play_cb} />
            <VersionsWindow host={versions_window_host} scenario_name={context.props().scenario.clone()} {load_cb} {save_cb} update_timestamp={self.versions_update_timestamp} />
            <SeedWindow host={seed_window_host} {current_seed} change_cb={change_seed_cb} />
            { self.render_overlay(context) }
        </>
        }
    }

    fn rendered(&mut self, context: &yew::Context<Self>, first_render: bool) {
        if self.overlay.is_some() {
            self.focus_overlay();
        } else if first_render && context.props().scenario != "welcome" {
            self.focus_editor(0);
        }
    }

    fn changed(&mut self, context: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = context.props();
        if props == old_props {
            return false;
        }

        if props.player0 != old_props.player0 {
            self.save_current_code(context, &props.scenario, None);
        }

        if props.scenario != old_props.scenario {
            self.save_current_code(context, &old_props.scenario, None);
            self.change_scenario(context, &props.scenario, true);
            return true;
        }

        if props.player0 != old_props.player0 || props.player1 != old_props.player1 {
            context.link().send_message(Msg::Start);
            return true;
        }

        self.run(context, ExecutionMode::Initial);

        true
    }
}

struct BackgroundSimSummary {
    count: usize,
    victory_count: usize,
    failed_seeds: Vec<u32>,
    average_time: Option<f64>,
    best_seed: Option<u32>,
    worst_seed: Option<u32>,
    scenario_name: String,
}

impl Game {
    fn on_simulation_finished(&mut self, context: &yew::Context<Self>, snapshot: Snapshot) -> bool {
        let status = snapshot.status;

        if !snapshot.errors.is_empty() {
            self.compiler_errors = Some(format!("Simulation errors: {:?}", snapshot.errors));
            return true;
        }

        if context.props().demo && status != Status::Running {
            self.run(context, ExecutionMode::Run);
            return false;
        }

        if self.execution_mode == ExecutionMode::Run {
            if let Status::Victory { team: 0 } = status {
                self.background_agents.clear();
                self.background_snapshots.clear();
                self.background_nonce = rand::thread_rng().gen();
                let codes: Vec<_> = self
                    .teams
                    .iter()
                    .map(|x| x.running_compiled_code.clone())
                    .collect();
                for seed in 0..NUM_BACKGROUND_SIMULATIONS {
                    let cb = {
                        let link = context.link().clone();
                        move |e| link.send_message(Msg::ReceivedBackgroundSimAgentResponse(e, seed))
                    };
                    let mut sim_agent = SimAgent::bridge(Rc::new(cb));
                    sim_agent.send(oort_simulation_worker::Request::StartScenario {
                        scenario_name: context.props().scenario.clone(),
                        seed,
                        codes: codes.clone(),
                        nonce: self.background_nonce,
                    });
                    self.background_agents.push(sim_agent);
                }

                self.overlay = Some(Overlay::MissionComplete);
                gtag::mission_complete(&context.props().scenario);
            }
        }

        self.last_snapshot = Some(snapshot);
        true
    }

    fn render_overlay(&self, context: &yew::Context<Self>) -> Html {
        let outer_click_cb = context.link().callback(|_| Msg::DismissOverlay);
        let close_overlay_cb = context.link().callback(|_| Msg::DismissOverlay);
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
                        Some(Overlay::MissionComplete) => self.render_mission_complete_overlay(context),
                        Some(Overlay::Compiling) => html! { <h1 class="compiling">{ "Compiling..." }</h1> },
                        Some(Overlay::Feedback) => html! { <crate::feedback::Feedback {close_overlay_cb} /> },
                        Some(Overlay::Error(e)) => html! { <><h1>{ "Error" }</h1><span>{ e }</span></> },
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

    fn focus_editor(&self, team: usize) {
        assert!(team < 2);
        let tab = if team == 0 {
            "editor.player"
        } else {
            "editor.opponent"
        };
        let link = self.editor_links[team].clone();
        let cb = Closure::once_into_js(move || {
            js::golden_layout::select_tab(tab);
            link.with_editor(|editor| editor.as_ref().focus());
        });
        gloo_utils::window()
            .set_timeout_with_callback(&cb.into())
            .unwrap();
    }

    fn focus_simulation(&self) {
        let canvas_ref = self.simulation_canvas_ref.clone();
        let cb = Closure::once_into_js(move || {
            js::golden_layout::select_tab("simulation");
            if let Some(element) = canvas_ref.cast::<web_sys::HtmlElement>() {
                element.focus().expect("focusing simulation canvas");
            }
        });
        gloo_utils::window()
            .set_timeout_with_callback(&cb.into())
            .unwrap();
    }

    fn summarize_background_simulations(
        &self,
        scenario_name: &str,
    ) -> Option<BackgroundSimSummary> {
        if self
            .background_snapshots
            .iter()
            .any(|(_, snapshot)| snapshot.nonce != self.background_nonce)
        {
            log::error!("Found unexpected nonce");
            return None;
        }

        let expected_seeds: Vec<u32> = (0..NUM_BACKGROUND_SIMULATIONS).collect();
        let mut found_seeds: Vec<u32> = self
            .background_snapshots
            .iter()
            .map(|(seed, _)| *seed)
            .collect();
        found_seeds.sort();
        if expected_seeds != found_seeds {
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
                    .map(|(_, snapshot)| snapshot.score_time)
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
            .map(|(seed, snapshot)| (*seed, snapshot.score_time))
            .collect();
        victory_seeds_by_time.sort_by_key(|(_, time)| (time / PHYSICS_TICK_LENGTH) as i64);
        let best_seed = victory_seeds_by_time.first().map(|(seed, _)| *seed);
        let mut worst_seed = victory_seeds_by_time.last().map(|(seed, _)| *seed);
        if worst_seed == best_seed {
            worst_seed = None;
        }

        Some(BackgroundSimSummary {
            count: found_seeds.len(),
            victory_count,
            failed_seeds,
            average_time,
            best_seed,
            worst_seed,
            scenario_name: scenario_name.to_owned(),
        })
    }

    fn render_mission_complete_overlay(&self, context: &yew::Context<Self>) -> Html {
        let score_time = if let Some(snapshot) = self.last_snapshot.as_ref() {
            snapshot.score_time
        } else {
            0.0
        };
        let source_code = code_to_string(&self.player_team().running_source_code);
        let code_size = crate::code_size::calculate(&source_code);
        let leaderboard_eligible = self.leaderboard_eligible();

        let next_scenario = scenario::load(&context.props().scenario).next_scenario();

        let make_seed_link_cb = |seed: u32| {
            let link = context.link().clone();
            let navigator = context.link().navigator().unwrap();
            let scenario_name = context.props().scenario.clone();
            context.link().batch_callback(move |_| {
                let location = link.location().expect("location");
                let mut query = query_params(&location);
                query.seed = Some(seed);
                navigator
                    .push_with_query(
                        &crate::Route::Scenario {
                            scenario: scenario_name.clone(),
                        },
                        &query,
                    )
                    .unwrap();
                vec![Msg::DismissOverlay]
            })
        };
        let make_seed_link =
            |seed| html! { <a href="#" onclick={make_seed_link_cb(seed)}>{ seed }</a> };

        let background_status = if let Some(summary) =
            self.summarize_background_simulations(&context.props().scenario)
        {
            let next_scenario_link = if summary.failed_seeds.is_empty() {
                match next_scenario {
                    Some(scenario_name) => {
                        let navigator = context.link().navigator().unwrap();
                        let next_scenario_cb = context.link().batch_callback(move |_| {
                            navigator.push(&crate::Route::Scenario {
                                scenario: scenario_name.clone(),
                            });
                            vec![Msg::DismissOverlay]
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
            let submit_button = if scenario::load(&context.props().scenario).is_tournament()
                && summary.victory_count > 0
                && !is_encrypted(&self.player_team().running_source_code)
            {
                let cb = context
                    .link()
                    .batch_callback(move |_| vec![Msg::SubmitToTournament, Msg::DismissOverlay]);
                html! {
                    <>
                        <button onclick={cb}>{ "Submit to tournament" }</button>
                        { "\u{00a0}" }  // nbsp
                    </>
                }
            } else {
                html! {}
            };
            let upload_shortcode_button = {
                if !is_encrypted(&self.player_team().running_source_code) {
                    let cb = context.link().callback(move |_| Msg::UploadShortcode);
                    html! {
                        <>
                            <button onclick={cb}>{ "Copy shortcode" }</button>
                        </>
                    }
                } else {
                    html! {}
                }
            };

            let play_cb = {
                let link = context.link().clone();
                let navigator = context.link().navigator().unwrap();
                let scenario_name = context.props().scenario.clone();
                context.link().batch_callback(move |args: (i32, String)| {
                    let (team, shortcode) = args;
                    let location = link.location().expect("location");
                    let mut query = query_params(&location);
                    if team == 0 {
                        query.player0 = Some(shortcode);
                    } else if team == 1 {
                        query.player1 = Some(shortcode);
                    }
                    navigator
                        .push_with_query(
                            &crate::Route::Scenario {
                                scenario: scenario_name.clone(),
                            },
                            &query,
                        )
                        .unwrap();
                    vec![Msg::DismissOverlay]
                })
            };
            let leaderboard_submission = (leaderboard_eligible && summary.failed_seeds.is_empty())
                .then(|| LeaderboardSubmission {
                    userid: userid::get_userid(),
                    username: userid::get_username(),
                    timestamp: chrono::Utc::now(),
                    scenario_name: summary.scenario_name.clone(),
                    code: source_code.clone(),
                    code_size,
                    time: summary.average_time.unwrap(),
                    rescored_version: None,
                });
            html! {
                <>
                    <span>{ "Simulations complete: " }{ summary.victory_count }{"/"}{ summary.count }{ " successful" }</span><br />
                    <span>
                        { "Average time: " }
                        {
                            if let Some(average_time) = summary.average_time {
                                format!("{average_time:.3} seconds")
                            } else {
                                "none".to_string()
                            }
                        }
                    </span>
                    { failures }
                    { best_and_worst_seeds }
                    <br />
                    { submit_button }
                    { upload_shortcode_button }
                    <br />
                    { next_scenario_link }
                    <br />
                    {
                        if leaderboard_eligible { html! { <Leaderboard scenario_name={ context.props().scenario.clone() } submission={leaderboard_submission} {play_cb} /> } }
                        else { html! { <p>{ "Leaderboard skipped due to modified opponent code" }</p> } }
                    }
                </>
            }
        } else {
            html! { <span>{ "Waiting for simulations (" }{ self.background_snapshots.len() }{ "/" }{ self.background_agents.len() }{ " complete)" }</span> }
        };

        html! {
            <div class="centered">
                <h1>{ "Mission Complete" }</h1>
                { "Time: " }{ format!("{score_time:.3}") }{ " seconds" }<br/>
                { "Code size: " }{ code_size }{ " bytes" }<br/><br/>
                { background_status }<br/><br/>
                <br/><br/>
            </div>
        }
    }

    pub fn start_compile(&mut self, context: &Context<Self>, execution_mode: ExecutionMode) {
        self.compiler_errors = None;
        self.overlay = Some(Overlay::Compiling);

        let finished_callback = context
            .link()
            .callback(move |results| Msg::CompileFinished(results, execution_mode));

        /// Sends code to the compiler service, returns a compiled WASM binary
        async fn compile(text: String) -> Result<Code, String> {
            if text.trim().is_empty() {
                return Ok(Code::None);
            }

            // Compilation time will be logged
            let start_time = instant::Instant::now();

            let url = format!("{}/compile", services::compiler_url());
            let result = Request::post(&url).body(text).send().await;
            if let Err(e) = result {
                log::error!("Compile error: {}", e);
                return Err(e.to_string());
            }

            let response = result.unwrap();
            if !response.ok() {
                let error = response.text().await.unwrap();
                log::error!("Compile error: {}", error);
                return Err(error);
            }

            let wasm = response.binary().await;
            if let Err(e) = wasm {
                log::error!("Compile error: {}", e);
                return Err(e.to_string());
            }

            let elapsed = instant::Instant::now() - start_time;
            log::info!("Compile succeeded in {:?}", elapsed);
            Ok(Code::Wasm(wasm.unwrap()))
        }

        let source_codes: Vec<_> = self
            .teams
            .iter()
            .map(|team| {
                if team.running_source_code == Code::Rust("".to_string()) {
                    Code::None
                } else if team.running_source_code == team.initial_source_code
                    && team.initial_compiled_code != Code::None
                {
                    // Avoid recompilation if using the initial source code
                    team.initial_compiled_code.clone()
                } else if let Some(compiled_code) =
                    self.compilation_cache.get(&team.running_source_code)
                {
                    // Avoid recompilation if current code has already been compilde and cached
                    compiled_code.clone()
                } else {
                    team.running_source_code.clone()
                }
            })
            .collect();

        wasm_bindgen_futures::spawn_local(async move {
            let mut results = vec![];

            // All uncompiled code is compiled, and the callback defined above is called on completion
            for source_code in source_codes {
                let result = match source_code {
                    Code::Rust(text) => compile(text).await,
                    Code::Builtin(name) => oort_simulator::vm::builtin::load_compiled(&name),
                    other => Ok(other),
                };
                results.push(result);
            }
            finished_callback.emit(results);
        });
    }

    /// Kicks off simulation using running compiled code from each team
    /// NOTE: Assumes no compiler errors
    pub fn run(&mut self, context: &Context<Self>, execution_mode: ExecutionMode) {
        self.compiler_errors = None;

        // Collect compiled code from each team
        let codes: Vec<_> = self
            .teams
            .iter()
            .map(|x| x.running_compiled_code.clone())
            .collect();
        let rand_seed = rand::thread_rng().gen();

        // If replaying, reuse previous seed if it exists
        // instead of using a newly generated seed
        let seed = match execution_mode {
            ExecutionMode::Initial | ExecutionMode::Run => {
                self.configured_seed(context).unwrap_or(rand_seed)
            }
            ExecutionMode::Replay { .. } => self
                .configured_seed(context)
                .unwrap_or(self.previous_seed.unwrap_or(rand_seed)),
        };
        let start_paused = matches!(execution_mode, ExecutionMode::Replay { paused: true });

        // Cache seed for replays
        self.previous_seed = Some(seed);
        self.execution_mode = execution_mode;

        if let Some(link) = self.simulation_window_link.as_ref() {
            link.send_message(crate::simulation_window::Msg::StartSimulation {
                scenario_name: context.props().scenario.clone(),
                seed,
                start_paused,
                codes: codes.to_vec(),
            });
        } else {
            log::error!("Missing SimulationWindow");
        }
        self.background_agents.clear();
        self.background_snapshots.clear();
        self.background_nonce = 0;
    }

    pub fn change_scenario(&mut self, context: &Context<Self>, scenario_name: &str, run: bool) {
        let codes = crate::codestorage::load(&context.props().scenario);
        let scenario = oort_simulator::scenario::load(&context.props().scenario);

        let to_source_code = |code: &Code| match code {
            Code::Builtin(name) => oort_simulator::vm::builtin::load_source(name).unwrap(),
            _ => code.clone(),
        };

        let mut player_team = Team::new(self.editor_links[0].clone());
        player_team.initial_source_code = to_source_code(&codes[0]);

        if context.props().demo || context.props().scenario == "welcome" {
            let solution = scenario.solution();
            player_team.initial_source_code = to_source_code(&solution);
            player_team.running_source_code = player_team.initial_source_code.clone();
            player_team.running_compiled_code = solution;
        } else if let Some(compiled_code) =
            self.compilation_cache.get(&player_team.initial_source_code)
        {
            if run {
                player_team.running_source_code = player_team.initial_source_code.clone();
                player_team.running_compiled_code = compiled_code.clone();
            }
        }

        if context.props().scenario == "welcome" {
            player_team.initial_source_code = Code::Rust(
                "\
// Welcome to Oort.
// Select a scenario from the list in the top-right of the page.
// If you're new, start with \"Tutorial: Guns\"."
                    .to_string(),
            );
        }

        player_team.set_editor_text(&code_to_string(&player_team.initial_source_code));
        self.teams = vec![player_team];

        let enemy_code = if codes.len() > 1 {
            codes[1].clone()
        } else {
            Code::None
        };

        let mut enemy_team = Team::new(self.editor_links[1].clone());
        enemy_team.initial_source_code = to_source_code(&enemy_code);
        enemy_team.running_source_code = to_source_code(&enemy_code);
        enemy_team.initial_compiled_code = enemy_code.clone();
        enemy_team.running_compiled_code = enemy_code;
        enemy_team.set_editor_text(&code_to_string(&enemy_team.initial_source_code));
        self.teams.push(enemy_team);

        if scenario_name == "welcome" {
            crate::js::golden_layout::show_welcome(true);
            crate::js::golden_layout::select_tab("welcome");
        } else {
            crate::js::golden_layout::show_welcome(false);
        }

        self.run(context, ExecutionMode::Initial);
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
                log::info!("Initial: {:?}", team.initial_source_code);
                log::info!("Running: {:?}", team.running_source_code);
                return false;
            }
        }
        !is_encrypted(&self.player_team().running_source_code)
    }

    pub fn save_current_code(
        &self,
        context: &Context<Self>,
        scenario_name: &str,
        label: Option<String>,
    ) {
        if self.teams.is_empty() {
            return;
        }
        let code = self.player_team().get_editor_code();
        log::info!("code {}", self.player_team().get_editor_text());
        if is_encrypted(&code) {
            return;
        }

        codestorage::save(scenario_name, &code);

        let scenario_name = scenario_name.to_string();
        try_send_future(context.link(), async move {
            let code = code_to_string(&code);
            let version_control = oort_version_control::VersionControl::new().await?;
            if label.is_some() || !version_control.check_code_exists(&code).await? {
                let version = oort_version_control::CreateVersionParams {
                    code,
                    scenario_name,
                    label,
                };
                version_control.create_version(&version).await?;
            }
            Ok::<_, oort_version_control::Error>(Msg::RefreshVersions)
        });
    }

    fn configured_seed(&self, context: &Context<Self>) -> Option<u32> {
        context.props().seed
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

            let ed: &monaco::sys::editor::IStandaloneCodeEditor = editor.as_ref();
            let options = monaco::sys::editor::IEditorOptions::from(empty());
            options.set_read_only(Some(false));
            ed.update_options(&options);
        });
        // TODO trigger analyzer run
    }

    pub fn set_editor_text_preserving_cursor(&self, text: &str) {
        self.editor_link.with_editor(|editor| {
            let saved = editor.as_ref().save_view_state();
            editor.get_model().unwrap().set_value(text);
            if let Some(view_state) = saved {
                editor.as_ref().restore_view_state(&view_state);
            }

            let ed: &monaco::sys::editor::IStandaloneCodeEditor = editor.as_ref();
            let options = monaco::sys::editor::IEditorOptions::from(empty());
            options.set_read_only(Some(false));
            ed.update_options(&options);
        });
        // TODO trigger analyzer run
    }

    pub fn display_compiler_errors(&mut self, errors: &[CompilerError]) {
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
        Code::Builtin(name) => format!("#builtin:{name}"),
    }
}

pub fn str_to_code(s: &str) -> Code {
    let re = Regex::new(r"#builtin:(.*)").unwrap();
    if let Some(m) = re.captures(s) {
        Code::Builtin(m[1].to_string())
    } else if s.trim().is_empty() {
        Code::None
    } else {
        Code::Rust(s.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct CompilerError {
    pub line: usize,
    pub msg: String,
}

fn make_editor_errors(error: &str) -> Vec<CompilerError> {
    let re = Regex::new(r"(?m)error.*?: (.*?)$\n.*?ai/src/user.rs:(\d+):").unwrap();
    re.captures_iter(error)
        .map(|m| CompilerError {
            line: m[2].parse().unwrap(),
            msg: m[1].to_string(),
        })
        .collect()
}

pub(crate) fn is_encrypted(code: &Code) -> bool {
    match code {
        Code::Rust(src) => src.starts_with("ENCRYPTED:"),
        _ => false,
    }
}

fn try_send_future<Fut, Msg, C, E>(link: &Scope<C>, future: Fut)
where
    C: Component,
    Msg: Into<C::Message>,
    Fut: std::future::Future<Output = Result<Msg, E>> + 'static,
    E: std::fmt::Debug,
{
    let link = link.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match future.await {
            Ok(msg) => link.send_message(msg.into()),
            Err(e) => log::error!("Async task failed: {:?}", e),
        }
    });
}
