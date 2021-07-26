use super::ship::{asteroid, fighter, ShipHandle};
use super::{
    bullet, ship, Simulation, BULLET_COLLISION_GROUP, SHIP_COLLISION_GROUP, WALL_COLLISION_GROUP,
    WORLD_SIZE,
};
use bullet::BulletData;
use nalgebra::{Point2, Translation2, Vector4};
use rand::seq::SliceRandom;
use rand::Rng;
use rapier2d_f64::prelude::*;
use Status::Running;

#[derive(PartialEq, Debug)]
pub enum Status {
    Running,
    Finished,
    Failed,
}

pub struct Line {
    pub a: Point2<f64>,
    pub b: Point2<f64>,
    pub color: Vector4<f32>,
}

fn check_tutorial_victory(sim: &Simulation) -> Status {
    let mut player_alive = false;
    let mut enemy_alive = false;
    for &handle in sim.ships.iter() {
        let team = sim.ship(handle).data().team;
        if team == 0 {
            player_alive = true;
        } else {
            enemy_alive = true;
        }
    }
    if !player_alive {
        Status::Failed
    } else if !enemy_alive {
        Status::Finished
    } else {
        Status::Running
    }
}

pub trait Scenario {
    fn name(&self) -> String;

    fn init(&mut self, sim: &mut Simulation);

    fn tick(&mut self, _: &mut Simulation) {}

    fn status(&self, _: &Simulation) -> Status {
        Running
    }

    fn lines(&self) -> Vec<Line> {
        Vec::new()
    }

    fn initial_code(&self) -> String {
        "".to_string()
    }

    fn solution(&self) -> String {
        "".to_string()
    }

    fn next_scenario(&self) -> Option<String> {
        None
    }
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
    let scenario: Box<dyn Scenario> = match name {
        "basic" => Box::new(BasicScenario {}),
        "asteroid-stress" => Box::new(AsteroidStressScenario {}),
        "bullet-stress" => Box::new(BulletStressScenario {}),
        "welcome" => Box::new(WelcomeScenario {}),
        "tutorial01" => Box::new(Tutorial01 {}),
        "tutorial02" => Box::new(Tutorial02::new()),
        "tutorial03" => Box::new(Tutorial03::new()),
        "tutorial04" => Box::new(Tutorial04::new()),
        "tutorial05" => Box::new(Tutorial05::new()),
        "tutorial06" => Box::new(Tutorial06::new()),
        "tutorial07" => Box::new(Tutorial07::new()),
        _ => panic!("Unknown scenario"),
    };
    assert_eq!(scenario.name(), name);
    scenario
}

struct BasicScenario {}

impl Scenario for BasicScenario {
    fn name(&self) -> String {
        "basic".into()
    }

    fn init(&mut self, sim: &mut Simulation) {
        add_walls(sim);
        ship::create(sim, -100.0, 0.0, 0.0, 0.0, 0.0, fighter(0));
        ship::create(sim, 100.0, 0.0, 0.0, 0.0, std::f64::consts::PI, fighter(1));
    }

    fn status(&self, sim: &Simulation) -> Status {
        if sim.ships.iter().len() > 1 {
            Running
        } else {
            Status::Finished
        }
    }
}

struct AsteroidStressScenario {}

impl Scenario for AsteroidStressScenario {
    fn name(&self) -> String {
        "asteroid-stress".into()
    }

    fn init(&mut self, sim: &mut Simulation) {
        let mut rng = rand::thread_rng();
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));

        let bound = (WORLD_SIZE / 2.0) * 0.9;
        for _ in 0..1000 {
            ship::create(
                sim,
                rng.gen_range(-bound..bound),
                rng.gen_range(-bound..bound),
                rng.gen_range(-30.0..30.0),
                rng.gen_range(-30.0..30.0),
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                asteroid(rng.gen_range(0..30)),
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        if sim.ships.iter().len() > 1 {
            Running
        } else {
            Status::Finished
        }
    }
}

struct BulletStressScenario {}

impl Scenario for BulletStressScenario {
    fn name(&self) -> String {
        "bullet-stress".into()
    }

