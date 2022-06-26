use crate::rng;
use crate::ship::{ShipClass, ShipHandle};
use crate::simulation::{Line, Simulation};
use nalgebra::{vector, Point2, UnitComplex, Vector2};
use rand::Rng;
use rand_distr::StandardNormal;
use rng::SeededRng;
use std::f64::consts::TAU;

#[derive(Clone, Debug)]
pub struct Radar {
    pub heading: f64,
    pub width: f64,
    pub power: f64,
    pub rx_cross_section: f64,
    pub min_rssi: f64,
    pub classify_rssi: f64,
    pub result: Option<ScanResult>,
}

struct RadarBeam {
    center: Point2<f64>,
    width: f64,
    start_bearing: f64,
    end_bearing: f64,
    power: f64,
    center_vec: Vector2<f64>,
}

#[derive(Copy, Clone, Debug)]
pub struct ScanResult {
    pub class: Option<ShipClass>,
    pub position: Vector2<f64>,
    pub velocity: Vector2<f64>,
}

pub fn scan(sim: &mut Simulation, own_ship: ShipHandle) -> Option<ScanResult> {
    if let Some(radar) = sim.ship(own_ship).data().radar.as_ref() {
        radar.result
    } else {
        None
    }
}

pub fn tick(sim: &mut Simulation) {
    let handle_snapshot: Vec<ShipHandle> = sim.ships.iter().cloned().collect();
    for own_ship in handle_snapshot {
        if let Some(radar) = sim.ship_mut(own_ship).data_mut().radar.clone() {
            let mut result = None;
            let own_team = sim.ship(own_ship).data().team;
            let own_position: Point2<f64> = sim.ship(own_ship).position().vector.into();
            let own_heading = sim.ship(own_ship).heading();
            let beam = compute_beam(&radar, own_position, own_heading);
            let mut best_rssi = 0.0;
            let mut rng = rng::new_rng(sim.tick());
            for &other in sim.ships.iter() {
                if sim.ship(other).data().team == own_team {
                    continue;
                }
                let rssi = compute_rssi(sim, &beam, own_ship, other);
                if rssi > radar.min_rssi && (result.is_none() || rssi > best_rssi) {
                    result = Some(ScanResult {
                        class: if rssi > radar.classify_rssi {
                            Some(sim.ship(other).data().class)
                        } else {
                            None
                        },
                        position: sim.ship(other).position().vector + noise(&mut rng, rssi),
                        velocity: sim.ship(other).velocity() + noise(&mut rng, rssi),
                    });
                    best_rssi = rssi;
                }
            }
            draw_beam(sim, own_ship, &radar, &beam);
            sim.ship_mut(own_ship)
                .data_mut()
                .radar
                .as_mut()
                .unwrap()
                .result = result;
        }
    }
}

fn compute_beam(radar: &Radar, ship_position: Point2<f64>, ship_heading: f64) -> RadarBeam {
    let h = radar.heading + ship_heading;
    let w = radar.width;
    RadarBeam {
        center: ship_position,
        power: radar.power,
        width: w,
        start_bearing: h - 0.5 * w,
        end_bearing: h + 0.5 * w,
        center_vec: UnitComplex::new(h).transform_vector(&vector![1.0, 0.0]),
    }
}

fn compute_rssi(sim: &Simulation, beam: &RadarBeam, source: ShipHandle, target: ShipHandle) -> f64 {
    let other_position: Point2<f64> = sim.ship(target).position().vector.into();
    if (other_position - beam.center).angle(&beam.center_vec) > beam.width * 0.5 {
        return 0.0;
    }
    let r_sq = nalgebra::distance_squared(&beam.center, &other_position);
    let target_cross_section = sim.ship(target).data().radar_cross_section;
    let rx_cross_section = sim
        .ship(source)
        .data()
        .radar
        .as_ref()
        .unwrap()
        .rx_cross_section;
    beam.power * target_cross_section * rx_cross_section / (TAU * beam.width * r_sq)
}

fn compute_approx_range(radar: &Radar, beam: &RadarBeam) -> f64 {
    let target_cross_section = 5.0;
    (beam.power * target_cross_section * radar.rx_cross_section
        / (TAU * beam.width * radar.min_rssi))
        .sqrt()
}

fn noise(rng: &mut SeededRng, rssi: f64) -> Vector2<f64> {
    vector![rng.sample(StandardNormal), rng.sample(StandardNormal)] * (1.0 / rssi)
}

fn draw_beam(sim: &mut Simulation, ship: ShipHandle, radar: &Radar, beam: &RadarBeam) {
    let color = vector![0.1, 0.2, 0.3, 1.0];
    let mut lines = vec![];
    let n = 20;
    let w = beam.end_bearing - beam.start_bearing;
    let center = beam.center;
    let r = compute_approx_range(radar, beam);
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
    sim.emit_debug_lines(ship, &lines);
}
