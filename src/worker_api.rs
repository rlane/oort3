use crate::simulation::scenario::Status;
use crate::simulation::Simulation;
use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

struct Worker {
    sim: Option<Box<Simulation>>,
}

unsafe impl Send for Worker {}

lazy_static! {
    static ref OORT_WORKER: Mutex<Option<Worker>> = Mutex::new(None);
}

static PANICKED: AtomicBool = AtomicBool::new(false);

fn has_panicked() -> bool {
    PANICKED.load(Ordering::SeqCst)
}

#[wasm_bindgen]
pub fn worker_initialize() {
    console_log::init_with_level(log::Level::Info).expect("initializing logging");
    log::info!("Worker initialized");
}

#[wasm_bindgen]
pub fn worker_run_scenario(scenario_name: &str, seed: u64, code: &str) -> JsValue {
    let mut sim = Simulation::new(scenario_name, seed, code);
    let mut i = 0;
    while sim.status() == Status::Running && i < 10000 {
        sim.step();
        i += 1;
    }
    JsValue::from_serde(&sim.status()).unwrap()
}

#[wasm_bindgen]
pub fn worker_start_scenario(scenario_name: &str, seed: u64, code: &str) -> Vec<u8> {
    if has_panicked() {
        return vec![];
    }
    let mut worker_lock = OORT_WORKER.lock().unwrap();
    *worker_lock = Some(Worker {
        sim: Some(Simulation::new(scenario_name, seed, code)),
    });
    bincode::serialize(
        &worker_lock
            .as_ref()
            .unwrap()
            .sim
            .as_ref()
            .unwrap()
            .snapshot(0),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn worker_request_snapshot(nonce: u64) -> Vec<u8> {
    if has_panicked() {
        return vec![];
    }
    let mut worker_lock = OORT_WORKER.lock().unwrap();
    let worker = worker_lock.as_mut().unwrap();
    if worker.sim.as_ref().unwrap().status() == Status::Running {
        worker.sim.as_mut().unwrap().step();
    }
    bincode::serialize(&worker.sim.as_ref().unwrap().snapshot(nonce)).unwrap()
}
