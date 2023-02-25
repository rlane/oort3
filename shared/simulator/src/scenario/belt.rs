use super::prelude::*;

pub struct Belt {}

impl Belt {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for Belt {
    fn name(&self) -> String {
        "belt".into()
    }

    fn human_name(&self) -> String {
        "Belt".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        add_walls(sim);
        let mut rng = new_rng(seed);
        for team in 0..2 {
            let signum = if team == 0 { -1.0 } else { 1.0 };
            let center = point![rng.gen_range(-6000.0..6000.0), signum * 8000.0];
            let heading = if team == 0 { TAU / 4.0 } else { -TAU / 4.0 };
            let num_fighters = 8;
            let num_frigates = 2;
            for i in 0..(num_fighters / 2) {
                for j in [-1.0, 1.0] {
                    ship::create(
                        sim,
                        vector![center.x + j * (1000.0 + i as f64 * 100.0), center.y],
                        vector![0.0, 0.0],
                        heading,
                        fighter(team),
                    );
                }
            }
            for i in 0..(num_frigates / 2) {
                for j in [-1.0, 1.0] {
                    ship::create(
                        sim,
                        vector![center.x + j * (500.0 + i as f64 * 200.0), center.y],
                        vector![0.0, 0.0],
                        heading,
                        frigate(team),
                    );
                }
            }
            ship::create(
                sim,
                center.coords,
                vector![0.0, 0.0],
                heading,
                cruiser(team),
            );
        }

        let bound = vector![(WORLD_SIZE / 2.0) * 0.9, (WORLD_SIZE / 4.0)];
        for _ in 0..100 {
            let mut data = asteroid(rng.gen_range(0..30));
            data.health = 10000.0;
            ship::create(
                sim,
                vector![
                    rng.gen_range(-bound.x..bound.x),
                    rng.gen_range(-bound.y..bound.y)
                ],
                vector![rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)],
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                data,
            );
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
}
