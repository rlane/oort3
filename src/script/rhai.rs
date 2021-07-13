use super::ShipController;
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use log::{error, info};
use rhai::{Engine, Scope, FLOAT, INT};

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
    // TODO share AST across ships
    ast: Option<rhai::AST>,
}

impl RhaiShipController {
    pub fn new(handle: ShipHandle, sim: *mut Simulation) -> Self {
        let api = Api::new(handle, sim);

        let mut engine = Engine::new();

        engine.on_print(|x| info!("Script: {}", x));
        engine.on_debug(|x, src, pos| info!("Script ({}:{:?}): {}", src.unwrap_or(""), pos, x));

        engine
            .register_type::<Api>()
            .register_fn("thrust_main", Api::thrust_main)
            .register_fn("thrust_lateral", Api::thrust_lateral)
            .register_fn("thrust_angular", Api::thrust_angular)
            .register_fn("fire_weapon", Api::fire_weapon)
            .register_fn("explode", Api::explode);

        let mut scope = Scope::new();
        scope.push("api", api);

        Self {
            engine,
            scope,
            ast: None,
        }
    }
}

impl ShipController for RhaiShipController {
    fn upload_code(&mut self, code: &str) {
        match self.engine.compile(code) {
            Ok(ast) => {
                self.ast = Some(ast);
            }
            Err(msg) => {
                error!("Compilation failed: {}", msg);
            }
        }
    }

    fn start(&mut self) {
        if let Some(ast) = &self.ast {
            if let Err(msg) = self.engine.consume_ast_with_scope(&mut self.scope, &ast) {
                error!("Script error: {}", msg);
                self.ast = None;
            }
        }
        if self.ast.is_some() {
            self.ast.as_mut().unwrap().clear_statements();
        }
    }

    fn tick(&mut self) {
        if let Some(ast) = &self.ast {
            let result: Result<(), _> = self.engine.call_fn(&mut self.scope, &ast, "tick", ());
            if let Err(msg) = result {
                error!("Script error: {}", msg);
                self.ast = None;
            }
        }
    }
}
