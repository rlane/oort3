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
        let placements = place_teams(&mut rng, self.world_size());
        let offsets = [
            vector![0.0, 0.0],
            vector![-100.0, 100.0],
            vector![-100.0, -100.0],
        ];

        for (team, placement) in placements.into_iter().enumerate() {
            let Placement { position, heading } = placement;
            for offset in &offsets {
                ship::create(
                    sim,
                    position + UnitComplex::new(heading).transform_vector(offset),
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
