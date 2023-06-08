mod asteroid_duel;
mod belt;
mod cruiser_duel;
mod fighter_duel;
mod fleet;
mod frigate_duel;
mod gunnery;
mod orbit;
mod planetary_defense;
mod primitive_duel;
mod radar_duel;
mod squadrons;
mod stress;
mod test;
mod tutorial_acceleration;
mod tutorial_acceleration2;
mod tutorial_cruiser;
mod tutorial_deflection;
mod tutorial_frigate;
mod tutorial_guns;
mod tutorial_missiles;
mod tutorial_radar;
mod tutorial_radio;
mod tutorial_rotation;
mod tutorial_search;
mod tutorial_squadron;
mod welcome;

use crate::ship::{asteroid, fighter, ShipAccessor, ShipClass, ShipData};
use crate::simulation::{Code, Line, Simulation};
use nalgebra::{vector, Vector2};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod prelude {
    pub use super::Scenario;
    pub use super::Status;
    pub use super::{builtin, empty_ai, reference_ai};
    pub use super::{
        check_capital_ship_tournament_victory, check_tournament_victory, check_tutorial_victory,
    };
    pub use super::{fighter_without_missiles, fighter_without_missiles_or_radar, target_asteroid};
    pub use super::{place_teams, Placement};
    pub use super::{DEFAULT_TUTORIAL_MAX_TICKS, TOURNAMENT_MAX_TICKS};
    pub use crate::rng::{new_rng, SeededRng};
    pub use crate::ship::{
        self, asteroid, cruiser, fighter, frigate, missile, target, torpedo, ShipHandle,
    };
    pub use crate::simulation::{Code, Line, Simulation};
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

    fn previous_names(&self) -> Vec<String> {
        vec![]
    }

    fn world_size(&self) -> f64 {
        40000.0
    }
}

pub fn load_safe(name: &str) -> Option<Box<dyn Scenario>> {
    let scenario: Option<Box<dyn Scenario>> = match name {
        // Tutorials
        "tutorial_guns" => Some(Box::new(tutorial_guns::TutorialGuns {})),
        "tutorial_acceleration" => {
            Some(Box::new(tutorial_acceleration::TutorialAcceleration::new()))
        }
        "tutorial_acceleration2" => Some(Box::new(
            tutorial_acceleration2::TutorialAcceleration2::new(),
        )),
        "tutorial_rotation" => Some(Box::new(tutorial_rotation::TutorialRotation::new())),
        "tutorial_deflection" => Some(Box::new(tutorial_deflection::TutorialDeflection::new())),
        "tutorial_radar" => Some(Box::new(tutorial_radar::TutorialRadar::new())),
        "tutorial_search" => Some(Box::new(tutorial_search::TutorialSearch::new())),
        "tutorial_radio" => Some(Box::new(tutorial_radio::TutorialRadio::new())),
        "tutorial_missiles" => Some(Box::new(tutorial_missiles::TutorialMissiles::new())),
        "tutorial_squadron" => Some(Box::new(tutorial_squadron::TutorialSquadron::new())),
        "tutorial_frigate" => Some(Box::new(tutorial_frigate::TutorialFrigate::new())),
        "tutorial_cruiser" => Some(Box::new(tutorial_cruiser::TutorialCruiser::new())),
        // Tournament
        "primitive_duel" => Some(Box::new(primitive_duel::PrimitiveDuel::new())),
        "radar_duel" => Some(Box::new(radar_duel::RadarDuel::new())),
        "fighter_duel" => Some(Box::new(fighter_duel::FighterDuel::new())),
        "frigate_duel" => Some(Box::new(frigate_duel::FrigateDuel::new())),
        "cruiser_duel" => Some(Box::new(cruiser_duel::CruiserDuel::new())),
        "asteroid_duel" => Some(Box::new(asteroid_duel::AsteroidDuel::new())),
        "squadrons" => Some(Box::new(squadrons::Squadrons::new())),
        "fleet" => Some(Box::new(fleet::Fleet::new())),
        "belt" => Some(Box::new(belt::Belt::new())),
        "orbit" => Some(Box::new(orbit::Orbit::new())),
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
        "radar_test" => Some(Box::new(test::RadarTest {})),
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

pub fn list() -> Vec<(String, Vec<String>)> {
    vec![
        ("Introduction", vec!["welcome"]),
        (
            "Tutorial",
            vec![
                "tutorial_guns",
                "tutorial_acceleration",
                "tutorial_acceleration2",
                "tutorial_rotation",
                "tutorial_deflection",
                "tutorial_radar",
                "tutorial_search",
                "tutorial_radio",
                "tutorial_missiles",
                "tutorial_squadron",
                "tutorial_frigate",
                "tutorial_cruiser",
            ],
        ),
        ("Challenge", vec!["gunnery", "planetary_defense"]),
        (
            "Tournament",
            vec![
                "primitive_duel",
                "fighter_duel",
                "frigate_duel",
                "cruiser_duel",
                "asteroid_duel",
                "squadrons",
                "fleet",
                "belt",
                "orbit",
            ],
        ),
    ]
    .iter()
    .map(|(category, scenario_names)| {
        (
            category.to_string(),
            scenario_names.iter().map(|name| name.to_string()).collect(),
        )
    })
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

pub fn target_asteroid(variant: i32) -> ShipData {
    let mut asteroid = asteroid(variant);
    asteroid.team = 1;
    asteroid
}

pub struct Placement {
    pub position: Vector2<f64>,
    pub heading: f64,
}

pub fn place_teams(rng: &mut dyn RngCore, world_size: f64) -> Vec<Placement> {
    let s = world_size * 0.45;
    let range = -s..s;
    vec![
        Placement {
            position: vector![-s, rng.gen_range(range.clone())],
            heading: 0.0,
        },
        Placement {
            position: vector![s, rng.gen_range(range)],
            heading: std::f64::consts::PI,
        },
    ]
}
