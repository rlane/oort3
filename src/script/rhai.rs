use super::ShipController;
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use log::{error, info};
use rhai::plugin::*;
use rhai::{Engine, Scope, FLOAT, INT};

type Vec2 = nalgebra::Vector2<f64>;

#[export_module]
mod vec2_module {
    pub fn vec2(x: f64, y: f64) -> Vec2 {
        Vec2::new(x, y)
    }

    #[rhai_fn(name = "+")]
    pub fn add(obj: &mut Vec2, other: Vec2) -> Vec2 {
        *obj + other
    }

    #[rhai_fn(name = "-")]
    pub fn sub(obj: &mut Vec2, other: Vec2) -> Vec2 {
        *obj - other
    }

    #[rhai_fn(name = "*")]
    pub fn scale(obj: &mut Vec2, other: f64) -> Vec2 {
        *obj * other
    }

    #[rhai_fn(name = "*")]
    pub fn scale2(obj: &mut f64, other: Vec2) -> Vec2 {
        *obj * other
    }

    #[rhai_fn(name = "/")]
    pub fn div(obj: &mut Vec2, other: f64) -> Vec2 {
        *obj / other
    }

    #[rhai_fn(get = "x", pure)]
    pub fn get_x(obj: &mut Vec2) -> f64 {
        obj.x
    }

    #[rhai_fn(set = "x")]
    pub fn set_x(obj: &mut Vec2, value: f64) {
        obj.x = value;
    }

    #[rhai_fn(get = "y", pure)]
    pub fn get_y(obj: &mut Vec2) -> f64 {
        obj.y
    }

    #[rhai_fn(set = "y")]
    pub fn set_y(obj: &mut Vec2, value: f64) {
        obj.y = value;
    }

    #[rhai_fn(name = "magnitude")]
    pub fn magnitude(obj: &mut Vec2) -> f64 {
        obj.magnitude()
    }

    #[rhai_fn(name = "dot")]
    pub fn dot(obj: &mut Vec2, other: Vec2) -> f64 {
        obj.dot(&other)
    }

    #[rhai_fn(name = "distance")]
    pub fn distance(obj: &mut Vec2, other: Vec2) -> f64 {
        obj.metric_distance(&other)
    }

    #[rhai_fn(name = "angle")]
    pub fn angle(obj: &mut Vec2) -> f64 {
        obj.y.atan2(obj.x)
    }

    #[rhai_fn(name = "normalize")]
    pub fn normalize(obj: &mut Vec2) -> Vec2 {
        obj.normalize()
    }

    fn assert_internal<T: PartialEq + std::fmt::Debug>(
        a: &mut T,
        b: T,
    ) -> Result<(), Box<EvalAltResult>> {
        if *a != b {
            Err(format!("assertion failed: {:?} != {:?}", *a, b).into())
        } else {
            Ok(())
        }
    }

    #[rhai_fn(name = "assert_eq", return_raw)]
    pub fn assert_eq_i64(a: &mut i64, b: i64) -> Result<(), Box<EvalAltResult>> {
        assert_internal(a, b)
    }

    #[rhai_fn(name = "assert_eq", return_raw)]
    pub fn assert_eq_f64(a: &mut f64, b: f64) -> Result<(), Box<EvalAltResult>> {
        assert_internal(a, b)
    }

    #[rhai_fn(name = "assert_eq", return_raw)]
    pub fn assert_eq_vec2(a: &mut Vec2, b: Vec2) -> Result<(), Box<EvalAltResult>> {
        assert_internal(a, b)
    }

    #[rhai_fn(name = "to_string")]
    pub fn to_string(obj: &mut Vec2) -> String {
        format!("({}, {})", obj.x, obj.y)
    }

    #[rhai_fn(name = "to_debug")]
    pub fn to_debug(obj: &mut Vec2) -> String {
        to_string(obj)
    }
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

    fn position(&mut self) -> Vec2 {
        self.sim().ship(self.handle).position().vector
    }

    fn velocity(&mut self) -> Vec2 {
        self.sim().ship(self.handle).velocity()
    }

    fn heading(&mut self) -> f64 {
        self.sim().ship(self.handle).heading()
    }

    fn accelerate(&mut self, acceleration: Vec2) {
        self.sim().ship_mut(self.handle).accelerate(acceleration);
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
        engine.set_max_expr_depths(64, 32);

        engine.on_print(|x| info!("Script: {}", x));
        engine.on_debug(|x, src, pos| info!("Script ({}:{:?}): {}", src.unwrap_or(""), pos, x));

        engine
            .register_type::<Api>()
            .register_fn("position", Api::position)
            .register_fn("velocity", Api::velocity)
            .register_fn("heading", Api::heading)
            .register_fn("accelerate", Api::accelerate)
            .register_fn("thrust_angular", Api::thrust_angular)
            .register_fn("fire_weapon", Api::fire_weapon)
            .register_fn("explode", Api::explode);

        engine.register_global_module(exported_module!(vec2_module).into());

        let mut scope = Scope::new();
        scope.push("api", api);

        Self {
            engine,
            scope,
            ast: None,
        }
    }

    pub fn test(&mut self, code: &str) {
        self.upload_code(code);
        if let Some(v) = self
            .engine
            .consume_ast_with_scope(&mut self.scope, self.ast.as_ref().unwrap())
            .err()
        {
            panic!("Test failed: {:?}", v);
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

    fn write_target(&mut self, target: Vec2) {
        self.scope.push("target", target);
    }
}

#[cfg(test)]
mod test {
    use crate::simulation::ship;
    use crate::simulation::Simulation;

    #[test]
    fn test_vec2() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, -100.0, 0.0, 100.0, 0.0, 0.0, ship::fighter());
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
        let v1 = vec2(1.0, 2.0);
        let v2 = vec2(3.0, 4.0);
        assert_eq((v1 + v2).x, 4.0);
        assert_eq(v1 + v2, vec2(4.0, 6.0));
        assert_eq(v2.magnitude(), 5.0);
        assert_eq(v1.distance(v2), 2.8284271247461903);
        assert_eq(v1.dot(v2), 11.0);
        ",
        );
    }

    #[test]
    fn test_pos_vel_hd() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(
            &mut sim,
            1.0,
            2.0,
            3.0,
            4.0,
            std::f64::consts::PI,
            ship::fighter(),
        );
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
        assert_eq(api.position(), vec2(1.0, 2.0));
        assert_eq(api.velocity(), vec2(3.0, 4.0));
        assert_eq(api.heading(), PI());
        ",
        );
    }

    #[test]
    fn test_angle() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter());
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
        assert_eq(vec2(1.0, 0.0).angle(), 0.0);
        assert_eq(vec2(0.0, 1.0).angle(), PI() / 2.0);
        assert_eq(vec2(-1.0, 0.0).angle(), PI());
        assert_eq(vec2(0.0, -1.0).angle(), -PI() / 2.0);
        ",
        );
    }
}
