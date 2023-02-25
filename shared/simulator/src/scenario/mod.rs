mod stress;
mod test;
mod tutorial01_guns;
mod tutorial02_acceleration;
mod tutorial03_acceleration2;
mod tutorial04_rotation;
mod tutorial05_deflection;
mod tutorial06_radar;
mod tutorial07_squadron;
mod tutorial08_search;
mod tutorial09_missiles;
mod tutorial10_frigate;
mod tutorial11_cruiser;
mod welcome;

use crate::rng::{new_rng, SeededRng};
use crate::ship::{
    asteroid, cruiser, fighter, frigate, missile, target, torpedo, ShipAccessor, ShipClass,
    ShipData, ShipHandle,
};
use crate::simulation::{Code, Line, Simulation, PHYSICS_TICK_LENGTH, WORLD_SIZE};
use crate::{collision, ship};
use nalgebra::{vector, Rotation2};
use rand::Rng;
use rapier2d_f64::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::TAU;

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Status {
    Running,
    Victory { team: i32 },
    Failed,
    Draw,
}

pub mod prelude {
    pub use super::Scenario;
    pub use super::Status;
    pub use super::{
        add_walls, fighter_without_guns, fighter_without_missiles,
        fighter_without_missiles_or_radar, target_asteroid,
    };
    pub use super::{builtin, empty_ai, reference_ai};
    pub use super::{check_tournament_victory, check_tutorial_victory};
    pub use super::{DEFAULT_TUTORIAL_MAX_TICKS, TOURNAMENT_MAX_TICKS};
    pub use crate::rng::{new_rng, SeededRng};
    pub use crate::ship::{
        self, asteroid, cruiser, fighter, frigate, missile, target, torpedo, ShipHandle,
    };
    pub use crate::simulation::{Code, Line, Simulation, WORLD_SIZE};
    pub use nalgebra::{point, vector, Point2, Rotation2, Vector2};
    pub use rand::Rng;
    pub use std::f64::consts::{PI, TAU};
}

pub const DEFAULT_TUTORIAL_MAX_TICKS: u32 = 30 * 60;
pub const TOURNAMENT_MAX_TICKS: u32 = 10000;
pub const MAX_TICKS: u32 = 10000;

pub fn builtin(name: &str) -> Code {
    Code::Builtin(name.to_string())
}

pub fn reference_ai() -> Code {
    builtin("reference")
}

pub fn empty_ai() -> Code {
    builtin("empty")
}

pub fn check_victory_with_filter(
    sim: &Simulation,
    max_ticks: u32,
    ship_filter: fn(&ShipAccessor) -> bool,
) -> Status {
    let mut team_health: HashMap<i32, u32> = HashMap::new();
    for &handle in sim.ships.iter() {
        let ship = sim.ship(handle);
        if ship_filter(&ship) {
            *team_health.entry(ship.data().team).or_insert(0) += ship.data().health as u32;
        }
    }
    if team_health.is_empty() {
        Status::Draw
    } else if team_health.len() == 1 {
        Status::Victory {
            team: *team_health.iter().next().unwrap().0,
        }
    } else if sim.tick() >= max_ticks - 1 {
        Status::Draw
    } else {
        Status::Running
    }
}

pub fn check_tutorial_victory(sim: &Simulation, max_ticks: u32) -> Status {
    match check_victory_with_filter(sim, max_ticks, |ship| {
        ![ShipClass::Missile, ShipClass::Torpedo].contains(&ship.data().class)
    }) {
        x @ Status::Victory { team: 0 } => x,
        Status::Victory { .. } => Status::Failed,
        x => x,
    }
}

pub fn check_tournament_victory(sim: &Simulation) -> Status {
    check_victory_with_filter(sim, TOURNAMENT_MAX_TICKS, |ship| {
        [ShipClass::Fighter, ShipClass::Frigate, ShipClass::Cruiser].contains(&ship.data().class)
            && ship.data().team < 2
    })
}

pub fn check_capital_ship_tournament_victory(sim: &Simulation) -> Status {
    check_victory_with_filter(sim, TOURNAMENT_MAX_TICKS, |ship| {
        [ShipClass::Frigate, ShipClass::Cruiser].contains(&ship.data().class)
            && ship.data().team < 2
    })
}