    fn init(&mut self, sim: &mut Simulation) {
        let mut rng = rand::thread_rng();
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));

        let bound = (WORLD_SIZE / 2.0) * 0.9;
        for _ in 0..1000 {
            let s = 1000.0;
            bullet::create(
                sim,
                rng.gen_range(-bound..bound),
                rng.gen_range(-bound..bound),
                rng.gen_range(-s..s),
                rng.gen_range(-s..s),
                BulletData {
                    damage: 10.0,
                    team: 0,
                },
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        if sim.bullets.is_empty() {
            Status::Finished
        } else {
            Status::Running
        }
    }
}

struct WelcomeScenario {}

impl Scenario for WelcomeScenario {
    fn name(&self) -> String {
        "welcome".into()
    }

    fn init(&mut self, sim: &mut Simulation) {
        let mut rng = rand::thread_rng();
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));
        let asteroid_variants = [1, 6, 14];
        let bound = (1000.0 / 2.0) * 0.9;
        for _ in 0..100 {
            ship::create(
                sim,
                rng.gen_range(-bound..bound),
                rng.gen_range(-bound..bound),
                rng.gen_range(-30.0..30.0),
                rng.gen_range(-30.0..30.0),
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                asteroid(*asteroid_variants.choose(&mut rng).unwrap()),
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
}

struct Tutorial01 {}

impl Scenario for Tutorial01 {
    fn name(&self) -> String {
        "tutorial01".into()
    }

    fn init(&mut self, sim: &mut Simulation) {
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));
        ship::create(sim, 100.0, 0.0, 0.0, 0.0, 0.1, asteroid(1));
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim)
    }

    fn initial_code(&self) -> String {
        "\
// Tutorial 01
// Destroy the asteroid.

fn tick() {
    // Uncomment me, then press ctrl-Enter to upload the code.
    // ship.fire_weapon();
}"
        .to_string()
    }

    fn solution(&self) -> String {
        "\
// Tutorial 01
// Destroy the asteroid.

fn tick() {
    // Uncomment me, then press ctrl-Enter to upload the code.
    ship.fire_weapon();
}"
        .to_string()
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial02".to_string())
    }
}

struct Tutorial02 {
    on_target_ticks: i32,
}

impl Tutorial02 {
    fn new() -> Self {
        Self { on_target_ticks: 0 }
    }
}

impl Scenario for Tutorial02 {
    fn name(&self) -> String {
        "tutorial02".into()
    }

    fn init(&mut self, sim: &mut Simulation) {
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));
        if let Some(&handle) = sim.ships.iter().next() {
            let c = sim.ship_controllers.get_mut(&handle);
            c.unwrap().write_target(vector![200.0, 0.0]);
        }
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if let Some(&handle) = sim.ships.iter().next() {
            let ship = sim.ship(handle);
            if (ship.position().vector - Translation2::new(200.0, 0.0).vector).magnitude() < 50.0
                && ship.velocity().magnitude() < 5.0
            {
                self.on_target_ticks += 1;
            } else {
                self.on_target_ticks = 0;
            }
        }
    }

    fn status(&self, _: &Simulation) -> Status {
        if self.on_target_ticks > 120 {
            Status::Finished
        } else {
            Status::Running
        }
    }

    fn lines(&self) -> Vec<Line> {
        let mut lines = vec![];
        let center: Point2<f64> = point![200.0, 0.0];
        let n = 20;
        let r = 50.0;
        let on_target_frac = self.on_target_ticks as f64 / 120.0;
        for i in 0..n {
            let frac = (i as f64) / (n as f64);
            let angle_a = std::f64::consts::TAU * frac;
            let angle_b = std::f64::consts::TAU * (frac + 1.0 / n as f64);
            let color = if on_target_frac > frac {
                vector![0.0, 1.0, 0.0, 1.0]
            } else {
                vector![1.0, 0.0, 0.0, 1.0]
            };
            lines.push(Line {
                a: center + vector![r * angle_a.cos(), r * angle_a.sin()],
                b: center + vector![r * angle_b.cos(), r * angle_b.sin()],
                color,
            });
        }
        lines
    }

    fn initial_code(&self) -> String {
        r#"
// Tutorial 02
// Fly to the target circle and stop. The target is in a location
// given by the "target" variable.

fn tick() {
    ship.accelerate(vec2(100.0, 0.0));
}"#
        .trim()
        .to_string()
    }

    fn solution(&self) -> String {
        r#"
// Tutorial 02
// Fly to the target circle and stop. The target is in a location
// given by the "target" variable.

fn tick() {
    let acc = 100.0;
    let x = ship.position().x;
    let dx = target.x - x;
    let vx = ship.velocity().x;
    let margin = 10.0;
    let t = abs(vx / acc);
    let pdx = (x + vx * t + 0.5 * -acc * t*t) - target.x;
    if pdx > -margin && pdx < margin {
        ship.accelerate(vec2(-vx * 10, 0.0));
    } else if pdx < -margin {
        ship.accelerate(vec2(acc, 0.0));
    } else if pdx > margin {
        ship.accelerate(vec2(-acc, 0.0));
    }
}
"#
        .trim()
        .to_string()
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial03".to_string())
    }
}

