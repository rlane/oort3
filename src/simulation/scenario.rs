use super::rng::{new_rng, SeededRng};
use super::ship::{asteroid, fighter, target, ShipHandle};
use super::{bullet, collision, ship, Line, Simulation, WORLD_SIZE};
use bullet::BulletData;
use nalgebra::{Point2, Rotation2, Translation2};
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

    fn init(&mut self, sim: &mut Simulation, seed: u64);

    fn tick(&mut self, _: &mut Simulation) {}

    fn status(&self, _: &Simulation) -> Status {
        Running
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

    fn lines(&self) -> Vec<Line> {
        vec![]
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
            .collision_groups(collision::wall_interaction_groups())
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
        "gunnery" => Box::new(GunneryScenario {}),
        "asteroid-stress" => Box::new(AsteroidStressScenario {}),
        "bullet-stress" => Box::new(BulletStressScenario {}),
        "welcome" => Box::new(WelcomeScenario::new()),
        "tutorial01" => Box::new(Tutorial01 {}),
        "tutorial02" => Box::new(Tutorial02::new()),
        "tutorial03" => Box::new(Tutorial03::new()),
        "tutorial04" => Box::new(Tutorial04::new()),
        "tutorial05" => Box::new(Tutorial05::new()),
        "tutorial06" => Box::new(Tutorial06::new()),
        "tutorial07" => Box::new(Tutorial07::new()),
        "tutorial08" => Box::new(Tutorial08::new()),
        "tutorial09" => Box::new(Tutorial09::new()),
        _ => panic!("Unknown scenario"),
    };
    assert_eq!(scenario.name(), name);
    scenario
}

