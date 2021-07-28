use super::ship::plugin::ShipApi;
use super::vec2::Vec2;
use rhai::plugin::*;

#[export_module]
pub mod plugin {
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
    pub fn assert_eq_bool(a: &mut bool, b: bool) -> Result<(), Box<EvalAltResult>> {
        assert_internal(a, b)
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

    pub fn cheat_set_position(obj: ShipApi, position: Vec2) {
        obj.sim().cheats = true;
        obj.ship_mut().body().set_translation(position, true);
    }

    pub fn cheat_set_velocity(obj: ShipApi, velocity: Vec2) {
        obj.sim().cheats = true;
        obj.ship_mut().body().set_linvel(velocity, true);
    }

    pub fn cheat_set_heading(obj: ShipApi, heading: f64) {
        obj.sim().cheats = true;
        obj.ship_mut().body().set_rotation(heading, true);
    }

    pub fn cheat_set_angular_velocity(obj: ShipApi, angular_velocity: f64) {
        obj.sim().cheats = true;
        obj.ship_mut().body().set_angvel(angular_velocity, true);
    }
}
