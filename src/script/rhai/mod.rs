mod ast_rewrite;
mod debug;
mod globals;
mod math;
mod radar;
mod random;
mod ship;
mod testing;
mod util;
mod vec2;

use self::vec2::Vec2;
use super::{ShipController, TeamController};
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use rhai::plugin::*;
use rhai::{Dynamic, Engine, Scope, AST};
use smartstring::alias::CompactString;
use std::rc::Rc;

pub fn new_engine() -> Engine {
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
    engine.register_global_module(exported_module!(testing::plugin).into());
    engine.register_global_module(exported_module!(debug::plugin).into());

    engine
}

pub struct RhaiTeamController {
    ast: Rc<AST>,
    stripped_ast: Rc<AST>,
}

impl RhaiTeamController {
    pub fn create(code: &str) -> Result<Box<dyn TeamController>, super::Error> {
        let engine = new_engine();
        match engine.compile(code) {
            Ok(ast) => {
                let ast = ast_rewrite::rewrite_ast(ast);
                let mut stripped_ast = ast.clone();
                stripped_ast.clear_statements();
                Ok(Box::new(RhaiTeamController {
                    ast: Rc::new(ast),
                    stripped_ast: Rc::new(stripped_ast),
                }))
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
}

impl TeamController for RhaiTeamController {
    fn create_ship_controller(
        &mut self,
        handle: ShipHandle,
        sim: *mut Simulation,
    ) -> Result<Box<dyn ShipController>, super::Error> {
        let mut engine = new_engine();

        let (i, j) = handle.0.into_raw_parts();
        let seed = ((i as i64) << 32) | j as i64;
        let rng = self::random::plugin::new_rng(seed);

        let ship = ship::plugin::ShipApi { handle, sim };
        let radar = radar::plugin::RadarApi { handle, sim };
        let dbg = debug::plugin::DebugApi { handle, sim };
        let mut globals_map = Box::new(std::collections::HashMap::new());
        let globals = globals::plugin::Globals {
            map: &mut *globals_map,
        };
        globals_map.insert("rng".into(), Dynamic::from(rng));
        engine.on_var(move |name, _index, _context| match name {
            "api" => Ok(Some(Dynamic::from(ship))),
            "ship" => Ok(Some(Dynamic::from(ship))),
            "radar" => Ok(Some(Dynamic::from(radar))),
            "dbg" => Ok(Some(Dynamic::from(dbg))),
            "globals" => Ok(Some(Dynamic::from(globals))),
            _ => Ok(None),
        });

        let mut ship_ctrl = Box::new(RhaiShipController {
            engine,
            scope: Scope::new(),
            ast: self.ast.clone(),
            globals_map,
        });

        let result = ship_ctrl
            .engine
            .consume_ast_with_scope(&mut ship_ctrl.scope, &self.ast);
        if let Err(e) = result {
            error!("Script error: {}", e);
            return Err(super::Error {
                line: extract_line(&e.to_string()),
                msg: e.to_string(),
            });
        }

        ship_ctrl.ast = self.stripped_ast.clone();

        Ok(ship_ctrl)
    }
}

pub struct RhaiShipController {
    engine: Engine,
    scope: Scope<'static>,
    #[allow(unused)]
    globals_map: Box<std::collections::HashMap<CompactString, Dynamic>>,
    ast: Rc<AST>,
}

impl ShipController for RhaiShipController {
    fn tick(&mut self) -> Result<(), super::Error> {
        let result: Result<(), _> = self.engine.call_fn(&mut self.scope, &*self.ast, "tick", ());
        if let Err(e) = result {
            error!("Script error: {}", e);
            return Err(super::Error {
                line: extract_line(&e.to_string()),
                msg: e.to_string(),
            });
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

pub fn check_errors(sim: &mut Simulation) {
    let events = sim.events();
    if !events.errors.is_empty() {
        panic!("Test failed: {:?}", events.errors);
    }
}

#[cfg(test)]
mod test {
    use super::check_errors;
    use crate::simulation::ship;
    use crate::simulation::Simulation;
    use test_env_log::test;

    #[test]
    fn test_vec2() {
        let mut sim = Simulation::new(
            "test",
            0,
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
        ship::create(&mut sim, -100.0, 0.0, 100.0, 0.0, 0.0, ship::fighter(0));
        check_errors(&mut sim);
    }

    #[test]
    fn test_vec2_angle() {
        let mut sim = Simulation::new(
            "test",
            0,
            "
        assert_eq(vec2(1.0, 0.0).angle(), 0.0);
        assert_eq(vec2(0.0, 1.0).angle(), PI() / 2.0);
        assert_eq(vec2(-1.0, 0.0).angle(), PI());
        assert_eq(vec2(0.0, -1.0).angle(), 3 * PI() / 2.0);
        ",
        );
        ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        check_errors(&mut sim);
    }

    #[test]
    fn test_pos_vel_hd() {
        let mut sim = Simulation::new(
            "test",
            0,
            "
        assert_eq(ship.position(), vec2(1.0, 2.0));
        assert_eq(ship.velocity(), vec2(3.0, 4.0));
        assert_eq(ship.heading(), PI());
        ",
        );
        ship::create(
            &mut sim,
            1.0,
            2.0,
            3.0,
            4.0,
            std::f64::consts::PI,
            ship::fighter(0),
        );
        check_errors(&mut sim);
    }

    #[test]
    fn test_function() {
        let mut sim = Simulation::new(
            "test",
            0,
            "
            fn foo() {
                assert_eq(ship.position(), vec2(0.0, 0.0));
            }

            assert_eq(ship.velocity(), vec2(0.0, 0.0));
            foo();
        ",
        );
        ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        check_errors(&mut sim);
    }

    #[test]
    fn test_globals() {
        let mut sim = Simulation::new(
            "test",
            0,
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
        ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        check_errors(&mut sim);
    }

    #[test]
    fn test_mixed_integer_float() {
        let mut sim = Simulation::new(
            "test",
            0,
            r#"
assert_eq(vec2(1, 2), vec2(1.0, 2.0));
assert_eq(vec2(1.0, 2), vec2(1, 2.0));
assert_eq(vec2(1, 1) * 2.0, vec2(1, 1) * 2);
assert_eq(2.0 * vec2(1, 1), 2 * vec2(1, 1));
assert_eq(vec2(1, 1) / 2.0, vec2(1, 1) / 2);
       "#,
        );
        ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        check_errors(&mut sim);
    }

    #[test]
    #[should_panic(expected = "Too many operations")]
    fn test_infinite_loop() {
        let mut sim = Simulation::new(
            "test",
            0,
            r#"
let i = 0;
while true {
    print(`i=${i}`);
    i += 1;
}
       "#,
        );
        ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        check_errors(&mut sim);
    }

    #[test]
    fn test_random() {
        let mut sim = Simulation::new(
            "test",
            0,
            r#"
let rng = new_rng(1);
assert_eq(rng.next(-10.0, 10.0), -5.130375501385842);
assert_eq(rng.next(-10.0, 10.0), -3.0351627041509293);
assert_eq(rng.next(-10.0, 10.0), -4.8407819174603075);
assert_eq(rng.next(-10.0, 10.0), 4.134284076597936);
       "#,
        );
        ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        check_errors(&mut sim);
    }

    #[test]
    fn test_scan() {
        let mut sim = Simulation::new(
            "test",
            0,
            "
let contact = ship.scan();
assert_eq(contact.found, true);
assert_eq(contact.position, vec2(100, 2));
assert_eq(contact.velocity, vec2(3, 4));
        ",
        );
        ship::create(
            &mut sim,
            100.0,
            2.0,
            3.0,
            4.0,
            std::f64::consts::PI,
            ship::fighter(1),
        );
        ship::create(&mut sim, 1.0, 2.0, 3.0, 4.0, 0.0, ship::fighter(0));
        check_errors(&mut sim);
    }

    #[test]
    fn test_angle_diff() {
        let mut sim = Simulation::new(
            "test",
            0,
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
        ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        check_errors(&mut sim);
    }
}
