use crate::ship::{ShipClass, ShipHandle};
use crate::simulation::{Line, Simulation, PHYSICS_TICK_LENGTH};
use crate::{rng, simulation};
use nalgebra::Rotation2;
use nalgebra::{vector, Point2, Vector2};
use rand::Rng;
use rand_distr::StandardNormal;
use rng::SeededRng;
use std::f64::consts::TAU;
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct Radar {
    pub(crate) heading: f64,
    pub(crate) width: f64,
    pub(crate) min_distance: f64,
    pub(crate) max_distance: f64,
    pub(crate) power: f64,
    pub(crate) rx_cross_section: f64,
    pub(crate) min_rssi: f64,
    pub(crate) classify_rssi: f64,
    pub(crate) result: Option<ScanResult>,
}

impl Default for Radar {
    fn default() -> Self {
        Radar {
            heading: 0.0,
            width: TAU / 6.0,
            min_distance: 0.0,
            max_distance: 1e9,
            power: 100e3,
            rx_cross_section: 10.0,
            min_rssi: 1e-2,
            classify_rssi: 1e-1,
            result: None,
        }
    }
}

impl Radar {
    pub fn get_heading(&self) -> f64 {
        self.heading
    }

    pub fn set_heading(&mut self, heading: f64) {
        self.heading = heading.rem_euclid(TAU);
    }

    pub fn get_width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width.rem_euclid(TAU);
    }

    pub fn get_min_distance(&self) -> f64 {
        self.min_distance
    }

    pub fn set_min_distance(&mut self, dist: f64) {
        self.min_distance = dist.clamp(0.0, simulation::WORLD_SIZE * 2.0);
    }

    pub fn get_max_distance(&self) -> f64 {
        self.max_distance
    }

    pub fn set_max_distance(&mut self, dist: f64) {
        self.max_distance = dist.clamp(0.0, simulation::WORLD_SIZE * 2.0);
    }

    pub fn scan(&self) -> Option<ScanResult> {
        self.result
    }
}

