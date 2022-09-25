use crate::ui::UI;
use oort_simulator::{scenario, simulation::Code, snapshot::Snapshot};
use oort_worker::SimAgent;
use rand::Rng;
use yew::html::Scope;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

#[derive(Debug)]
pub enum Msg {
    StartSimulation {
        scenario_name: String,
        seed: u32,
        codes: Vec<Code>,
    },
    Render,
    KeyEvent(web_sys::KeyboardEvent),
    WheelEvent(web_sys::WheelEvent),
    MouseEvent(web_sys::MouseEvent),
    RequestSnapshot,
    ReceivedSimAgentResponse(oort_worker::Response),
}

#[derive(Properties, Clone, PartialEq)]
pub struct SimulationWindowProps {
    pub host: web_sys::Element,
    pub on_simulation_finished: Callback<Snapshot>,
    pub register_link: Callback<Scope<SimulationWindow>>,
    pub version: String,
}

pub struct SimulationWindow {
    ui: Option<Box<UI>>,
    nonce: u32,
    sim_agent: Box<dyn Bridge<SimAgent>>,
    last_status: scenario::Status,
}

impl Component for SimulationWindow {
    type Message = Msg;
    type Properties = SimulationWindowProps;

    fn create(context: &yew::Context<Self>) -> Self {
        context.props().register_link.emit(context.link().clone());
        let sim_agent = SimAgent::bridge(context.link().callback(Msg::ReceivedSimAgentResponse));
        Self {
            ui: None,
            nonce: 0,
            sim_agent,
            last_status: scenario::Status::Running,
        }
    }

    fn update(&mut self, context: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StartSimulation {
                scenario_name,
                seed,
                codes,
            } => {
                self.nonce = rand::thread_rng().gen();
                self.ui = Some(Box::new(UI::new(
                    context.link().callback(|_| Msg::RequestSnapshot),
                    self.nonce,
                    context.props().version.clone(),
                )));
                self.sim_agent.send(oort_worker::Request::StartScenario {
                    scenario_name,
                    seed,
                    codes: codes.to_vec(),
                    nonce: self.nonce,
                });
                false
            }
            Msg::Render => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.render();
                }
                self.check_status(context)
            }
            Msg::RequestSnapshot => {
                self.sim_agent
                    .send(oort_worker::Request::Snapshot { nonce: self.nonce });
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
                if let Some(ui) = self.ui.as_mut() {
                    ui.on_snapshot(snapshot);
                }
                false
            }
        }
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let key_event_cb = context.link().callback(Msg::KeyEvent);
        let wheel_event_cb = context.link().callback(Msg::WheelEvent);
        let mouse_event_cb = context.link().callback(Msg::MouseEvent);

        create_portal(
            html! {
                <>
                    <canvas id="glcanvas"
                        tabindex="1"
                        onkeydown={key_event_cb.clone()}
                        onkeyup={key_event_cb}
                        onwheel={wheel_event_cb}
                        onclick={mouse_event_cb} />
                    <div id="status" />
                    <div id="picked"><pre id="picked_text"></pre></div>
                </>
            },
            context.props().host.clone(),
        )
    }
}

impl SimulationWindow {
    fn check_status(&mut self, context: &Context<Self>) -> bool {
        if let Some(ui) = self.ui.as_ref() {
            let status = ui.status();
            if self.last_status == status || status == scenario::Status::Running {
                return false;
            }
            context
                .props()
                .on_simulation_finished
                .emit(ui.snapshot().unwrap());
            self.last_status = status;
        }
        false
    }
}
