use super::index_set::{HasIndex, Index};
use crate::collision;
use crate::ship::ShipHandle;
use crate::simulation::{Simulation, PHYSICS_TICK_LENGTH};
use nalgebra::{Vector2, Vector4};
use rapier2d_f64::prelude::*;

const LAZY_COLLIDERS: bool = true;

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct BulletHandle(pub Index);

impl HasIndex for BulletHandle {
    fn index(self) -> Index {
        self.0
    }
}

pub struct BulletData {
    pub mass: f64,
    pub team: i32,
    pub ttl: f32,
    pub color: Vector4<f32>,
}

pub fn create(
    sim: &mut Simulation,
    position: Vector2<f64>,
    velocity: Vector2<f64>,
    data: BulletData,
) -> BulletHandle {
    let rigid_body = RigidBodyBuilder::dynamic()
        .translation(position)
        .linvel(velocity)
        .ccd_enabled(true)
        .build();
    let body_handle = sim.bodies.insert(rigid_body);
    if !LAZY_COLLIDERS {
        let collider = ColliderBuilder::ball(1.0)
            .restitution(1.0)
            .collision_groups(collision::bullet_interaction_groups(data.team))
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .sensor(true)
            .build();
        sim.colliders
            .insert_with_parent(collider, body_handle, &mut sim.bodies);
    }
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

    pub fn position(&self) -> Vector2<f64> {
        *self.body().translation()
    }
}

pub struct BulletAccessorMut<'a> {
    pub(crate) simulation: &'a mut Simulation,
    pub(crate) handle: BulletHandle,
}

impl<'a: 'b, 'b> BulletAccessorMut<'a> {
    pub fn position(&'a mut self) -> Vector2<f64> {
        *self.body().translation()
    }

    pub fn body(&'a mut self) -> &'a mut RigidBody {
        self.simulation
            .bodies
            .get_mut(RigidBodyHandle(self.handle.index()))
            .unwrap()
    }

    pub fn data_mut(&mut self) -> &mut BulletData {
        self.simulation.bullet_data.get_mut(&self.handle).unwrap()
    }

    pub fn tick(&mut self, dt: f64) {
        let team;
        {
            let data = self.data_mut();
            data.ttl -= dt as f32;
            if data.ttl <= 0.0 {
                self.destroy();
                return;
            }
            team = data.team;
        }

        if LAZY_COLLIDERS {
            let has_collider;
            let mut needs_collider = false;
            {
                let body = self
                    .simulation
                    .bodies
                    .get_mut(RigidBodyHandle(self.handle.index()))
                    .unwrap();
                has_collider = !body.colliders().is_empty();

                let aabb = Aabb::from_half_extents(
                    body.position().translation.vector.into(),
                    vector![1.0, 1.0] * body.linvel().magnitude() * 2.0 * PHYSICS_TICK_LENGTH,
                );

                self.simulation
                    .query_pipeline
                    .colliders_with_aabb_intersecting_aabb(&aabb, |&collider_handle| {
                        let get_index = |h| {
                            self.simulation
                                .colliders
                                .get(h)
                                .and_then(|x| x.parent())
                                .map(|x| x.0)
                        };
                        if let Some(index) = get_index(collider_handle) {
                            if self.simulation.ships.contains(ShipHandle(index))
                                && self.simulation.ship(ShipHandle(index)).data().team != team
                            {
                                needs_collider = true;
                            }
                        }
                        true
                    });
            }

            if needs_collider && !has_collider {
                self.add_collider();
            } else if has_collider && !needs_collider {
                self.remove_collider();
            }
        }
    }

    pub fn add_collider(&mut self) {
        let collider = ColliderBuilder::ball(1.0)
            .restitution(1.0)
            .collision_groups(collision::bullet_interaction_groups(self.data_mut().team))
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .sensor(true)
            .build();
        self.simulation.colliders.insert_with_parent(
            collider,
            RigidBodyHandle(self.handle.index()),
            &mut self.simulation.bodies,
        );
    }

    pub fn remove_collider(&mut self) {
        let colliders = self
            .simulation
            .bodies
            .get_mut(RigidBodyHandle(self.handle.index()))
            .unwrap()
            .colliders()
            .to_vec();
        for collider_handle in colliders {
            self.simulation.colliders.remove(
                collider_handle,
                &mut self.simulation.island_manager,
                &mut self.simulation.bodies,
                true,
            );
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
