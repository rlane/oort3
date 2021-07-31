use super::vec2::Vec2;
use crate::simulation::ship::ShipHandle;
use crate::simulation::{Line, Simulation};
use nalgebra::vector;
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

    #[rhai_fn(pure)]
    pub fn draw_line(obj: &mut DebugApi, a: Vec2, b: Vec2, c: i64) {
        let extract_color = |k: i64| -> f32 { ((((c as u32) >> (k * 8)) & 0xff) as f32) / 255.0 };
        let color = vector![extract_color(2), extract_color(1), extract_color(0), 1.0];
        obj.sim().emit_debug_lines(&[Line {
            a: a.into(),
            b: b.into(),
            color,
        }]);
    }
}