struct Tutorial03 {
    on_target_ticks: i32,
    target: Point2<f64>,
}

impl Tutorial03 {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let size = 500.0;
        let range = -size..size;
        Self {
            on_target_ticks: 0,
            target: point![rng.gen_range(range.clone()), rng.gen_range(range)],
        }
    }
}

impl Scenario for Tutorial03 {
    fn name(&self) -> String {
        "tutorial03".into()
    }

    fn init(&mut self, sim: &mut Simulation) {
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));
        if let Some(&handle) = sim.ships.iter().next() {
            let c = sim.ship_controllers.get_mut(&handle);
            c.unwrap().write_target(self.target.coords);
        }
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if let Some(&handle) = sim.ships.iter().next() {
            let ship = sim.ship(handle);
            if (ship.position().vector - self.target.coords).magnitude() < 50.0
                && ship.velocity().magnitude() < 5.0
            {
                self.on_target_ticks += 1;
            } else {
                self.on_target_ticks = 0;
            }
        }
    }

    fn status(&self, _: &Simulation) -> Status {
        if self.on_target_ticks > 120 {
            Status::Finished
        } else {
            Status::Running
        }
    }

    fn lines(&self) -> Vec<Line> {
        let mut lines = vec![];
        let center: Point2<f64> = self.target;
        let n = 20;
        let r = 50.0;
        let on_target_frac = self.on_target_ticks as f64 / 120.0;
        for i in 0..n {
            let frac = (i as f64) / (n as f64);
            let angle_a = std::f64::consts::TAU * frac;
            let angle_b = std::f64::consts::TAU * (frac + 1.0 / n as f64);
            let color = if on_target_frac > frac {
                vector![0.0, 1.0, 0.0, 1.0]
            } else {
                vector![1.0, 0.0, 0.0, 1.0]
            };
            lines.push(Line {
                a: center + vector![r * angle_a.cos(), r * angle_a.sin()],
                b: center + vector![r * angle_b.cos(), r * angle_b.sin()],
                color,
            });
        }
        lines
    }

    fn initial_code(&self) -> String {
        r#"
// Tutorial 03
// Fly to the target circle and stop. The target is in a random
// location given by the "target" variable.

fn tick() {
    ship.accelerate(0.1 * (target - ship.position()));
}
"#
        .trim()
        .to_string()
    }

    fn solution(&self) -> String {
        r#"
// Tutorial 03
// Fly to the target circle and stop. The target is in a random
// location given by the "target" variable.

fn tick() {
    let dp = target - ship.position();
    if dp.magnitude() < 50.0 {
        ship.accelerate(ship.velocity() * -10.0);
    } else {
        if ship.velocity().magnitude() < 100.0 {
            ship.accelerate(dp.normalize() * 100.0);
        }
    }
}
"#
        .trim()
        .to_string()
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial04".to_string())
    }
}

struct Tutorial04 {
    target: Point2<f64>,
}

impl Tutorial04 {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let size = 500.0;
        let range = -size..size;
        Self {
            target: point![rng.gen_range(range.clone()), rng.gen_range(range)],
        }
    }
}

impl Scenario for Tutorial04 {
    fn name(&self) -> String {
        "tutorial04".into()
    }

