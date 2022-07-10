use crate::ship::ShipHandle;
use crate::simulation::Simulation;
use nalgebra::{vector, Point2, Vector4};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Line {
    pub a: Point2<f64>,
    pub b: Point2<f64>,
    pub color: Vector4<f32>,
}

pub fn emit_ship(sim: &mut Simulation, handle: ShipHandle) {
    let mut lines = vec![];
    lines.reserve(3);
    let body = sim.ship(handle).body();
    let p = body.position().translation.vector.into();
    lines.push(Line {
        a: p,
        b: p + body.linvel(),
        color: vector![0.0, 0.81, 1.0, 1.0],
    });
    lines.push(Line {
        a: p,
        b: p + body.rotation().transform_vector(&vector![50.0, 0.0]),
        color: vector![1.0, 0.2, 0.0, 1.0],
    });
    lines.push(Line {
        a: p,
        b: p + body
            .rotation()
            .transform_vector(&sim.ship(handle).data().acceleration),
        color: vector![0.0, 1.0, 0.2, 1.0],
    });
    sim.emit_debug_lines(handle, &lines);
}

pub fn convert_color(c: u32) -> Vector4<f32> {
    let extract_color = |k: i64| -> f32 { ((((c as u32) >> (k * 8)) & 0xff) as f32) / 255.0 };
    vector![extract_color(2), extract_color(1), extract_color(0), 1.0]
}
