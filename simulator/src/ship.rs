use super::index_set::{HasIndex, Index};
use super::rng::new_rng;
use crate::model;
use crate::radar::Radar;
use crate::simulation;
use crate::simulation::Simulation;
use crate::{bullet, collision};
use bullet::BulletData;
use nalgebra::{vector, Rotation2, UnitComplex, Vector2};
use rand::Rng;
use rapier2d_f64::prelude::*;
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug)]
pub struct Weapon {
    pub reload_time: f64,
    pub reload_time_remaining: f64,
    pub damage: f64,
    pub speed: f64,
    pub offset: Vector2<f64>,
    pub angle: f64,
}

#[derive(Debug, Clone)]
pub struct MissileLauncher {
    pub reload_time: f64,
    pub reload_time_remaining: f64,
    pub initial_speed: f64,
    pub offset: Vector2<f64>,
    pub angle: f64,
}

#[derive(Debug)]
pub struct ShipData {
    pub class: ShipClass,
    pub weapons: Vec<Weapon>,
    pub missile_launchers: Vec<MissileLauncher>,
    pub health: f64,
    pub team: i32,
    pub acceleration: Vector2<f64>,
    pub angular_acceleration: f64,
    pub max_acceleration: Vector2<f64>,
    pub max_angular_acceleration: f64,
    pub destroyed: bool,
    pub radar: Option<Radar>,
    pub radar_cross_section: f64,
    pub ttl: Option<u64>,
}

impl Default for ShipData {
    fn default() -> ShipData {
        ShipData {
            class: ShipClass::Fighter,
            weapons: vec![],
            missile_launchers: vec![],
            health: 100.0,
            team: 0,
            acceleration: vector![0.0, 0.0],
            angular_acceleration: 0.0,
            max_acceleration: vector![0.0, 0.0],
            max_angular_acceleration: 0.0,
            destroyed: false,
            radar: None,
            radar_cross_section: 10.0,
            ttl: None,
        }
    }
}

pub fn fighter(team: i32) -> ShipData {
    ShipData {
        class: ShipClass::Fighter,
        weapons: vec![Weapon {
            reload_time: 0.2,
            reload_time_remaining: 0.0,
            damage: 20.0,
            speed: 1000.0,
            offset: vector![20.0, 0.0],
            angle: 0.0,
        }],
        missile_launchers: vec![MissileLauncher {
            reload_time: 5.0,
            reload_time_remaining: 0.0,
            initial_speed: 100.0,
            offset: vector![20.0, 0.0],
            angle: 0.0,
        }],
        health: 100.0,
        team,
        max_acceleration: vector![200.0, 100.0],
        max_angular_acceleration: std::f64::consts::TAU,
        radar: Some(Radar {
            heading: 0.0,
            width: std::f64::consts::TAU / 6.0,
            power: 20e3,
            rx_cross_section: 5.0,
            min_rssi: 1e-2,
            scanned: false,
        }),
        ..Default::default()
    }
}

pub fn frigate(team: i32) -> ShipData {
    ShipData {
        class: ShipClass::Frigate,
        weapons: vec![
            Weapon {
                reload_time: 1.0,
                reload_time_remaining: 0.0,
                damage: 1000.0,
                speed: 4000.0,
                offset: vector![40.0, 0.0],
                angle: 0.0,
            },
            Weapon {
                reload_time: 0.2,
                reload_time_remaining: 0.0,
                damage: 20.0,
                speed: 1000.0,
                offset: vector![0.0, 15.0],
                angle: 0.0,
            },
            Weapon {
                reload_time: 0.2,
                reload_time_remaining: 0.0,
                damage: 20.0,
                speed: 1000.0,
                offset: vector![0.0, -15.0],
                angle: 0.0,
            },
        ],
        missile_launchers: vec![MissileLauncher {
            reload_time: 2.0,
            reload_time_remaining: 0.0,
            initial_speed: 100.0,
            offset: vector![32.0, 0.0],
            angle: 0.0,
        }],
        health: 10000.0,
        team,
        max_acceleration: vector![20.0, 10.0],
        max_angular_acceleration: std::f64::consts::TAU / 8.0,
        radar: Some(Radar {
            heading: 0.0,
            width: std::f64::consts::TAU / 6.0,
            power: 100e3,
            rx_cross_section: 50.0,
            min_rssi: 1e-2,
            scanned: false,
        }),
        ..Default::default()
    }
}