    fn init(&mut self, sim: &mut Simulation) {
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));
        if let Some(&handle) = sim.ships.iter().next() {
            let c = sim.ship_controllers.get_mut(&handle);
            c.unwrap().write_target(self.target.coords);
        }
        ship::create(
            sim,
            self.target.x,
            self.target.y,
            0.0,
            0.0,
            0.0,
            asteroid(1),
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim)
    }

    fn initial_code(&self) -> String {
        r#"
// Tutorial 04
// Destroy the asteroid. The target is in a random
// location given by the "target" variable.

fn tick() {
    ship.accelerate(0.1 * (target - ship.position()));
    ship.fire_weapon();
}
"#
        .trim()
        .to_string()
    }

    fn solution(&self) -> String {
        r#"
// Tutorial 04
// Destroy the asteroid. The target is in a random
// location given by the "target" variable.

fn turn(speed) {
    let acc = 10.0;
    let margin = 0.01;
    let av = ship.angular_velocity();
    if av < speed - margin {
        ship.torque(acc);
    } else if av > speed + margin {
        ship.torque(-acc);
    }
}

fn normalize_heading(h) {
    while h < 0.0 {
        h += 2 * PI();
    }
    while h > 2 * PI() {
        h -= 2 * PI();
    }
    h
}

fn turn_to(target_heading) {
    let speed = 1.0;
    let margin = 0.1;
    let dh = (ship.heading() - target_heading) % (2 * PI());
    if dh - margin > 0.0 {
        turn(-speed);
    } else if dh + margin < 0.0 {
        turn(speed);
    } else {
         turn(-dh);
    }
}

fn tick() {
    turn_to((target - ship.position()).angle());
    ship.fire_weapon();
}
"#
        .trim()
        .to_string()
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial05".to_string())
    }
}

struct Tutorial05 {
    ship_handle: Option<ShipHandle>,
    target_handle: Option<ShipHandle>,
}

impl Tutorial05 {
    fn new() -> Self {
        Self {
            ship_handle: None,
            target_handle: None,
        }
    }
}

impl Scenario for Tutorial05 {
    fn name(&self) -> String {
        "tutorial05".into()
    }

    fn init(&mut self, sim: &mut Simulation) {
        add_walls(sim);
        self.ship_handle = Some(ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0)));

        let mut rng = rand::thread_rng();
        let size = 500.0;
        let range = -size..size;
        let target = point![rng.gen_range(range.clone()), rng.gen_range(range)];
        self.target_handle = Some(ship::create(
            sim,
            target.x,
            target.y,
            rng.gen_range(0.0..std::f64::consts::TAU),
            rng.gen_range(-400.0..400.0),
            rng.gen_range(-400.0..400.0),
            fighter(1),
        ));

        sim.upload_code(
            r#"
let target = ship.position();

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.1 * ship.angular_velocity()));
}

fn tick() {
    if (target - ship.position()).magnitude() < 50 {
        target = vec2(rng.next(200.0, 500.0), 0).rotate(rng.next(0.0, 2*PI()));
    }
    ship.accelerate((target - ship.position() - ship.velocity()).rotate(-ship.heading()));
    turn_to((target - ship.position()).angle());
}
        "#,
            1,
        );
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if sim.ships.len() < 2 {
            return;
        }
        {
            let target_position = sim.ship(self.target_handle.unwrap()).position();
            let c = sim.ship_controllers.get_mut(&self.ship_handle.unwrap());
            c.unwrap().write_target(target_position.vector);
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim)
    }

    fn initial_code(&self) -> String {
        r#"
// Tutorial 05
// Destroy the enemy ship. Its location is given by the
// "target" variable.

fn tick() {
    ship.accelerate(0.1 * (target - ship.position()));
    ship.fire_weapon();
}
"#
        .trim()
        .to_string()
    }

    fn solution(&self) -> String {
        r#"
// Tutorial 05
// Destroy the enemy ship. Its location is given by the
// "target" variable.

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.1*ship.angular_velocity()));
}

fn tick() {
    turn_to((target - ship.position()).angle());
    ship.accelerate((target - ship.position() - ship.velocity())
        .normalize().rotate(-ship.heading()) * 200.0);
    ship.fire_weapon();
}
"#
        .trim()
        .to_string()
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial06".to_string())
    }
}

