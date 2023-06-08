use super::prelude::*;

pub struct CruiserDuel {}

impl CruiserDuel {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for CruiserDuel {
    fn name(&self) -> String {
        "cruiser_duel".into()
    }

    fn human_name(&self) -> String {
        "Cruiser Duel".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        let placements = place_teams(&mut rng, self.world_size());

        for (team, placement) in placements.into_iter().enumerate() {
            let Placement { position, heading } = placement;
            ship::create(
                sim,
                position,
                vector![0.0, 0.0],
                heading,
                cruiser(team as i32),
            );
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
        100000.0
    }
}