pub fn cruiser(team: i32) -> ShipData {
    let base_launcher = MissileLauncher {
        reload_time: 1.2,
        reload_time_remaining: 0.0,
        initial_speed: 100.0,
        offset: vector![0.0, 0.0],
        angle: 0.0,
    };
    let left_launcher = MissileLauncher {
        angle: std::f64::consts::TAU / 4.0,
        ..base_launcher
    };
    let right_launcher = MissileLauncher {
        angle: -std::f64::consts::TAU / 4.0,
        ..base_launcher
    };
    ShipData {
        class: ShipClass::Cruiser,
        weapons: vec![Weapon {
            reload_time: 0.1,
            reload_time_remaining: 0.0,
            damage: 20.0,
            speed: 1000.0,
            offset: vector![0.0, 0.0],
            angle: 0.0,
        }],
        missile_launchers: vec![
            MissileLauncher {
                offset: vector![10.0, 30.0],
                ..left_launcher
            },
            MissileLauncher {
                offset: vector![0.0, 30.0],
                ..left_launcher
            },
            MissileLauncher {
                offset: vector![-10.0, 30.0],
                ..left_launcher
            },
            MissileLauncher {
                offset: vector![10.0, -30.0],
                ..right_launcher
            },
            MissileLauncher {
                offset: vector![0.0, -30.0],
                ..right_launcher
            },
            MissileLauncher {
                offset: vector![-10.0, -30.0],
                ..right_launcher
            },
        ],
        health: 30000.0,
        team,
        max_acceleration: vector![10.0, 50.0],
        max_angular_acceleration: std::f64::consts::TAU / 16.0,
        radar: Some(Radar {
            heading: 0.0,
            width: std::f64::consts::TAU / 6.0,
            power: 200e3,
            rx_cross_section: 100.0,
            min_rssi: 1e-2,
            scanned: false,
        }),
        ..Default::default()
    }
}
pub fn asteroid(variant: i32) -> ShipData {
    ShipData {
        class: ShipClass::Asteroid { variant },
        weapons: vec![],
        health: 200.0,
        team: 9,
        ..Default::default()
    }
}

pub fn target(team: i32) -> ShipData {
    ShipData {
        class: ShipClass::Target,
        health: 1.0,
        team,
        ..Default::default()
    }
}

pub fn missile(team: i32) -> ShipData {
    ShipData {
        class: ShipClass::Missile,
        health: 1.0,
        max_acceleration: vector![400.0, 100.0],
        max_angular_acceleration: 2.0 * std::f64::consts::TAU,
        team,
        radar: Some(Radar {
            heading: 0.0,
            width: std::f64::consts::TAU / 6.0,
            power: 10e3,
            rx_cross_section: 3.0,
            min_rssi: 1e-2,
            scanned: false,
        }),
        radar_cross_section: 4.0,
        ttl: Some(600),
        ..Default::default()
    }
}

