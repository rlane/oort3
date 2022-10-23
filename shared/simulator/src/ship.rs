use super::index_set::{HasIndex, Index};
use super::rng::new_rng;
use crate::model;
use crate::radar::Radar;
use crate::radio::Radio;
use crate::rng;
use crate::simulation::Simulation;
use crate::simulation::{self, PHYSICS_TICK_LENGTH};
use crate::{bullet, collision};
use bullet::BulletData;
use nalgebra::{vector, Rotation2, UnitComplex, Vector2};
use oort_api::Ability;
use rand::Rng;
use rapier2d_f64::prelude::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug)]
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct Gun {
    pub magazine_size: i32,
    pub magazine_remaining: i32,
    pub cycle_time: f64,
    pub cycle_time_remaining: f64,
    pub reload_time: f64,
    pub speed: f64,
    pub speed_error: f64,
    pub offset: Vector2<f64>,
    pub angle: f64,
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
    pub reload_time: f64,
    pub reload_time_remaining: f64,
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
pub struct ShipData {
    pub class: ShipClass,
    pub team: i32,
    pub health: f64,
    pub acceleration: Vector2<f64>,
    pub angular_acceleration: f64,
    pub max_forward_acceleration: f64,
    pub max_backward_acceleration: f64,
    pub max_lateral_acceleration: f64,
    pub max_angular_acceleration: f64,
    pub destroyed: bool,
    pub ttl: Option<u64>,
    pub guns: Vec<Gun>,
    pub missile_launchers: Vec<MissileLauncher>,
    pub radar: Option<Radar>,
    pub radar_cross_section: f64,
    pub radio: Option<Radio>,
    pub abilities: Vec<ShipAbility>,
}

impl Default for ShipData {
    fn default() -> ShipData {
        ShipData {
            class: ShipClass::Fighter,
            team: 0,
            health: 100.0,
            acceleration: vector![0.0, 0.0],
            angular_acceleration: 0.0,
            max_forward_acceleration: 0.0,
            max_backward_acceleration: 0.0,
            max_lateral_acceleration: 0.0,
            max_angular_acceleration: 0.0,
            destroyed: false,
            ttl: None,
            guns: vec![],
            missile_launchers: vec![],
            radar: None,
            radar_cross_section: 10.0,
            radio: None,
            abilities: vec![],
        }
    }
}

impl Default for Gun {
    fn default() -> Gun {
        Gun {
            magazine_size: 10,
            magazine_remaining: 0,
            cycle_time: 1.0,
            reload_time: 1.0,
            cycle_time_remaining: 0.0,
            speed: 1000.0,
            speed_error: 0.0,
            offset: vector![00.0, 0.0],
            angle: 0.0,
            min_angle: 0.0,
            max_angle: 0.0,
            inaccuracy: 0.0,
            burst_size: 1,
            ttl: 5.0,
            bullet_mass: 1.0,
        }
    }
}

impl Default for ShipAbility {
    fn default() -> Self {
        Self {
            ability: Ability::None,
            active_time: 0.0,
            reload_time: 0.0,
            active_time_remaining: 0.0,
            reload_time_remaining: 0.0,
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
        cycle_time: PHYSICS_TICK_LENGTH * 4.0,
        reload_time: 1.0,
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
            reload_time: 5.0,
            reload_time_remaining: 0.0,
            initial_speed: 100.0,
            offset: vector![20.0, 0.0],
            angle: 0.0,
        }],
        radar: Some(Radar {
            power: 20e3,
            rx_cross_section: 5.0,
            ..Default::default()
        }),
        radar_cross_section: 10.0,
        radio: Some(radio()),
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
        max_forward_acceleration: 10.0,
        max_backward_acceleration: 5.0,
        max_lateral_acceleration: 5.0,
        max_angular_acceleration: TAU / 8.0,
        guns: vec![
            Gun {
                magazine_size: 1,
                cycle_time: 2.0,
                reload_time: 0.0,
                speed: 4000.0,
                offset: vector![40.0, 0.0],
                bullet_mass: 1.0,
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
            reload_time: 2.0,
            reload_time_remaining: 0.0,
            initial_speed: 100.0,
            offset: vector![32.0, 0.0],
            angle: 0.0,
        }],
        radar: Some(Radar {
            power: 100e3,
            rx_cross_section: 10.0,
            ..Default::default()
        }),
        radar_cross_section: 30.0,
        radio: Some(radio()),
        ..Default::default()
    }
}

pub const CRUISER_RADAR_CROSS_SECTION: f64 = 40.0;

