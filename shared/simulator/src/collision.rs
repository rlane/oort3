use crate::bullet::{self, BulletHandle};
use crate::index_set::HasIndex;
use crate::ship::{ShipClass, ShipHandle};
use crate::simulation::{Particle, Simulation, PHYSICS_TICK_LENGTH};
use nalgebra::{ComplexField, Rotation2, UnitComplex};
use oort_api::Ability;
use rand::Rng;
use rapier2d_f64::prelude::*;
use std::f64::consts::TAU;

const DAMAGE_FACTOR: f64 = 0.00014;
const WALL_COLLISION_GROUP: Group = Group::GROUP_1;
const SHIP_COLLISION_GROUP: Group = Group::GROUP_2;
const PLANET_COLLISION_GROUP: Group = Group::GROUP_3;
const BULLET_GROUPS: &[Group] = &[
    Group::GROUP_4,
    Group::GROUP_5,
    Group::GROUP_6,
    Group::GROUP_7,
    Group::GROUP_8,
    Group::GROUP_9,
    Group::GROUP_10,
    Group::GROUP_11,
    Group::GROUP_12,
    Group::GROUP_13,
];

fn bullet_group(team: i32) -> Group {
    BULLET_GROUPS[team as usize]
}

fn all_bullet_groups() -> Group {
    let mut r = Group::empty();
    r.extend(BULLET_GROUPS.iter().cloned());
    r
}

pub fn bullet_interaction_groups(team: i32) -> InteractionGroups {
    InteractionGroups::new(
        bullet_group(team),
        WALL_COLLISION_GROUP | SHIP_COLLISION_GROUP | PLANET_COLLISION_GROUP,
    )
}

pub fn wall_interaction_groups() -> InteractionGroups {
    InteractionGroups::new(
        WALL_COLLISION_GROUP,
        SHIP_COLLISION_GROUP | all_bullet_groups(),
    )
}

pub fn ship_interaction_groups(team: i32) -> InteractionGroups {
    let bullet_groups = all_bullet_groups() ^ bullet_group(team);
    InteractionGroups::new(
        SHIP_COLLISION_GROUP,
        WALL_COLLISION_GROUP | SHIP_COLLISION_GROUP | PLANET_COLLISION_GROUP | bullet_groups,
    )
}

pub fn planet_interaction_groups() -> InteractionGroups {
    let bullet_groups = all_bullet_groups();
    InteractionGroups::new(
        PLANET_COLLISION_GROUP,
        SHIP_COLLISION_GROUP | PLANET_COLLISION_GROUP | bullet_groups,
    )
}

