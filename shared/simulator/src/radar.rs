use crate::ship::{self, ShipClass, ShipHandle};
use crate::simulation::{Line, Simulation};
use crate::{rng, simulation};
use nalgebra::Rotation2;
use nalgebra::{vector, Point2, Vector2};
use oort_api::Ability;
use rand::Rng;
use rand_distr::StandardNormal;
use rapier2d_f64::prelude::*;
use static_aabb2d_index::{StaticAABB2DIndex, StaticAABB2DIndexBuilder};
use std::collections::HashMap;
use std::f64::consts::TAU;
use std::ops::Range;

const DEBUG: bool = false;

#[derive(Clone, Debug)]
pub struct Radar {
    pub heading: f64,
    pub width: f64,
    pub min_width: f64,
    pub max_width: f64,
    pub min_distance: f64,
    pub max_distance: f64,
    pub power: f64,
    pub rx_cross_section: f64,
    pub reliable_rssi: f64,
    pub min_rssi: f64,
    pub result: Option<ScanResult>,
}

impl Default for Radar {
    fn default() -> Self {
        Radar {
            heading: 0.0,
            width: TAU / 16.0,
            min_width: TAU / 360.0,
            max_width: TAU / 16.0,
            min_distance: 0.0,
            max_distance: 1e9,
            power: 100e3,
            rx_cross_section: 10.0,
            reliable_rssi: from_dbm(-90.0),
            min_rssi: from_dbm(-100.0),
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
        self.width = width.clamp(self.min_width, self.max_width);
    }

    pub fn get_min_distance(&self) -> f64 {
        self.min_distance
    }

    pub fn set_min_distance(&mut self, dist: f64) {
        self.min_distance = dist.clamp(0.0, simulation::MAX_WORLD_SIZE * 2.0);
    }

    pub fn get_max_distance(&self) -> f64 {
        self.max_distance
    }

    pub fn set_max_distance(&mut self, dist: f64) {
        self.max_distance = dist.clamp(0.0, simulation::MAX_WORLD_SIZE * 2.0);
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
    bearing: f64,
    end_bearing: f64,
    min_distance: f64,
    max_distance: f64,
    square_distance_range: Range<f64>,
    power: f64,
    rx_cross_section: f64,
    reliable_rssi: f64,
    min_rssi: f64,
    team: i32,
}

struct RadarReflector {
    position: Point2<f64>,
    velocity: Vector2<f64>,
    radar_cross_section: f64,
    class: ShipClass,
}

#[derive(Copy, Clone, Debug)]
pub struct ScanResult {
    pub class: ShipClass,
    pub position: Vector2<f64>,
    pub velocity: Vector2<f64>,
}

struct ReflectorTeam {
    reflectors: Vec<RadarReflector>,
    index: StaticAABB2DIndex<f64>,
}

fn into_dbm(x: f64) -> f64 {
    10.0 * x.log10() + 30.0
}

fn from_dbm(x: f64) -> f64 {
    10.0_f64.powf((x - 30.0) / 10.0)
}

#[inline(never)]
fn build_reflector_team(sim: &Simulation) -> HashMap<i32, ReflectorTeam> {
    let mut aabbs_by_team: HashMap<i32, Vec<Aabb>> = HashMap::new();
    let mut reflectors_by_team: HashMap<i32, Vec<RadarReflector>> = HashMap::new();

    for handle in sim.ships.iter() {
        let ship = sim.ship(*handle);
        let ship_data = ship.data();
        let body = sim.ship(*handle).body();
        let aabb =
            Aabb::from_half_extents(point![0.0, 0.0] + body.translation(), vector![1.0, 1.0]);
        aabbs_by_team.entry(ship_data.team).or_default().push(aabb);

        let mut class = ship_data.class;
        let mut radar_cross_section = ship_data.radar_cross_section;
        if ship.is_ability_active(Ability::Decoy) {
            class = ShipClass::Cruiser;
            radar_cross_section = ship::CRUISER_RADAR_CROSS_SECTION / 2.0;
        }
        reflectors_by_team
            .entry(ship_data.team)
            .or_default()
            .push(RadarReflector {
                position: ship.position().vector.into(),
                velocity: ship.velocity(),
                radar_cross_section,
                class,
            });
    }

    let mut indices_by_team: HashMap<i32, StaticAABB2DIndex<f64>> = HashMap::new();
    for (team, aabbs) in aabbs_by_team {
        let mut builder = StaticAABB2DIndexBuilder::new(aabbs.len());
        for aabb in aabbs {
            builder.add(aabb.mins.x, aabb.mins.y, aabb.maxs.x, aabb.maxs.y);
        }
        indices_by_team.insert(team, builder.build().unwrap());
    }

    let mut result: HashMap<i32, ReflectorTeam> = HashMap::new();
    for (team, reflectors) in reflectors_by_team.drain() {
        result.insert(
            team,
            ReflectorTeam {
                reflectors,
                index: indices_by_team.remove(&team).unwrap(),
            },
        );
    }

    result
}

#[inline(never)]
pub fn tick(sim: &mut Simulation) {
    let handle_snapshot: Vec<ShipHandle> = sim.ships.iter().cloned().collect();
    let indices_by_team = build_reflector_team(sim);

    for handle in handle_snapshot.iter().cloned() {
        let ship = sim.ship(handle);
        let ship_data = ship.data();

        if let Some(radar) = ship_data.radar.as_ref() {
            let h = radar.heading;
            let w = radar.width;
            let max_distance = compute_max_detection_range(radar, 40.0 /*cruiser*/)
                .min(radar.max_distance)
                .min(simulation::MAX_WORLD_SIZE);
            let reliable_distance = compute_reliable_detection_range(radar, 10.0 /*fighter*/)
                .min(radar.max_distance)
                .min(simulation::MAX_WORLD_SIZE);
            let emitter = RadarEmitter {
                handle,
                team: ship_data.team,
                center: ship.position().vector.into(),
                power: radar.power,
                reliable_rssi: radar.reliable_rssi,
                min_rssi: radar.min_rssi,
                rx_cross_section: radar.rx_cross_section,
                width: w,
                start_bearing: h - 0.5 * w,
                bearing: h,
                end_bearing: h + 0.5 * w,
                min_distance: radar.min_distance,
                max_distance,
                square_distance_range: radar.min_distance.powi(2)..max_distance.powi(2),
            };
            let mut rng = rng::new_rng(sim.tick());
            let aabb = make_aabb(&emitter);

            let mut best_rssi = emitter.min_rssi;
            let mut best_reflector: Option<&RadarReflector> = None;

            for (team2, reflector_team) in indices_by_team.iter() {
                if emitter.team == *team2 {
                    continue;
                }

                for reflector_idx in reflector_team.index.query_iter(
                    aabb.mins.x,
                    aabb.mins.y,
                    aabb.maxs.x,
                    aabb.maxs.y,
                ) {
                    let reflector = &reflector_team.reflectors[reflector_idx];

                    if !check_inside_beam(&emitter, &reflector.position) {
                        continue;
                    }

                    let rssi = compute_rssi(&emitter, reflector);
                    if rssi > best_rssi {
                        best_reflector = Some(reflector);
                        best_rssi = rssi;
                    }
                }
            }

            if DEBUG {
                if let Some(reflector) = best_reflector {
                    sim.emit_debug_text(
                        handle,
                        format!(
                            "Radar contact range {:.1} km rssi {:.1} dBm",
                            (reflector.position - emitter.center).norm() * 1e-3,
                            into_dbm(best_rssi)
                        ),
                    );
                }
            }

            let result = if best_rssi < emitter.min_rssi
                || (best_rssi < emitter.reliable_rssi
                    && decide_unreliable_rssi(&mut rng, best_rssi, emitter.reliable_rssi))
            {
                None
            } else {
                best_reflector.map(|reflector| {
                    const BEARING_NOISE: f64 = 1e-15;
                    const DISTANCE_NOISE: f64 = 1e-10;
                    const VELOCITY_NOISE: f64 = 1e-10;
                    let dp = reflector.position - emitter.center;
                    let beam_rot = Rotation2::new(emitter.bearing);
                    let reflector_rot = Rotation2::rotation_between(&Vector2::x(), &dp);
                    let mut noisy_bearing: f64 = reflector_rot.angle()
                        + rng.sample::<f64, _>(StandardNormal) * (BEARING_NOISE / best_rssi);
                    {
                        let angle_to = Rotation2::new(noisy_bearing).angle_to(&beam_rot);
                        if angle_to > emitter.width * 0.5 {
                            noisy_bearing = emitter.bearing - emitter.width * 0.5;
                        } else if angle_to < -emitter.width * 0.5 {
                            noisy_bearing = emitter.bearing + emitter.width * 0.5;
                        }
                    }

                    let mut distance = (reflector.position - emitter.center).magnitude();
                    distance += rng.sample::<f64, _>(StandardNormal) * (DISTANCE_NOISE / best_rssi);
                    distance = distance.clamp(emitter.min_distance, emitter.max_distance);

                    let position = emitter.center.coords
                        + Rotation2::new(noisy_bearing).transform_vector(&vector![distance, 0.0]);
                    let velocity = reflector.velocity
                        + vector![rng.sample(StandardNormal), rng.sample(StandardNormal)]
                            * (VELOCITY_NOISE / best_rssi);

                    ScanResult {
                        class: reflector.class,
                        position,
                        velocity,
                    }
                })
            };

            {
                let mut ship = sim.ship_mut(emitter.handle);
                let ship_data = ship.data_mut();
                let radar = ship_data.radar.as_mut().unwrap();
                radar.result = result;
            }

            draw_emitter(sim, &emitter, reliable_distance);
            if let Some(contact) = &result {
                draw_contact(sim, emitter.handle, contact);
            }
        }
    }
}

fn decide_unreliable_rssi(rng: &mut impl Rng, rssi: f64, reliable_rssi: f64) -> bool {
    rng.gen_bool(1.0 / (2.0 * reliable_rssi / rssi).log2())
}

fn make_aabb(emitter: &RadarEmitter) -> Aabb {
    let mut points = vec![];
    points.reserve(48);
    let w = emitter.end_bearing - emitter.start_bearing;
    let center = emitter.center;
    let mut generate_arc = |r| {
        let n = (((20.0 / TAU) * w) as i32).max(3);
        for i in 0..(n + 1) {
            let frac = (i as f64) / (n as f64);
            let angle = emitter.start_bearing + w * frac;
            points.push(center + vector![r * angle.cos(), r * angle.sin()]);
        }
    };
    generate_arc(emitter.min_distance * 0.9);
    generate_arc(emitter.max_distance * 1.1);
    Aabb::from_points(&points)
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
        / (TAU * emitter.width * r_sq * r_sq)
}

fn compute_max_detection_range(radar: &Radar, target_cross_section: f64) -> f64 {
    (radar.power * target_cross_section * radar.rx_cross_section
        / (TAU * radar.width * radar.min_rssi))
        .powf(0.25)
}

fn compute_reliable_detection_range(radar: &Radar, target_cross_section: f64) -> f64 {
    (radar.power * target_cross_section * radar.rx_cross_section
        / (TAU * radar.width * radar.reliable_rssi))
        .powf(0.25)
}

fn draw_emitter(sim: &mut Simulation, emitter: &RadarEmitter, reliable_distance: f64) {
    let color = vector![0.2, 0.66, 0.97, 1.0];
    let mut lines = vec![];
    lines.reserve(48);
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
    draw_arc(reliable_distance);
    lines.push(Line {
        a: center,
        b: center
            + vector![
                reliable_distance * emitter.start_bearing.cos(),
                reliable_distance * emitter.start_bearing.sin()
            ],
        color,
    });
    lines.push(Line {
        a: center,
        b: center
            + vector![
                reliable_distance * emitter.end_bearing.cos(),
                reliable_distance * emitter.end_bearing.sin()
            ],
        color,
    });
    sim.emit_debug_lines(emitter.handle, lines);
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
    sim.emit_debug_lines(emitter_handle, lines);
}

#[cfg(test)]
mod test {
    use crate::ship;
    use crate::ship::ShipClass;
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
    fn test_detection_range() {
        let class_to_ship_data = |class, team| match class {
            ShipClass::Fighter => ship::fighter(team),
            ShipClass::Frigate => ship::frigate(team),
            ShipClass::Cruiser => ship::cruiser(team),
            ShipClass::Missile => ship::missile(team),
            ShipClass::Torpedo => ship::torpedo(team),
            _ => unimplemented!(),
        };

        let check_detection = |emitter_class, reflector_class, range| {
            let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);
            let ship0 = ship::create(
                &mut sim,
                vector![0.0, 0.0],
                vector![0.0, 0.0],
                0.0,
                class_to_ship_data(emitter_class, 0),
            );
            ship::create(
                &mut sim,
                vector![range, 0.0],
                vector![0.0, 0.0],
                0.0,
                class_to_ship_data(reflector_class, 1),
            );
            sim.ship_mut(ship0).radar_mut().unwrap().heading = 0.0;
            sim.ship_mut(ship0).radar_mut().unwrap().width = TAU / 360.0;

            (0..10)
                .map(|_| {
                    sim.step();
                    sim.ship(ship0).radar().unwrap().result.is_some()
                })
                .filter(|x| *x)
                .count()
                > 6
        };

        use ShipClass::*;
        assert!(check_detection(Fighter, Missile, 30e3));
        assert!(!check_detection(Fighter, Missile, 40e3));
        assert!(check_detection(Fighter, Torpedo, 40e3));
        assert!(!check_detection(Fighter, Torpedo, 50e3));
        assert!(check_detection(Fighter, Fighter, 90e3));
        assert!(!check_detection(Fighter, Fighter, 120e3));
        assert!(check_detection(Fighter, Frigate, 120e3));
        assert!(!check_detection(Fighter, Frigate, 150e3));
        assert!(check_detection(Fighter, Cruiser, 120e3));
        assert!(!check_detection(Fighter, Cruiser, 150e3));
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

            let ship0 = ship::create(&mut sim, p0, vector![0.0, 0.0], 0.0, ship::fighter(0));
            let _ship1 = ship::create(&mut sim, p1, vector![0.0, 0.0], 0.0, ship::target(1));
            sim.ship_mut(ship0).radar_mut().unwrap().heading = h;
            sim.ship_mut(ship0).radar_mut().unwrap().width = w;
            sim.step();

            let dp = p1 - p0;
            let center_vec = UnitComplex::new(h).transform_vector(&vector![1.0, 0.0]);
            let expected = dp.angle(&center_vec).abs() < w * 0.5;
            let got = sim.ship(ship0).radar().unwrap().result.is_some();
            assert_eq!(
                got, expected,
                "p0={p0:?} p1={p1:?} h={h} w={w} expected={expected} got={got}"
            );
        }
    }
}
