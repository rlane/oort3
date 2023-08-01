use super::prelude::*;

pub struct TutorialGuns {}

impl Scenario for TutorialGuns {
    fn name(&self) -> String {
        "tutorial_guns".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 1: Guns".into()
    }

    fn init(&mut self, sim: &mut Simulation, _seed: u32) {
        ship::create(
            sim,
            vector![-250.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            fighter_without_missiles_or_radar(0),
        );
        ship::create(
            sim,
            vector![250.0, 0.0],
            vector![0.0, 0.0],
            0.1,
            target_asteroid(1),
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![builtin("tutorial/tutorial_guns_initial")]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial_guns_solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial_acceleration".to_string())
    }

    fn previous_names(&self) -> Vec<String> {
        vec!["tutorial01".into()]
    }
}
