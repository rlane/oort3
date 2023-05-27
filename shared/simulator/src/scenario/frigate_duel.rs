use super::prelude::*;

pub struct FrigateDuel {}

impl FrigateDuel {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for FrigateDuel {
    fn name(&self) -> String {
        "frigate_duel".into()
    }

    fn human_name(&self) -> String {
        "Frigate Duel".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        let s = self.world_size() * 0.45;
        let range = -s..s;
        let p0 = vector![rng.gen_range(range.clone()), rng.gen_range(range.clone())];
        let p1 = vector![rng.gen_range(range.clone()), rng.gen_range(range)];

        ship::create(sim, p0, vector![0.0, 0.0], 0.0, frigate(0));
        ship::create(sim, p1, vector![0.0, 0.0], 0.0, frigate(1));
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
