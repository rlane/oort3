use crate::simulation::scenario::Status;
use crate::simulation::Simulation;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Worker {
    sim: Option<Box<Simulation>>,
}

#[wasm_bindgen]
impl Worker {
    pub fn start_scenario(&mut self, scenario_name: &str, seed: u64, code: &str) -> Vec<u8> {
        self.sim = Some(Simulation::new(scenario_name, seed, code));
        bincode::serialize(&self.sim.as_ref().unwrap().snapshot(0)).unwrap()
    }

    pub fn request_snapshot(&mut self, nonce: u64) -> Vec<u8> {
        if self.sim.as_ref().unwrap().status() == Status::Running {
            self.sim.as_mut().unwrap().step();
        }
        bincode::serialize(&self.sim.as_ref().unwrap().snapshot(nonce)).unwrap()
    }
}

#[wasm_bindgen]
pub fn create_worker() -> Worker {
    console_log::init_with_level(log::Level::Info).expect("initializing logging");
    log::info!("Worker initialized");
    Worker { sim: None }
}
