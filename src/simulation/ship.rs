use super::index_set::{HasIndex, Index};
use super::rng::new_rng;
use crate::simulation;
use crate::simulation::{bullet, Simulation};
use bullet::BulletData;
use nalgebra::{vector, Rotation2, Vector2};
use rand::Rng;
use rapier2d_f64::prelude::*;

#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug)]
pub struct ShipHandle(pub Index);

impl HasIndex for ShipHandle {
    fn index(self) -> Index {
        self.0
    }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum ShipClass {
    Fighter,
    Asteroid { variant: i32 },
    Target,
    Missile,
}

pub struct Weapon {
    pub reload_time: f64,
    pub reload_time_remaining: f64,
    pub damage: f64,
}

pub struct ShipData {
    pub class: ShipClass,
    pub weapons: Vec<Weapon>,
    pub missile: Option<Weapon>,
    pub health: f64,
    pub team: i32,
    pub acceleration: Vector2<f64>,
    pub angular_acceleration: f64,
    pub max_acceleration: Vector2<f64>,
    pub max_angular_acceleration: f64,
    pub destroyed: bool,
}

impl Default for ShipData {
    fn default() -> ShipData {
        ShipData {
            class: ShipClass::Fighter,
            weapons: vec![],
            missile: None,
            health: 100.0,
            team: 0,
            acceleration: vector![0.0, 0.0],
            angular_acceleration: 0.0,
            max_acceleration: vector![0.0, 0.0],
            max_angular_acceleration: 0.0,
            destroyed: false,
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
        }],
        missile: Some(Weapon {
            reload_time: 5.0,
            reload_time_remaining: 0.0,
            damage: 0.0,
        }),
        health: 100.0,
        team,
        max_acceleration: vector![200.0, 100.0],
        max_angular_acceleration: std::f64::consts::TAU,
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
        max_acceleration: vector![400.0, 0.0],
        max_angular_acceleration: 2.0 * std::f64::consts::TAU,
        team,
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
    let rigid_body = RigidBodyBuilder::new_dynamic()
        .translation(vector![x, y])
        .linvel(vector![vx, vy])
        .rotation(h)
        .ccd_enabled(true)
        .build();
    let body_handle = sim.bodies.insert(rigid_body);
    let handle = ShipHandle(body_handle.0);
    let team = data.team;
    match data.class {
        ShipClass::Fighter => {
            let vertices = crate::renderer::model::ship()
                .iter()
                .map(|&v| point![v.x as f64, v.y as f64])
                .collect::<Vec<_>>();
            let collider = ColliderBuilder::convex_hull(&vertices)
                .unwrap()
                .restitution(1.0)
                .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
                .collision_groups(InteractionGroups::new(
                    1 << simulation::SHIP_COLLISION_GROUP,
                    1 << simulation::WALL_COLLISION_GROUP
                        | 1 << simulation::SHIP_COLLISION_GROUP
                        | 1 << simulation::BULLET_COLLISION_GROUP,
                ))
                .build();
            sim.colliders
                .insert_with_parent(collider, body_handle, &mut sim.bodies);
        }
        ShipClass::Asteroid { variant } => {
            let vertices = crate::renderer::model::asteroid(variant)
                .iter()
                .map(|&v| point![v.x as f64, v.y as f64])
                .collect::<Vec<_>>();
            let collider = ColliderBuilder::convex_hull(&vertices)
                .unwrap()
                .restitution(1.0)
                .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
                .collision_groups(InteractionGroups::new(
                    1 << simulation::SHIP_COLLISION_GROUP,
                    1 << simulation::WALL_COLLISION_GROUP
                        | 1 << simulation::SHIP_COLLISION_GROUP
                        | 1 << simulation::BULLET_COLLISION_GROUP,
                ))
                .build();
            sim.colliders
                .insert_with_parent(collider, body_handle, &mut sim.bodies);
        }
        ShipClass::Target => {
            let vertices = crate::renderer::model::target()
                .iter()
                .map(|&v| point![v.x as f64, v.y as f64])
                .collect::<Vec<_>>();
            let collider = ColliderBuilder::convex_hull(&vertices)
                .unwrap()
                .restitution(1.0)
                .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
                .collision_groups(InteractionGroups::new(
                    1 << simulation::SHIP_COLLISION_GROUP,
                    1 << simulation::WALL_COLLISION_GROUP
                        | 1 << simulation::SHIP_COLLISION_GROUP
                        | 1 << simulation::BULLET_COLLISION_GROUP,
                ))
                .build();
            sim.colliders
                .insert_with_parent(collider, body_handle, &mut sim.bodies);
        }
        ShipClass::Missile => {
            let vertices = crate::renderer::model::missile()
                .iter()
                .map(|&v| point![v.x as f64, v.y as f64])
                .collect::<Vec<_>>();
            let collider = ColliderBuilder::convex_hull(&vertices)
                .unwrap()
                .restitution(1.0)
                .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
                .collision_groups(InteractionGroups::new(
                    1 << simulation::SHIP_COLLISION_GROUP,
                    1 << simulation::WALL_COLLISION_GROUP
                        | 1 << simulation::SHIP_COLLISION_GROUP
                        | 1 << simulation::BULLET_COLLISION_GROUP,
                ))
                .build();
            sim.colliders
                .insert_with_parent(collider, body_handle, &mut sim.bodies);
        }
    }

