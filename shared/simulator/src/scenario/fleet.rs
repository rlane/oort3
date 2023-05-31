use super::prelude::*;

pub struct Fleet {}

impl Fleet {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for Fleet {
    fn name(&self) -> String {
        "fleet".into()
    }

    fn human_name(&self) -> String {
        "Fleet".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        for team in 0..2 {
            let w = self.world_size() / 2.0;
            let signum = if team == 0 { -1.0 } else { 1.0 };
            let center = point![signum * w * 0.9, rng.gen_range(-1.0..1.0) * w * 0.7];
            let heading = if team == 0 { 0.0 } else { std::f64::consts::PI };
            let scale = 1;
            let num_fighters = scale * 40;
            let num_frigates = scale * 4;
            let num_cruisers = scale * 2;
            for i in 0..num_fighters {
                ship::create(
                    sim,
                    vector![
                        center.x - signum * 200.0,
                        center.y + i as f64 * 50.0 - (num_fighters - 1) as f64 * 25.0
                    ],
                    vector![0.0, 0.0],
                    heading,
                    fighter(team),
                );
            }
            for i in 0..num_frigates {
                ship::create(
                    sim,
                    vector![
                        center.x,
                        center.y + i as f64 * 300.0 - 150.0 * (num_frigates - 1) as f64
                    ],
                    vector![0.0, 0.0],
                    heading,
                    frigate(team),
                );
            }
            for i in 0..num_cruisers {
                ship::create(
                    sim,
                    vector![
                        center.x + signum * 500.0,
                        center.y + 400.0 * i as f64 - 200.0 * (num_cruisers - 1) as f64
                    ],
                    vector![0.0, 0.0],
                    heading,
                    cruiser(team),
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
