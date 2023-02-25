use super::prelude::*;

pub struct Tutorial09 {}

impl Tutorial09 {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for Tutorial09 {
    fn name(&self) -> String {
        "tutorial09".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 9: Missiles".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);

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
            builtin("tutorial/tutorial09.initial"),
            builtin("tutorial/tutorial09.enemy"),
        ]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial09.solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial10".to_string())
    }
}
