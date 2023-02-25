use super::prelude::*;

pub struct Tutorial01 {}

impl Scenario for Tutorial01 {
    fn name(&self) -> String {
        "tutorial01".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 1: Guns".into()
    }

    fn init(&mut self, sim: &mut Simulation, _seed: u32) {
        ship::create(
            sim,
            vector![-1000.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            fighter_without_missiles_or_radar(0),
        );
        ship::create(
            sim,
            vector![1000.0, 0.0],
            vector![0.0, 0.0],
            0.1,
            target_asteroid(1),
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![builtin("tutorial/tutorial01.initial")]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial01.solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial02".to_string())
    }
}
