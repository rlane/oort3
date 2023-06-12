use super::prelude::*;

pub struct TutorialSquadron {}

impl TutorialSquadron {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for TutorialSquadron {
    fn name(&self) -> String {
        "tutorial_squadron".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 10: Squadron".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);

        for i in 1..6 {
            let center = vector![-8000.0, 0.0];
            let offset = vector![
                (i / 2) as f64 * -100.0,
                ((i / 2) as f64) * ((i % 2) as f64 - 0.5) * 300.0
            ];
            ship::create(sim, center + offset, vector![0.0, 0.0], 0.0, fighter(0));
        }

        for _ in 0..4 {
            let size = 500.0;
            let range = -size..size;
            let center = vector![8000.0, 0.0];
            let offset = vector![rng.gen_range(range.clone()), rng.gen_range(range.clone())];
            ship::create(sim, center + offset, vector![0.0, 0.0], PI, fighter(1));
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS * 3)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![
            builtin("tutorial/tutorial_squadron_initial"),
            builtin("tutorial/tutorial_squadron_enemy"),
        ]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial_squadron_solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial_frigate".to_string())
    }

    fn previous_names(&self) -> Vec<String> {
        vec!["tutorial07".into()]
    }
}
