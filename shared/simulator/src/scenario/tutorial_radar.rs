use super::prelude::*;

pub struct TutorialRadar {}

impl TutorialRadar {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for TutorialRadar {
    fn name(&self) -> String {
        "tutorial_radar".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 6: Radar".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        ship::create(
            sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            fighter_without_missiles(0),
        );

        let mut rng = new_rng(seed);
        let size = 500.0;
        let range = -size..size;
        for _ in 0..3 {
            let target = point![rng.gen_range(range.clone()), rng.gen_range(range.clone())];
            ship::create(
                sim,
                target.coords,
                vector![
                    rng.gen_range(0.0..std::f64::consts::TAU),
                    rng.gen_range(-400.0..400.0)
                ],
                rng.gen_range(-400.0..400.0),
                fighter(1),
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS * 2)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![
            builtin("tutorial/tutorial_radar.initial"),
            builtin("tutorial/tutorial_radar.enemy"),
        ]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial_radar.solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial_squadron".to_string())
    }

    fn previous_names(&self) -> Vec<String> {
        vec!["tutorial06".into()]
    }
}
