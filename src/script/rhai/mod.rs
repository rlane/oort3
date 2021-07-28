mod ast_rewrite;
mod globals;
mod math;
mod radar;
mod random;
mod ship;
mod util;
mod vec2;

use self::vec2::Vec2;
use super::ShipController;
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use rhai::plugin::*;
use rhai::{Dynamic, Engine, Scope, AST};
use smartstring::alias::CompactString;

pub struct RhaiShipController {
    engine: Engine,
    scope: Scope<'static>,
    #[allow(unused)]
    globals_map: Box<std::collections::HashMap<CompactString, Dynamic>>,
    // TODO share AST across ships
    ast: Option<AST>,
}

impl RhaiShipController {
    pub fn new(handle: ShipHandle, sim: *mut Simulation) -> Self {
        let mut engine = Engine::new();
        engine.set_max_expr_depths(64, 64);
        engine.set_max_operations(1000);

        engine.on_print(|x| info!("Script: {}", x));
        engine.on_debug(|x, src, pos| info!("Script ({}:{:?}): {}", src.unwrap_or(""), pos, x));

        engine.register_global_module(exported_module!(ship::plugin).into());
        engine.register_global_module(exported_module!(vec2::plugin).into());
        engine.register_global_module(exported_module!(globals::plugin).into());
        engine.register_global_module(exported_module!(radar::plugin).into());
        engine.register_global_module(exported_module!(self::random::plugin).into());
        engine.register_global_module(exported_module!(self::util::plugin).into());
        engine.register_global_module(exported_module!(self::math::plugin).into());

        let (i, j) = handle.0.into_raw_parts();
        let seed = ((i as i64) << 32) | j as i64;
        let rng = self::random::plugin::new_rng(seed);

        let ship = ship::plugin::ShipApi { handle, sim };
        let radar = radar::plugin::RadarApi { handle, sim };
        let mut globals_map = Box::new(std::collections::HashMap::new());
        let globals = globals::plugin::Globals {
            map: &mut *globals_map,
        };
        globals_map.insert("rng".into(), Dynamic::from(rng));
        engine.on_var(move |name, _index, _context| match name {
            "api" => Ok(Some(Dynamic::from(ship))),
            "ship" => Ok(Some(Dynamic::from(ship))),
            "radar" => Ok(Some(Dynamic::from(radar))),
            "globals" => Ok(Some(Dynamic::from(globals))),
            _ => Ok(None),
        });

        Self {
            engine,
            scope: Scope::new(),
            ast: None,
            globals_map,
        }
    }

    pub fn test(&mut self, code: &str) {
        self.upload_code(code).expect("Uploading code failed");
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
    fn upload_code(&mut self, code: &str) -> Result<(), super::Error> {
        match self.engine.compile(code) {
            Ok(ast) => {
                self.ast = Some(ast_rewrite::rewrite_ast(ast));
                Ok(())
            }
            Err(e) => {
                error!("Compilation failed: {}", e);
                Err(super::Error {
                    line: extract_line(&e.to_string()),
                    msg: e.to_string(),
                })
            }
        }
    }

    fn start(&mut self) -> Result<(), super::Error> {
        if let Some(ast) = &self.ast {
            let result = self.engine.consume_ast_with_scope(&mut self.scope, &ast);
            if let Err(e) = result {
                error!("Script error: {}", e);
                self.ast = None;
                return Err(super::Error {
                    line: extract_line(&e.to_string()),
                    msg: e.to_string(),
                });
            }
        }
        if self.ast.is_some() {
            self.ast.as_mut().unwrap().clear_statements();
        }
        Ok(())
    }

    fn tick(&mut self) -> Result<(), super::Error> {
        if let Some(ast) = &self.ast {
            let result: Result<(), _> = self.engine.call_fn(&mut self.scope, &ast, "tick", ());
            if let Err(e) = result {
                error!("Script error: {}", e);
                self.ast = None;
                return Err(super::Error {
                    line: extract_line(&e.to_string()),
                    msg: e.to_string(),
                });
            }
        }
        Ok(())
    }

    fn write_target(&mut self, target: Vec2) {
        self.scope.push("target", target);
    }
}

fn extract_line(msg: &str) -> usize {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"line (?P<line>\d+)").unwrap();
    }
    RE.captures(msg)
        .and_then(|cap| cap.name("line").map(|line| line.as_str().parse()))
        .unwrap_or(Ok(0))
        .unwrap_or(0)
}

#[cfg(test)]
mod test {
    use crate::simulation::ship;
    use crate::simulation::Simulation;
    use test_env_log::test;

