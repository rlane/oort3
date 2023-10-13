use super::prelude::*;

pub struct TutorialSearch {}

impl TutorialSearch {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for TutorialSearch {
    fn name(&self) -> String {
        "tutorial_search".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 9: Search".into()
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
                fighter_without_missiles(0),
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
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS * 3)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![
            builtin("tutorial/tutorial_search_initial"),
            builtin("tutorial/tutorial_search_enemy"),
        ]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial_search_solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial_radio".to_string())
    }

    fn previous_names(&self) -> Vec<String> {
        vec!["tutorial08".into()]
    }
}