struct RadarEmitter {
    handle: ShipHandle,
    center: Point2<f64>,
    width: f64,
    start_bearing: f64,
    end_bearing: f64,
    min_distance: f64,
    max_distance: f64,
    square_distance_range: Range<f64>,
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

#[inline(never)]
pub fn tick(sim: &mut Simulation) {
    let handle_snapshot: Vec<ShipHandle> = sim.ships.iter().cloned().collect();

    let reflectors: Vec<RadarReflector> = handle_snapshot
        .iter()
        .cloned()
        .map(|handle| {
            let ship = sim.ship(handle);
            let ship_data = ship.data();
            RadarReflector {
                team: ship_data.team,
                position: ship.position().vector.into(),
                velocity: ship.velocity(),
                radar_cross_section: ship_data.radar_cross_section,
                class: ship_data.class,
            }
        })
        .collect();

    for handle in handle_snapshot.iter().cloned() {
        let ship = sim.ship(handle);
        let ship_data = ship.data();

        if let Some(radar) = ship_data.radar.as_ref() {
            let energy_used = radar.power * PHYSICS_TICK_LENGTH;
            if energy_used > ship_data.energy {
                continue;
            }
            let h = radar.heading + ship.heading();
            let w = radar.width;
            let max_distance = compute_max_detection_range(radar, 40.0 /*cruiser*/);
            let emitter = RadarEmitter {
                handle,
                team: ship_data.team,
                center: ship.position().vector.into(),
                power: radar.power,
                min_rssi: radar.min_rssi,
                classify_rssi: radar.classify_rssi,
                rx_cross_section: radar.rx_cross_section,
                width: w,
                start_bearing: h - 0.5 * w,
                end_bearing: h + 0.5 * w,
                min_distance: radar.min_distance,
                max_distance,
                square_distance_range: radar.min_distance.powi(2)..radar.max_distance.powi(2),
            };
            let mut rng = rng::new_rng(sim.tick());

            let mut best_rssi = emitter.min_rssi;
            let mut best_reflector: Option<&RadarReflector> = None;
            for reflector in &reflectors {
                if emitter.team == reflector.team {
                    continue;
                }

                if !check_inside_beam(&emitter, &reflector.position) {
                    continue;
                }

                let rssi = compute_rssi(&emitter, reflector);
                if rssi > best_rssi {
                    best_reflector = Some(reflector);
                    best_rssi = rssi;
                }
            }

            let result = best_reflector.map(|reflector| ScanResult {
                class: if best_rssi > emitter.classify_rssi {
                    Some(reflector.class)
                } else {
                    None
                },
                position: reflector.position.coords + noise(&mut rng, best_rssi),
                velocity: reflector.velocity + noise(&mut rng, best_rssi),
            });

            {
                let mut ship = sim.ship_mut(emitter.handle);
                let ship_data = ship.data_mut();
                let radar = ship_data.radar.as_mut().unwrap();
                ship_data.energy -= energy_used;
                radar.result = result;
            }

            draw_emitter(sim, &emitter);
            if let Some(contact) = &result {
                draw_contact(sim, emitter.handle, contact);
            }
        }
    }
}

fn check_inside_beam(emitter: &RadarEmitter, point: &Point2<f64>) -> bool {
    if !emitter
        .square_distance_range
        .contains(&nalgebra::distance_squared(&emitter.center, point))
    {
        return false;
    }
    if emitter.width >= TAU {
        return true;
    }
    let ray0 = Rotation2::new(emitter.start_bearing).transform_vector(&vector![1.0, 0.0]);
    let ray1 = Rotation2::new(emitter.end_bearing).transform_vector(&vector![1.0, 0.0]);
    let dp = point - emitter.center;
    let is_clockwise = |v0: Vector2<f64>, v1: Vector2<f64>| -v0.x * v1.y + v0.y * v1.x > 0.0;
    if is_clockwise(ray1, ray0) {
        !is_clockwise(ray0, dp) && is_clockwise(ray1, dp)
    } else {
        is_clockwise(ray1, dp) || !is_clockwise(ray0, dp)
    }
}

fn compute_rssi(emitter: &RadarEmitter, reflector: &RadarReflector) -> f64 {
    let r_sq = nalgebra::distance_squared(&emitter.center, &reflector.position);
    emitter.power * reflector.radar_cross_section * emitter.rx_cross_section
        / (TAU * emitter.width * r_sq)
}

fn compute_max_detection_range(radar: &Radar, target_cross_section: f64) -> f64 {
    (radar.power * target_cross_section * radar.rx_cross_section
        / (TAU * radar.width * radar.min_rssi))
        .sqrt()
}

fn noise(rng: &mut SeededRng, rssi: f64) -> Vector2<f64> {
    vector![rng.sample(StandardNormal), rng.sample(StandardNormal)] * (1.0 / rssi)
}

fn draw_emitter(sim: &mut Simulation, emitter: &RadarEmitter) {
    let color = vector![0.1, 0.2, 0.3, 1.0];
    let mut lines = vec![];
    let w = emitter.end_bearing - emitter.start_bearing;
    let center = emitter.center;
    let mut draw_arc = |r| {
        if r < 0.01 {
            return;
        }
        let n = (((20.0 / TAU) * w) as i32).max(3);
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
    };
    draw_arc(emitter.min_distance);
    draw_arc(emitter.max_distance);
    lines.push(Line {
        a: center,
        b: center
            + vector![
                emitter.max_distance * emitter.start_bearing.cos(),
                emitter.max_distance * emitter.start_bearing.sin()
            ],
        color,
    });
    lines.push(Line {
        a: center,
        b: center
            + vector![
                emitter.max_distance * emitter.end_bearing.cos(),
                emitter.max_distance * emitter.end_bearing.sin()
            ],
        color,
    });
    sim.emit_debug_lines(emitter.handle, &lines);
}

fn draw_contact(sim: &mut Simulation, emitter_handle: ShipHandle, contact: &ScanResult) {
    let color = vector![0.9, 0.9, 0.9, 1.0];
    let w = 10.0;
    let center: Point2<f64> = contact.position.into();
    let v0 = center + vector![w, w];
    let v1 = center + vector![w, -w];
    let v2 = center + vector![-w, -w];
    let v3 = center + vector![-w, w];
    let lines = vec![
        Line {
            a: v0,
            b: v1,
            color,
        },
        Line {
            a: v1,
            b: v2,
            color,
        },
        Line {
            a: v2,
            b: v3,
            color,
        },
        Line {
            a: v3,
            b: v0,
            color,
        },
    ];
    sim.emit_debug_lines(emitter_handle, &lines);
}

#[cfg(test)]
mod test {
    use crate::ship;
    use crate::simulation::Code;
    use crate::simulation::Simulation;
    use nalgebra::{vector, UnitComplex};
    use rand::Rng;
    use std::f64::consts::TAU;
    use test_log::test;

