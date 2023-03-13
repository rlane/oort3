use crate::ui::UI;
use gloo_render::{request_animation_frame, AnimationFrame};
use oort_simulation_worker::SimAgent;
use oort_simulator::{scenario, simulation::Code, snapshot::Snapshot};
use rand::Rng;
use std::rc::Rc;
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
    PointerEvent(web_sys::PointerEvent),
    BlurEvent(web_sys::FocusEvent),
    RequestSnapshot,
    ReceivedSimAgentResponse(oort_simulation_worker::Response),
}

#[derive(Properties, Clone, PartialEq)]
pub struct SimulationWindowProps {
    pub host: web_sys::Element,
    pub on_simulation_finished: Callback<Snapshot>,
    pub register_link: Callback<Scope<SimulationWindow>>,
    pub version: String,
    pub canvas_ref: NodeRef,
}

pub struct SimulationWindow {
    ui: Option<Box<UI>>,
    render_handle: Option<AnimationFrame>,
    nonce: u32,
    sim_agent: Box<dyn Bridge<SimAgent>>,
    last_status: scenario::Status,
    canvas_ref: NodeRef,
    status_ref: NodeRef,
    picked_ref: NodeRef,
}

impl Component for SimulationWindow {
    type Message = Msg;
    type Properties = SimulationWindowProps;

    fn create(context: &yew::Context<Self>) -> Self {
        context.props().register_link.emit(context.link().clone());
        let cb = {
            let link = context.link().clone();
            move |e| link.send_message(Msg::ReceivedSimAgentResponse(e))
        };
        let sim_agent = SimAgent::bridge(Rc::new(cb));
        let render_handle = {
            let link2 = context.link().clone();
            Some(request_animation_frame(move |_ts| {
                link2.send_message(Msg::Render)
            }))
        };
        Self {
            ui: None,
            render_handle,
            nonce: 0,
            sim_agent,
            last_status: scenario::Status::Running,
            canvas_ref: context.props().canvas_ref.clone(),
            status_ref: NodeRef::default(),
            picked_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, context: &yew::Context<Self>, msg: Self::Message) -> bool {
        let result = match msg {
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
                    self.canvas_ref.clone(),
                    self.status_ref.clone(),
                    self.picked_ref.clone(),
                )));
                self.sim_agent
                    .send(oort_simulation_worker::Request::StartScenario {
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
                    .send(oort_simulation_worker::Request::Snapshot {
                        ticks: 1,
                        nonce: self.nonce,
                    });
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
            Msg::PointerEvent(e) => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.on_pointer_event(e);
                }
                false
            }
            Msg::BlurEvent(e) => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.on_blur_event(e);
                }
                false
            }
            Msg::ReceivedSimAgentResponse(oort_simulation_worker::Response::Snapshot {
                snapshot,
            }) => {
                if let Some(ui) = self.ui.as_mut() {
                    ui.on_snapshot(snapshot);
                }
                false
            }
        };

        if let Some(ui) = self.ui.as_ref() {
            if ui.needs_render() {
                self.render_handle = {
                    let link = context.link().clone();
                    Some(request_animation_frame(move |_ts| {
                        link.send_message(Msg::Render)
                    }))
                };
            }
        }

        result
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let key_event_cb = context.link().callback(Msg::KeyEvent);
        let wheel_event_cb = context.link().callback(Msg::WheelEvent);
        let pointer_event_cb = context.link().callback(Msg::PointerEvent);
        let blur_event_cb = context.link().callback(Msg::BlurEvent);

        create_portal(
            html! {
                <>
                    <canvas id="simcanvas" class="glcanvas"
                        ref={self.canvas_ref.clone()}
                        tabindex="1"
                        onkeydown={key_event_cb.clone()}
                        onkeyup={key_event_cb}
                        onwheel={wheel_event_cb}
                        onpointermove={pointer_event_cb.clone()}
                        onpointerup={pointer_event_cb.clone()}
                        onpointerdown={pointer_event_cb}
                        onblur={blur_event_cb} />
                    <div class="status" ref={self.status_ref.clone()} />
                    <div class="picked">
                        <pre ref={self.picked_ref.clone()}></pre>
                    </div>
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
            if self.last_status != status && status != scenario::Status::Running {
                context
                    .props()
                    .on_simulation_finished
                    .emit(ui.snapshot().unwrap());
            }
            self.last_status = status;
        }
        false
    }
}
