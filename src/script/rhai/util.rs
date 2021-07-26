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

    fn normalize_angle(a: f64) -> f64 {
        use std::f64::consts::TAU;
        let mut a = a;
        if a.abs() > TAU {
            a %= TAU;
        }
        if a < 0.0 {
            a += TAU;
        }
        a
    }

    #[rhai_fn(name = "angle_diff")]
    pub fn angle_diff_ff(a: f64, b: f64) -> f64 {
        use std::f64::consts::PI;
        use std::f64::consts::TAU;
        let c = normalize_angle(b - a);
        if c > PI {
            c - TAU
        } else {
            c
        }
    }

    #[rhai_fn(name = "angle_diff")]
    pub fn angle_diff_fi(a: f64, b: i64) -> f64 {
        angle_diff_ff(a, b as f64)
    }

    #[rhai_fn(name = "angle_diff")]
    pub fn angle_diff_if(a: i64, b: f64) -> f64 {
        angle_diff_ff(a as f64, b)
    }

    #[rhai_fn(name = "angle_diff")]
    pub fn angle_diff_ii(a: i64, b: i64) -> f64 {
        angle_diff_ff(a as f64, b as f64)
    }
}