pub fn create(
    sim: &mut Simulation,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    h: f64,
    data: ShipData,
) -> ShipHandle {
    let rigid_body = RigidBodyBuilder::dynamic()
        .translation(vector![x, y])
        .linvel(vector![vx, vy])
        .rotation(h)
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
        .build();
    sim.colliders
        .insert_with_parent(collider, body_handle, &mut sim.bodies);

    sim.ships.insert(handle);
    sim.ship_data.insert(handle, data);

    if let Some(team_ctrl) = sim.get_team_controller(team) {
        match team_ctrl.borrow_mut().create_ship_controller(handle, sim) {
            Ok(ship_ctrl) => {
                sim.ship_controllers.insert(handle, ship_ctrl);
            }
            Err(e) => {
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

fn normalize_heading(mut h: f64) -> f64 {
    while h < 0.0 {
        h += std::f64::consts::TAU;
    }
    while h > std::f64::consts::TAU {
        h -= std::f64::consts::TAU;
    }
    h
}

impl<'a> ShipAccessor<'a> {
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
        normalize_heading(self.body().rotation().angle())
    }

    pub fn angular_velocity(&self) -> Real {
        self.body().angvel()
    }

    pub fn data(&self) -> &ShipData {
        self.simulation.ship_data.get(&self.handle).unwrap()
    }
}

pub struct ShipAccessorMut<'a> {
    pub(crate) simulation: &'a mut Simulation,
    pub(crate) handle: ShipHandle,
}

impl<'a: 'b, 'b> ShipAccessorMut<'a> {
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

    pub fn accelerate(&mut self, acceleration: Vector2<f64>) {
        let max_acceleration = self.data().max_acceleration;
        let clamped_acceleration = acceleration.inf(&max_acceleration).sup(&-max_acceleration);
        self.data_mut().acceleration = clamped_acceleration;
    }

    pub fn torque(&mut self, angular_acceleration: f64) {
        let max_angular_acceleration = self.data().max_angular_acceleration;
        let clamped_angular_acceleration =
            angular_acceleration.clamp(-max_angular_acceleration, max_angular_acceleration);
        self.data_mut().angular_acceleration = clamped_angular_acceleration;
    }

    pub fn fire_weapon(&mut self, index: i64) {
        let ship_data = self.data_mut();
        if index as usize >= ship_data.weapons.len() {
            return;
        }
        let team = ship_data.team;
        let damage;
        let offset;
        let speed;
        let angle;
        {
            let weapon = &mut ship_data.weapons[index as usize];
            if weapon.reload_time_remaining > 0.0 {
                return;
            }
            damage = weapon.damage;
            offset = weapon.offset;
            speed = weapon.speed;
            angle = weapon.angle;
            weapon.reload_time_remaining += weapon.reload_time;
        }

        let body = self.body();
        let rot = body.position().rotation * UnitComplex::new(angle);
        let p = body.position().translation.vector + rot.transform_vector(&offset);
        let v = body.linvel() + rot.transform_vector(&vector![speed, 0.0]);
        let alpha = ((damage as f32).log(10.0) / 3.0).clamp(0.5, 1.0);
        let color = vector![1.00, 0.63, 0.00, alpha];
        let ttl = 5.0;
        bullet::create(
            self.simulation,
            p.x,
            p.y,
            v.x,
            v.y,
            BulletData {
                damage,
                team,
                color,
                ttl,
            },
        );
    }

    pub fn launch_missile(&mut self, index: i64) {
        let missile_launcher = if let Some(missile_launcher) = self
            .data_mut()
            .missile_launchers
            .get_mut(index as usize)
            .as_mut()
        {
            if missile_launcher.reload_time_remaining > 0.0 {
                return;
            }
            missile_launcher.reload_time_remaining += missile_launcher.reload_time;
            missile_launcher.clone()
        } else {
            return;
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
            p.x,
            p.y,
            v.x,
            v.y,
            rot2.angle(),
            missile(team),
        );
    }

    pub fn aim_weapon(&mut self, index: i64, angle: f64) {
        let ship_data = self.data_mut();
        if index as usize >= ship_data.weapons.len() {
            return;
        }
        let weapon = &mut ship_data.weapons[index as usize];
        weapon.angle = angle;
    }

    pub fn explode(&mut self) {
        if self.data().destroyed {
            return;
        }
        self.data_mut().destroyed = true;

        let team = self.data().team;
        let speed = 1000.0;
        let p = self.body().position().translation;
        let color = vector![0.5, 0.5, 0.5, 0.30];
        let ttl = 1.0;
        let mut rng = new_rng(0);
        for _ in 0..25 {
            let rot = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU));
            let v = self.body().linvel() + rot.transform_vector(&vector![speed, 0.0]);
            bullet::create(
                self.simulation,
                p.x,
                p.y,
                v.x,
                v.y,
                BulletData {
                    damage: 20.0,
                    team,
                    color,
                    ttl,
                },
            );
        }
    }

    pub fn tick(&mut self) {
        // Weapons.
        {
            let ship_data = self.simulation.ship_data.get_mut(&self.handle).unwrap();
            for weapon in ship_data.weapons.iter_mut() {
                weapon.reload_time_remaining =
                    (weapon.reload_time_remaining - simulation::PHYSICS_TICK_LENGTH).max(0.0);
            }

            for missile_launcher in ship_data.missile_launchers.iter_mut() {
                missile_launcher.reload_time_remaining = (missile_launcher.reload_time_remaining
                    - simulation::PHYSICS_TICK_LENGTH)
                    .max(0.0);
            }
        }

        // Radar.
        {
            let ship_data = self.simulation.ship_data.get_mut(&self.handle).unwrap();
            if let Some(radar) = ship_data.radar.as_mut() {
                radar.scanned = false;
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

        // Destruction.
        if self.data().destroyed {
            self.simulation.ships.remove(self.handle);
            self.simulation.bodies.remove(
                RigidBodyHandle(self.handle.index()),
                &mut self.simulation.island_manager,
                &mut self.simulation.colliders,
                &mut self.simulation.impulse_joints,
                &mut self.simulation.multibody_joints,
                /*remove_attached_colliders=*/ true,
            );
        }
    }
}
