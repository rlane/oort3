use crate::ship::ShipHandle;
use crate::simulation::Simulation;
use nalgebra::{vector, Point2, UnitComplex, Vector4};
use serde::{Deserialize, Serialize};

const DEBUG_RADAR_RADIUS: bool = false;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Line {
    pub a: Point2<f64>,
    pub b: Point2<f64>,
    pub color: Vector4<f32>,
}

pub fn emit_ship(sim: &mut Simulation, handle: ShipHandle) {
    let mut lines = Vec::with_capacity(2 + sim.ship(handle).data().guns.len());
    let body = sim.ship(handle).body();
    let p = body.position().translation.vector.into();
    lines.push(Line {
        a: p,
        b: p + body.linvel(),
        color: vector![0.0, 0.81, 1.0, 1.0],
    });
    lines.push(Line {
        a: p,
        b: p + sim.ship(handle).data().last_acceleration,
        color: vector![0.0, 1.0, 0.2, 1.0],
    });
    for gun in sim.ship(handle).data().guns.iter() {
        if gun.min_angle == gun.max_angle {
            continue;
        }
        let turret_rot = UnitComplex::new(gun.heading);
        let p0 = p + body.rotation().transform_vector(&gun.offset);
        let p1 = p0 + turret_rot.transform_vector(&vector![10.0, 0.0]);
        lines.push(Line {
            a: p0,
            b: p1,
            color: vector![1.0, 0.0, 0.0, 1.0],
        });
    }
    if DEBUG_RADAR_RADIUS {
        let sides = 20;
        let delta_angle = std::f64::consts::TAU / sides as f64;
        let rotation = UnitComplex::new(delta_angle);
        let mut v = vector![sim.ship(handle).data().radar_radius as f64, 0.0];
        for _ in 0..sides {
            let v2 = rotation.transform_vector(&v);
            lines.push(Line {
                a: p + v,
                b: p + v2,
                color: vector![1.0, 1.0, 0.0, 1.0],
            });
            v = v2;
        }
    }
    sim.emit_debug_lines(handle, lines);
}
