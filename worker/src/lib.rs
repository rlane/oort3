use oort_simulator::scenario::Status;
use oort_simulator::simulation::Simulation;
use oort_simulator::snapshot::Snapshot;
use serde::{Deserialize, Serialize};
use yew_agent::{Agent, AgentLink, HandlerId, Private};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    StartScenario {
        scenario_name: String,
        seed: u32,
        code: String,
        nonce: u32,
    },
    RunScenario {
        scenario_name: String,
        seed: u32,
        code: String,
    },
    Snapshot {
        nonce: u32,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Snapshot { snapshot: Snapshot },
}

pub struct SimAgent {
    link: AgentLink<SimAgent>,
    sim: Option<Box<Simulation>>,
    errored: bool,
}

impl Agent for SimAgent {
    type Reach = Private<Self>;
    type Message = ();
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            sim: None,
            errored: false,
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, request: Self::Input, who: HandlerId) {
        match request {
            Request::StartScenario {
                scenario_name,
                seed,
                code,
                nonce,
            } => {
                self.sim = Some(Simulation::new(&scenario_name, seed, &code));
                let snapshot = self.sim().snapshot(nonce);
                self.errored = !snapshot.errors.is_empty();
                self.link.respond(who, Response::Snapshot { snapshot });
            }
            Request::RunScenario {
                scenario_name,
                seed,
                code,
            } => {
                let mut sim = Simulation::new(&scenario_name, seed, &code);
                while sim.status() == Status::Running && sim.tick() < 10000 {
                    sim.step();
                }
                let snapshot = sim.snapshot(0);
                self.link.respond(who, Response::Snapshot { snapshot });
            }
            Request::Snapshot { nonce } => {
                if self.errored {
                    return;
                }
                if self.sim().status() == Status::Running {
                    self.sim().step();
                }
                let snapshot = self.sim().snapshot(nonce);
                self.errored = !snapshot.errors.is_empty();
                self.link.respond(who, Response::Snapshot { snapshot });
            }
        };
    }

    fn name_of_resource() -> &'static str {
        "oort_worker.js"
    }
}

impl SimAgent {
    fn sim(&mut self) -> &mut Simulation {
        self.sim.as_mut().unwrap()
    }
}