pub fn list() -> Vec<String> {
    vec![
        "welcome",
        "tutorial01",
        "tutorial02",
        "tutorial03",
        "tutorial04",
        "tutorial05",
        "tutorial06",
        "tutorial07",
        "tutorial08",
        "tutorial09",
        "gunnery",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect()
}

struct BasicScenario {}

impl Scenario for BasicScenario {
    fn name(&self) -> String {
        "basic".into()
    }

    fn init(&mut self, sim: &mut Simulation, _seed: u64) {
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

struct GunneryScenario {}

impl Scenario for GunneryScenario {
    fn name(&self) -> String {
        "gunnery".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u64) {
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));
        let mut rng = new_rng(seed);
        for _ in 0..4 {
            ship::create(
                sim,
                2000.0 + rng.gen_range(-500.0..500.0),
                -2000.0 + rng.gen_range(-500.0..500.0),
                0.0 + rng.gen_range(-10.0..10.0),
                700.0 + rng.gen_range(-300.0..300.0),
                std::f64::consts::PI,
                target(1),
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

    fn solution(&self) -> String {
        r#"
fn turn_to(target_heading, target_angular_velocity) {
    let acc = 2 * PI();
    let dh = angle_diff(ship.heading(), target_heading);
    let vh = ship.angular_velocity() - target_angular_velocity;
    let margin = 0.001;
    let t = abs(vh / acc);
    let pdh = vh * t + 0.5 * -acc * t*t - dh;
    if pdh < 0 {
        ship.torque(acc);
    } else if pdh > 0 {
        ship.torque(-acc);
    }
}

let last_target_heading = 0.0;

fn tick() {
    let contact = radar.scan();
    if (contact.found) {
        let dp = contact.position - ship.position();
        let dv = contact.velocity - ship.velocity();
        let bullet_speed = 1000.0;
        let bullet_offset = 20.0;
        let predicted_dp = dp;
        for i in range(0, 4) {
            let dist = predicted_dp.magnitude() - bullet_offset;
            let t = dist / bullet_speed;
            predicted_dp = dp + t * dv;
        }
        let target_heading = predicted_dp.angle();
        let target_angular_velocity = (target_heading - last_target_heading) * 60.0;
        turn_to(target_heading, target_angular_velocity);
        if vec2(predicted_dp.magnitude(), 0).rotate(ship.heading()).distance(predicted_dp) <= 5 {
            ship.fire_weapon();
        }
        last_target_heading = target_heading;
    }
}
    "#
        .trim()
        .to_string()
    }
}

struct AsteroidStressScenario {}

impl Scenario for AsteroidStressScenario {
    fn name(&self) -> String {
        "asteroid-stress".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u64) {
        let mut rng = new_rng(seed);
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

    fn init(&mut self, sim: &mut Simulation, seed: u64) {
        let mut rng = new_rng(seed);
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

struct WelcomeScenario {
    rng: Option<SeededRng>,
}

impl WelcomeScenario {
    fn new() -> Self {
        Self {
            rng: None as Option<SeededRng>,
        }
    }
}

impl Scenario for WelcomeScenario {
    fn name(&self) -> String {
        "welcome".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u64) {
        self.rng = Some(new_rng(seed));
        add_walls(sim);
        sim.upload_code(
            0,
            r#"
fn tick() {
    if ship.class() == "missile" {
        missile_tick();
    } else {
        ship_tick();
    }
}

let initial_position = ship.position();
let target_position = initial_position;
let target_velocity = vec2(0.0, 0.0);

fn ship_tick() {
    let contact = radar.scan();
    if (contact.found) {
        target_position = contact.position;
        target_velocity = contact.velocity;
        ship.fire_weapon();
        ship.launch_missile();
    } else {
        if (target_position - ship.position()).magnitude() < 100 {
            target_position = vec2(rng.next(3500.0, 4500.0), 0).rotate(rng.next(0.0, 2*PI()));
            target_velocity = vec2(0.0, 0.0);
        }
    }
    let dp = target_position - ship.position();
    let dist = dp.magnitude();
    let bullet_speed = 1000.0;
    let t = dist / bullet_speed;
    let predicted_dp = dp + t * (target_velocity - ship.velocity());
    turn_to(predicted_dp.angle(), 0.0);

    if contact.found && dist < 1000 {
        ship.accelerate(-ship.velocity().rotate(-ship.heading()));
    } else {
        ship.accelerate((dp - ship.velocity()).rotate(-ship.heading()));
    }
}

fn turn_to(target_heading, target_angular_velocity) {
    let acc = 2 * PI();
    let dh = angle_diff(ship.heading(), target_heading);
    let vh = ship.angular_velocity() - target_angular_velocity;
    let margin = 0.001;
    let t = abs(vh / acc);
    let pdh = vh * t + 0.5 * -acc * t*t - dh;
    if pdh < 0 {
        ship.torque(acc);
    } else if pdh > 0 {
        ship.torque(-acc);
    }
}

fn missile_tick() {
    let acc = 400;

    let contact = radar.scan();
    if (!contact.found) {
        ship.explode();
        return;
    }

    let dp = contact.position - ship.position();
    let dv = contact.velocity - ship.velocity();

    let dist = dp.magnitude();
    let next_dist = (dp + dv / 60).magnitude();
    if next_dist < 30 || dist < 100 && next_dist > dist {
        ship.explode();
        return;
    }

    let badv = -(dv - dot(dv, dp) * dp.normalize() / dp.magnitude());
    let a = (dp - badv * 10).rotate(-ship.heading()).normalize() * acc;
    ship.accelerate(a);
    turn_to(a.rotate(ship.heading()).angle(), 0);

    dbg.draw_diamond(contact.position, 20.0, 0xffff00);
    dbg.draw_diamond(ship.position() + dp, 5.0, 0xffffff);
    dbg.draw_line(ship.position(), ship.position() + dp, 0x222222);
    dbg.draw_line(ship.position(), ship.position() - dv, 0xffffff);
    dbg.draw_line(ship.position(), ship.position() + badv, 0x222299);
}
            "#,
        );
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));
    }

    fn tick(&mut self, sim: &mut Simulation) {
        let rng = self.rng.as_mut().unwrap();
        let asteroid_variants = [1, 6, 14];
        while sim.ships.len() < 20 {
            let p = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                .transform_point(&point![rng.gen_range(500.0..2000.0), 0.0]);
            ship::create(
                sim,
                p.x,
                p.y,
                rng.gen_range(-30.0..30.0),
                rng.gen_range(-30.0..30.0),
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                asteroid(*asteroid_variants.choose(rng).unwrap()),
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

    fn init(&mut self, sim: &mut Simulation, _seed: u64) {
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

    fn init(&mut self, sim: &mut Simulation, _seed: u64) {
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

    fn status(&self, _: &Simulation) -> Status {
        if self.on_target_ticks > 120 {
            Status::Finished
        } else {
            Status::Running
        }
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
    target: Option<Point2<f64>>,
}

impl Tutorial03 {
    fn new() -> Self {
        Self {
            on_target_ticks: 0,
            target: None,
        }
    }
}

impl Scenario for Tutorial03 {
    fn name(&self) -> String {
        "tutorial03".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u64) {
        let mut rng = new_rng(seed);
        let size = 500.0;
        let range = -size..size;
        self.target = Some(point![rng.gen_range(range.clone()), rng.gen_range(range)]);
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));
        if let Some(&handle) = sim.ships.iter().next() {
            let c = sim.ship_controllers.get_mut(&handle);
            c.unwrap().write_target(self.target.unwrap().coords);
        }
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if let Some(&handle) = sim.ships.iter().next() {
            let ship = sim.ship(handle);
            if (ship.position().vector - self.target.unwrap().coords).magnitude() < 50.0
                && ship.velocity().magnitude() < 5.0
            {
                self.on_target_ticks += 1;
            } else {
                self.on_target_ticks = 0;
            }
        }
    }

    fn lines(&self) -> Vec<Line> {
        let mut lines = vec![];
        let center: Point2<f64> = self.target.unwrap();
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

    fn status(&self, _: &Simulation) -> Status {
        if self.on_target_ticks > 120 {
            Status::Finished
        } else {
            Status::Running
        }
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

struct Tutorial04 {}

impl Tutorial04 {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for Tutorial04 {
    fn name(&self) -> String {
        "tutorial04".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u64) {
        add_walls(sim);
        let mut rng = new_rng(seed);
        let size = 500.0;
        let range = -size..size;
        let target = point![rng.gen_range(range.clone()), rng.gen_range(range)];
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));
        if let Some(&handle) = sim.ships.iter().next() {
            let c = sim.ship_controllers.get_mut(&handle);
            c.unwrap().write_target(target.coords);
        }
        ship::create(sim, target.x, target.y, 0.0, 0.0, 0.0, asteroid(1));
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

    fn init(&mut self, sim: &mut Simulation, seed: u64) {
        add_walls(sim);
        self.ship_handle = Some(ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0)));

        let mut rng = new_rng(seed);
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
            1,
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

    fn init(&mut self, sim: &mut Simulation, seed: u64) {
        add_walls(sim);
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, fighter(0));

        let mut rng = new_rng(seed);
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
            1,
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

    fn init(&mut self, sim: &mut Simulation, seed: u64) {
        add_walls(sim);
        let mut rng = new_rng(seed);
        for team in 0..2 {
            for _ in 0..10 {
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
            1,
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

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial08".to_string())
    }
}

struct Tutorial08 {}

impl Tutorial08 {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for Tutorial08 {
    fn name(&self) -> String {
        "tutorial08".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u64) {
        add_walls(sim);

        sim.upload_code(
            1,
            r#"
let initial_position = ship.position();
let target = initial_position;

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.5 * ship.angular_velocity()));
}

fn tick() {
    if (target - ship.position()).magnitude() < 50 {
        target = initial_position + vec2(rng.next(0.0, 200.0), 0).rotate(rng.next(0.0, 2*PI()));
    }
    ship.accelerate((target - ship.position() - ship.velocity()).rotate(-ship.heading()));
    let contact = radar.scan();
    if (contact.position.distance(ship.position()) < 1000.0) {
        turn_to((contact.position - ship.position()).angle());
        ship.fire_weapon();
    } else {
        turn_to((target - ship.position()).angle());
    }
}
        "#,
        );

        let mut rng = new_rng(seed);
        {
            for _ in 0..3 {
                let position = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                    .transform_point(&point![rng.gen_range(100.0..500.0), 0.0]);
                ship::create(
                    sim,
                    position.x,
                    position.y,
                    0.0,
                    0.0,
                    rng.gen_range(0.0..std::f64::consts::TAU),
                    fighter(0),
                );
            }
        }
        {
            for _ in 0..3 {
                let position = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                    .transform_point(&point![rng.gen_range(3500.0..4500.0), 0.0]);
                ship::create(
                    sim,
                    position.x,
                    position.y,
                    0.0,
                    0.0,
                    rng.gen_range(0.0..std::f64::consts::TAU),
                    fighter(1),
                );
            }
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim)
    }

    fn initial_code(&self) -> String {
        r#"
// tutorial08
// Destroy the enemy ships. They are initially outside of your radar range.

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
// tutorial08
// Destroy the enemy ships. They are initially outside of your radar range.

let initial_position = ship.position();
let target_position = initial_position;
let target_velocity = vec2(0.0, 0.0);
let last_target_heading = 0.0;

fn turn_to(target_heading, target_angular_velocity) {
    let acc = 2 * PI();
    let dh = angle_diff(ship.heading(), target_heading);
    let vh = ship.angular_velocity() - target_angular_velocity;
    let margin = 0.001;
    let t = abs(vh / acc);
    let pdh = vh * t + 0.5 * -acc * t*t - dh;
    if pdh < 0 {
        ship.torque(acc);
    } else if pdh > 0 {
        ship.torque(-acc);
    }
}

fn tick() {
    let contact = radar.scan();
    if (contact.found) {
        target_position = contact.position;
        target_velocity = contact.velocity;
    } else {
        if (target_position - ship.position()).magnitude() < 100 {
            target_position = vec2(rng.next(3500.0, 4500.0), 0).rotate(rng.next(0.0, 2*PI()));
            target_velocity = vec2(0.0, 0.0);
        }
    }

    let dp = target_position - ship.position();
    let dv = target_velocity - ship.velocity();
    let bullet_speed = 1000.0;
    let bullet_offset = 20.0;
    let predicted_dp = dp;
    for i in range(0, 4) {
        let dist = predicted_dp.magnitude() - bullet_offset;
        let t = dist / bullet_speed;
        predicted_dp = dp + t * dv;
    }
    let target_heading = predicted_dp.angle();
    let target_angular_velocity = (target_heading - last_target_heading) * 60.0;
    turn_to(target_heading, target_angular_velocity);
    if contact.found && vec2(predicted_dp.magnitude(), 0).rotate(ship.heading()).distance(predicted_dp) <= 10 {
        ship.fire_weapon();
    }
    last_target_heading = target_heading;
    ship.accelerate((target_position - ship.position() - ship.velocity())
        .normalize().rotate(-ship.heading()) * 100.0);
}
"#
        .trim()
        .to_string()
    }
}

struct Tutorial09 {}

impl Tutorial09 {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for Tutorial09 {
    fn name(&self) -> String {
        "tutorial09".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u64) {
        add_walls(sim);

        sim.upload_code(
            1,
            r#"
let initial_position = ship.position();
let target = initial_position;

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.5 * ship.angular_velocity()));
}

fn tick() {
    if (target - ship.position()).magnitude() < 50 {
        target = initial_position + vec2(rng.next(0.0, 1000.0), 0).rotate(rng.next(0.0, 2*PI()));
    }
    ship.accelerate((target - ship.position() - ship.velocity()).rotate(-ship.heading()));
    let contact = radar.scan();
    if (contact.position.distance(ship.position()) < 1000.0 && contact.velocity.magnitude() > 100) {
        turn_to((contact.position - ship.position()).angle());
        ship.fire_weapon();
    } else {
        turn_to((target - ship.position()).angle());
    }
}
        "#,
        );

        let mut shipdata = fighter(0);
        shipdata.weapons.clear();
        ship::create(sim, 0.0, 0.0, 0.0, 0.0, 0.0, shipdata);

        let mut rng = new_rng(seed);
        for _ in 0..3 {
            let p = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                .transform_vector(&vector![rng.gen_range(1000.0..1500.0), 0.0]);
            let v = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                .transform_vector(&vector![rng.gen_range(0.0..300.0), 0.0]);
            ship::create(sim, p.x, p.y, v.x, v.y, std::f64::consts::PI, fighter(1));
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim)
    }

    fn initial_code(&self) -> String {
        r#"
// tutorial09
// Destroy the enemy ships with your missiles.

fn tick() {
    let contact = ship.scan();
    if contact.found {
        if ship.class() == "missile" {
            let dp = contact.position - ship.position();
            ship.torque(20 * (angle_diff(ship.heading(), dp.angle())
                - 0.1 * ship.angular_velocity()));
            ship.accelerate(dp.rotate(-ship.heading()));
            if dp.magnitude() < 100 {
                ship.explode();
            }
        } else {
            ship.launch_missile();
        }
    }
}
"#
        .trim()
        .to_string()
    }

    fn solution(&self) -> String {
        r#"
// tutorial09
// Destroy the enemy ships with your missiles.

fn tick() {
    if ship.class() == "missile" {
        missile_tick();
    } else {
        let contact = radar.scan();
        if contact.found {
            turn_to((contact.position - ship.position()).angle(), 0.0);
            ship.launch_missile();
        }
    }
}

fn turn_to(target_heading, target_angular_velocity) {
    let acc = 4 * PI();
    let dh = angle_diff(ship.heading(), target_heading);
    let vh = ship.angular_velocity() - target_angular_velocity;
    let margin = 0.001;
    let t = abs(vh / acc);
    let pdh = vh * t + 0.5 * -acc * t*t - dh;
    if pdh < 0 {
        ship.torque(acc);
    } else if pdh > 0 {
        ship.torque(-acc);
    }
}

fn missile_tick() {
    let acc = 400;

    let contact = radar.scan();
    if (!contact.found) {
        return;
    }

    let dp = contact.position - ship.position();
    let dv = contact.velocity - ship.velocity();

    let dist = dp.magnitude();
    let next_dist = (dp + dv / 60).magnitude();
    if next_dist < 10 || dist < 100 && next_dist > dist {
        ship.explode();
        return;
    }

    let dp = dp.rotate(dp.magnitude() / 1e4);  // Evade guns.
    let badv = -(dv - dot(dv, dp) * dp.normalize() / dp.magnitude());
    let a = (dp - badv * 10).rotate(-ship.heading()).normalize() * acc;
    ship.accelerate(a);
    turn_to(a.rotate(ship.heading()).angle(), 0);

    dbg.draw_diamond(contact.position, 20.0, 0xffff00);
    dbg.draw_diamond(ship.position() + dp, 5.0, 0xffffff);
    dbg.draw_line(ship.position(), ship.position() + dp, 0x222222);
    dbg.draw_line(ship.position(), ship.position() - dv, 0xffffff);
    dbg.draw_line(ship.position(), ship.position() + badv, 0x222299);
}
"#
        .trim()
        .to_string()
    }
}
