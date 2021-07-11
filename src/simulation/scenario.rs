use super::ship::ShipClass::*;
use super::ship::{fighter, ShipData};
use super::{
    ship, Simulation, BULLET_COLLISION_GROUP, SHIP_COLLISION_GROUP, WALL_COLLISION_GROUP,
    WORLD_SIZE,
};
use rand::Rng;
use rapier2d_f64::prelude::*;
use Status::Running;

#[derive(PartialEq)]
pub enum Status {
    Running,
    Finished,
}

pub trait Scenario {
    fn init(&self, sim: &mut Simulation);
    fn tick(&self, sim: &mut Simulation) -> Status;
}

pub fn add_walls(sim: &mut Simulation) {
    let mut make_edge = |x: f64, y: f64, a: f64| {
        let edge_length = WORLD_SIZE as f64;
        let edge_width = 10.0;
        let rigid_body = RigidBodyBuilder::new_static()
            .translation(vector![x, y])
            .rotation(a)
            .build();
        let body_handle = sim.bodies.insert(rigid_body);
        let collider = ColliderBuilder::cuboid(edge_length / 2.0, edge_width / 2.0)
            .restitution(1.0)
            .collision_groups(InteractionGroups::new(
                1 << WALL_COLLISION_GROUP,
                1 << SHIP_COLLISION_GROUP | 1 << BULLET_COLLISION_GROUP,
            ))
            .build();
        sim.colliders
            .insert_with_parent(collider, body_handle, &mut sim.bodies);
    };
    make_edge(0.0, WORLD_SIZE / 2.0, 0.0);
    make_edge(0.0, -WORLD_SIZE / 2.0, std::f64::consts::PI);
    make_edge(WORLD_SIZE / 2.0, 0.0, std::f64::consts::PI / 2.0);
    make_edge(-WORLD_SIZE / 2.0, 0.0, 3.0 * std::f64::consts::PI / 2.0);
}

pub fn load(name: &str) -> Box<dyn Scenario> {
    match name {
        "basic" => Box::new(BasicScenario {}),
        "asteroid" => Box::new(AsteroidScenario {}),
        _ => panic!("Unknown scenario"),
    }
}

struct BasicScenario {}

impl Scenario for BasicScenario {
    fn init(&self, sim: &mut Simulation) {
        add_walls(sim);
        ship::create(sim, -100.0, 0.0, 0.0, 0.0, 0.0, fighter());
        ship::create(sim, 100.0, 0.0, 0.0, 0.0, std::f64::consts::PI, fighter());
    }

    fn tick(&self, sim: &mut Simulation) -> Status {
        if sim.ships.iter().len() > 1 {
            Running
        } else {
            Status::Finished
        }
    }
}

struct AsteroidScenario {}

impl Scenario for AsteroidScenario {
    fn init(&self, sim: &mut Simulation) {
        let mut rng = rand::thread_rng();
        add_walls(sim);
        ship::create(sim, -100.0, 0.0, 0.0, 0.0, 0.0, fighter());

        let bound = (WORLD_SIZE / 2.0) * 0.9;
        for _ in 1..1000 {
            ship::create(
                sim,
                rng.gen_range(-bound..bound),
                rng.gen_range(-bound..bound),
                rng.gen_range(-30.0..30.0),
                rng.gen_range(-30.0..30.0),
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                ShipData { class: Asteroid },
            );
        }
    }

    fn tick(&self, sim: &mut Simulation) -> Status {
        if sim.ships.iter().len() > 1 {
            Running
        } else {
            Status::Finished
        }
    }
}
