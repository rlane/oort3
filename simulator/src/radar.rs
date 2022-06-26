use crate::rng;
use crate::ship::{ShipClass, ShipHandle};
use crate::simulation::{Line, Simulation};
use nalgebra::Rotation2;
use nalgebra::{vector, Point2, Vector2};
use rand::Rng;
use rand_distr::StandardNormal;
use rapier2d_f64::geometry::Triangle;
use rng::SeededRng;
use std::f64::consts::TAU;

const MAX_RADAR_RANGE: f64 = 10000.0;

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

struct RadarEmitter {
    handle: ShipHandle,
    center: Point2<f64>,
    width: f64,
    start_bearing: f64,
    end_bearing: f64,
    power: f64,
    rx_cross_section: f64,
    min_rssi: f64,
    classify_rssi: f64,
    team: i32,
}

struct RadarReflector {
    position: Point2<f64>,
    velocity: Vector2<f64>,
    radar_cross_section: f64,
    team: i32,
    class: ShipClass,
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

    let mut emitters: Vec<RadarEmitter> = Vec::new();
    emitters.reserve(handle_snapshot.len());
    let mut reflectors: Vec<RadarReflector> = Vec::new();
    reflectors.reserve(handle_snapshot.len());
    for handle in handle_snapshot {
        let ship = sim.ship(handle);
        let ship_data = ship.data();
        let position: Point2<f64> = sim.ship(handle).position().vector.into();
        if let Some(radar) = ship_data.radar.as_ref() {
            let heading = sim.ship(handle).heading();
            let h = radar.heading + heading;
            let w = radar.width;
            emitters.push(RadarEmitter {
                handle,
                team: ship_data.team,
                center: position,
                power: radar.power,
                min_rssi: radar.min_rssi,
                classify_rssi: radar.classify_rssi,
                rx_cross_section: radar.rx_cross_section,
                width: w,
                start_bearing: h - 0.5 * w,
                end_bearing: h + 0.5 * w,
            });
        }

        {
            reflectors.push(RadarReflector {
                team: ship_data.team,
                position,
                velocity: sim.ship(handle).velocity(),
                radar_cross_section: ship_data.radar_cross_section,
                class: ship_data.class,
            });
        }
    }

    for emitter in &emitters {
        let mut result = None;
        let mut best_rssi = 0.0;
        let mut rng = rng::new_rng(sim.tick());
        let shape = make_beam_shape(emitter);

        for reflector in &reflectors {
            if emitter.team == reflector.team {
                continue;
            }

            if !shape.contains_point(&reflector.position) {
                continue;
            }

            let rssi = compute_rssi(emitter, reflector);
            if rssi > emitter.min_rssi && (result.is_none() || rssi > best_rssi) {
                result = Some(ScanResult {
                    class: if rssi > emitter.classify_rssi {
                        Some(reflector.class)
                    } else {
                        None
                    },
                    position: reflector.position.coords + noise(&mut rng, rssi),
                    velocity: reflector.velocity + noise(&mut rng, rssi),
                });
                best_rssi = rssi;
            }
        }
        sim.ship_mut(emitter.handle)
            .data_mut()
            .radar
            .as_mut()
            .unwrap()
            .result = result;
    }

    for emitter in &emitters {
        draw_emitter(sim, emitter);
    }
}

fn make_beam_shape(emitter: &RadarEmitter) -> Triangle {
    Triangle::new(
        emitter.center,
        Rotation2::new(emitter.start_bearing)
            .transform_vector(&vector![MAX_RADAR_RANGE, 0.0])
            .into(),
        Rotation2::new(emitter.end_bearing)
            .transform_vector(&vector![MAX_RADAR_RANGE, 0.0])
            .into(),
    )
}

fn compute_rssi(emitter: &RadarEmitter, reflector: &RadarReflector) -> f64 {
    let r_sq = nalgebra::distance_squared(&emitter.center, &reflector.position);
    emitter.power * reflector.radar_cross_section * emitter.rx_cross_section
        / (TAU * emitter.width * r_sq)
}

fn compute_approx_range(emitter: &RadarEmitter) -> f64 {
    let target_cross_section = 5.0;
    (emitter.power * target_cross_section * emitter.rx_cross_section
        / (TAU * emitter.width * emitter.min_rssi))
        .sqrt()
        .min(MAX_RADAR_RANGE)
}

fn noise(rng: &mut SeededRng, rssi: f64) -> Vector2<f64> {
    vector![rng.sample(StandardNormal), rng.sample(StandardNormal)] * (1.0 / rssi)
}

fn draw_emitter(sim: &mut Simulation, emitter: &RadarEmitter) {
    let color = vector![0.1, 0.2, 0.3, 1.0];
    let mut lines = vec![];
    let n = 20;
    let w = emitter.end_bearing - emitter.start_bearing;
    let center = emitter.center;
    let r = compute_approx_range(emitter);
    for i in 0..n {
        let frac = (i as f64) / (n as f64);
        let angle_a = emitter.start_bearing + w * frac;
        let angle_b = emitter.start_bearing + w * (frac + 1.0 / n as f64);
        lines.push(Line {
            a: center + vector![r * angle_a.cos(), r * angle_a.sin()],
            b: center + vector![r * angle_b.cos(), r * angle_b.sin()],
            color,
        });
    }
    lines.push(Line {
        a: center,
        b: center
            + vector![
                r * emitter.start_bearing.cos(),
                r * emitter.start_bearing.sin()
            ],
        color,
    });
    lines.push(Line {
        a: center,
        b: center + vector![r * emitter.end_bearing.cos(), r * emitter.end_bearing.sin()],
        color,
    });
    sim.emit_debug_lines(emitter.handle, &lines);
}
