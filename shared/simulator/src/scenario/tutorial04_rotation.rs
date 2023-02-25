use super::prelude::*;

pub struct Tutorial04 {}

impl Tutorial04 {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for Tutorial04 {
    fn name(&self) -> String {
        "tutorial04".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 4: Rotation".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        let target = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
            .transform_point(&point![rng.gen_range(600.0..1000.0), 0.0]);
        let handle = ship::create(
            sim,
            Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                .transform_vector(&vector![rng.gen_range(100.0..500.0), 0.0]),
            vector![0.0, 0.0],
            0.0,
            fighter_without_missiles_or_radar(0),
        );
        sim.write_target(handle, target.coords, vector![0.0, 0.0]);
        ship::create(
            sim,
            target.coords,
            vector![0.0, 0.0],
            0.0,
            target_asteroid(1),
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![builtin("tutorial/tutorial04.initial")]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial04.solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial05".to_string())
    }
}
