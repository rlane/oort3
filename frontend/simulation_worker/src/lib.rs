use oort_simulator::scenario::{Status, MAX_TICKS};
use oort_simulator::simulation::Code;
use oort_simulator::simulation::Simulation;
use oort_simulator::snapshot::Snapshot;
use serde::{Deserialize, Serialize};
use yew_agent::{HandlerId, Private, WorkerLink};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    StartScenario {
        scenario_name: String,
        seed: u32,
        codes: Vec<Code>,
        nonce: u32,
    },
    Snapshot {
        ticks: u32,
        nonce: u32,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Snapshot { snapshot: Snapshot },
}

pub struct SimAgent {
    link: WorkerLink<Self>,
    sim: Option<Box<Simulation>>,
    errored: bool,
}

impl yew_agent::Worker for SimAgent {
    type Reach = Private<Self>;
    type Message = ();
    type Input = Request;
    type Output = Response;

    fn create(link: WorkerLink<Self>) -> Self {
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
                codes,
                nonce,
            } => {
                self.sim = Some(Simulation::new(&scenario_name, seed, &codes));
                // Snapshot of the starting state of the simulation
                let snapshot = self.sim().snapshot(nonce);
                self.errored = !snapshot.errors.is_empty();
                self.link.respond(who, Response::Snapshot { snapshot });
            }
            Request::Snapshot { ticks, nonce } => {
                if self.errored || self.sim().status() != Status::Running {
                    return;
                }

                for _ in 0..ticks {
                    if self.sim().status() == Status::Running && self.sim().tick() < MAX_TICKS {
                        self.sim().step();
                    }
                }
                let snapshot = self.sim().snapshot(nonce);
                self.errored = !snapshot.errors.is_empty();
                self.link.respond(who, Response::Snapshot { snapshot });
            }
        };
    }

    fn name_of_resource() -> &'static str {
        "oort_simulation_worker.js"
    }
}

impl SimAgent {
    fn sim(&mut self) -> &mut Simulation {
        self.sim.as_mut().unwrap()
    }
}
