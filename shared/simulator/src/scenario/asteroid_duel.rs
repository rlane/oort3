use super::prelude::*;

pub struct AsteroidDuel {}

impl AsteroidDuel {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for AsteroidDuel {
    fn name(&self) -> String {
        "asteroid_duel".into()
    }

    fn human_name(&self) -> String {
        "Asteroid Duel".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        let bound = vector![(sim.world_size() / 2.0) * 0.9, (sim.world_size() / 2.0) * 0.9];

        ship::create(
            sim,
            vector![
                rng.gen_range(-bound.x..bound.x),
                rng.gen_range(-bound.y..bound.y)
            ],
            vector![0.0, 0.0],
            0.0,
            frigate(0),
        );
        ship::create(
            sim,
            vector![
                rng.gen_range(-bound.x..bound.x),
                rng.gen_range(-bound.y..bound.y)
            ],
            vector![0.0, 0.0],
            std::f64::consts::PI,
            frigate(1),
        );

        let bound = vector![(sim.world_size() / 2.0) * 0.9, (sim.world_size() / 2.0) * 0.9];
        for _ in 0..200 {
            let mut data = asteroid(rng.gen_range(0..30));
            data.health = 10000.0;
            ship::create(
                sim,
                vector![
                    rng.gen_range(-bound.x..bound.x),
                    rng.gen_range(-bound.y..bound.y)
                ],
                vector![rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)] * 10.0,
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                data,
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
}
