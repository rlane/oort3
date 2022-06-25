use super::index_set::{HasIndex, Index};
use crate::ship::ShipHandle;
use crate::simulation::{Simulation, PHYSICS_TICK_LENGTH};
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
        .build();
    let body_handle = sim.bodies.insert(rigid_body);
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

    pub fn tick(&mut self, dt: f64, collisions: &mut Vec<BulletCollisionData>) {
        let team = {
            let data = self.data_mut();
            data.ttl -= dt as f32;
            if data.ttl <= 0.0 {
                self.destroy();
                return;
            }
            data.team
        };

        let body = self
            .simulation
            .bodies
            .get(RigidBodyHandle(self.handle.index()))
            .unwrap();

        let position = body.position();
        let velocity = body.linvel();

        let shape = rapier2d_f64::geometry::Ball::new(3.0);
        let max_toi = PHYSICS_TICK_LENGTH;

        let groups = InteractionGroups::all();
        let collision_filter = |other: ColliderHandle| {
            let index = self
                .simulation
                .colliders
                .get(other)
                .and_then(|x| x.parent())
                .map(|x| x.0);
            if let Some(index) = index {
                if self.simulation.ships.contains(ShipHandle(index)) {
                    let ship = self.simulation.ship(ShipHandle(index));
                    return ship.data().team != team;
                }
            }
            false
        };
        let filter_fn: &dyn Fn(ColliderHandle) -> bool = &collision_filter;
        let filter = Some(filter_fn);

        if let Some((collider_handle, toi)) = self.simulation.query_pipeline.cast_shape(
            &self.simulation.colliders,
            position,
            velocity,
            &shape,
            max_toi,
            groups,
            filter,
        ) {
            collisions.push(BulletCollisionData {
                bullet_handle: self.handle,
                collider_handle,
                impact_point: toi.witness1,
            });
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

pub struct BulletCollisionData {
    pub bullet_handle: BulletHandle,
    pub collider_handle: ColliderHandle,
    pub impact_point: Point<f64>,
}
