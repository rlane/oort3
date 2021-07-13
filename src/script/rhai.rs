use super::ShipController;
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

    #[allow(clippy::mut_from_ref)]
    fn sim(&self) -> &mut Simulation {
        unsafe { &mut *self.sim }
    }

    fn thrust_main(&mut self, force: FLOAT) {
        self.sim().ship_mut(self.handle).thrust_main(force);
    }

    fn thrust_lateral(&mut self, force: FLOAT) {
        self.sim().ship_mut(self.handle).thrust_lateral(force);
    }

    fn thrust_angular(&mut self, force: FLOAT) {
        self.sim().ship_mut(self.handle).thrust_angular(force);
    }

    fn fire_weapon(&mut self, index: INT) {
        self.sim().ship_mut(self.handle).fire_weapon(index);
    }

    fn explode(&mut self) {
        self.sim().ship_mut(self.handle).explode();
    }
}

pub struct RhaiShipController {
    engine: Engine,
    scope: Scope<'static>,
    code: String,
}

impl RhaiShipController {
    pub fn new(handle: ShipHandle, sim: *mut Simulation) -> Self {
        let api = Api::new(handle, sim);

        let mut engine = Engine::new_raw();

        engine
            .register_type::<Api>()
            .register_fn("thrust_main", Api::thrust_main)
            .register_fn("thrust_lateral", Api::thrust_lateral)
            .register_fn("thrust_angular", Api::thrust_angular)
            .register_fn("fire_weapon", Api::fire_weapon)
            .register_fn("explode", Api::explode);

        engine.register_fn("log", log);

        let mut scope = Scope::new();
        scope.push("api", api);

        Self {
            engine,
            scope,
            code: String::new(),
        }
    }
}

impl ShipController for RhaiShipController {
    fn upload_code(&mut self, code: &str) {
        self.code = code.to_string();
    }

    fn tick(&mut self) {
        if let Err(msg) = self.engine.consume_with_scope(&mut self.scope, &self.code) {
            error!("Script error: {}", msg);
            self.code = String::new();
        }
    }
}
