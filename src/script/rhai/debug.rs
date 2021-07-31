use super::vec2::Vec2;
use crate::simulation::ship::ShipHandle;
use crate::simulation::{Line, Simulation};
use nalgebra::{vector, Point2, Vector4};
use rhai::plugin::*;

#[export_module]
pub mod plugin {
    #[derive(Copy, Clone)]
    pub struct DebugApi {
        pub handle: ShipHandle,
        pub sim: *mut Simulation,
    }

    impl DebugApi {
        #[allow(clippy::mut_from_ref)]
        fn sim(&self) -> &mut Simulation {
            unsafe { &mut *self.sim }
        }
    }

    fn make_color(c: i64) -> Vector4<f32> {
        let extract_color = |k: i64| -> f32 { ((((c as u32) >> (k * 8)) & 0xff) as f32) / 255.0 };
        vector![extract_color(2), extract_color(1), extract_color(0), 1.0]
    }

    fn draw_polygon(obj: &mut DebugApi, p: Vec2, r: f64, n: i64, color: i64) {
        let color = make_color(color);
        let mut lines = vec![];
        let center: Point2<f64> = p.into();
        for i in 0..n {
            let frac = (i as f64) / (n as f64);
            let angle_a = std::f64::consts::TAU * frac;
            let angle_b = std::f64::consts::TAU * (frac + 1.0 / n as f64);
            lines.push(Line {
                a: center + vector![r * angle_a.cos(), r * angle_a.sin()],
                b: center + vector![r * angle_b.cos(), r * angle_b.sin()],
                color,
            });
        }
        obj.sim().emit_debug_lines(&lines);
    }

    #[rhai_fn(pure)]
    pub fn draw_line(obj: &mut DebugApi, a: Vec2, b: Vec2, color: i64) {
        obj.sim().emit_debug_lines(&[Line {
            a: a.into(),
            b: b.into(),
            color: make_color(color),
        }]);
    }

    #[rhai_fn(pure)]
    pub fn draw_triangle(obj: &mut DebugApi, p: Vec2, r: f64, color: i64) {
        draw_polygon(obj, p, r, 4, color);
    }

    #[rhai_fn(pure)]
    pub fn draw_diamond(obj: &mut DebugApi, p: Vec2, r: f64, color: i64) {
        draw_polygon(obj, p, r, 4, color);
    }

    #[rhai_fn(pure)]
    pub fn draw_pentagon(obj: &mut DebugApi, p: Vec2, r: f64, color: i64) {
        draw_polygon(obj, p, r, 5, color);
    }

    #[rhai_fn(pure)]
    pub fn draw_hexagon(obj: &mut DebugApi, p: Vec2, r: f64, color: i64) {
        draw_polygon(obj, p, r, 6, color);
    }
}