pub fn fighter_without_missiles(team: i32) -> ShipData {
    let mut data = fighter(team);
    data.missile_launchers.pop();
    data
}

pub fn fighter_without_missiles_or_radar(team: i32) -> ShipData {
    let mut data = fighter(team);
    data.missile_launchers.pop();
    data.radar = None;
    data
}

pub fn fighter_without_guns(team: i32) -> ShipData {
    let mut data = fighter(team);
    data.guns.pop();
    data
}

pub fn target_asteroid(variant: i32) -> ShipData {
    let mut asteroid = asteroid(variant);
    asteroid.team = 1;
    asteroid
}

pub trait Scenario {
    fn name(&self) -> String;

    fn human_name(&self) -> String {
        self.name()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32);

    fn tick(&mut self, _: &mut Simulation) {}

    fn status(&self, _: &Simulation) -> Status {
        Status::Running
    }

    // Indexed by team ID.
    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai()]
    }

    fn solution(&self) -> Code {
        Code::None
    }

    fn solution_codes(&self) -> Vec<Code> {
        let mut codes = self.initial_code();
        codes[0] = self.solution();
        codes
    }

    fn next_scenario(&self) -> Option<String> {
        None
    }

    fn lines(&self) -> Vec<Line> {
        vec![]
    }

    fn is_tournament(&self) -> bool {
        false
    }

    fn score_time(&self, sim: &Simulation) -> f64 {
        sim.time()
    }
}

