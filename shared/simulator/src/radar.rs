use crate::ship::{self, ShipClass, ShipHandle};
use crate::simulation::{Line, Simulation};
use crate::{model, rng, simulation};
use nalgebra::{vector, ComplexField, Point2, Rotation2, Vector2};
use oort_api::{Ability, EcmMode};
use rand::Rng;
use rand_distr::StandardNormal;
use rapier2d_f64::parry;
use rapier2d_f64::prelude::*;
use std::collections::HashMap;
use std::f64::consts::TAU;
use std::ops::Range;
use wide::{f32x4, CmpGt, CmpLt};

const DEBUG: bool = false;
const BACKGROUND_NOISE: f64 = 1e-13; // -100 dBm
const JAMMER_COEFF: f64 = 1e-9; // Account for frequency hopping and pulse length
const BEARING_NOISE_FACTOR: f64 = 1e1 * (TAU / 360.0);
const DISTANCE_NOISE_FACTOR: f64 = 1e4;
const VELOCITY_NOISE_FACTOR: f64 = 1e2;

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
    pub ecm_mode: EcmMode,
    pub result: Option<ScanResult>,
}

impl Default for Radar {
    fn default() -> Self {
        Radar {
            heading: 0.0,
            width: TAU / 16.0,
            min_width: TAU / 3600.0,
            max_width: TAU / 4.0,
            min_distance: 0.0,
            max_distance: 1e9,
            power: 100e3,
            rx_cross_section: 10.0,
            reliable_rssi: from_dbm(-90.0),
            min_rssi: from_dbm(-100.0),
            ecm_mode: EcmMode::None,
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

    pub fn set_ecm_mode(&mut self, mode: EcmMode) {
        self.ecm_mode = mode;
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
    rays: [Vector2<f64>; 2],
}

#[derive(Clone)]
struct RadarReflector {
    position: Point2<f64>,
    velocity: Vector2<f64>,
    radar_cross_section: f64,
    class: ShipClass,
    jammer: Option<RadarJammer>,
}

#[derive(Clone)]
struct RadarJammer {
    width: f64,
    bearing: f64,
    power: f64,
    ecm_mode: EcmMode,
}

#[derive(Copy, Clone, Debug)]
pub struct ScanResult {
    pub class: ShipClass,
    pub position: Vector2<f64>,
    pub velocity: Vector2<f64>,
    pub rssi: f64,
    pub snr: f64,
}

#[derive(Clone)]
struct ReflectorTeam {
    xs: Vec<f32x4>,
    ys: Vec<f32x4>,
    reflectors: Vec<RadarReflector>,
}

fn into_dbm(x: f64) -> f64 {
    10.0 * ComplexField::log10(x) + 30.0
}

fn from_dbm(x: f64) -> f64 {
    ComplexField::powf(10.0_f64, (x - 30.0) / 10.0)
}

#[inline(never)]
fn build_reflector_team(sim: &Simulation) -> Vec<ReflectorTeam> {
    let mut reflectors_by_team: HashMap<i32, Vec<RadarReflector>> = HashMap::new();

    for handle in sim.ships.iter() {
        let ship = sim.ship(*handle);
        let ship_data = ship.data();

        let mut class = ship_data.class;
        let mut radar_cross_section = ship_data.radar_cross_section;
        if ship.is_ability_active(Ability::Decoy) {
            class = ShipClass::Cruiser;
            radar_cross_section = ship::CRUISER_RADAR_CROSS_SECTION / 2.0;
        }
        if class == ShipClass::Planet {
            continue;
        }
        let jammer = ship_data
            .radar
            .as_ref()
            .and_then(|radar| match radar.ecm_mode {
                EcmMode::None => None,
                _ => Some(RadarJammer {
                    width: radar.width,
                    bearing: radar.heading,
                    power: radar.power,
                    ecm_mode: radar.ecm_mode,
                }),
            });
        reflectors_by_team
            .entry(ship_data.team)
            .or_default()
            .push(RadarReflector {
                position: ship.position().vector.into(),
                velocity: ship.velocity(),
                radar_cross_section,
                class,
                jammer,
            });
    }

    let mut result: Vec<ReflectorTeam> = Vec::new();
    result.resize(
        10,
        ReflectorTeam {
            xs: Vec::new(),
            ys: Vec::new(),
            reflectors: Vec::new(),
        },
    );
    for (team, reflectors) in reflectors_by_team.drain() {
        let positions: Vec<Point2<f32>> = reflectors
            .iter()
            .map(|r| r.position.cast::<f32>())
            .collect();
        let xs = positions
            .chunks(4)
            .map(|chunk| {
                let mut xs = [0.0; 4];
                for (i, p) in chunk.iter().enumerate() {
                    xs[i] = p.x;
                }
                f32x4::from(xs)
            })
            .collect();
        let ys = positions
            .chunks(4)
            .map(|chunk| {
                let mut ys = [0.0; 4];
                for (i, p) in chunk.iter().enumerate() {
                    ys[i] = p.y;
                }
                f32x4::from(ys)
            })
            .collect();
        result[team as usize] = ReflectorTeam { xs, ys, reflectors };
    }

    result
}

#[inline(never)]
pub fn tick(sim: &mut Simulation) {
    let handle_snapshot: Vec<ShipHandle> = sim.ships.iter().cloned().collect();
    let reflector_teams = build_reflector_team(sim);
    let mut candidates: Vec<(i32, usize)> = Vec::new();
    let planets = sim
        .ships
        .iter()
        .filter(|handle| sim.ship(**handle).data().class == ShipClass::Planet)
        .cloned()
        .collect::<Vec<_>>();

    for handle in handle_snapshot.iter().cloned() {
        let ship = sim.ship(handle);
        let ship_data = ship.data();

        if let Some(radar) = ship_data.radar.as_ref() {
            let h = radar.heading;
            let w = radar.width;
            assert!(w < TAU / 2.0);
            let max_distance = compute_max_detection_range(radar, 40.0 /*cruiser*/)
                .min(radar.max_distance)
                .min(simulation::MAX_WORLD_SIZE);
            let reliable_distance = compute_reliable_detection_range(radar, 10.0 /*fighter*/)
                .min(radar.max_distance)
                .min(simulation::MAX_WORLD_SIZE);

            let start_bearing = h - 0.5 * w;
            let end_bearing = h + 0.5 * w;
            let ray0 = Rotation2::new(start_bearing).transform_vector(&vector![1.0, 0.0]);
            let ray1 = Rotation2::new(end_bearing).transform_vector(&vector![1.0, 0.0]);
            assert!(is_clockwise(ray1, ray0));
            let rays = [ray0, ray1];
            let mut emitter = RadarEmitter {
                handle,
                team: ship_data.team,
                center: ship.position().vector.into(),
                power: radar.power,
                reliable_rssi: radar.reliable_rssi,
                min_rssi: radar.min_rssi,
                rx_cross_section: radar.rx_cross_section,
                width: w,
                start_bearing,
                bearing: h,
                end_bearing,
                min_distance: radar.min_distance,
                max_distance,
                square_distance_range: ComplexField::powi(radar.min_distance, 2)
                    ..ComplexField::powi(max_distance, 2),
                rays,
            };

            if radar.ecm_mode != EcmMode::None {
                {
                    let mut ship = sim.ship_mut(emitter.handle);
                    let ship_data = ship.data_mut();
                    let radar = ship_data.radar.as_mut().unwrap();
                    radar.result = None;
                }
                draw_emitter(sim, &emitter, reliable_distance);
                continue;
            }

            let mut rng = rng::new_rng(sim.tick());

            let mut best_rssi = emitter.min_rssi;
            let mut best_reflector: Option<&RadarReflector> = None;
            let mut received_noise =
                BACKGROUND_NOISE * ComplexField::powf(2.0f64, rng.gen_range(-1.0..1.0));
            candidates.clear();

            let planet_contact = check_planet_contact(sim, &emitter, &planets);
            if let Some(planet_distance) =
                planet_contact.map(|c| nalgebra::distance(&c, &emitter.center))
            {
                emitter.square_distance_range.end = ComplexField::powi(planet_distance, 2);
            }

            find_candidates(&emitter, &reflector_teams, &mut candidates);

            for (team, reflector_index) in candidates.iter() {
                let reflector = &reflector_teams[*team as usize].reflectors[*reflector_index];
                if let Some(jammer) = reflector.jammer.as_ref() {
                    match jammer.ecm_mode {
                        EcmMode::None => {}
                        EcmMode::Noise => {
                            if check_inside_beam_raw(
                                &reflector.position,
                                jammer.bearing,
                                jammer.width,
                                &emitter.center,
                            ) {
                                let r_sq = nalgebra::distance_squared(
                                    &emitter.center,
                                    &reflector.position,
                                );
                                received_noise +=
                                    JAMMER_COEFF * jammer.power * emitter.rx_cross_section
                                        / (TAU * jammer.width * r_sq);
                            }
                        }
                    }
                }

                if emitter
                    .square_distance_range
                    .contains(&nalgebra::distance_squared(
                        &emitter.center,
                        &reflector.position,
                    ))
                {
                    let rssi = compute_rssi(&emitter, reflector)
                        * ComplexField::powf(1.2f64, rng.gen_range(-1.0..1.0));
                    if rssi > best_rssi {
                        best_reflector = Some(reflector);
                        best_rssi = rssi;
                    }
                }
            }

            let best_rssi_dbm = into_dbm(best_rssi);
            let received_noise_dbm = into_dbm(received_noise);
            let signal_db = best_rssi_dbm - received_noise_dbm;

            if DEBUG {
                if let Some(reflector) = best_reflector {
                    sim.emit_debug_text(
                        handle,
                        format!(
                            "Radar contact range {:.1} km rssi {:.1} dBm noise {:.1} dBm signal {:.1} dB",
                            (reflector.position - emitter.center).norm() * 1e-3,
                            into_dbm(best_rssi),
                            into_dbm(received_noise),
                            signal_db,
                        ),
                    );
                }
            }

            let result = if signal_db < 3.0
                || best_rssi < emitter.min_rssi
                || (best_rssi < emitter.reliable_rssi
                    && decide_unreliable_rssi(&mut rng, best_rssi, emitter.reliable_rssi))
            {
                None
            } else {
                best_reflector.map(|reflector| {
                    make_scan_result(
                        &emitter,
                        reflector,
                        best_rssi_dbm,
                        received_noise_dbm,
                        &mut rng,
                    )
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

#[inline(never)]
fn find_candidates(
    emitter: &RadarEmitter,
    reflector_teams: &[ReflectorTeam],
    candidates: &mut Vec<(i32, usize)>,
) {
    let rays = [emitter.rays[0].cast::<f32>(), emitter.rays[1].cast::<f32>()];
    let emitter_position = emitter.center.cast::<f32>();

    let wex = f32x4::splat(emitter_position.x);
    let wey = f32x4::splat(emitter_position.y);
    let wrx0 = f32x4::splat(rays[0].x);
    let wry0 = f32x4::splat(rays[0].y);
    let wrx1 = f32x4::splat(rays[1].x);
    let wry1 = f32x4::splat(rays[1].y);

    for (team, reflector_team) in reflector_teams.iter().enumerate() {
        let team = team as i32;
        if emitter.team == team || reflector_team.reflectors.is_empty() {
            continue;
        }

        let n = reflector_team.reflectors.len();
        for (i, (&wx, &wy)) in reflector_team.xs.iter().zip(&reflector_team.ys).enumerate() {
            let wdx = wx - wex;
            let wdy = wy - wey;

            // Positive if true.
            fn is_clockwise(wx0: f32x4, wy0: f32x4, wx1: f32x4, wy1: f32x4) -> f32x4 {
                -wx0 * wy1 + wy0 * wx1
            }

            let mask = is_clockwise(wrx0, wry0, wdx, wdy).cmp_lt(f32x4::ZERO)
                & is_clockwise(wrx1, wry1, wdx, wdy).cmp_gt(f32x4::ZERO);
            if mask.any() {
                for (j, &v) in mask.to_array().iter().enumerate() {
                    let reflector_index = i * 4 + j;
                    if v != 0.0 && reflector_index < n {
                        candidates.push((team, reflector_index));
                    }
                }
            }
        }
    }
}

#[inline(never)]
fn make_scan_result(
    emitter: &RadarEmitter,
    reflector: &RadarReflector,
    rssi_dbm: f64,
    noise_dbm: f64,
    rng: &mut impl Rng,
) -> ScanResult {
    let signal_db = rssi_dbm - noise_dbm;
    let error_factor = ComplexField::powf(10.0f64, -signal_db / 10.0);
    let dp = reflector.position - emitter.center;
    let beam_rot = Rotation2::new(emitter.bearing);
    let reflector_rot = Rotation2::rotation_between(&Vector2::x(), &dp);
    let mut noisy_bearing: f64 = reflector_rot.angle()
        + rng.sample::<f64, _>(StandardNormal) * (BEARING_NOISE_FACTOR * error_factor);
    {
        let angle_to = Rotation2::new(noisy_bearing).angle_to(&beam_rot);
        if angle_to > emitter.width * 0.5 {
            noisy_bearing = emitter.bearing - emitter.width * 0.5;
        } else if angle_to < -emitter.width * 0.5 {
            noisy_bearing = emitter.bearing + emitter.width * 0.5;
        }
    }

    let mut distance = (reflector.position - emitter.center).magnitude();
    distance += rng.sample::<f64, _>(StandardNormal) * (DISTANCE_NOISE_FACTOR * error_factor);
    distance = distance.clamp(emitter.min_distance, emitter.max_distance);

    let position = emitter.center.coords
        + Rotation2::new(noisy_bearing).transform_vector(&vector![distance, 0.0]);
    let velocity = reflector.velocity
        + vector![rng.sample(StandardNormal), rng.sample(StandardNormal)]
            * (VELOCITY_NOISE_FACTOR * error_factor);

    ScanResult {
        class: reflector.class,
        position,
        velocity,
        rssi: rssi_dbm,
        snr: signal_db,
    }
}

fn decide_unreliable_rssi(rng: &mut impl Rng, rssi: f64, reliable_rssi: f64) -> bool {
    rng.gen_bool(1.0 / ComplexField::log2(2.0 * reliable_rssi / rssi))
}

fn is_clockwise(v0: Vector2<f64>, v1: Vector2<f64>) -> bool {
    -v0.x * v1.y + v0.y * v1.x > 0.0
}

fn check_inside_beam_raw(
    src_position: &Point2<f64>,
    bearing: f64,
    width: f64,
    dst_position: &Point2<f64>,
) -> bool {
    let ray0 = Rotation2::new(bearing - width * 0.5).transform_vector(&vector![1.0, 0.0]);
    let ray1 = Rotation2::new(bearing + width * 0.5).transform_vector(&vector![1.0, 0.0]);
    let dp = dst_position - src_position;
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
    ComplexField::powf(
        radar.power * target_cross_section * radar.rx_cross_section
            / (TAU * radar.width * radar.min_rssi),
        0.25,
    )
}

fn compute_reliable_detection_range(radar: &Radar, target_cross_section: f64) -> f64 {
    ComplexField::powf(
        radar.power * target_cross_section * radar.rx_cross_section
            / (TAU * radar.width * radar.reliable_rssi),
        0.25,
    )
}

fn check_planet_contact(
    sim: &Simulation,
    emitter: &RadarEmitter,
    planets: &[ShipHandle],
) -> Option<Point2<f64>> {
    if planets.is_empty() {
        return None;
    }

    let v = Rotation2::new(emitter.width / 2.0).transform_point(&point![1e6, 0.0]);
    let emitter_shape = parry::shape::Triangle::new(point![0.0, 0.0], v, point![v.x, -v.y]);
    let emitter_isometry = Isometry::new(emitter.center.coords, emitter.bearing);

    planets
        .iter()
        .filter_map(|handle| {
            let radius = model::radius(sim.ship(*handle).data().class) as f64;
            let planet_shape = parry::shape::Ball::new(radius);
            let planet_isometry = *sim.ship(*handle).body().position();

            parry::query::contact(
                &emitter_isometry,
                &emitter_shape,
                &planet_isometry,
                &planet_shape,
                0.0,
            )
            .unwrap()
            .map(|contact| [contact.point1, contact.point2])
        })
        .flatten()
        .min_by_key(|p| (emitter.center - p.coords).coords.norm_squared() as i64)
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
            let a = center + Rotation2::new(angle_a).transform_vector(&vector![r, 0.0]);
            let b = center + Rotation2::new(angle_b).transform_vector(&vector![r, 0.0]);
            lines.push(Line { a, b, color });
        }
    };
    draw_arc(emitter.min_distance);
    draw_arc(reliable_distance);
    lines.push(Line {
        a: center,
        b: center
            + Rotation2::new(emitter.start_bearing)
                .transform_vector(&vector![reliable_distance, 0.0]),
        color,
    });
    lines.push(Line {
        a: center,
        b: center
            + Rotation2::new(emitter.end_bearing)
                .transform_vector(&vector![reliable_distance, 0.0]),
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
    use oort_api::EcmMode;
    use rand::Rng;
    use std::f64::consts::{PI, TAU};
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
            let offset = vector![-90e3, 0.0];
            let ship0 = ship::create(
                &mut sim,
                vector![0.0, 0.0] + offset,
                vector![0.0, 0.0],
                0.0,
                class_to_ship_data(emitter_class, 0),
            );
            ship::create(
                &mut sim,
                vector![range, 0.0] + offset,
                vector![0.0, 0.0],
                0.0,
                class_to_ship_data(reflector_class, 1),
            );
            sim.ship_mut(ship0).radar_mut().unwrap().heading = 0.0;
            sim.ship_mut(ship0).radar_mut().unwrap().width = TAU / 360.0;

            (0..100)
                .map(|_| {
                    sim.step();
                    sim.ship(ship0).radar().unwrap().result.is_some()
                })
                .filter(|x| *x)
                .count()
                > 50
        };

        use ShipClass::*;
        assert!(check_detection(Fighter, Missile, 15e3));
        assert!(!check_detection(Fighter, Missile, 40e3));
        assert!(check_detection(Fighter, Torpedo, 30e3));
        assert!(!check_detection(Fighter, Torpedo, 50e3));
        assert!(check_detection(Fighter, Fighter, 70e3));
        assert!(!check_detection(Fighter, Fighter, 120e3));
        assert!(check_detection(Fighter, Frigate, 70e3));
        assert!(!check_detection(Fighter, Frigate, 150e3));
        assert!(check_detection(Fighter, Cruiser, 100e3));
        assert!(!check_detection(Fighter, Cruiser, 150e3));
    }

    #[test]
    fn test_jamming() {
        let check_detection = |range| {
            let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);
            let ship0 = ship::create(
                &mut sim,
                vector![0.0, 0.0],
                vector![0.0, 0.0],
                0.0,
                ship::fighter(0),
            );
            let ship1 = ship::create(
                &mut sim,
                vector![range, 0.0],
                vector![0.0, 0.0],
                0.0,
                ship::fighter(1),
            );
            sim.ship_mut(ship0).radar_mut().unwrap().heading = 0.0;
            sim.ship_mut(ship0).radar_mut().unwrap().width = TAU / 360.0;
            sim.ship_mut(ship1).radar_mut().unwrap().heading = PI;
            sim.ship_mut(ship1).radar_mut().unwrap().width = TAU / 360.0;
            sim.ship_mut(ship1).radar_mut().unwrap().ecm_mode = EcmMode::Noise;
            (0..10)
                .map(|_| {
                    sim.step();
                    sim.ship(ship0).radar().unwrap().result.is_some()
                })
                .filter(|x| *x)
                .count()
                > 6
        };

        assert!(check_detection(50e3));
        assert!(!check_detection(70e3));
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
            if (p1 - p0).magnitude() < 20.0 {
                continue;
            }
            let h = rng.gen_range(0.0..TAU);
            let w = rng.gen_range(0.0..(TAU / 4.0));

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
