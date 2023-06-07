use super::prelude::*;
use nalgebra::UnitComplex;

pub struct Squadrons {}

impl Squadrons {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for Squadrons {
    fn name(&self) -> String {
        "squadrons".into()
    }

    fn human_name(&self) -> String {
        "Squadrons".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        let s = self.world_size() * 0.45;
        let range = -s..s;
        let centers = [
            vector![-s, rng.gen_range(range.clone())],
            vector![s, rng.gen_range(range)],
        ];
        let headings = [0.0, std::f64::consts::PI];
        let offsets = [
            vector![0.0, 0.0],
            vector![-100.0, 100.0],
            vector![-100.0, -100.0],
        ];

        for team in 0..2 {
            let center = centers[team];
            let heading = headings[team];
            for offset in &offsets {
                ship::create(
                    sim,
                    center + UnitComplex::new(heading).transform_vector(offset),
                    vector![0.0, 0.0],
                    heading,
                    fighter(team as i32),
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

    fn world_size(&self) -> f64 {
        60e3
    }
}
