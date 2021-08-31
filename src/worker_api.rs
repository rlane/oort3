use crate::simulation::scenario::Status;
use crate::simulation::Simulation;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkerRequest {
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

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkerResponse {
    Snapshot {
        #[serde(with = "serde_bytes")]
        snapshot: Vec<u8>,
    },
}

#[wasm_bindgen]
pub struct Worker {
    sim: Option<Box<Simulation>>,
}

#[wasm_bindgen]
impl Worker {
    pub fn on_message(&mut self, request: JsValue) -> JsValue {
        let request: WorkerRequest =
            serde_wasm_bindgen::from_value(request).expect("deserializing worker request");
        let response = match request {
            WorkerRequest::StartScenario {
                scenario_name,
                seed,
                code,
            } => {
                self.sim = Some(Simulation::new(&scenario_name, seed, &code));
                Worker::make_snapshot_response(self.sim(), 0)
            }
            WorkerRequest::RunScenario {
                scenario_name,
                seed,
                code,
            } => {
                let mut sim = Simulation::new(&scenario_name, seed, &code);
                while sim.status() == Status::Running && sim.tick() < 10000 {
                    sim.step();
                }
                Worker::make_snapshot_response(&sim, 0)
            }
            WorkerRequest::Snapshot { nonce } => {
                if self.sim().status() == Status::Running {
                    self.sim().step();
                }
                Worker::make_snapshot_response(self.sim(), nonce)
            }
        };
        serde_wasm_bindgen::to_value(&response).expect("serializing worker reply")
    }

    fn sim(&mut self) -> &mut Simulation {
        self.sim.as_mut().unwrap()
    }

    fn make_snapshot_response(sim: &Simulation, nonce: u32) -> WorkerResponse {
        WorkerResponse::Snapshot {
            snapshot: bincode::serialize(&sim.snapshot(nonce)).unwrap(),
        }
    }
}

#[wasm_bindgen]
pub fn create_worker() -> Worker {
    console_log::init_with_level(log::Level::Info).expect("initializing logging");
    Worker { sim: None }
}
