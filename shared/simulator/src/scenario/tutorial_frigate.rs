use super::prelude::*;

pub struct TutorialFrigate {}

impl TutorialFrigate {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for TutorialFrigate {
    fn name(&self) -> String {
        "tutorial_frigate".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 10: Frigate".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        ship::create(sim, vector![0.0, 0.0], vector![0.0, 0.0], 0.0, frigate(0));

        let mut rng = new_rng(seed);
        for _ in 0..5 {
            let p = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                .transform_vector(&vector![rng.gen_range(1000.0..1500.0), 0.0]);
            let v = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                .transform_vector(&vector![rng.gen_range(0.0..300.0), 0.0]);
            ship::create(sim, p, v, std::f64::consts::PI, fighter(1));
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS * 2)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![
            builtin("tutorial/tutorial_frigate.initial"),
            builtin("tutorial/tutorial_frigate.enemy"),
        ]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial_frigate.solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial_cruiser".to_string())
    }

    fn previous_names(&self) -> Vec<String> {
        vec!["tutorial10".into()]
    }
}