    sim.ships.insert(handle);
    sim.ship_data.insert(handle, data);

    let sim_ptr: *mut Simulation = sim;
    if let Some(team_ctrl) = sim.team_controllers.get_mut(&team) {
        match team_ctrl.create_ship_controller(handle, sim_ptr) {
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
        self.data_mut().acceleration = acceleration;
    }

    pub fn torque(&mut self, angular_acceleration: f64) {
        self.data_mut().angular_acceleration = angular_acceleration;
    }

    pub fn fire_weapon(&mut self, index: i64) {
        let ship_data = self.data_mut();
        let team = ship_data.team;
        let damage;
        {
            let weapon = &mut ship_data.weapons[index as usize];
            damage = weapon.damage;
            if weapon.reload_time_remaining > 0.0 {
                return;
            }
            weapon.reload_time_remaining += weapon.reload_time;
        }

        let speed = 1000.0;
        let offset = vector![20.0, 0.0];
        let body = self.body();
        let rot = body.position().rotation;
        let p = body.position().translation.vector + rot.transform_vector(&offset);
        let v = body.linvel() + rot.transform_vector(&vector![speed, 0.0]);
        bullet::create(
            &mut self.simulation,
            p.x,
            p.y,
            v.x,
            v.y,
            BulletData { damage, team },
        );
    }

    pub fn launch_missile(&mut self) {
        if let Some(missile) = self.data_mut().missile.as_mut() {
            if missile.reload_time_remaining > 0.0 {
                return;
            }
            missile.reload_time_remaining += missile.reload_time;
        } else {
            return;
        }

        let speed = 100.0;
        let offset = vector![20.0, 0.0];
        let body = self.body();
        let rot = body.position().rotation;
        let p = body.position().translation.vector + rot.transform_vector(&offset);
        let v = body.linvel() + rot.transform_vector(&vector![speed, 0.0]);
        let team = self.data().team;
        create(
            &mut self.simulation,
            p.x,
            p.y,
            v.x,
            v.y,
            rot.angle(),
            missile(team),
        );
    }

    pub fn explode(&mut self) {
        if self.data().destroyed {
            return;
        }
        self.data_mut().destroyed = true;

        let team = self.data().team;
        let speed = 1000.0;
        let p = self.body().position().translation;
        let mut rng = new_rng(0);
        for _ in 0..25 {
            let rot = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU));
            let v = self.body().linvel() + rot.transform_vector(&vector![speed, 0.0]);
            bullet::create(
                &mut self.simulation,
                p.x,
                p.y,
                v.x,
                v.y,
                BulletData { damage: 20.0, team },
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

            if let Some(missile) = ship_data.missile.as_mut() {
                missile.reload_time_remaining =
                    (missile.reload_time_remaining - simulation::PHYSICS_TICK_LENGTH).max(0.0);
            }
        }
        // Acceleration.
        {
            let max_acceleration = self.data().max_acceleration;
            let acceleration = self
                .data()
                .acceleration
                .inf(&max_acceleration)
                .sup(&-max_acceleration);
            let mass = self.body().mass();
            let rotation_matrix = self.body().position().rotation.to_rotation_matrix();
            self.body()
                .apply_force(rotation_matrix * acceleration * mass, true);
            self.data_mut().acceleration = vector![0.0, 0.0];
        }

        // Torque.
        {
            let max_angular_acceleration = self.data().max_angular_acceleration;
            let angular_acceleration = self
                .data()
                .angular_acceleration
                .clamp(-max_angular_acceleration, max_angular_acceleration);
            let inertia_sqrt = 1.0 / self.body().mass_properties().inv_principal_inertia_sqrt;
            let torque = angular_acceleration * inertia_sqrt * inertia_sqrt;
            self.body().apply_torque(torque, true);
            self.data_mut().angular_acceleration = 0.0;
        }

        // Destruction.
        if self.data().destroyed {
            self.simulation.ships.remove(self.handle);
            self.simulation.bodies.remove(
                RigidBodyHandle(self.handle.index()),
                &mut self.simulation.island_manager,
                &mut self.simulation.colliders,
                &mut self.simulation.joints,
            );
        }
    }
}
