use super::ship::ShipClass::*;
use super::ship::{fighter, ShipData};
use super::{
    bullet, ship, Simulation, BULLET_COLLISION_GROUP, SHIP_COLLISION_GROUP, WALL_COLLISION_GROUP,
    WORLD_SIZE,
};
use nalgebra::{Point2, Vector4};
use rand::Rng;
use rapier2d_f64::prelude::*;
use Status::Running;

#[derive(PartialEq)]
pub enum Status {
    Running,
    Finished,
}

pub struct Line {
    pub a: Point2<f32>,
    pub b: Point2<f32>,
    pub color: Vector4<f32>,
}

pub trait Scenario {
    fn init(&self, sim: &mut Simulation);
    fn initial_code(&self) -> String;
    fn tick(&self, sim: &mut Simulation) -> Status;
    fn lines(&self) -> Vec<Line>;
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
        "bullet-stress" => Box::new(BulletStressScenario {}),
        "welcome" => Box::new(WelcomeScenario {}),
        "tutorial01" => Box::new(Tutorial01 {}),
        "tutorial02" => Box::new(Tutorial02 {}),
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

    fn initial_code(&self) -> String {
        "".to_string()
    }

    fn tick(&self, sim: &mut Simulation) -> Status {
        if sim.ships.iter().len() > 1 {
            Running
        } else {
            Status::Finished
        }
    }

    fn lines(&self) -> Vec<Line> {
        Vec::new()
    }
}

struct AsteroidScenario {}

impl Scenario for AsteroidScenario {
    fn init(&self, sim: &mut Simulation) {
        let mut rng = rand::thread_rng();
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter());

        let bound = (WORLD_SIZE / 2.0) * 0.9;
        for _ in 0..1000 {
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

    fn initial_code(&self) -> String {
        "".to_string()
    }

    fn tick(&self, sim: &mut Simulation) -> Status {
        if sim.ships.iter().len() > 1 {
            Running
        } else {
            Status::Finished
        }
    }

    fn lines(&self) -> Vec<Line> {
        Vec::new()
    }
}

struct BulletStressScenario {}

impl Scenario for BulletStressScenario {
    fn init(&self, sim: &mut Simulation) {
        let mut rng = rand::thread_rng();
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter());

        let bound = (WORLD_SIZE / 2.0) * 0.9;
        for _ in 0..1000 {
            let s = 1000.0;
            bullet::create(
                sim,
                rng.gen_range(-bound..bound),
                rng.gen_range(-bound..bound),
                rng.gen_range(-s..s),
                rng.gen_range(-s..s),
            );
        }
    }

    fn initial_code(&self) -> String {
        "".to_string()
    }

    fn tick(&self, _: &mut Simulation) -> Status {
        Running
    }

    fn lines(&self) -> Vec<Line> {
        Vec::new()
    }
}

struct WelcomeScenario {}

impl Scenario for WelcomeScenario {
    fn init(&self, sim: &mut Simulation) {
        let mut rng = rand::thread_rng();
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter());

        let bound = (1000.0 / 2.0) * 0.9;
        for _ in 0..100 {
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

    fn initial_code(&self) -> String {
        "\
// Welcome to Oort.
// Select a scenario from the list in the top-right of the page.
// If you're new, start with \"tutorial01\"."
            .to_string()
    }

    fn tick(&self, _: &mut Simulation) -> Status {
        Running
    }

    fn lines(&self) -> Vec<Line> {
        Vec::new()
    }
}

struct Tutorial01 {}

impl Scenario for Tutorial01 {
    fn init(&self, sim: &mut Simulation) {
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter());
        ship::create(sim, 100.0, 0.0, 0.0, 0.0, 0.1, ShipData { class: Asteroid });
    }

    fn initial_code(&self) -> String {
        "\
// Tutorial 01
// Destroy the asteroid.

fn tick() {
    // Uncomment me, then press ctrl-Enter to upload the code.
    // api.fire_weapon(0);
}"
        .to_string()
    }

    fn tick(&self, sim: &mut Simulation) -> Status {
        if sim.ships.iter().len() > 1 {
            Running
        } else {
            Status::Finished
        }
    }

    fn lines(&self) -> Vec<Line> {
        Vec::new()
    }
}

struct Tutorial02 {}

impl Scenario for Tutorial02 {
    fn init(&self, sim: &mut Simulation) {
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter());
    }

    fn initial_code(&self) -> String {
        "\
// Tutorial 01
// Fly to the target circle.

fn tick() {
    api.thrust_main(1e3);
}"
        .to_string()
    }

    fn tick(&self, _sim: &mut Simulation) -> Status {
        Running
    }

    fn lines(&self) -> Vec<Line> {
        let mut lines = vec![];
        let center: Point2<f32> = point![100.0, 0.0];
        let n = 20;
        let r = 10.0;
        for i in 0..n {
            let angle_a = std::f32::consts::TAU * (i as f32) / (n as f32);
            let angle_b = std::f32::consts::TAU * ((i + 1) as f32) / (n as f32);
            lines.push(Line {
                a: center + vector![r * angle_a.cos(), r * angle_a.sin()],
                b: center + vector![r * angle_b.cos(), r * angle_b.sin()],
                color: vector![0.87, 0.78, 0.7, 1.0],
            });
        }
        lines
    }
}
