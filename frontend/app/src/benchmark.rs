use oort_simulation_worker::SimAgent;
use oort_simulator::snapshot::{Snapshot, Timing};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::rc::Rc;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

const BATCH_SIZE: usize = 10;

pub enum Msg {
    ReceivedSimAgentResponse(oort_simulation_worker::Response),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Props {
    pub scenario: String,
}

pub struct Benchmark {
    scenario_name: String,
    sim_agent: Box<dyn Bridge<SimAgent>>,
    time: f64,
    cumulative_timing: Timing,
    num_slow_ticks: usize,
    slowest_snapshot: Option<Snapshot>,
    hash: Option<String>,
}

impl Component for Benchmark {
    type Message = Msg;
    type Properties = Props;

    fn create(context: &yew::Context<Self>) -> Self {
        let scenario_name = context.props().scenario.clone();
        let seed = 0;
        let nonce = rand::thread_rng().gen();
        let scenario = oort_simulator::scenario::load(&scenario_name);
        let mut codes = scenario.initial_code();
        codes[0] = scenario.solution();
        let cb = {
            let link = context.link().clone();
            move |e| link.send_message(Msg::ReceivedSimAgentResponse(e))
        };
        let mut sim_agent = SimAgent::bridge(Rc::new(cb));
        sim_agent.send(oort_simulation_worker::Request::StartScenario {
            scenario_name: scenario_name.clone(),
            seed,
            codes,
            nonce,
        });
        sim_agent.send(oort_simulation_worker::Request::Snapshot {
            ticks: BATCH_SIZE as u32,
            nonce: 0,
        });
        Self {
            scenario_name,
            sim_agent,
            time: 0.0,
            cumulative_timing: Timing::default(),
            num_slow_ticks: 0,
            slowest_snapshot: None,
            hash: None,
        }
    }

    fn update(&mut self, _context: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ReceivedSimAgentResponse(oort_simulation_worker::Response::Snapshot {
                snapshot,
            }) => {
                if snapshot.status == oort_simulator::scenario::Status::Running {
                    self.time = snapshot.time;
                    if snapshot.timing.total() > oort_simulator::simulation::PHYSICS_TICK_LENGTH {
                        self.num_slow_ticks += BATCH_SIZE;
                    }
                    if self.slowest_snapshot.is_none()
                        || snapshot.timing.total()
                            > self.slowest_snapshot.as_ref().unwrap().timing.total()
                    {
                        self.slowest_snapshot = Some(snapshot.clone());
                    }
                    self.cumulative_timing += snapshot.timing;
                    self.sim_agent
                        .send(oort_simulation_worker::Request::Snapshot {
                            ticks: BATCH_SIZE as u32,
                            nonce: 0,
                        });
                    true
                } else {
                    if self.hash.is_none() {
                        let mut snapshot = snapshot;
                        snapshot.timing = Timing::default();
                        let bytes = bincode::serialize(&snapshot).unwrap();
                        let mut hasher = Sha256::new();
                        hasher.update(&bytes);
                        self.hash = Some(format!("{:x}", hasher.finalize()));
                    }
                    true
                }
            }
        }
    }

    fn view(&self, _context: &yew::Context<Self>) -> Html {
        let slowest_snapshot = if let Some(snapshot) = self.slowest_snapshot.as_ref() {
            html! {
                <div>
                    <p><b>{ "Slowest snapshot:" }</b></p>
                    <p>{ format!("Simulated time: {:.1}s", snapshot.time) }</p>
                    <p>{ format!("CPU time: {:.2}ms", snapshot.timing.total() * 1e3 * BATCH_SIZE as f64) }</p>
                    <p>{ format!("Physics: {:.2}ms", snapshot.timing.physics * 1e3 * BATCH_SIZE as f64) }</p>
                    <p>{ format!("Script: {:.2}ms", snapshot.timing.script * 1e3 * BATCH_SIZE as f64) }</p>
                    <p>{ format!("Ships: {}", snapshot.ships.len()) }</p>
                    <p>{ format!("Bullets: {}", snapshot.bullets.len()) }</p>
                </div>
            }
        } else {
            html! {}
        };
        html! {
            <div id="overlay">
                <h1>{ "Benchmark: " }{ &self.scenario_name }</h1>
                <p><b>{ "Cumulative:" }</b></p>
                <p>{ format!("Simulated time: {:.1}s", self.time) }</p>
                <p>{ format!("CPU time: {:.1}s", self.cumulative_timing.total() * BATCH_SIZE as f64) }</p>
                <p>{ format!("Physics: {:.1}s", self.cumulative_timing.physics * BATCH_SIZE as f64 ) }</p>
                <p>{ format!("Script: {:.1}s", self.cumulative_timing.script * BATCH_SIZE as f64 ) }</p>
                <p>{ format!("Slow ticks: {}", self.num_slow_ticks) }</p>
                <p>{ format!("Hash: {:?}", self.hash) }</p>
                { slowest_snapshot }
            </div>
        }
    }
}
