use crate::index_set::{HasIndex, Index};
use crate::simulation;
use crate::simulation::Simulation;
use rapier2d_f64::prelude::*;

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct ShipHandle(pub Index);

impl HasIndex for ShipHandle {
    fn index(self) -> Index {
        self.0
    }
}

pub fn create(sim: &mut Simulation, x: f64, y: f64, vx: f64, vy: f64, h: f64) -> ShipHandle {
    let rigid_body = RigidBodyBuilder::new_dynamic()
        .translation(vector![x, y])
        .linvel(vector![vx, vy])
        .rotation(h)
        .ccd_enabled(true)
        .build();
    let body_handle = sim.bodies.insert(rigid_body);
    let vertices = crate::model::ship()
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
    let handle = ShipHandle(body_handle.0);
    sim.ships.insert(handle);
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

    pub fn thrust_main(&mut self, force: f64) {
        let body = self.body();
        let rotation_matrix = body.position().rotation.to_rotation_matrix();
        body.apply_force(rotation_matrix * vector![force, 0.0], true);
    }

    pub fn thrust_lateral(&mut self, force: f64) {
        let body = self.body();
        let rotation_matrix = body.position().rotation.to_rotation_matrix();
        body.apply_force(rotation_matrix * vector![0.0, force], true);
    }

    pub fn thrust_angular(&mut self, torque: f64) {
        self.body().apply_torque(torque, true);
    }

    pub fn fire_weapon(&mut self) {
        let speed = 1000.0;
        let offset = vector![20.0, 0.0];
        let body = self.body();
        let rot = body.position().rotation;
        let p = body.position().translation.vector + rot.transform_vector(&offset);
        let v = body.linvel() + rot.transform_vector(&vector![speed, 0.0]);
        crate::bullet::create(&mut self.simulation, p.x, p.y, v.x, v.y);
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