    #[test]
    fn test_vec2() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, -100.0, 0.0, 100.0, 0.0, 0.0, ship::fighter(0));
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
        assert_eq(-v1, vec2(-1, -2));
        assert_eq(vec2(1, 2).rotate(PI() / 2), vec2(-2, 1.0000000000000002));
        assert_eq(vec2(1, 2).rotate(PI()), vec2(-1.0000000000000002, -1.9999999999999998));
        assert_eq(vec2(1, 2).rotate(3 * PI() / 2), vec2(1.9999999999999998, -1.0000000000000004));
        assert_eq(vec2(3, 4).normalize(), vec2(0.6, 0.8));
        ",
        );
    }

    #[test]
    fn test_vec2_angle() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
        assert_eq(vec2(1.0, 0.0).angle(), 0.0);
        assert_eq(vec2(0.0, 1.0).angle(), PI() / 2.0);
        assert_eq(vec2(-1.0, 0.0).angle(), PI());
        assert_eq(vec2(0.0, -1.0).angle(), 3 * PI() / 2.0);
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
            ship::fighter(0),
        );
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
        assert_eq(ship.position(), vec2(1.0, 2.0));
        assert_eq(ship.velocity(), vec2(3.0, 4.0));
        assert_eq(ship.heading(), PI());
        ",
        );
    }

    #[test]
    fn test_function() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
            fn foo() {
                assert_eq(ship.position(), vec2(0.0, 0.0));
            }

            assert_eq(ship.velocity(), vec2(0.0, 0.0));
            foo();
        ",
        );
    }

    #[test]
    fn test_globals() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            r#"
           let a = 1;
           let b = 2.0;
           let c = b;
           fn foo() {
               assert_eq(a, 1);
               assert_eq(b, 2.0);
               a += 1;
               b += 1.0;
               bar();
           }
           fn bar() {
               assert_eq(a, 2);
               a += 1;
           }
           foo();
           assert_eq(a, 3);
           assert_eq(b, 3.0);
           assert_eq(c, 2.0);
           if 1 == 1 {
               assert_eq(a, 3);
           }
           while a < 4 {
               a += 1;
           }
           assert_eq(a, 4);
           print(`a=${a}`);
       "#,
        );
    }

    #[test]
    fn test_mixed_integer_float() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            r#"
assert_eq(vec2(1, 2), vec2(1.0, 2.0));
assert_eq(vec2(1.0, 2), vec2(1, 2.0));
assert_eq(vec2(1, 1) * 2.0, vec2(1, 1) * 2);
assert_eq(2.0 * vec2(1, 1), 2 * vec2(1, 1));
assert_eq(vec2(1, 1) / 2.0, vec2(1, 1) / 2);
       "#,
        );
    }

    #[test]
    #[should_panic(expected = "ErrorTooManyOperations")]
    fn test_infinite_loop() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            r#"
let i = 0;
while true {
    print(`i=${i}`);
    i += 1;
}
       "#,
        );
    }

    #[test]
    fn test_random() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            r#"
let rng = new_rng(1);
assert_eq(rng.next(-10.0, 10.0), -5.130375501385842);
assert_eq(rng.next(-10.0, 10.0), -3.0351627041509293);
assert_eq(rng.next(-10.0, 10.0), -4.8407819174603075);
assert_eq(rng.next(-10.0, 10.0), 4.134284076597936);
       "#,
        );
    }

    #[test]
    fn test_scan() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(
            &mut sim,
            1.0,
            2.0,
            3.0,
            4.0,
            std::f64::consts::PI,
            ship::fighter(0),
        );
        let _ship1 = ship::create(
            &mut sim,
            100.0,
            2.0,
            3.0,
            4.0,
            std::f64::consts::PI,
            ship::fighter(1),
        );
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
let contact = ship.scan();
assert_eq(contact.found, true);
assert_eq(contact.position, vec2(100, 2));
assert_eq(contact.velocity, vec2(3, 4));
        ",
        );
    }

    #[test]
    fn test_angle_diff() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            r#"
assert_eq(angle_diff(0.0, 0.0), 0.0);
assert_eq(angle_diff(0.0, PI()/2), PI()/2);
assert_eq(angle_diff(0.0, PI()), PI());
assert_eq(angle_diff(0.0, 3*PI()/2), -PI()/2);

assert_eq(angle_diff(PI()/2, PI()/2), 0.0);
assert_eq(angle_diff(PI()/2, PI()), PI()/2);
assert_eq(angle_diff(PI()/2, 3*PI()/2), PI());
assert_eq(angle_diff(PI()/2, 0.0), -PI()/2);

assert_eq(angle_diff(-PI()/2, -PI()/2), 0.0);
assert_eq(angle_diff(-PI()/2, 0.0), PI()/2);
assert_eq(angle_diff(-PI()/2, PI()/2), PI());
assert_eq(angle_diff(-PI()/2, PI()), -PI()/2);
       "#,
        );
    }
}
