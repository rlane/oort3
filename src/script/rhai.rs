use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use log::{error, info};
use rhai::{Engine, Scope, FLOAT, INT};

fn log(x: INT) {
    info!("Script logged: {}", x);
}

#[derive(Clone)]
struct Api {
    handle: ShipHandle,
    sim: *mut Simulation,
}

impl Api {
    fn new(handle: ShipHandle, sim: *mut Simulation) -> Self {
        Self { handle, sim }
    }

    fn thrust_main(&mut self, force: FLOAT) {
        info!("thrust_main");
        unsafe {
            (*self.sim).ship_mut(self.handle).thrust_main(force);
        }
    }
}

pub fn exec_script(code: &str, handle: ShipHandle, sim: &mut Simulation) {
    let api = Api::new(handle, sim as *mut Simulation);

    let mut engine = Engine::new();

    engine
        .register_type::<Api>()
        .register_fn("thrust_main", Api::thrust_main);

    engine.register_fn("log", log);

    let mut scope = Scope::new();
    scope.push("z", 40_i64);
    scope.push("api", api);

    if let Err(msg) = engine.consume_with_scope(&mut scope, code) {
        error!("Script error: {}", msg);
    }
}