    const EPSILON: f64 = 0.01;

    #[test]
    fn test_basic() {
        let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

        // Initial state.
        let ship0 = ship::create(
            &mut sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::fighter(0),
        );
        let ship1 = ship::create(
            &mut sim,
            vector![1000.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::target(1),
        );
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Explicit heading and width.
        sim.ship_mut(ship0).radar_mut().unwrap().heading = 0.0;
        sim.ship_mut(ship0).radar_mut().unwrap().width = TAU / 6.0;
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Just outside of sector (clockwise).
        sim.ship_mut(ship0).radar_mut().unwrap().heading = TAU / 12.0 + EPSILON;
        sim.ship_mut(ship0).radar_mut().unwrap().width = TAU / 6.0;
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), false);

        // Just inside of sector (clockwise).
        sim.ship_mut(ship0).radar_mut().unwrap().heading -= 2.0 * EPSILON;
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Just outside of sector (counter-clockwise).
        sim.ship_mut(ship0).radar_mut().unwrap().heading = -TAU / 12.0 - EPSILON;
        sim.ship_mut(ship0).radar_mut().unwrap().width = TAU / 6.0;
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), false);

        // Just inside of sector (counter-clockwise).
        sim.ship_mut(ship0).radar_mut().unwrap().heading += 2.0 * EPSILON;
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Out of range.
        sim.ship_mut(ship0).radar_mut().unwrap().heading = 0.0;
        sim.ship_mut(ship0).radar_mut().unwrap().width = TAU / 6.0;
        sim.ship_mut(ship1)
            .body()
            .set_translation(vector![1e6, 0.0], true);
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), false);
    }

    #[test]
    fn test_distance_filter() {
        let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

        // Initial state.
        let ship0 = ship::create(
            &mut sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::fighter(0),
        );
        let _ship1 = ship::create(
            &mut sim,
            vector![1000.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::target(1),
        );
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        sim.ship_mut(ship0).radar_mut().unwrap().min_distance = 900.0;
        sim.ship_mut(ship0).radar_mut().unwrap().max_distance = 1100.0;
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        sim.ship_mut(ship0).radar_mut().unwrap().min_distance = 1050.0;
        sim.ship_mut(ship0).radar_mut().unwrap().max_distance = 1100.0;
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), false);

        sim.ship_mut(ship0).radar_mut().unwrap().min_distance = 900.0;
        sim.ship_mut(ship0).radar_mut().unwrap().max_distance = 950.0;
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), false);
    }

    #[test]
    fn test_180_degrees() {
        let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

        // Initial state.
        let ship0 = ship::create(
            &mut sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::fighter(0),
        );
        let ship1 = ship::create(
            &mut sim,
            vector![1000.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::target(1),
        );
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Set width to 180 degrees.
        sim.ship_mut(ship0).radar_mut().unwrap().heading = 0.0;
        sim.ship_mut(ship0).radar_mut().unwrap().width = TAU / 2.0;
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Move target north.
        sim.ship_mut(ship1)
            .body()
            .set_translation(vector![EPSILON, 1000.0], true);
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Move just out of range to the north west.
        sim.ship_mut(ship1)
            .body()
            .set_translation(vector![-EPSILON, 1000.0], true);
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), false);

        // Move target south.
        sim.ship_mut(ship1)
            .body()
            .set_translation(vector![EPSILON, -1000.0], true);
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Move just out of range to the south west.
        sim.ship_mut(ship1)
            .body()
            .set_translation(vector![-EPSILON, -1000.0], true);
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), false);

        // Move target west.
        sim.ship_mut(ship1)
            .body()
            .set_translation(vector![-1000.0, 0.0], true);
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), false);
    }

    #[test]
    fn test_270_degrees() {
        let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

        // Initial state.
        let ship0 = ship::create(
            &mut sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::fighter(0),
        );
        let ship1 = ship::create(
            &mut sim,
            vector![1000.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::target(1),
        );
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Set width to 270 degrees.
        sim.ship_mut(ship0).radar_mut().unwrap().heading = 0.0;
        sim.ship_mut(ship0).radar_mut().unwrap().width = TAU * 3.0 / 4.0;
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Move target up.
        sim.ship_mut(ship1)
            .body()
            .set_translation(vector![0.0, 1000.0], true);
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Move target down.
        sim.ship_mut(ship1)
            .body()
            .set_translation(vector![0.0, -1000.0], true);
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Move target left.
        sim.ship_mut(ship1)
            .body()
            .set_translation(vector![-1000.0, 100.0], true);
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), false);
    }

    #[test]
    fn test_360_degrees() {
        let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

        // Initial state.
        let ship0 = ship::create(
            &mut sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::fighter(0),
        );
        let ship1 = ship::create(
            &mut sim,
            vector![1000.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::target(1),
        );
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Set width to 360 degrees.
        sim.ship_mut(ship0).radar_mut().unwrap().heading = 0.0;
        sim.ship_mut(ship0).radar_mut().unwrap().width = TAU;
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Move target up.
        sim.ship_mut(ship1)
            .body()
            .set_translation(vector![0.0, 1000.0], true);
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Move target down.
        sim.ship_mut(ship1)
            .body()
            .set_translation(vector![0.0, -1000.0], true);
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);

        // Move target left.
        sim.ship_mut(ship1)
            .body()
            .set_translation(vector![-1000.0, 100.0], true);
        sim.step();
        assert_eq!(sim.ship(ship0).radar().unwrap().result.is_some(), true);
    }

    #[test]
    fn test_random() {
        let mut rng = crate::rng::new_rng(1);
        for _ in 0..1000 {
            let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);
            let mut rand_vector =
                || vector![rng.gen_range(-100.0..100.0), rng.gen_range(-100.0..100.0)];
            let p0 = rand_vector();
            let p1 = rand_vector();
            let h = rng.gen_range(0.0..TAU);
            let w = rng.gen_range(0.0..TAU);

            let ship0 = ship::create(&mut sim, p0, vector![0.0, 0.0], h, ship::fighter(0));
            let _ship1 = ship::create(&mut sim, p1, vector![0.0, 0.0], 0.0, ship::target(1));
            sim.ship_mut(ship0).radar_mut().unwrap().width = w;
            sim.step();

            let dp = p1 - p0;
            let center_vec = UnitComplex::new(h).transform_vector(&vector![1.0, 0.0]);
            let expected = dp.angle(&center_vec).abs() < w * 0.5;
            let got = sim.ship(ship0).radar().unwrap().result.is_some();
            assert_eq!(
                got, expected,
                "p0={:?} p1={:?} h={} w={} expected={} got={}",
                p0, p1, h, w, expected, got
            );
        }
    }
}
