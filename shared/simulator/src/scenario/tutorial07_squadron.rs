use super::prelude::*;

pub struct Tutorial07 {}

impl Tutorial07 {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for Tutorial07 {
    fn name(&self) -> String {
        "tutorial07".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 7: Squadron".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);

        let mut rng = new_rng(seed);
        for team in 0..2 {
            for _ in 0..4 {
                let size = 500.0;
                let range = -size..size;
                let center = vector![(team as f64 - 0.5) * 6000.0, 0.0];
                let offset = vector![rng.gen_range(range.clone()), rng.gen_range(range.clone())];
                let heading = if team == 0 { 0.0 } else { std::f64::consts::PI };
                ship::create(
                    sim,
                    center + offset,
                    vector![0.0, 0.0],
                    heading,
                    fighter_without_missiles(team),
                );
            }
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS * 3)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![
            builtin("tutorial/tutorial07.initial"),
            builtin("tutorial/tutorial07.enemy"),
        ]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial07.solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial08".to_string())
    }
}
