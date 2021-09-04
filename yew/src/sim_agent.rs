use oort_simulator::scenario::Status;
use oort_simulator::simulation::Simulation;
use oort_simulator::snapshot::Snapshot;
use serde::{Deserialize, Serialize};
use yew::agent::{Agent, AgentLink, HandlerId, Job};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    StartScenario {
        scenario_name: String,
        seed: u32,
        code: String,
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
}

impl Agent for SimAgent {
    type Reach = Job<Self>;
    type Message = ();
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        Self { link, sim: None }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, request: Self::Input, who: HandlerId) {
        let response = match request {
            Request::StartScenario {
                scenario_name,
                seed,
                code,
            } => {
                self.sim = Some(Simulation::new(&scenario_name, seed, &code));
                Self::make_snapshot_response(self.sim(), 0)
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
                Self::make_snapshot_response(&sim, 0)
            }
            Request::Snapshot { nonce } => {
                if self.sim().status() == Status::Running {
                    self.sim().step();
                }
                Self::make_snapshot_response(self.sim(), nonce)
            }
        };
        self.link.respond(who, response);
    }
}

impl SimAgent {
    fn sim(&mut self) -> &mut Simulation {
        self.sim.as_mut().unwrap()
    }

    fn make_snapshot_response(sim: &Simulation, nonce: u32) -> Response {
        Response::Snapshot {
            snapshot: sim.snapshot(nonce),
        }
    }
}
