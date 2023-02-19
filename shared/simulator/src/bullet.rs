use super::index_set::{HasIndex, Index};
use crate::collision;
use crate::ship::ShipHandle;
use crate::simulation::{Simulation, PHYSICS_TICK_LENGTH};
use nalgebra::{Vector2, Vector4};
use rapier2d_f64::prelude::*;

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct BulletHandle(pub Index);

impl HasIndex for BulletHandle {
    fn index(self) -> Index {
        self.0
    }
}

impl From<BulletHandle> for RigidBodyHandle {
    fn from(handle: BulletHandle) -> Self {
        RigidBodyHandle(handle.index())
    }
}

#[derive(Default, Clone)]
pub struct BulletData {
    pub mass: f64,
    pub team: i32,
    pub ttl: f32,
    pub color: Vector4<f32>,
}

pub fn body(sim: &Simulation, handle: BulletHandle) -> &RigidBody {
    sim.bodies.get(handle.into()).unwrap()
}

pub fn body_mut(sim: &mut Simulation, handle: BulletHandle) -> &mut RigidBody {
    sim.bodies.get_mut(handle.into()).unwrap()
}

pub fn data(sim: &Simulation, handle: BulletHandle) -> &BulletData {
    sim.bullet_data.get(handle.index()).unwrap()
}

pub fn data_mut(sim: &mut Simulation, handle: BulletHandle) -> &mut BulletData {
    sim.bullet_data.get_mut(handle.index()).unwrap()
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
    let handle = BulletHandle(body_handle.0);
    sim.bullet_data.insert(handle.index(), data);
    sim.bullets.insert(handle);
    handle
}

pub fn destroy(sim: &mut Simulation, handle: BulletHandle) {
    sim.bullet_data
        .remove(handle.index(), BulletData::default());
    sim.bullets.remove(handle);
    sim.bodies.remove(
        RigidBodyHandle(handle.index()),
        &mut sim.island_manager,
        &mut sim.colliders,
        &mut sim.impulse_joints,
        &mut sim.multibody_joints,
        /*remove_attached_colliders=*/ true,
    );
}

pub fn tick(sim: &mut Simulation) {
    let dt = PHYSICS_TICK_LENGTH;
    let bullets: Vec<BulletHandle> = sim.bullets.iter().cloned().collect();
    for handle in bullets {
        let team = {
            let data = data_mut(sim, handle);
            data.ttl -= dt as f32;
            if data.ttl <= 0.0 {
                destroy(sim, handle);
                continue;
            }
            data.team
        };

        let has_collider;
        let mut needs_collider = false;
        {
            let body = sim.bodies.get_mut(RigidBodyHandle(handle.index())).unwrap();
            has_collider = !body.colliders().is_empty();

            let aabb = Aabb::from_half_extents(
                body.position().translation.vector.into(),
                vector![1.0, 1.0] * body.linvel().magnitude() * 2.0 * PHYSICS_TICK_LENGTH,
            );

            sim.query_pipeline
                .colliders_with_aabb_intersecting_aabb(&aabb, |&collider_handle| {
                    let get_index = |h| sim.colliders.get(h).and_then(|x| x.parent()).map(|x| x.0);
                    if let Some(index) = get_index(collider_handle) {
                        if sim.ships.contains(ShipHandle(index))
                            && sim.ship(ShipHandle(index)).data().team != team
                        {
                            needs_collider = true;
                        }
                    }
                    true
                });
        }

        if needs_collider && !has_collider {
            add_collider(sim, handle);
        } else if has_collider && !needs_collider {
            remove_collider(sim, handle);
        }
    }
}

fn add_collider(sim: &mut Simulation, handle: BulletHandle) {
    let team = data(sim, handle).team;
    let collider = ColliderBuilder::ball(1.0)
        .restitution(1.0)
        .collision_groups(collision::bullet_interaction_groups(team))
        .active_events(ActiveEvents::COLLISION_EVENTS)
        .sensor(true)
        .build();
    sim.colliders
        .insert_with_parent(collider, RigidBodyHandle(handle.index()), &mut sim.bodies);
}

fn remove_collider(sim: &mut Simulation, handle: BulletHandle) {
    let colliders = sim
        .bodies
        .get_mut(RigidBodyHandle(handle.index()))
        .unwrap()
        .colliders()
        .to_vec();
    for collider_handle in colliders {
        sim.colliders.remove(
            collider_handle,
            &mut sim.island_manager,
            &mut sim.bodies,
            true,
        );
    }
}
