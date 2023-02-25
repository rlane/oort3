use super::prelude::*;

pub struct Tutorial10 {}

impl Tutorial10 {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for Tutorial10 {
    fn name(&self) -> String {
        "tutorial10".into()
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
            builtin("tutorial/tutorial10.initial"),
            builtin("tutorial/tutorial10.enemy"),
        ]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial10.solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial11".to_string())
    }
}