struct Tutorial06 {}

impl Tutorial06 {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for Tutorial06 {
    fn name(&self) -> String {
        "tutorial06".into()
    }

    fn init(&mut self, sim: &mut Simulation) {
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));

        let mut rng = rand::thread_rng();
        let size = 500.0;
        let range = -size..size;
        for _ in 0..3 {
            let target = point![rng.gen_range(range.clone()), rng.gen_range(range.clone())];
            ship::create(
                sim,
                target.x,
                target.y,
                rng.gen_range(0.0..std::f64::consts::TAU),
                rng.gen_range(-400.0..400.0),
                rng.gen_range(-400.0..400.0),
                fighter(1),
            );
        }

        sim.upload_code(
            r#"
let target = ship.position();

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.5 * ship.angular_velocity()));
}

fn tick() {
    if (target - ship.position()).magnitude() < 50 {
        target = vec2(rng.next(200.0, 500.0), 0).rotate(rng.next(0.0, 2*PI()));
    }
    ship.accelerate((target - ship.position() - ship.velocity()).rotate(-ship.heading()));
    turn_to((target - ship.position()).angle());
}
        "#,
            1,
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim)
    }

    fn initial_code(&self) -> String {
        r#"
// Tutorial 06
// Destroy the enemy ships. Use your radar to find them.

fn tick() {
    let contact = radar.scan();
    ship.accelerate(0.1 * (contact.position - ship.position()));
    ship.fire_weapon();
}
"#
        .trim()
        .to_string()
    }

    fn solution(&self) -> String {
        r#"
// Tutorial 06
// Destroy the enemy ships. Use your radar to find them.

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.1*ship.angular_velocity()));
}

fn tick() {
    let contact = radar.scan();
    turn_to((contact.position - ship.position()).angle());
    ship.accelerate((contact.position - ship.position() - ship.velocity())
        .normalize().rotate(-ship.heading()) * 200.0);
    ship.fire_weapon();
}
"#
        .trim()
        .to_string()
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial07".to_string())
    }
}

struct Tutorial07 {}

impl Tutorial07 {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for Tutorial07 {
    fn name(&self) -> String {
        "tutorial07".into()
    }

    fn init(&mut self, sim: &mut Simulation) {
        add_walls(sim);
        for team in 0..2 {
            for _ in 0..10 {
                let mut rng = rand::thread_rng();
                let size = 500.0;
                let range = -size..size;
                let center = point![(team as f64 - 0.5) * 1000.0, 0.0];
                let offset = point![rng.gen_range(range.clone()), rng.gen_range(range.clone())];
                ship::create(
                    sim,
                    center.x + offset.x,
                    center.y + offset.y,
                    rng.gen_range(0.0..std::f64::consts::TAU),
                    0.0,
                    0.0,
                    fighter(team),
                );
            }
        }

        sim.upload_code(
            r#"
let target = ship.position();

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.5 * ship.angular_velocity()));
}

fn tick() {
    if (target - ship.position()).magnitude() < 50 {
        target = vec2(rng.next(200.0, 500.0), 0).rotate(rng.next(0.0, 2*PI()));
    }
    ship.accelerate((target - ship.position() - ship.velocity()).rotate(-ship.heading()));
    turn_to((target - ship.position()).angle());
    ship.fire_weapon();
}
        "#,
            1,
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim)
    }

    fn initial_code(&self) -> String {
        r#"
// tutorial07
// Destroy the enemy ships.

fn tick() {
    let contact = ship.scan();
    ship.accelerate(0.1 * (contact.position - ship.position()));
    ship.fire_weapon();
}
"#
        .trim()
        .to_string()
    }

    fn solution(&self) -> String {
        r#"
// tutorial07
// Destroy the enemy ships.

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.1*ship.angular_velocity()));
}

fn tick() {
    let contact = radar.scan();
    turn_to((contact.position - ship.position()).angle());
    ship.accelerate((contact.position - ship.position() - ship.velocity())
        .normalize().rotate(-ship.heading()) * 100.0);
    ship.fire_weapon();
}
"#
        .trim()
        .to_string()
    }
}
