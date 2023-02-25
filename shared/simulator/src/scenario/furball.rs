use super::prelude::*;

pub struct Furball {}

impl Furball {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for Furball {
    fn name(&self) -> String {
        "furball".into()
    }

    fn human_name(&self) -> String {
        "Furball".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        for team in 0..2 {
            let fleet_radius = 500.0;
            let range = -fleet_radius..fleet_radius;
            let center = vector![(team as f64 - 0.5) * 2000.0 * 2.0, 0.0];
            let heading = if team == 0 { 0.0 } else { std::f64::consts::PI };
            for _ in 0..10 {
                let offset = vector![rng.gen_range(range.clone()), rng.gen_range(range.clone())];
                ship::create(
                    sim,
                    center + offset,
                    vector![0.0, 0.0],
                    heading,
                    fighter(team),
                );
            }
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }
}
