use super::vec2::Vec2;
use crate::simulation::ship::{Radar, ShipAccessor, ShipAccessorMut, ShipHandle};
use crate::simulation::{Line, Simulation};
use nalgebra::{vector, Point2, UnitComplex, Vector2};
use rhai::plugin::*;
use std::f64::consts::TAU;

struct RadarBeam {
    center: Point2<f64>,
    width: f64,
    start_bearing: f64,
    end_bearing: f64,
    range: f64,
    center_vec: Vector2<f64>,
}

#[export_module]
pub mod plugin {
    #[derive(Copy, Clone)]
    pub struct RadarApi {
        pub handle: ShipHandle,
        pub sim: *mut Simulation,
    }

    impl RadarApi {
        #[allow(clippy::mut_from_ref)]
        fn sim(&self) -> &mut Simulation {
            unsafe { &mut *self.sim }
        }

        fn ship(&self) -> ShipAccessor {
            self.sim().ship(self.handle)
        }

        fn ship_mut(&self) -> ShipAccessorMut {
            self.sim().ship_mut(self.handle)
        }
    }

    pub fn set_heading(obj: RadarApi, heading: f64) {
        if let Some(radar) = obj.ship_mut().data_mut().radar.as_mut() {
            radar.heading = heading;
        }
    }

    pub fn set_width(obj: RadarApi, width: f64) {
        if let Some(radar) = obj.ship_mut().data_mut().radar.as_mut() {
            radar.width = width.clamp(TAU / 60.0, TAU);
        }
    }

    pub fn scan(obj: RadarApi) -> ScanResult {
        let mut result = ScanResult {
            found: false,
            position: vector![0.0, 0.0],
            velocity: vector![0.0, 0.0],
        };
        if let Some(radar) = obj.ship_mut().data_mut().radar.clone() {
            let sim = obj.sim();
            let own_team = obj.ship().data().team;
            let own_position: Point2<f64> = obj.ship().position().vector.into();
            let own_heading = obj.ship().heading();
            let beam = compute_beam(&radar, own_position, own_heading);
            let mut best_distance = 0.0;
            for &other in sim.ships.iter() {
                if sim.ship(other).data().team == own_team {
                    continue;
                }
                let other_position: Point2<f64> = sim.ship(other).position().vector.into();
                let distance = nalgebra::distance(&own_position, &other_position);
                if distance > beam.range {
                    continue;
                }
                if (other_position - own_position).angle(&beam.center_vec) > beam.width * 0.5 {
                    continue;
                }
                if !result.found || distance < best_distance {
                    result = ScanResult {
                        found: true,
                        position: other_position.coords,
                        velocity: sim.ship(other).velocity(),
                    };
                    best_distance = distance;
                }
            }
            draw_beam(sim, &beam);
        }
        result
    }

    #[derive(Copy, Clone)]
    pub struct ScanResult {
        pub found: bool,
        pub position: Vec2,
        pub velocity: Vec2,
    }

    #[rhai_fn(get = "found", pure)]
    pub fn get_found(obj: &mut ScanResult) -> bool {
        obj.found
    }

    #[rhai_fn(get = "position", pure)]
    pub fn get_position(obj: &mut ScanResult) -> Vec2 {
        obj.position
    }

    #[rhai_fn(get = "velocity", pure)]
    pub fn get_velocity(obj: &mut ScanResult) -> Vec2 {
        obj.velocity
    }
}

fn compute_beam(radar: &Radar, ship_position: Point2<f64>, ship_heading: f64) -> RadarBeam {
    let h = radar.heading + ship_heading;
    let w = radar.width;
    RadarBeam {
        center: ship_position,
        range: radar.power / (radar.width / TAU),
        width: w,
        start_bearing: h - 0.5 * w,
        end_bearing: h + 0.5 * w,
        center_vec: UnitComplex::new(h).transform_vector(&vector![1.0, 0.0]),
    }
}

fn draw_beam(sim: &mut Simulation, beam: &RadarBeam) {
    let color = vector![0.1, 0.2, 0.3, 1.0];
    let mut lines = vec![];
    let n = 20;
    let w = beam.end_bearing - beam.start_bearing;
    let center = beam.center;
    let r = beam.range;
    for i in 0..n {
        let frac = (i as f64) / (n as f64);
        let angle_a = beam.start_bearing + w * frac;
        let angle_b = beam.start_bearing + w * (frac + 1.0 / n as f64);
        lines.push(Line {
            a: center + vector![r * angle_a.cos(), r * angle_a.sin()],
            b: center + vector![r * angle_b.cos(), r * angle_b.sin()],
            color,
        });
    }
    lines.push(Line {
        a: center,
        b: center + vector![r * beam.start_bearing.cos(), r * beam.start_bearing.sin()],
        color,
    });
    lines.push(Line {
        a: center,
        b: center + vector![r * beam.end_bearing.cos(), r * beam.end_bearing.sin()],
        color,
    });
    sim.emit_debug_lines(&lines);
}
