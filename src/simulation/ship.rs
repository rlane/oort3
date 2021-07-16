use super::index_set::{HasIndex, Index};
use crate::script;
use crate::simulation;
use crate::simulation::{bullet, Simulation};
use nalgebra::Vector2;
use rapier2d_f64::prelude::*;

#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug)]
pub struct ShipHandle(pub Index);

impl HasIndex for ShipHandle {
    fn index(self) -> Index {
        self.0
    }
}

#[derive(Hash, Eq, PartialEq)]
pub enum ShipClass {
    Fighter,
    Asteroid,
}

pub struct ShipData {
    pub class: ShipClass,
}

pub fn fighter() -> ShipData {
    ShipData {
        class: ShipClass::Fighter,
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
            let sim_ptr = sim as *mut Simulation;
            sim.ship_controllers
                .insert(handle, script::new_ship_controller(handle, sim_ptr));
        }
        ShipClass::Asteroid => {
            let vertices = crate::renderer::model::asteroid()
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
    handle
}

pub struct ShipAccessor<'a> {
    pub(crate) simulation: &'a Simulation,
    pub(crate) handle: ShipHandle,
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
        self.body().rotation().angle()
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

    pub fn accelerate(&mut self, acceleration: Vector2<f64>) {
        let body = self.body();
        let rotation_matrix = body.position().rotation.to_rotation_matrix();
        body.apply_force(rotation_matrix * acceleration * body.mass(), true);
    }

    pub fn thrust_angular(&mut self, torque: f64) {
        self.body().apply_torque(torque, true);
    }

    pub fn fire_weapon(&mut self, _index: i64) {
        let speed = 1000.0;
        let offset = vector![20.0, 0.0];
        let body = self.body();
        let rot = body.position().rotation;
        let p = body.position().translation.vector + rot.transform_vector(&offset);
        let v = body.linvel() + rot.transform_vector(&vector![speed, 0.0]);
        bullet::create(&mut self.simulation, p.x, p.y, v.x, v.y);
    }

    pub fn explode(&mut self) {
        self.simulation.ships.remove(self.handle);
        self.simulation.bodies.remove(
            RigidBodyHandle(self.handle.index()),
            &mut self.simulation.island_manager,
            &mut self.simulation.colliders,
            &mut self.simulation.joints,
        );
    }
}
