use super::prelude::*;

pub struct TutorialRadio {}

impl TutorialRadio {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for TutorialRadio {
    fn name(&self) -> String {
        "tutorial_radio".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 8: Radio".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        {
            let position = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                .transform_point(&point![rng.gen_range(100.0..500.0), 0.0]);
            ship::create(
                sim,
                position.coords,
                vector![0.0, 0.0],
                rng.gen_range(0.0..std::f64::consts::TAU),
                fighter_without_missiles_or_radar(0),
            );
        }
        {
            let position = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                .transform_point(&point![rng.gen_range(6000.0..8000.0), 0.0]);
            ship::create(
                sim,
                position.coords,
                vector![0.0, 0.0],
                rng.gen_range(0.0..std::f64::consts::TAU),
                fighter(1),
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS * 2)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![
            builtin("tutorial/tutorial_radio.initial"),
            builtin("tutorial/tutorial_radio.enemy"),
        ]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial_radio.solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial_missiles".to_string())
    }
}
