use super::prelude::*;

pub struct MissileDuel {}

impl MissileDuel {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for MissileDuel {
    fn name(&self) -> String {
        "missile_duel".into()
    }

    fn human_name(&self) -> String {
        "Missile Duel".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        let angle = rng.gen_range(0.0..TAU);
        let rot = Rotation2::new(angle);
        let distance = rng.gen_range(15000.0..17000.0);
        let offset = vector![rng.gen_range(-10e3..10e3), rng.gen_range(-10e3..10e3)];

        ship::create(
            sim,
            offset + rot.transform_vector(&vector![-0.5, 0.0]) * distance,
            vector![0.0, 0.0],
            0.0,
            fighter_without_guns(0),
        );
        ship::create(
            sim,
            offset + rot.transform_vector(&vector![0.5, 0.0]) * distance,
            vector![0.0, 0.0],
            std::f64::consts::PI,
            fighter_without_guns(1),
        );
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
