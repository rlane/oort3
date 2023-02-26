use std::collections::HashMap;

use super::index_set::{HasIndex, Index};
use crate::simulation::{Simulation, PHYSICS_TICK_LENGTH, WORLD_SIZE};
use crate::{collision, simulation};
use bitvec::vec::BitVec;
use nalgebra::Vector2;
use rapier2d_f64::prelude::*;
use static_aabb2d_index::*;

const COLOR_COLLIDERS: bool = false;

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
    pub mass: f32,
    pub team: i32,
    pub ttl: f32,
    pub color: u32,
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
    mut data: BulletData,
) -> BulletHandle {
    let rigid_body = RigidBodyBuilder::dynamic()
        .translation(position)
        .linvel(velocity)
        .ccd_enabled(true)
        .build();
    let body_handle = sim.bodies.insert(rigid_body);
    let handle = BulletHandle(body_handle.0);
    if COLOR_COLLIDERS {
        data.color = 0xff0000ff;
    }
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
    let (indices_by_team, coarse_grids_by_team) = build_indices(sim, dt);
    let mut stack = Vec::new();
    let shape = rapier2d_f64::geometry::Ball { radius: 1.0 };
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
        let coarse_grid_hit;
        let mut needs_collider = false;
        {
            let body = sim.bodies.get_mut(RigidBodyHandle(handle.index())).unwrap();
            has_collider = !body.colliders().is_empty();

            let position = *body.translation();
            if position.x < -WORLD_SIZE / 2.0
                || position.x > WORLD_SIZE / 2.0
                || position.y < -WORLD_SIZE / 2.0
                || position.y > WORLD_SIZE / 2.0
            {
                destroy(sim, handle);
                continue;
            }

            coarse_grid_hit = coarse_grids_by_team
                .iter()
                .any(|(other_team, grid)| *other_team != team && grid.lookup(position));
            if coarse_grid_hit {
                let aabb = shape.compute_swept_aabb(
                    body.position(),
                    &body.predict_position_using_velocity_and_forces(dt),
                );

                for (other_team, index) in indices_by_team.iter() {
                    if team != *other_team {
                        needs_collider = needs_collider
                            || index
                                .query_iter_with_stack(
                                    aabb.mins.x,
                                    aabb.mins.y,
                                    aabb.maxs.x,
                                    aabb.maxs.y,
                                    &mut stack,
                                )
                                .next()
                                .is_some();
                    }
                }
            }
        }

        if needs_collider && !has_collider {
            add_collider(sim, handle);
        } else if has_collider && !needs_collider {
            remove_collider(sim, handle);
        }

        if COLOR_COLLIDERS {
            if needs_collider {
                data_mut(sim, handle).color = 0x00ff00ff;
            } else if coarse_grid_hit {
                data_mut(sim, handle).color = 0x0000ffff;
            } else {
                data_mut(sim, handle).color = 0xff0000ff;
            }
        }
    }
}

fn build_indices(
    sim: &Simulation,
    dt: f64,
) -> (
    HashMap<i32, StaticAABB2DIndex<f64>>,
    HashMap<i32, CoarseGrid>,
) {
    let mut aabbs_by_team: HashMap<i32, Vec<Aabb>> = HashMap::new();
    let mut coarse_grids_by_team: HashMap<i32, CoarseGrid> = HashMap::new();

    for handle in sim.ships.iter() {
        let body = sim.ship(*handle).body();
        let collider_handle = body.colliders()[0];
        let collider = sim.colliders.get(collider_handle).unwrap();
        let aabb =
            collider.compute_swept_aabb(&body.predict_position_using_velocity_and_forces(dt));
        let team = sim.ship(*handle).data().team;
        aabbs_by_team.entry(team).or_default().push(aabb);
        coarse_grids_by_team.entry(team).or_default().insert(aabb);
    }

    let mut indices_by_team: HashMap<i32, StaticAABB2DIndex<f64>> = HashMap::new();
    for (team, aabbs) in aabbs_by_team {
        let mut builder = StaticAABB2DIndexBuilder::new(aabbs.len());
        for aabb in aabbs {
            builder.add(aabb.mins.x, aabb.mins.y, aabb.maxs.x, aabb.maxs.y);
        }
        indices_by_team.insert(team, builder.build().unwrap());
    }

    (indices_by_team, coarse_grids_by_team)
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

struct CoarseGrid {
    cells: BitVec,
}

impl CoarseGrid {
    const CELL_SIZE: f64 = 100.0;
    const RECIP_CELL_SIZE: f64 = 1.0 / Self::CELL_SIZE;
    const WORLD_WIDTH: f64 = simulation::WORLD_SIZE + Self::CELL_SIZE * 2.0;
    const WIDTH: i32 = (Self::WORLD_WIDTH / Self::CELL_SIZE) as i32;

    fn to_cell(p: Vector2<f64>) -> usize {
        let x = ((Self::WORLD_WIDTH / 2.0 + p.x) * Self::RECIP_CELL_SIZE) as i32;
        let y = ((Self::WORLD_WIDTH / 2.0 + p.y) * Self::RECIP_CELL_SIZE) as i32;
        (Self::WIDTH * y + x) as usize
    }

    pub fn new() -> Self {
        let mut cells = BitVec::new();
        cells.resize((Self::WIDTH * Self::WIDTH) as usize, false);
        Self { cells }
    }

    pub fn lookup(&self, p: Vector2<f64>) -> bool {
        let index: usize = Self::to_cell(p);
        if index < self.cells.len() {
            self.cells[index]
        } else {
            false
        }
    }

    pub fn insert(&mut self, mut aabb: Aabb) {
        aabb.mins -= vector![Self::CELL_SIZE, Self::CELL_SIZE];
        aabb.maxs += vector![Self::CELL_SIZE, Self::CELL_SIZE];
        if let Some(aabb) = aabb.intersection(&Aabb::from_half_extents(
            point![0.0, 0.0],
            vector![WORLD_SIZE / 2.0, WORLD_SIZE / 2.0],
        )) {
            let w = ((aabb.maxs.x - aabb.mins.x) * Self::RECIP_CELL_SIZE).ceil() as i32;
            let h = ((aabb.maxs.y - aabb.mins.y) * Self::RECIP_CELL_SIZE).ceil() as i32;
            let min_index = Self::to_cell(aabb.mins.coords);
            for y in 0..h {
                for x in 0..w {
                    let index = (min_index as i32 + x + y * Self::WIDTH) as usize;
                    self.cells.set(index, true);
                }
            }
        }
    }
}

impl Default for CoarseGrid {
    fn default() -> Self {
        Self::new()
    }
}
