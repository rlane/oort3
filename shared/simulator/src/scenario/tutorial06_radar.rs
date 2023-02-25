use super::prelude::*;

pub struct Tutorial06 {}

impl Tutorial06 {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for Tutorial06 {
    fn name(&self) -> String {
        "tutorial06".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 6: Radar".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);
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
            builtin("tutorial/tutorial06.initial"),
            builtin("tutorial/tutorial06.enemy"),
        ]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial06.solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial07".to_string())
    }
}
