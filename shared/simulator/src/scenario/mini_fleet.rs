use super::prelude::*;

pub struct MiniFleet {}

impl MiniFleet {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for MiniFleet {
    fn name(&self) -> String {
        "mini_fleet".into()
    }

    fn human_name(&self) -> String {
        "Mini-Fleet".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        let placements = place_teams(&mut rng, self.world_size());

        for (team, placement) in placements.into_iter().enumerate() {
            let Placement { position, heading } = placement;
            let fighter_separation = 1000.0;

            // Frigate
            ship::create(
                sim,
                vector![position.x, position.y],
                vector![0.0, 0.0],
                heading,
                frigate(team as i32),
            );

            // Fighters
            for s in [-1.0, 1.0] {
                ship::create(
                    sim,
                    vector![position.x, position.y + s * fighter_separation],
                    vector![0.0, 0.0],
                    heading,
                    fighter(team as i32),
                );
            }
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_capital_ship_tournament_victory(sim)
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
        100e3
    }
}