pub fn handle_collisions(sim: &mut Simulation, events: &[CollisionEvent]) {
    for event in events {
        if let CollisionEvent::Started(h1, h2, _flags) = event {
            let get_index = |h| sim.colliders.get(h).and_then(|x| x.parent()).map(|x| x.0);
            let handle_hit = |sim: &mut Simulation, ship, bullet: BulletHandle| {
                let (bullet_position, bullet_velocity) = {
                    let body = bullet::body(sim, bullet);
                    (body.position().translation.vector, *body.linvel())
                };
                if sim.ship(ship).is_ability_active(Ability::Shield) {
                    let dp = bullet_position - sim.ship(ship).position().vector;
                    let normal = dp.normalize();
                    let new_bullet_velocity = normal * bullet_velocity.magnitude();
                    {
                        let body = bullet::body_mut(sim, bullet);
                        body.set_linvel(new_bullet_velocity, false);
                        body.set_translation(
                            bullet_position + new_bullet_velocity * PHYSICS_TICK_LENGTH,
                            false,
                        );
                    }
                    bullet::data_mut(sim, bullet).team = sim.ship(ship).data().team;
                    return;
                }
                if bullet::data(sim, bullet).team == sim.ship(ship).data().team {
                    bullet::destroy(sim, bullet);
                    return;
                }
                let dv = bullet_velocity - sim.ship(ship).velocity();
                let energy = 0.5 * bullet::data(sim, bullet).mass as f64 * dv.magnitude_squared();
                let damage = energy * DAMAGE_FACTOR;
                for _ in 0..((damage as i32 / 10).clamp(1, 20)) {
                    let rot = Rotation2::new(sim.rng.gen_range(0.0..TAU));
                    let v = rot.transform_vector(&vector![sim.rng.gen_range(0.0..1000.0), 0.0]);
                    let p = bullet_position + v * sim.rng.gen_range(0.0..0.1);
                    sim.events.particles.push(Particle {
                        position: p,
                        velocity: v,
                        color: vector![1.0, 1.0, 1.0, sim.rng.gen_range(0.5..1.0)],
                        lifetime: (PHYSICS_TICK_LENGTH * 30.0) as f32,
                    });
                }
                let ship_destroyed = {
                    let ship_data = sim.ship_data.get_mut(ship.index()).unwrap();
                    ship_data.health -= damage;
                    ship_data.health <= 0.0
                };
                if ship_destroyed {
                    for _ in 0..10 {
                        let rot = Rotation2::new(sim.rng.gen_range(0.0..TAU));
                        let v = rot.transform_vector(&vector![sim.rng.gen_range(0.0..200.0), 0.0]);
                        let p = sim.ship(ship).body().position().translation.vector
                            + v * sim.rng.gen_range(0.0..0.1);
                        let lifetime =
                            (ComplexField::log2(sim.ship_data.get(ship.index()).unwrap().base_stats.mass)
                                * PHYSICS_TICK_LENGTH) as f32;
                        sim.events.particles.push(Particle {
                            position: p,
                            velocity: v,
                            color: vector![1.0, 1.0, 1.0, sim.rng.gen_range(0.5..1.0)],
                            lifetime,
                        });
                    }
                    sim.ship_mut(ship).data_mut().destroyed = true;
                    bullet::data_mut(sim, bullet).mass *= 0.5;
                    let rotation = UnitComplex::new(sim.rng.gen_range(-0.1..0.1));
                    let new_bullet_velocity = rotation.transform_vector(&bullet_velocity);
                    bullet::body_mut(sim, bullet).set_linvel(new_bullet_velocity, false);
                } else {
                    bullet::destroy(sim, bullet);
                }
            };
            if let (Some(idx1), Some(idx2)) = (get_index(*h1), get_index(*h2)) {
                #[derive(Ord, Eq, PartialOrd, PartialEq)]
                enum Collider {
                    Bullet(BulletHandle),
                    Ship(ShipHandle),
                    Wall,
                }
                let classify_collider = |idx| {
                    if sim.bullets.contains(BulletHandle(idx)) {
                        Collider::Bullet(BulletHandle(idx))
                    } else if sim.ships.contains(ShipHandle(idx)) {
                        Collider::Ship(ShipHandle(idx))
                    } else {
                        Collider::Wall
                    }
                };
                let mut collider_types = [classify_collider(idx1), classify_collider(idx2)];
                collider_types.sort();
                match collider_types {
                    [Collider::Bullet(b), Collider::Ship(s)] => {
                        handle_hit(sim, s, b);
                    }
                    [Collider::Bullet(b), Collider::Wall] => {
                        bullet::destroy(sim, b);
                    }
                    [Collider::Ship(s1), Collider::Ship(s2)] => {
                        if sim.ship(s1).data().team != sim.ship(s2).data().team {
                            sim.ship_mut(s1).handle_collision();
                            sim.ship_mut(s2).handle_collision();
                        }
                    }
                    [Collider::Ship(s), Collider::Wall] => {
                        if sim.ship(s).data().class != ShipClass::Planet {
                            sim.ship_mut(s).explode();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn add_walls(sim: &mut Simulation) {
    let world_size = sim.world_size();
    let mut make_edge = |x: f64, y: f64, a: f64| {
        let edge_length = world_size;
        let edge_width = 10.0;
        let rigid_body = RigidBodyBuilder::fixed()
            .translation(vector![x, y])
            .rotation(a)
            .build();
        let body_handle = sim.bodies.insert(rigid_body);
        let collider = ColliderBuilder::cuboid(edge_length / 2.0, edge_width / 2.0)
            .restitution(1.0)
            .collision_groups(wall_interaction_groups())
            .build();
        sim.colliders
            .insert_with_parent(collider, body_handle, &mut sim.bodies);
    };
    make_edge(0.0, world_size / 2.0, 0.0);
    make_edge(0.0, -world_size / 2.0, std::f64::consts::PI);
    make_edge(world_size / 2.0, 0.0, std::f64::consts::PI / 2.0);
    make_edge(-world_size / 2.0, 0.0, 3.0 * std::f64::consts::PI / 2.0);
}
