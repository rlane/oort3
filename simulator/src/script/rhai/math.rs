use rhai::plugin::*;

#[export_module]
pub mod plugin {
    #[rhai_fn(name = "min")]
    pub fn min_ii(a: i64, b: i64) -> i64 {
        a.min(b)
    }

    #[rhai_fn(name = "min")]
    pub fn min_if(a: i64, b: f64) -> f64 {
        (a as f64).min(b)
    }

    #[rhai_fn(name = "min")]
    pub fn min_fi(a: f64, b: i64) -> f64 {
        a.min(b as f64)
    }

    #[rhai_fn(name = "min")]
    pub fn min_ff(a: f64, b: f64) -> f64 {
        a.min(b)
    }

    #[rhai_fn(name = "max")]
    pub fn max_ii(a: i64, b: i64) -> i64 {
        a.max(b)
    }

    #[rhai_fn(name = "max")]
    pub fn max_if(a: i64, b: f64) -> f64 {
        (a as f64).max(b)
    }

    #[rhai_fn(name = "max")]
    pub fn max_fi(a: f64, b: i64) -> f64 {
        a.max(b as f64)
    }

    #[rhai_fn(name = "max")]
    pub fn max_ff(a: f64, b: f64) -> f64 {
        a.max(b)
    }
}

#[cfg(test)]
mod test {
    use crate::script::rhai::check_errors;
    use crate::ship;
    use crate::simulation::Simulation;
    use test_env_log::test;

    #[test]
    fn test_min_max() {
        let mut sim = Simulation::new(
            "test",
            0,
            r#"
assert_eq(min(5, 7), 5);
assert_eq(min(5, 7.0), 5.0);
assert_eq(min(5.0, 7), 5.0);
assert_eq(min(5.0, 7.0), 5.0);
assert_eq(min(7.0, 5.0), 5.0);

assert_eq(max(5, 7), 7);
assert_eq(max(5, 7.0), 7.0);
assert_eq(max(5.0, 7), 7.0);
assert_eq(max(5.0, 7.0), 7.0);
assert_eq(max(7.0, 5.0), 7.0);

assert_eq(7.min(10).max(3), 7);
assert_eq(-1.min(10).max(3), 3);
assert_eq(12.min(10).max(3), 10);
       "#,
        );
        ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        check_errors(&mut sim);
    }
}