pub fn cruiser(team: i32) -> ShipData {
    let missile_launcher = MissileLauncher {
        class: ShipClass::Missile,
        reload_time: 1.2,
        reload_time_remaining: 0.0,
        initial_speed: 100.0,
        offset: vector![0.0, 0.0],
        angle: 0.0,
    };
    ShipData {
        class: ShipClass::Cruiser,
        team,
        health: 20000.0,
        max_forward_acceleration: 5.0,
        max_backward_acceleration: 2.5,
        max_lateral_acceleration: 2.5,
        max_angular_acceleration: TAU / 16.0,
        guns: vec![Gun {
            magazine_size: 30,
            cycle_time: 0.4,
            reload_time: 1.0,
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
                offset: vector![0.0, 30.0],
                angle: TAU / 4.0,
                ..missile_launcher
            },
            MissileLauncher {
                offset: vector![0.0, -30.0],
                angle: -TAU / 4.0,
                ..missile_launcher
            },
            MissileLauncher {
                class: ShipClass::Torpedo,
                reload_time: 3.0,
                reload_time_remaining: 0.0,
                initial_speed: 100.0,
                offset: vector![100.0, 0.0],
                angle: 0.0,
            },
        ],
        radar: Some(Radar {
            power: 200e3,
            rx_cross_section: 20.0,
            ..Default::default()
        }),
        radar_cross_section: CRUISER_RADAR_CROSS_SECTION,
        radio: Some(radio()),
        ..Default::default()
    }
}

pub fn asteroid(variant: i32) -> ShipData {
    ShipData {
        class: ShipClass::Asteroid { variant },
        team: 9,
        health: 200.0,
        ..Default::default()
    }
}

pub fn target(team: i32) -> ShipData {
    ShipData {
        class: ShipClass::Target,
        team,
        health: 1.0,
        ..Default::default()
    }
}

