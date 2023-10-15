use super::index_set::{HasIndex, Index};
use super::rng::new_rng;
use crate::color;
use crate::model;
use crate::radar::Radar;
use crate::radio::Radio;
use crate::rng;
use crate::simulation::{self, PHYSICS_TICK_LENGTH};
use crate::simulation::{Particle, Simulation};
use crate::{bullet, collision};
use bullet::BulletData;
use nalgebra::{vector, Rotation2, UnitComplex, Vector2};
use oort_api::Ability;
use rand::Rng;
use rapier2d_f64::prelude::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug, Ord, PartialOrd)]
pub struct ShipHandle(pub Index);

impl HasIndex for ShipHandle {
    fn index(self) -> Index {
        self.0
    }
}

impl From<ShipHandle> for u64 {
    fn from(handle: ShipHandle) -> u64 {
        let (gen, idx) = handle.0.into_raw_parts();
        ((gen as u64) << 32) | idx as u64
    }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum ShipClass {
    Fighter,
    Frigate,
    Cruiser,
    Asteroid { variant: i32 },
    Target,
    Missile,
    Torpedo,
    Planet,
}

impl ShipClass {
    pub fn name(&self) -> &'static str {
        match self {
            ShipClass::Fighter => "fighter",
            ShipClass::Frigate => "frigate",
            ShipClass::Cruiser => "cruiser",
            ShipClass::Asteroid { .. } => "asteroid",
            ShipClass::Target => "target",
            ShipClass::Missile => "missile",
            ShipClass::Torpedo => "torpedo",
            ShipClass::Planet => "planet",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Gun {
    pub magazine_size: i32,
    pub magazine_remaining: i32,
    pub magazine_reload_ticks: u32,
    pub reload_ticks: u32,
    pub reload_ticks_remaining: u32,
    pub speed: f64,
    pub speed_error: f64,
    pub offset: Vector2<f64>,
    pub heading: f64,
    pub min_angle: f64,
    pub max_angle: f64,
    pub inaccuracy: f64,
    pub burst_size: i32,
    pub ttl: f32,
    pub bullet_mass: f64,
}

#[derive(Debug, Clone)]
pub struct MissileLauncher {
    pub class: ShipClass,
    pub reload_ticks: u32,
    pub reload_ticks_remaining: u32,
    pub initial_speed: f64,
    pub offset: Vector2<f64>,
    pub angle: f64,
}

#[derive(Debug, Clone)]
pub struct ShipAbility {
    pub ability: Ability,
    pub active_time: f64,
    pub reload_time: f64,
    pub active_time_remaining: f64,
    pub reload_time_remaining: f64,
}

#[derive(Debug, Clone)]
pub struct Warhead {
    pub count: i32,
    pub mass: f32,
    pub width: f64,
    pub speed: f64,
    pub ttl: f32,
}

#[derive(Debug, Clone)]
pub struct ShipData {
    pub class: ShipClass,
    pub team: i32,
    pub health: f64,
    pub mass: f64,
    pub acceleration: Vector2<f64>,
    pub last_acceleration: Vector2<f64>,
    pub angular_acceleration: f64,
    pub max_forward_acceleration: f64,
    pub max_backward_acceleration: f64,
    pub max_lateral_acceleration: f64,
    pub max_angular_acceleration: f64,
    pub destroyed: bool,
    pub crash_message: Option<String>,
    pub ttl: Option<u64>,
    pub fuel: Option<f64>,
    pub guns: Vec<Gun>,
    pub missile_launchers: Vec<MissileLauncher>,
    pub radar: Option<Radar>,
    pub radar_cross_section: f64,
    pub radios: Vec<Radio>,
    pub abilities: Vec<ShipAbility>,
    pub target: Option<Box<Target>>,
    pub warhead: Warhead,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub position: Vector2<f64>,
    pub velocity: Vector2<f64>,
}

impl Default for ShipData {
    fn default() -> ShipData {
        ShipData {
            class: ShipClass::Fighter,
            team: 0,
            health: 100.0,
            mass: 1000.0,
            acceleration: vector![0.0, 0.0],
            last_acceleration: vector![0.0, 0.0],
            angular_acceleration: 0.0,
            max_forward_acceleration: 0.0,
            max_backward_acceleration: 0.0,
            max_lateral_acceleration: 0.0,
            max_angular_acceleration: 0.0,
            destroyed: false,
            crash_message: None,
            ttl: None,
            fuel: None,
            guns: vec![],
            missile_launchers: vec![],
            radar: None,
            radar_cross_section: 10.0,
            radios: vec![],
            abilities: vec![],
            target: None,
            warhead: Default::default(),
        }
    }
}

impl Default for Gun {
    fn default() -> Gun {
        Gun {
            magazine_size: 10,
            magazine_remaining: 0,
            magazine_reload_ticks: 60,
            reload_ticks: 60,
            reload_ticks_remaining: 0,
            speed: 1000.0,
            speed_error: 0.0,
            offset: vector![00.0, 0.0],
            heading: 0.0,
            min_angle: 0.0,
            max_angle: 0.0,
            inaccuracy: 0.0,
            burst_size: 1,
            ttl: 10.0,
            bullet_mass: 1.0,
        }
    }
}

impl Default for ShipAbility {
    fn default() -> Self {
        Self {
            ability: Ability::Boost,
            active_time: 0.0,
            reload_time: 0.0,
            active_time_remaining: 0.0,
            reload_time_remaining: 0.0,
        }
    }
}

impl Default for Warhead {
    fn default() -> Self {
        Self {
            count: 20,
            mass: 0.2,
            width: TAU,
            speed: 1e3,
            ttl: (PHYSICS_TICK_LENGTH * 5.0) as f32,
        }
    }
}

fn radio() -> Radio {
    // TODO tune this
    Radio {
        power: 20e3,
        rx_cross_section: 5.0,
        min_rssi: 1e-5,
        channel: 0,
        sent: None,
        received: None,
    }
}

pub fn vulcan_gun() -> Gun {
    Gun {
        magazine_size: 30,
        reload_ticks: 4,
        magazine_reload_ticks: 60,
        speed: 1000.0,
        inaccuracy: 0.0025,
        bullet_mass: 0.1,
        ..Default::default()
    }
}

pub fn fighter(team: i32) -> ShipData {
    ShipData {
        class: ShipClass::Fighter,
        team,
        health: 100.0,
        mass: 15000.0,
        max_forward_acceleration: 60.0,
        max_backward_acceleration: 30.0,
        max_lateral_acceleration: 30.0,
        max_angular_acceleration: TAU,
        guns: vec![Gun {
            offset: vector![20.0, 0.0],
            ..vulcan_gun()
        }],
        missile_launchers: vec![MissileLauncher {
            class: ShipClass::Missile,
            reload_ticks: 5 * 60,
            reload_ticks_remaining: 0,
            initial_speed: 100.0,
            offset: vector![20.0, 0.0],
            angle: 0.0,
        }],
        radar: Some(Radar {
            power: 20e3,
            rx_cross_section: 5.0,
            min_width: TAU / 720.0,
            ..Default::default()
        }),
        radar_cross_section: 10.0,
        radios: vec![radio(), radio()],
        abilities: vec![ShipAbility {
            ability: Ability::Boost,
            active_time: 2.0,
            reload_time: 10.0,
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub fn frigate(team: i32) -> ShipData {
    ShipData {
        class: ShipClass::Frigate,
        team,
        health: 10000.0,
        mass: 4e6,
        max_forward_acceleration: 10.0,
        max_backward_acceleration: 5.0,
        max_lateral_acceleration: 5.0,
        max_angular_acceleration: TAU / 8.0,
        guns: vec![
            Gun {
                magazine_size: 1,
                magazine_reload_ticks: 0,
                reload_ticks: 2 * 60,
                speed: 4000.0,
                offset: vector![40.0, 0.0],
                bullet_mass: 1.0,
                ttl: 60.0,
                ..Default::default()
            },
            Gun {
                offset: vector![0.0, 15.0],
                max_angle: TAU,
                ..vulcan_gun()
            },
            Gun {
                offset: vector![0.0, -15.0],
                max_angle: TAU,
                ..vulcan_gun()
            },
        ],
        missile_launchers: vec![MissileLauncher {
            class: ShipClass::Missile,
            reload_ticks: 2 * 60,
            reload_ticks_remaining: 0,
            initial_speed: 100.0,
            offset: vector![60.0, 0.0],
            angle: 0.0,
        }],
        radar: Some(Radar {
            power: 100e3,
            rx_cross_section: 10.0,
            ..Default::default()
        }),
        radar_cross_section: 30.0,
        radios: vec![radio(), radio(), radio(), radio()],
        ..Default::default()
    }
}

pub const CRUISER_RADAR_CROSS_SECTION: f64 = 40.0;

pub fn cruiser(team: i32) -> ShipData {
    let missile_launcher = MissileLauncher {
        class: ShipClass::Missile,
        reload_ticks: 72,
        reload_ticks_remaining: 0,
        initial_speed: 100.0,
        offset: vector![0.0, 0.0],
        angle: 0.0,
    };
    ShipData {
        class: ShipClass::Cruiser,
        team,
        health: 20000.0,
        mass: 9e6,
        max_forward_acceleration: 5.0,
        max_backward_acceleration: 2.5,
        max_lateral_acceleration: 2.5,
        max_angular_acceleration: TAU / 16.0,
        guns: vec![Gun {
            magazine_size: 30,
            magazine_reload_ticks: 60,
            reload_ticks: 24,
            speed: 1000.0,
            speed_error: 50.0,
            offset: vector![0.0, 0.0],
            max_angle: TAU,
            inaccuracy: 0.02,
            burst_size: 6,
            ttl: 1.0,
            bullet_mass: 0.1,
            ..Default::default()
        }],
        missile_launchers: vec![
            MissileLauncher {
                offset: vector![0.0, 50.0],
                angle: TAU / 4.0,
                ..missile_launcher
            },
            MissileLauncher {
                offset: vector![0.0, -50.0],
                angle: -TAU / 4.0,
                ..missile_launcher
            },
            MissileLauncher {
                class: ShipClass::Torpedo,
                reload_ticks: 180,
                reload_ticks_remaining: 0,
                initial_speed: 100.0,
                offset: vector![140.0, 0.0],
                angle: 0.0,
            },
        ],
        radar: Some(Radar {
            power: 200e3,
            rx_cross_section: 20.0,
            ..Default::default()
        }),
        radar_cross_section: CRUISER_RADAR_CROSS_SECTION,
        radios: vec![
            radio(),
            radio(),
            radio(),
            radio(),
            radio(),
            radio(),
            radio(),
            radio(),
        ],
        abilities: vec![ShipAbility {
            ability: Ability::Shield,
            active_time: 1.0,
            reload_time: 5.0,
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub fn asteroid(variant: i32) -> ShipData {
    ShipData {
        class: ShipClass::Asteroid { variant },
        team: 9,
        health: 200.0,
        mass: 20e6,
        radar_cross_section: 50.0,
        ..Default::default()
    }
}

pub fn target(team: i32) -> ShipData {
    ShipData {
        class: ShipClass::Target,
        team,
        health: 1.0,
        mass: 10.0,
        ..Default::default()
    }
}

pub fn missile(team: i32) -> ShipData {
    ShipData {
        class: ShipClass::Missile,
        team,
        health: 20.0,
        mass: 150.0,
        max_forward_acceleration: 300.0,
        max_backward_acceleration: 0.0,
        max_lateral_acceleration: 100.0,
        max_angular_acceleration: 4.0 * TAU,
        radar: Some(Radar {
            power: 1e3,
            rx_cross_section: 3.0,
            min_width: TAU / 720.0,
            ..Default::default()
        }),
        radar_cross_section: 0.1,
        radios: vec![radio()],
        ttl: Some(60 * 60),
        fuel: Some(2000.0),
        abilities: vec![ShipAbility {
            ability: Ability::Boost,
            active_time: 2.0,
            reload_time: 10.0,
            ..Default::default()
        }],
        warhead: Warhead {
            count: 20,
            mass: 0.05,
            width: 0.4,
            speed: 1e3,
            ttl: 0.2,
        },
        ..Default::default()
    }
}

pub fn torpedo(team: i32) -> ShipData {
    ShipData {
        class: ShipClass::Torpedo,
        team,
        health: 100.0,
        mass: 500.0,
        max_forward_acceleration: 70.0,
        max_backward_acceleration: 0.0,
        max_lateral_acceleration: 20.0,
        max_angular_acceleration: 2.0 * TAU,
        radar: Some(Radar {
            power: 10e3,
            rx_cross_section: 3.0,
            min_width: TAU / 720.0,
            ..Default::default()
        }),
        radar_cross_section: 0.3,
        radios: vec![radio()],
        ttl: Some(60 * 60),
        fuel: Some(3000.0),
        abilities: vec![ShipAbility {
            ability: Ability::Decoy,
            active_time: 0.5,
            reload_time: 10.0,
            ..Default::default()
        }],
        warhead: Warhead {
            count: 50,
            mass: 0.25,
            width: 0.5,
            speed: 1e3,
            ttl: 0.2,
        },
        ..Default::default()
    }
}

pub fn create(
    sim: &mut Simulation,
    position: Vector2<f64>,
    velocity: Vector2<f64>,
    heading: f64,
    mut data: ShipData,
) -> ShipHandle {
    let mut builder = RigidBodyBuilder::dynamic()
        .translation(position)
        .linvel(velocity)
        .rotation(heading)
        .ccd_enabled(true)
        .can_sleep(false);
    if data.class == ShipClass::Planet {
        builder = builder.lock_translations()
    }
    let rigid_body = builder.build();
    let body_handle = sim.bodies.insert(rigid_body);
    let handle = ShipHandle(body_handle.0);
    let team = data.team;
    let model = model::load(data.class);
    let restitution = match data.class {
        ShipClass::Missile => 0.0,
        _ => 0.1,
    };
    let vertices = model
        .iter()
        .map(|&v| point![v.x as f64, v.y as f64])
        .collect::<Vec<_>>();
    let collider = ColliderBuilder::convex_hull(&vertices)
        .unwrap()
        .mass(data.mass)
        .restitution(restitution)
        .collision_groups(if data.class == ShipClass::Planet {
            collision::planet_interaction_groups()
        } else {
            collision::ship_interaction_groups(team)
        })
        .active_events(ActiveEvents::COLLISION_EVENTS)
        .build();
    sim.colliders
        .insert_with_parent(collider, body_handle, &mut sim.bodies);

    for gun in data.guns.iter_mut() {
        gun.magazine_remaining = gun.magazine_size;
    }

    sim.ships.insert(handle);
    sim.new_ships.push((data.team, handle));
    sim.ship_data.insert(handle.index(), data);

    handle
}

pub struct ShipAccessor<'a> {
    pub(crate) simulation: &'a Simulation,
    pub(crate) handle: ShipHandle,
}

impl<'a> ShipAccessor<'a> {
    pub fn exists(&self) -> bool {
        self.simulation.ship_data.get(self.handle.index()).is_some()
    }

    pub fn body(&self) -> &'a RigidBody {
        self.simulation
            .bodies
            .get(RigidBodyHandle(self.handle.index()))
            .unwrap()
    }

    pub fn position(&self) -> Translation<Real> {
        self.body().position().translation
    }

    pub fn velocity(&self) -> Vector<Real> {
        *self.body().linvel()
    }

    pub fn heading(&self) -> Real {
        self.body().rotation().angle().rem_euclid(TAU)
    }

    pub fn angular_velocity(&self) -> Real {
        self.body().angvel()
    }

    pub fn data(&self) -> &ShipData {
        self.simulation.ship_data.get(self.handle.index()).unwrap()
    }

    pub fn radar(&self) -> Option<&Radar> {
        self.data().radar.as_ref()
    }

    pub fn radio(&self, idx: usize) -> Option<&Radio> {
        self.data().radios.get(idx)
    }

    pub fn is_ability_active(&self, ability: oort_api::Ability) -> bool {
        self.data()
            .abilities
            .iter()
            .find(|x| x.ability == ability)
            .map(|x| x.active_time_remaining > 0.0)
            .unwrap_or(false)
    }

    pub fn active_abilities(&self) -> Vec<oort_api::Ability> {
        self.data()
            .abilities
            .iter()
            .filter(|x| x.active_time_remaining > 0.0)
            .map(|x| x.ability)
            .collect()
    }

    pub fn get_reload_ticks(&self, idx: usize) -> u32 {
        if let Some(gun) = self.data().guns.get(idx) {
            gun.reload_ticks_remaining
        } else if let Some(missile) = self
            .data()
            .missile_launchers
            .get(idx - self.data().guns.len())
        {
            missile.reload_ticks_remaining
        } else {
            0
        }
    }
}

pub struct ShipAccessorMut<'a> {
    pub(crate) simulation: &'a mut Simulation,
    pub(crate) handle: ShipHandle,
}

impl<'a: 'b, 'b> ShipAccessorMut<'a> {
    pub fn readonly(&self) -> ShipAccessor {
        ShipAccessor {
            simulation: self.simulation,
            handle: self.handle,
        }
    }

    pub fn body(&'b mut self) -> &'b mut RigidBody {
        self.simulation
            .bodies
            .get_mut(RigidBodyHandle(self.handle.index()))
            .unwrap()
    }

    pub fn data(&self) -> &ShipData {
        self.simulation.ship_data.get(self.handle.index()).unwrap()
    }

    pub fn data_mut(&mut self) -> &mut ShipData {
        self.simulation
            .ship_data
            .get_mut(self.handle.index())
            .unwrap()
    }

    pub fn radar_mut(&mut self) -> Option<&mut Radar> {
        self.data_mut().radar.as_mut()
    }

    pub fn radio_mut(&mut self, idx: usize) -> Option<&mut Radio> {
        self.data_mut().radios.get_mut(idx)
    }

    pub fn accelerate(&mut self, acceleration: Vector2<f64>) {
        let data = self.data();
        let clamped_acceleration = acceleration
            .inf(&vector![
                data.max_forward_acceleration,
                data.max_lateral_acceleration
            ])
            .sup(&vector![
                -data.max_backward_acceleration,
                -data.max_lateral_acceleration
            ]);
        self.data_mut().acceleration = clamped_acceleration;
    }

    pub fn torque(&mut self, angular_acceleration: f64) {
        let max_angular_acceleration = self.data().max_angular_acceleration;
        let clamped_angular_acceleration =
            angular_acceleration.clamp(-max_angular_acceleration, max_angular_acceleration);
        self.data_mut().angular_acceleration = clamped_angular_acceleration;
    }

    pub fn fire(&mut self, index: i64) {
        let num_guns = self.data().guns.len() as i64;
        if index >= num_guns {
            self.launch_missile(index - num_guns);
        } else {
            self.fire_gun(index);
        }
    }

    pub fn fire_gun(&mut self, index: i64) {
        let ship_data = self.data_mut();
        if index as usize >= ship_data.guns.len() {
            return;
        }
        let team = ship_data.team;
        let gun = {
            let gun = &mut ship_data.guns[index as usize];
            if gun.reload_ticks_remaining > 0 {
                return;
            }
            gun.reload_ticks_remaining = gun.reload_ticks;
            gun.magazine_remaining -= gun.burst_size;
            if gun.magazine_remaining <= 0 {
                gun.magazine_remaining = gun.magazine_size;
                gun.reload_ticks_remaining += gun.magazine_reload_ticks;
            }
            gun.clone()
        };

        let mut rng =
            rng::new_rng(self.simulation.tick() ^ u64::from(self.handle) as u32 ^ index as u32);
        let alpha = (gun.bullet_mass as f32).clamp(0.7, 1.0);
        let color = color::to_u32(vector![1.0, 1.0, 1.0, alpha]);
        let mut t = 0.0;
        let dt = simulation::PHYSICS_TICK_LENGTH / gun.burst_size as f64;

        let relative_heading = (gun.heading - self.readonly().heading())
            .rem_euclid(TAU)
            .clamp(gun.min_angle, gun.max_angle);

        for _ in 0..gun.burst_size {
            let relative_heading = if gun.inaccuracy > 0.0 {
                relative_heading + rng.gen_range(-gun.inaccuracy..gun.inaccuracy)
            } else {
                relative_heading
            };
            let speed = if gun.speed_error > 0.0 {
                gun.speed + rng.gen_range(-gun.speed_error..gun.speed_error)
            } else {
                gun.speed
            };
            let body = self.body();
            let rot = body.position().rotation * UnitComplex::new(relative_heading);
            let v = body.linvel() + rot.transform_vector(&vector![speed, 0.0]);
            let p = body.position().translation.vector
                + body.position().rotation.transform_vector(&gun.offset)
                + v * t;
            bullet::create(
                self.simulation,
                p,
                v,
                BulletData {
                    mass: gun.bullet_mass as f32,
                    team,
                    color,
                    ttl: gun.ttl + t as f32,
                },
            );
            t += dt;
        }
    }

    pub fn launch_missile(&mut self, index: i64) {
        let missile_launcher = {
            let ship_data = self.data_mut();
            if let Some(missile_launcher) =
                ship_data.missile_launchers.get_mut(index as usize).as_mut()
            {
                if missile_launcher.reload_ticks_remaining > 0 {
                    return;
                }
                missile_launcher.reload_ticks_remaining = missile_launcher.reload_ticks;
                missile_launcher.clone()
            } else {
                return;
            }
        };

        let speed = missile_launcher.initial_speed;
        let offset = missile_launcher.offset;
        let body = self.body();
        let rot = body.position().rotation;
        let p = body.position().translation.vector + rot.transform_vector(&offset);
        let rot2 = rot * UnitComplex::new(missile_launcher.angle);
        let v = body.linvel() + rot2.transform_vector(&vector![speed, 0.0]);
        let team = self.data().team;
        create(
            self.simulation,
            p,
            v,
            rot2.angle(),
            match missile_launcher.class {
                ShipClass::Missile => missile(team),
                ShipClass::Torpedo => torpedo(team),
                _ => unimplemented!(),
            },
        );
    }

    pub fn aim(&mut self, index: i64, heading: f64) {
        let ship_data = self.data_mut();
        if index as usize >= ship_data.guns.len() {
            return;
        }
        let gun = &mut ship_data.guns[index as usize];
        gun.heading = heading;
    }

    pub fn explode(&mut self) {
        if self.data().destroyed {
            return;
        }
        self.data_mut().destroyed = true;

        let warhead = self.data().warhead.clone();
        let team = self.data().team;
        let p =
            self.body().position().translation.vector - self.body().linvel() * PHYSICS_TICK_LENGTH;
        let mut rng = new_rng(0);
        for _ in 0..warhead.count {
            let color = vector![rng.gen_range(0.7..1.0), 0.5, 0.5, rng.gen_range(0.5..1.0)];
            let rot = self.body().rotation()
                * Rotation2::new(rng.gen_range((-warhead.width / 2.0)..(warhead.width / 2.0)));
            let speed = warhead.speed * 2.0 * rng.gen_range(0.0..1.0);
            let v = self.body().linvel() + rot.transform_vector(&vector![speed, 0.0]);
            let offset = v * rng.gen_range(0.0..PHYSICS_TICK_LENGTH);
            bullet::create(
                self.simulation,
                p + offset,
                v,
                BulletData {
                    mass: warhead.mass,
                    team,
                    color: color::to_u32(color),
                    ttl: warhead.ttl,
                },
            );
            self.simulation.events.particles.push(Particle {
                position: p + offset,
                velocity: v,
                color,
                lifetime: warhead.ttl,
            });
        }
    }

    pub fn activate_ability(&mut self, ability: oort_api::Ability) {
        if let Some(ship_ability) = self
            .data_mut()
            .abilities
            .iter_mut()
            .find(|x| x.ability == ability)
        {
            if ship_ability.reload_time_remaining > 0.0 {
                return;
            }
            ship_ability.active_time_remaining = ship_ability.active_time - PHYSICS_TICK_LENGTH;
            ship_ability.reload_time_remaining = ship_ability.reload_time;
        }
    }

    pub fn deactivate_ability(&mut self, ability: oort_api::Ability) {
        if let Some(ship_ability) = self
            .data_mut()
            .abilities
            .iter_mut()
            .find(|x| x.ability == ability)
        {
            ship_ability.active_time_remaining = 0.0;
        }
    }

    pub fn tick(&mut self) {
        // Weapons.
        {
            let ship_data = self
                .simulation
                .ship_data
                .get_mut(self.handle.index())
                .unwrap();
            for gun in ship_data.guns.iter_mut() {
                if gun.reload_ticks_remaining > 0 {
                    gun.reload_ticks_remaining -= 1;
                }
            }

            for missile_launcher in ship_data.missile_launchers.iter_mut() {
                if missile_launcher.reload_ticks_remaining > 0 {
                    missile_launcher.reload_ticks_remaining -= 1;
                }
            }
        }

        // Acceleration.
        {
            let mut acceleration = self.data().acceleration;
            if self.readonly().is_ability_active(Ability::Boost) {
                acceleration += vector![100.0, 0.0];
            }
            let fuel_consumption = (acceleration * PHYSICS_TICK_LENGTH).norm();
            if let Some(fuel) = self.data_mut().fuel {
                if fuel < fuel_consumption {
                    acceleration *= fuel / fuel_consumption;
                    self.data_mut().fuel = Some(0.0);
                } else {
                    self.data_mut().fuel = Some(fuel - fuel_consumption);
                }
            }
            let mass = self.body().mass();
            let rotation_matrix = self.body().position().rotation.to_rotation_matrix();
            let inertial_acceleration = rotation_matrix * acceleration;
            self.body().reset_forces(false);
            self.body().add_force(inertial_acceleration * mass, true);
            self.data_mut().last_acceleration = inertial_acceleration;
            self.data_mut().acceleration = vector![0.0, 0.0];
        }

        // Torque.
        {
            let inertia_sqrt = 1.0
                / self
                    .body()
                    .mass_properties()
                    .local_mprops
                    .inv_principal_inertia_sqrt;
            let torque = self.data().angular_acceleration * inertia_sqrt * inertia_sqrt;
            self.body().reset_torques(false);
            self.body().add_torque(torque, true);
            self.data_mut().angular_acceleration = 0.0;
        }

        // TTL
        {
            if let Some(ttl) = self.data_mut().ttl {
                self.data_mut().ttl = Some(ttl - 1);
                if self.data().ttl.unwrap() == 0 {
                    self.explode();
                }
            }
        }

        // Special abilities.
        {
            for ship_ability in self.data_mut().abilities.iter_mut() {
                ship_ability.active_time_remaining =
                    (ship_ability.active_time_remaining - PHYSICS_TICK_LENGTH).max(0.0);
                ship_ability.reload_time_remaining =
                    (ship_ability.reload_time_remaining - PHYSICS_TICK_LENGTH).max(0.0);
            }
        }

        // Destruction.
        if self.data().destroyed {
            if let Some(team_ctrl) = self.simulation.get_team_controller(self.data().team) {
                team_ctrl.borrow_mut().remove_ship(self.handle);
            }
            self.simulation.ships.remove(self.handle);
            self.simulation.bodies.remove(
                RigidBodyHandle(self.handle.index()),
                &mut self.simulation.island_manager,
                &mut self.simulation.colliders,
                &mut self.simulation.impulse_joints,
                &mut self.simulation.multibody_joints,
                /*remove_attached_colliders=*/ true,
            );
            self.simulation
                .ship_data
                .remove(self.handle.index(), ShipData::default());
        }
    }

    pub fn handle_collision(&mut self) {
        if self.data().class == ShipClass::Missile || self.data().class == ShipClass::Torpedo {
            self.explode();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ship;
    use crate::simulation::Code;
    use crate::simulation::Simulation;
    use nalgebra::vector;
    use test_log::test;

    #[test]
    fn test_gun_reload_ticks() {
        let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

        // Initial state.
        let ship0 = ship::create(
            &mut sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::fighter(0),
        );

        assert_eq!(sim.ship(ship0).get_reload_ticks(0), 0);
        sim.ship_mut(ship0).fire(0);
        assert_eq!(sim.bullets.len(), 1);

        for i in [3, 2, 1, 0].iter() {
            sim.ship_mut(ship0).fire(0);
            assert_eq!(sim.bullets.len(), 1);

            sim.step();
            assert_eq!(sim.ship(ship0).get_reload_ticks(0), *i);
        }

        sim.ship_mut(ship0).fire(0);
        assert_eq!(sim.bullets.len(), 2);
    }

    #[test]
    fn test_missile_reload_ticks() {
        let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

        let mut data = ship::fighter(0);
        data.missile_launchers[0].reload_ticks = 4;

        // Initial state.
        let ship0 = ship::create(&mut sim, vector![0.0, 0.0], vector![0.0, 0.0], 0.0, data);

        assert_eq!(sim.ships.len(), 1);

        assert_eq!(sim.ship(ship0).get_reload_ticks(0), 0);
        sim.ship_mut(ship0).fire(1);
        assert_eq!(sim.ships.len(), 2);

        for i in [3, 2, 1, 0].iter() {
            sim.ship_mut(ship0).fire(1);
            assert_eq!(sim.ships.len(), 2);

            sim.step();
            assert_eq!(sim.ship(ship0).get_reload_ticks(1), *i);
        }

        sim.ship_mut(ship0).fire(1);
        assert_eq!(sim.ships.len(), 3);
    }

    #[test]
    fn test_center_of_mass() {
        for ship_data in [
            super::fighter(0),
            super::frigate(0),
            super::cruiser(0),
            super::missile(0),
            super::torpedo(0),
        ] {
            let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

            let ship = ship::create(
                &mut sim,
                vector![0.0, 0.0],
                vector![0.0, 0.0],
                0.0,
                ship_data,
            );
            let com = sim.ship(ship).body().center_of_mass().coords;
            assert!(
                com.magnitude() < 1e-6,
                "class {:?} center of mass {:?}",
                sim.ship(ship).data().class,
                com
            );
        }
    }

    #[test]
    fn test_rotate_in_place() {
        for ship_data in [
            super::fighter(0),
            super::frigate(0),
            super::cruiser(0),
            super::missile(0),
            super::torpedo(0),
        ] {
            let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

            let ship0 = ship::create(
                &mut sim,
                vector![0.0, 0.0],
                vector![0.0, 0.0],
                0.0,
                ship_data,
            );

            let dist = sim.ship(ship0).position().vector.magnitude();
            assert!(
                dist < 1.0,
                "class {:?} dist {}",
                sim.ship(ship0).data().class,
                dist
            );

            for _ in 0..10 {
                sim.ship_mut(ship0).torque(6.28);
                sim.step();
                let dist = sim.ship(ship0).position().vector.magnitude();
                assert!(
                    dist < 1.0,
                    "class {:?} dist {} tick {}",
                    sim.ship(ship0).data().class,
                    dist,
                    sim.tick()
                );
            }
        }
    }
}