pub fn add_walls(sim: &mut Simulation) {
    let mut make_edge = |x: f64, y: f64, a: f64| {
        let edge_length = WORLD_SIZE;
        let edge_width = 10.0;
        let rigid_body = RigidBodyBuilder::fixed()
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

pub fn load_safe(name: &str) -> Option<Box<dyn Scenario>> {
    let scenario: Option<Box<dyn Scenario>> = match name {
        // Tutorials
        "tutorial01" => Some(Box::new(tutorial01_guns::Tutorial01 {})),
        "tutorial02" => Some(Box::new(tutorial02_acceleration::Tutorial02::new())),
        "tutorial03" => Some(Box::new(tutorial03_acceleration2::Tutorial03::new())),
        "tutorial04" => Some(Box::new(tutorial04_rotation::Tutorial04::new())),
        "tutorial05" => Some(Box::new(tutorial05_deflection::Tutorial05::new())),
        "tutorial06" => Some(Box::new(tutorial06_radar::Tutorial06::new())),
        "tutorial07" => Some(Box::new(tutorial07_squadron::Tutorial07::new())),
        "tutorial08" => Some(Box::new(tutorial08_search::Tutorial08::new())),
        "tutorial09" => Some(Box::new(tutorial09_missiles::Tutorial09::new())),
        "tutorial10" => Some(Box::new(tutorial10_frigate::Tutorial10::new())),
        "tutorial11" => Some(Box::new(tutorial11_cruiser::Tutorial11::new())),
        // Tournament
        "primitive_duel" => Some(Box::new(PrimitiveDuel::new())),
        "fighter_duel" => Some(Box::new(FighterDuel::new())),
        "missile_duel" => Some(Box::new(MissileDuel::new())),
        "frigate_duel" => Some(Box::new(FrigateDuel::new())),
        "cruiser_duel" => Some(Box::new(CruiserDuel::new())),
        "asteroid_duel" => Some(Box::new(AsteroidDuel::new())),
        "furball" => Some(Box::new(Furball::new())),
        "fleet" => Some(Box::new(Fleet::new())),
        "belt" => Some(Box::new(Belt::new())),
        // Challenge
        "gunnery" => Some(Box::new(GunneryScenario {})),
        "planetary_defense" => Some(Box::new(PlanetaryDefense::new())),
        // Testing
        "test" => Some(Box::new(test::TestScenario {})),
        "basic" => Some(Box::new(test::BasicScenario {})),
        "missile_test" => Some(Box::new(test::MissileTest::new())),
        "frigate_vs_cruiser" => Some(Box::new(test::FrigateVsCruiser::new())),
        "cruiser_vs_frigate" => Some(Box::new(test::CruiserVsFrigate::new())),
        "frigate_point_defense" => Some(Box::new(test::FrigatePointDefense {})),
        // Stress
        "stress" => Some(Box::new(stress::StressScenario {})),
        "asteroid-stress" => Some(Box::new(stress::AsteroidStressScenario {})),
        "bullet-stress" => Some(Box::new(stress::BulletStressScenario {})),
        "missile-stress" => Some(Box::new(stress::MissileStressScenario {})),
        // Miscellaneous
        "welcome" => Some(Box::new(welcome::Welcome::new())),
        _ => None,
    };
    if let Some(scenario) = scenario.as_ref() {
        assert_eq!(scenario.name(), name);
    }
    scenario
}

pub fn load(name: &str) -> Box<dyn Scenario> {
    match load_safe(name) {
        Some(scenario) => scenario,
        None => panic!("Unknown scenario"),
    }
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
        "tutorial10",
        "tutorial11",
        "gunnery",
        "fighter_duel",
        "missile_duel",
        "frigate_duel",
        "cruiser_duel",
        "asteroid_duel",
        "furball",
        "fleet",
        "belt",
        "planetary_defense",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect()
}

struct GunneryScenario {}

impl Scenario for GunneryScenario {
    fn name(&self) -> String {
        "gunnery".into()
    }

    fn human_name(&self) -> String {
        "Gunnery".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);
        let mut ship_data = frigate(0);
        ship_data.guns.pop();
        ship_data.guns.pop();
        ship_data.missile_launchers.pop();
        ship_data.acceleration = vector![0.0, 0.0];
        ship::create(
            sim,
            vector![-9000.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship_data,
        );
        let mut rng = new_rng(seed);
        for _ in 0..4 {
            ship::create(
                sim,
                vector![
                    9000.0 + rng.gen_range(-500.0..500.0),
                    -9000.0 + rng.gen_range(-500.0..500.0)
                ],
                vector![
                    0.0 + rng.gen_range(-10.0..10.0),
                    700.0 + rng.gen_range(-300.0..600.0)
                ],
                std::f64::consts::PI,
                target(1),
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS * 2)
    }

    fn solution(&self) -> Code {
        builtin("gunnery")
    }
}

struct PrimitiveDuel {
    ship0: Option<ShipHandle>,
    ship1: Option<ShipHandle>,
}

impl PrimitiveDuel {
    fn new() -> Self {
        Self {
            ship0: None,
            ship1: None,
        }
    }
}

impl Scenario for PrimitiveDuel {
    fn name(&self) -> String {
        "primitive_duel".into()
    }

    fn human_name(&self) -> String {
        "Primitive Duel".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);

        let mut rng = new_rng(seed);
        let angle = rng.gen_range(0.0..TAU);
        let rot = Rotation2::new(angle);
        let distance = rng.gen_range(2000.0..4000.0);

        self.ship0 = Some(ship::create(
            sim,
            rot.transform_vector(&vector![-0.5, 0.0]) * distance,
            vector![0.0, 0.0],
            0.0,
            fighter_without_missiles_or_radar(0),
        ));
        self.ship1 = Some(ship::create(
            sim,
            rot.transform_vector(&vector![0.5, 0.0]) * distance,
            vector![0.0, 0.0],
            std::f64::consts::PI,
            fighter_without_missiles_or_radar(1),
        ));
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if !sim.ships.contains(self.ship0.unwrap()) || !sim.ships.contains(self.ship1.unwrap()) {
            return;
        }

        let ship0_position = sim.ship(self.ship0.unwrap()).position().vector;
        let ship0_velocity = sim.ship(self.ship0.unwrap()).velocity();
        let ship1_position = sim.ship(self.ship1.unwrap()).position().vector;
        let ship1_velocity = sim.ship(self.ship1.unwrap()).velocity();

        sim.write_target(self.ship0.unwrap(), ship1_position, ship1_velocity);
        sim.write_target(self.ship1.unwrap(), ship0_position, ship0_velocity);
    }
}

struct FighterDuel {}

impl FighterDuel {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for FighterDuel {
    fn name(&self) -> String {
        "fighter_duel".into()
    }

    fn human_name(&self) -> String {
        "Fighter Duel".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);

        let mut rng = new_rng(seed);
        let angle = rng.gen_range(0.0..TAU);
        let rot = Rotation2::new(angle);
        let distance = rng.gen_range(2000.0..4000.0);

        ship::create(
            sim,
            rot.transform_vector(&vector![-0.5, 0.0]) * distance,
            vector![0.0, 0.0],
            0.0,
            fighter(0),
        );
        ship::create(
            sim,
            rot.transform_vector(&vector![0.5, 0.0]) * distance,
            vector![0.0, 0.0],
            std::f64::consts::PI,
            fighter(1),
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }
}

struct MissileDuel {}

impl MissileDuel {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for MissileDuel {
    fn name(&self) -> String {
        "missile_duel".into()
    }

    fn human_name(&self) -> String {
        "Missile Duel".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);

        let mut rng = new_rng(seed);
        let angle = rng.gen_range(0.0..TAU);
        let rot = Rotation2::new(angle);
        let distance = rng.gen_range(4000.0..6000.0);

        ship::create(
            sim,
            rot.transform_vector(&vector![-0.5, 0.0]) * distance,
            vector![0.0, 0.0],
            0.0,
            fighter_without_guns(0),
        );
        ship::create(
            sim,
            rot.transform_vector(&vector![0.5, 0.0]) * distance,
            vector![0.0, 0.0],
            std::f64::consts::PI,
            fighter_without_guns(1),
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }
}

struct FrigateDuel {}

impl FrigateDuel {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for FrigateDuel {
    fn name(&self) -> String {
        "frigate_duel".into()
    }

    fn human_name(&self) -> String {
        "Frigate Duel".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);

        let mut rng = new_rng(seed);
        let angle = rng.gen_range(0.0..TAU);
        let rot = Rotation2::new(angle);
        let distance = rng.gen_range(4000.0..8000.0);

        ship::create(
            sim,
            rot.transform_vector(&vector![-0.5, 0.0]) * distance,
            vector![0.0, 0.0],
            0.0,
            frigate(0),
        );
        ship::create(
            sim,
            rot.transform_vector(&vector![0.5, 0.0]) * distance,
            vector![0.0, 0.0],
            std::f64::consts::PI,
            frigate(1),
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }
}

struct CruiserDuel {}

impl CruiserDuel {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for CruiserDuel {
    fn name(&self) -> String {
        "cruiser_duel".into()
    }

    fn human_name(&self) -> String {
        "Cruiser Duel".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);

        let mut rng = new_rng(seed);
        let angle = rng.gen_range(0.0..TAU);
        let rot = Rotation2::new(angle);
        let distance = rng.gen_range(5000.0..10000.0);

        ship::create(
            sim,
            rot.transform_vector(&vector![-0.5, 0.0]) * distance,
            vector![0.0, 0.0],
            0.0,
            cruiser(0),
        );
        ship::create(
            sim,
            rot.transform_vector(&vector![0.5, 0.0]) * distance,
            vector![0.0, 0.0],
            std::f64::consts::PI,
            cruiser(1),
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }
}

struct AsteroidDuel {}

impl AsteroidDuel {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for AsteroidDuel {
    fn name(&self) -> String {
        "asteroid_duel".into()
    }

    fn human_name(&self) -> String {
        "Asteroid Duel".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);

        let mut rng = new_rng(seed);
        let bound = vector![(WORLD_SIZE / 2.0) * 0.9, (WORLD_SIZE / 2.0) * 0.9];

        ship::create(
            sim,
            vector![
                rng.gen_range(-bound.x..bound.x),
                rng.gen_range(-bound.y..bound.y)
            ],
            vector![0.0, 0.0],
            0.0,
            frigate(0),
        );
        ship::create(
            sim,
            vector![
                rng.gen_range(-bound.x..bound.x),
                rng.gen_range(-bound.y..bound.y)
            ],
            vector![0.0, 0.0],
            std::f64::consts::PI,
            frigate(1),
        );

        let bound = vector![(WORLD_SIZE / 2.0) * 0.9, (WORLD_SIZE / 2.0) * 0.9];
        for _ in 0..200 {
            let mut data = asteroid(rng.gen_range(0..30));
            data.health = 10000.0;
            ship::create(
                sim,
                vector![
                    rng.gen_range(-bound.x..bound.x),
                    rng.gen_range(-bound.y..bound.y)
                ],
                vector![rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)] * 10.0,
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                data,
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }
}

struct Furball {}

impl Furball {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for Furball {
    fn name(&self) -> String {
        "furball".into()
    }

    fn human_name(&self) -> String {
        "Furball".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);
        let mut rng = new_rng(seed);
        for team in 0..2 {
            let fleet_radius = 500.0;
            let range = -fleet_radius..fleet_radius;
            let center = vector![(team as f64 - 0.5) * 2000.0 * 2.0, 0.0];
            let heading = if team == 0 { 0.0 } else { std::f64::consts::PI };
            for _ in 0..10 {
                let offset = vector![rng.gen_range(range.clone()), rng.gen_range(range.clone())];
                ship::create(
                    sim,
                    center + offset,
                    vector![0.0, 0.0],
                    heading,
                    fighter(team),
                );
            }
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }
}

struct Fleet {}

impl Fleet {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for Fleet {
    fn name(&self) -> String {
        "fleet".into()
    }

    fn human_name(&self) -> String {
        "Fleet".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);
        let mut rng = new_rng(seed);
        for team in 0..2 {
            let signum = if team == 0 { -1.0 } else { 1.0 };
            let center = point![signum * 8000.0, rng.gen_range(-6000.0..6000.0)];
            let heading = if team == 0 { 0.0 } else { std::f64::consts::PI };
            let scale = 1;
            let num_fighters = scale * 40;
            let num_frigates = scale * 4;
            let num_cruisers = scale * 2;
            for i in 0..num_fighters {
                ship::create(
                    sim,
                    vector![
                        center.x - signum * 200.0,
                        center.y + i as f64 * 50.0 - (num_fighters - 1) as f64 * 25.0
                    ],
                    vector![0.0, 0.0],
                    heading,
                    fighter(team),
                );
            }
            for i in 0..num_frigates {
                ship::create(
                    sim,
                    vector![
                        center.x,
                        center.y + i as f64 * 300.0 - 150.0 * (num_frigates - 1) as f64
                    ],
                    vector![0.0, 0.0],
                    heading,
                    frigate(team),
                );
            }
            for i in 0..num_cruisers {
                ship::create(
                    sim,
                    vector![
                        center.x + signum * 500.0,
                        center.y + 400.0 * i as f64 - 200.0 * (num_cruisers - 1) as f64
                    ],
                    vector![0.0, 0.0],
                    heading,
                    cruiser(team),
                );
            }
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_capital_ship_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }
}

struct Belt {}

impl Belt {
    fn new() -> Self {
        Self {}
    }
}

impl Scenario for Belt {
    fn name(&self) -> String {
        "belt".into()
    }

    fn human_name(&self) -> String {
        "Belt".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);
        let mut rng = new_rng(seed);
        for team in 0..2 {
            let signum = if team == 0 { -1.0 } else { 1.0 };
            let center = point![rng.gen_range(-6000.0..6000.0), signum * 8000.0];
            let heading = if team == 0 { TAU / 4.0 } else { -TAU / 4.0 };
            let num_fighters = 8;
            let num_frigates = 2;
            for i in 0..(num_fighters / 2) {
                for j in [-1.0, 1.0] {
                    ship::create(
                        sim,
                        vector![center.x + j * (1000.0 + i as f64 * 100.0), center.y],
                        vector![0.0, 0.0],
                        heading,
                        fighter(team),
                    );
                }
            }
            for i in 0..(num_frigates / 2) {
                for j in [-1.0, 1.0] {
                    ship::create(
                        sim,
                        vector![center.x + j * (500.0 + i as f64 * 200.0), center.y],
                        vector![0.0, 0.0],
                        heading,
                        frigate(team),
                    );
                }
            }
            ship::create(
                sim,
                center.coords,
                vector![0.0, 0.0],
                heading,
                cruiser(team),
            );
        }

        let bound = vector![(WORLD_SIZE / 2.0) * 0.9, (WORLD_SIZE / 4.0)];
        for _ in 0..100 {
            let mut data = asteroid(rng.gen_range(0..30));
            data.health = 10000.0;
            ship::create(
                sim,
                vector![
                    rng.gen_range(-bound.x..bound.x),
                    rng.gen_range(-bound.y..bound.y)
                ],
                vector![rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)],
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                data,
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_capital_ship_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }
}

struct PlanetaryDefense {
    rng: SeededRng,
}

impl PlanetaryDefense {
    const PLANET_HEALTH: f64 = 1.5e5;
    const SPAWN_DURATION: f64 = 60.0;

    fn new() -> Self {
        Self { rng: new_rng(0) }
    }
}

impl Scenario for PlanetaryDefense {
    fn name(&self) -> String {
        "planetary_defense".into()
    }

    fn human_name(&self) -> String {
        "Planetary Defense".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);
        self.rng = new_rng(seed);

        {
            let team = 0;
            let center = point![0.0, -3000.0];
            let heading = TAU / 4.0;
            let num_fighters = 8;
            let num_frigates = 2;
            for i in 0..(num_fighters / 2) {
                for j in [-1.0, 1.0] {
                    ship::create(
                        sim,
                        vector![center.x + j * (1000.0 + i as f64 * 100.0), center.y],
                        vector![0.0, 0.0],
                        heading,
                        fighter(team),
                    );
                }
            }
            for i in 0..(num_frigates / 2) {
                for j in [-1.0, 1.0] {
                    ship::create(
                        sim,
                        vector![center.x + j * (500.0 + i as f64 * 200.0), center.y],
                        vector![0.0, 0.0],
                        heading,
                        frigate(team),
                    );
                }
            }
            ship::create(
                sim,
                center.coords,
                vector![0.0, 0.0],
                heading,
                cruiser(team),
            );

            ship::create(
                sim,
                vector![0.0, -WORLD_SIZE / 2.0 + 2500.0],
                vector![0.0, 0.0],
                0.0,
                ShipData {
                    class: ShipClass::Planet,
                    team: 2,
                    health: Self::PLANET_HEALTH,
                    mass: 20e6,
                    radar_cross_section: 50.0,
                    ..Default::default()
                },
            );
        }
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if sim.time() < Self::SPAWN_DURATION {
            let bound = (WORLD_SIZE / 2.0) * 0.9;
            if self
                .rng
                .gen_bool(PHYSICS_TICK_LENGTH * (sim.time() / Self::SPAWN_DURATION) * 2.0)
            {
                let mut ship_data = if self.rng.gen_bool(0.1) {
                    torpedo(1)
                } else {
                    missile(1)
                };
                ship_data.ttl = None;
                ship::create(
                    sim,
                    vector![self.rng.gen_range(-bound..bound), WORLD_SIZE / 2.0 - 30.0],
                    vector![
                        self.rng.gen_range(-30.0..30.0),
                        self.rng.gen_range(-1500.0..-500.0)
                    ],
                    -TAU / 4.0,
                    ship_data,
                );
            }
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        let planet_alive = sim
            .ships
            .iter()
            .any(|&handle| sim.ship(handle).data().class == ShipClass::Planet);
        let enemy_alive = sim
            .ships
            .iter()
            .any(|&handle| sim.ship(handle).data().team == 1);
        if !planet_alive {
            Status::Victory { team: 1 }
        } else if sim.time() > Self::SPAWN_DURATION && !enemy_alive {
            Status::Victory { team: 0 }
        } else {
            Status::Running
        }
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), builtin("planetary_defense.enemy")]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }

    fn score_time(&self, sim: &Simulation) -> f64 {
        if let Some(&planet) = sim
            .ships
            .iter()
            .find(|&&handle| sim.ship(handle).data().class == ShipClass::Planet)
        {
            sim.time() + (1.0 - sim.ship(planet).data().health / Self::PLANET_HEALTH) * 60.0
        } else {
            1e6
        }
    }
}