pub fn missile(team: i32) -> ShipData {
    ShipData {
        class: ShipClass::Missile,
        team,
        health: 20.0,
        max_forward_acceleration: 200.0,
        max_backward_acceleration: 0.0,
        max_lateral_acceleration: 50.0,
        max_angular_acceleration: 2.0 * TAU,
        radar: Some(Radar {
            power: 1e3,
            rx_cross_section: 3.0,
            ..Default::default()
        }),
        radar_cross_section: 3.0,
        radio: Some(radio()),
        ttl: Some(20 * 60),
        abilities: vec![ShipAbility {
            ability: Ability::ShapedCharge,
            active_time: 1e6,
            reload_time: 0.0,
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub fn torpedo(team: i32) -> ShipData {
    ShipData {
        class: ShipClass::Torpedo,
        team,
        health: 100.0,
        max_forward_acceleration: 70.0,
        max_backward_acceleration: 0.0,
        max_lateral_acceleration: 20.0,
        max_angular_acceleration: 2.0 * TAU,
        radar: Some(Radar {
            power: 10e3,
            rx_cross_section: 3.0,
            ..Default::default()
        }),
        radar_cross_section: 8.0,
        radio: Some(radio()),
        ttl: Some(30 * 60),
        abilities: vec![ShipAbility {
            ability: Ability::Decoy,
            active_time: 0.5,
            reload_time: 10.0,
            ..Default::default()
        }],
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
    let rigid_body = RigidBodyBuilder::dynamic()
        .translation(position)
        .linvel(velocity)
        .rotation(heading)
        .ccd_enabled(true)
        .build();
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
        .restitution(restitution)
        .collision_groups(collision::ship_interaction_groups(team))
        .active_events(ActiveEvents::COLLISION_EVENTS)
        .build();
    sim.colliders
        .insert_with_parent(collider, body_handle, &mut sim.bodies);

    for gun in data.guns.iter_mut() {
        gun.magazine_remaining = gun.magazine_size;
    }

    sim.ships.insert(handle);
    sim.ship_data.insert(handle, data);

    if let Some(team_ctrl) = sim.get_team_controller(team) {
        match team_ctrl.borrow_mut().create_ship_controller(handle, sim) {
            Ok(ship_ctrl) => {
                sim.ship_controllers.insert(handle, ship_ctrl);
            }
            Err(e) => {
                log::warn!("Ship creation error: {:?}", e);
                sim.events.errors.push(e);
            }
        }
    }
    handle
}

pub struct ShipAccessor<'a> {
    pub(crate) simulation: &'a Simulation,
    pub(crate) handle: ShipHandle,
}

impl<'a> ShipAccessor<'a> {
    pub fn exists(&self) -> bool {
        self.simulation.ship_data.contains_key(&self.handle)
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
        self.simulation.ship_data.get(&self.handle).unwrap()
    }

    pub fn radar(&self) -> Option<&Radar> {
        self.data().radar.as_ref()
    }

    pub fn radio(&self) -> Option<&Radio> {
        self.data().radio.as_ref()
    }

    pub fn is_ability_active(&self, ability: oort_api::Ability) -> bool {
        self.data()
            .abilities
            .iter()
            .find(|x| x.ability == ability)
            .map(|x| x.active_time_remaining > 0.0)
            .unwrap_or(false)
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
        self.simulation.ship_data.get(&self.handle).unwrap()
    }

    pub fn data_mut(&mut self) -> &mut ShipData {
        self.simulation.ship_data.get_mut(&self.handle).unwrap()
    }

    pub fn radar_mut(&mut self) -> Option<&mut Radar> {
        self.data_mut().radar.as_mut()
    }

    pub fn radio_mut(&mut self) -> Option<&mut Radio> {
        self.data_mut().radio.as_mut()
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
            if gun.cycle_time_remaining > 1e-6 {
                return;
            }
            gun.cycle_time_remaining = gun.cycle_time;
            gun.magazine_remaining -= gun.burst_size;
            if gun.magazine_remaining <= 0 {
                gun.magazine_remaining = gun.magazine_size;
                gun.cycle_time_remaining += gun.reload_time;
            }
            gun.clone()
        };

        let mut rng =
            rng::new_rng(self.simulation.tick() ^ u64::from(self.handle) as u32 ^ index as u32);
        let alpha = (gun.bullet_mass as f32).clamp(0.5, 1.0);
        let color = vector![1.00, 0.63, 0.00, alpha];
        let mut t = 0.0;
        let dt = simulation::PHYSICS_TICK_LENGTH / gun.burst_size as f64;

        for _ in 0..gun.burst_size {
            let angle = if gun.inaccuracy > 0.0 {
                gun.angle + rng.gen_range(-gun.inaccuracy..gun.inaccuracy)
            } else {
                gun.angle
            };
            let speed = if gun.speed_error > 0.0 {
                gun.speed + rng.gen_range(-gun.speed_error..gun.speed_error)
            } else {
                gun.speed
            };
            let body = self.body();
            let rot = body.position().rotation * UnitComplex::new(angle);
            let v = body.linvel() + rot.transform_vector(&vector![speed, 0.0]);
            let p = body.position().translation.vector
                + body.position().rotation.transform_vector(&gun.offset)
                + v * t;
            bullet::create(
                self.simulation,
                p,
                v,
                BulletData {
                    mass: gun.bullet_mass,
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
                if missile_launcher.reload_time_remaining > 0.0 {
                    return;
                }
                missile_launcher.reload_time_remaining += missile_launcher.reload_time;
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

    pub fn aim(&mut self, index: i64, angle: f64) {
        let ship_data = self.data_mut();
        if index as usize >= ship_data.guns.len() {
            return;
        }
        let gun = &mut ship_data.guns[index as usize];
        gun.angle = angle.rem_euclid(TAU).clamp(gun.min_angle, gun.max_angle);
    }

    pub fn explode(&mut self) {
        if self.data().destroyed {
            return;
        }
        self.data_mut().destroyed = true;

        let (mass, num) = match self.data().class {
            ShipClass::Missile => (0.25, 20),
            ShipClass::Torpedo => (0.25, 50),
            _ => (0.2, 20),
        };

        let team = self.data().team;
        let p =
            self.body().position().translation.vector - self.body().linvel() * PHYSICS_TICK_LENGTH;
        let color = vector![0.5, 0.5, 0.5, 0.70];
        let ttl = (PHYSICS_TICK_LENGTH * 5.0) as f32;
        let h = if self.readonly().is_ability_active(Ability::ShapedCharge) {
            0.1
        } else if self.data().class == ShipClass::Torpedo {
            0.5
        } else {
            TAU
        };
        let mut rng = new_rng(0);
        for _ in 0..num {
            let rot = self.body().rotation() * Rotation2::new(rng.gen_range((-h / 2.0)..(h / 2.0)));
            let speed = 2000.0 * rng.gen_range(0.0..1.0);
            let v = self.body().linvel() + rot.transform_vector(&vector![speed, 0.0]);
            let offset = v * rng.gen_range(0.0..PHYSICS_TICK_LENGTH);
            bullet::create(
                self.simulation,
                p + offset,
                v,
                BulletData {
                    mass,
                    team,
                    color,
                    ttl,
                },
            );
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

    pub fn tick(&mut self) {
        // Guns.
        {
            let ship_data = self.simulation.ship_data.get_mut(&self.handle).unwrap();
            for gun in ship_data.guns.iter_mut() {
                gun.cycle_time_remaining =
                    (gun.cycle_time_remaining - simulation::PHYSICS_TICK_LENGTH).max(0.0);
            }

            for missile_launcher in ship_data.missile_launchers.iter_mut() {
                missile_launcher.reload_time_remaining = (missile_launcher.reload_time_remaining
                    - simulation::PHYSICS_TICK_LENGTH)
                    .max(0.0);
            }
        }

        // Acceleration.
        {
            let acceleration = self.data().acceleration;
            let mass = self.body().mass();
            let rotation_matrix = self.body().position().rotation.to_rotation_matrix();
            self.body().reset_forces(false);
            self.body()
                .add_force(rotation_matrix * acceleration * mass, true);
            if self.readonly().is_ability_active(Ability::Boost) {
                self.body()
                    .add_force(rotation_matrix * vector![100.0, 0.0] * mass, true);
            }
            self.data_mut().acceleration = vector![0.0, 0.0];
        }

        // Torque.
        {
            let inertia_sqrt = 1.0 / self.body().mass_properties().inv_principal_inertia_sqrt;
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
            if let Some(mut controller) = self.simulation.ship_controllers.remove(&self.handle) {
                controller.delete();
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
            self.simulation.ship_data.remove(&self.handle);
        }
    }

    pub fn handle_collision(&mut self) {
        if self.data().class == ShipClass::Missile || self.data().class == ShipClass::Torpedo {
            self.explode();
        }
    }
}
