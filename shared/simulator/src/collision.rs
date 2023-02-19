use crate::bullet::{self, BulletHandle};
use crate::index_set::HasIndex;
use crate::ship::ShipHandle;
use crate::simulation::{Particle, Simulation, PHYSICS_TICK_LENGTH};
use nalgebra::{Rotation2, UnitComplex};
use oort_api::Ability;
use rand::Rng;
use rapier2d_f64::prelude::*;
use std::f64::consts::TAU;

const DAMAGE_FACTOR: f64 = 0.00014;
const WALL_COLLISION_GROUP: Group = Group::GROUP_1;
const SHIP_COLLISION_GROUP: Group = Group::GROUP_2;
const BULLET_GROUPS: &[Group] = &[
    Group::GROUP_3,
    Group::GROUP_4,
    Group::GROUP_5,
    Group::GROUP_6,
    Group::GROUP_7,
    Group::GROUP_8,
    Group::GROUP_9,
    Group::GROUP_10,
    Group::GROUP_11,
    Group::GROUP_12,
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
        WALL_COLLISION_GROUP | SHIP_COLLISION_GROUP,
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
        WALL_COLLISION_GROUP | SHIP_COLLISION_GROUP | bullet_groups,
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
                    sim.bullet_mut(bullet).data_mut().team = sim.ship(ship).data().team;
                    return;
                }
                if sim.bullet(bullet).data().team == sim.ship(ship).data().team {
                    sim.bullet_mut(bullet).destroy();
                    return;
                }
                let dv = bullet_velocity - sim.ship(ship).velocity();
                let energy = 0.5 * sim.bullet(bullet).data().mass * dv.magnitude_squared();
                let damage = energy * DAMAGE_FACTOR;
                for _ in 0..((damage as i32 / 10).clamp(1, 20)) {
                    let rot = Rotation2::new(sim.rng.gen_range(0.0..TAU));
                    let v = rot.transform_vector(&vector![sim.rng.gen_range(0.0..500.0), 0.0]);
                    let p = bullet_position + v * sim.rng.gen_range(0.0..0.1);
                    sim.events.particles.push(Particle {
                        position: p,
                        velocity: v,
                        color: vector![1.0, 1.0, 1.0, sim.rng.gen_range(0.5..1.0)],
                        lifetime: (PHYSICS_TICK_LENGTH * 10.0) as f32,
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
                        let lifetime = (sim.ship_data.get(ship.index()).unwrap().mass.log2()
                            * PHYSICS_TICK_LENGTH) as f32;
                        sim.events.particles.push(Particle {
                            position: p,
                            velocity: v,
                            color: vector![1.0, 1.0, 1.0, sim.rng.gen_range(0.5..1.0)],
                            lifetime,
                        });
                    }
                    sim.ship_mut(ship).data_mut().destroyed = true;
                    sim.bullet_mut(bullet).data_mut().mass *= 0.5;
                    let rotation = UnitComplex::new(sim.rng.gen_range(-0.1..0.1));
                    let new_bullet_velocity = rotation.transform_vector(&bullet_velocity);
                    bullet::body_mut(sim, bullet).set_linvel(new_bullet_velocity, false);
                } else {
                    sim.bullet_mut(bullet).destroy();
                }
            };
            if let (Some(idx1), Some(idx2)) = (get_index(*h1), get_index(*h2)) {
                if sim.ships.contains(ShipHandle(idx1))
                    && sim.ships.contains(ShipHandle(idx2))
                    && sim.ship(ShipHandle(idx1)).data().team
                        != sim.ship(ShipHandle(idx2)).data().team
                {
                    sim.ship_mut(ShipHandle(idx1)).handle_collision();
                    sim.ship_mut(ShipHandle(idx2)).handle_collision();
                }

                if sim.bullets.contains(BulletHandle(idx1)) {
                    if sim.ships.contains(ShipHandle(idx2)) {
                        handle_hit(sim, ShipHandle(idx2), BulletHandle(idx1));
                    } else {
                        sim.bullet_mut(BulletHandle(idx1)).destroy();
                    }
                } else if sim.bullets.contains(BulletHandle(idx2)) {
                    if sim.ships.contains(ShipHandle(idx1)) {
                        handle_hit(sim, ShipHandle(idx1), BulletHandle(idx2));
                    } else {
                        sim.bullet_mut(BulletHandle(idx2)).destroy();
                    }
                }
            }
        }
    }
}
