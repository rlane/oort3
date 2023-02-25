mod asteroid_duel;
mod belt;
mod cruiser_duel;
mod fighter_duel;
mod fleet;
mod frigate_duel;
mod furball;
mod gunnery;
mod missile_duel;
mod planetary_defense;
mod primitive_duel;
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

use crate::collision;
use crate::ship::{asteroid, fighter, ShipAccessor, ShipClass, ShipData};
use crate::simulation::{Code, Line, Simulation, WORLD_SIZE};
use nalgebra::vector;

use rapier2d_f64::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod prelude {
    pub use super::Scenario;
    pub use super::Status;
    pub use super::{
        add_walls, fighter_without_guns, fighter_without_missiles,
        fighter_without_missiles_or_radar, target_asteroid,
    };
    pub use super::{builtin, empty_ai, reference_ai};
    pub use super::{
        check_capital_ship_tournament_victory, check_tournament_victory, check_tutorial_victory,
    };
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

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Status {
    Running,
    Victory { team: i32 },
    Failed,
    Draw,
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
        "primitive_duel" => Some(Box::new(primitive_duel::PrimitiveDuel::new())),
        "fighter_duel" => Some(Box::new(fighter_duel::FighterDuel::new())),
        "missile_duel" => Some(Box::new(missile_duel::MissileDuel::new())),
        "frigate_duel" => Some(Box::new(frigate_duel::FrigateDuel::new())),
        "cruiser_duel" => Some(Box::new(cruiser_duel::CruiserDuel::new())),
        "asteroid_duel" => Some(Box::new(asteroid_duel::AsteroidDuel::new())),
        "furball" => Some(Box::new(furball::Furball::new())),
        "fleet" => Some(Box::new(fleet::Fleet::new())),
        "belt" => Some(Box::new(belt::Belt::new())),
        // Challenge
        "gunnery" => Some(Box::new(gunnery::GunneryScenario {})),
        "planetary_defense" => Some(Box::new(planetary_defense::PlanetaryDefense::new())),
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
