use super::prelude::*;

pub struct TutorialMissiles {}

impl TutorialMissiles {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for TutorialMissiles {
    fn name(&self) -> String {
        "tutorial_missiles".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 8: Missiles".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut shipdata = fighter(0);
        shipdata.guns[0].cycle_time_remaining = 1e9;
        ship::create(sim, vector![0.0, 0.0], vector![0.0, 0.0], 0.0, shipdata);

        let mut rng = new_rng(seed);
        let p = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
            .transform_vector(&vector![rng.gen_range(2000.0..2500.0), 0.0]);
        let v = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
            .transform_vector(&vector![rng.gen_range(0.0..300.0), 0.0]);
        let mut shipdata = fighter(1);
        shipdata.health /= 2.0;
        ship::create(sim, p, v, std::f64::consts::PI, shipdata);
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS * 2)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![
            builtin("tutorial/tutorial_missiles.initial"),
            builtin("tutorial/tutorial_missiles.enemy"),
        ]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial_missiles.solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial_squadron".to_string())
    }

    fn previous_names(&self) -> Vec<String> {
        vec!["tutorial09".into()]
    }
}
