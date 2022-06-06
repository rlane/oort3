use super::index_set::{HasIndex, Index};
use crate::collision;
use crate::simulation::Simulation;
use nalgebra::Vector4;
use rapier2d_f64::prelude::*;

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct BulletHandle(pub Index);

impl HasIndex for BulletHandle {
    fn index(self) -> Index {
        self.0
    }
}

pub struct BulletData {
    pub damage: f64,
    pub team: i32,
    pub ttl: f32,
    pub color: Vector4<f32>,
}

pub fn create(
    sim: &mut Simulation,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    data: BulletData,
) -> BulletHandle {
    let rigid_body = RigidBodyBuilder::dynamic()
        .translation(vector![x, y])
        .linvel(vector![vx, vy])
        .ccd_enabled(true)
        .build();
    let body_handle = sim.bodies.insert(rigid_body);
    let collider = ColliderBuilder::ball(1.0)
        .restitution(1.0)
        .collision_groups(collision::bullet_interaction_groups(data.team))
        .active_events(ActiveEvents::COLLISION_EVENTS)
        .build();
    sim.colliders
        .insert_with_parent(collider, body_handle, &mut sim.bodies);
    let handle = BulletHandle(body_handle.0);
    sim.bullet_data.insert(handle, data);
    sim.bullets.insert(handle);
    handle
}

pub struct BulletAccessor<'a> {
    pub(crate) simulation: &'a Simulation,
    pub(crate) handle: BulletHandle,
}

impl<'a> BulletAccessor<'a> {
    pub fn body(&self) -> &'a RigidBody {
        self.simulation
            .bodies
            .get(RigidBodyHandle(self.handle.index()))
            .unwrap()
    }

    pub fn data(&self) -> &BulletData {
        self.simulation.bullet_data.get(&self.handle).unwrap()
    }
}

pub struct BulletAccessorMut<'a> {
    pub(crate) simulation: &'a mut Simulation,
    pub(crate) handle: BulletHandle,
}

impl<'a: 'b, 'b> BulletAccessorMut<'a> {
    pub fn data_mut(&mut self) -> &mut BulletData {
        self.simulation.bullet_data.get_mut(&self.handle).unwrap()
    }

    pub fn tick(&mut self, dt: f64) {
        let data = self.data_mut();
        data.ttl -= dt as f32;
        if data.ttl <= 0.0 {
            self.destroy();
        }
    }

    pub fn destroy(&mut self) {
        self.simulation.bullets.remove(self.handle);
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
