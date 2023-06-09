use nalgebra::UnitComplex;

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
        let placements = place_teams(&mut rng, self.world_size());

        for (team, placement) in placements.into_iter().enumerate() {
            let Placement { position, heading } = placement;
            let signum = if team == 0 { 1.0 } else { -1.0 };
            let frigate_separation = 1000.0;

            // Cruiser
            ship::create(
                sim,
                vector![position.x, position.y],
                vector![0.0, 0.0],
                heading,
                cruiser(team as i32),
            );

            // Frigates
            for s in [-1.0, 1.0] {
                ship::create(
                    sim,
                    vector![position.x, position.y + s * frigate_separation],
                    vector![0.0, 0.0],
                    heading,
                    frigate(team as i32),
                );
            }

            // Fighters
            for s in [-1.0, 1.0] {
                let n = 5;
                for i in 0..n {
                    ship::create(
                        sim,
                        position
                            + vector![signum * 1000.0, s * frigate_separation]
                            + wedge(i, heading) * 100.0,
                        vector![0.0, 0.0],
                        heading,
                        fighter(team as i32),
                    );
                }
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

fn wedge(i: usize, heading: f64) -> Vector2<f64> {
    if i == 0 {
        return vector![0.0, 0.0];
    }

    let s = [-1.0, 1.0][i % 2];
    let j = ((i + 1) / 2) as f64;
    UnitComplex::new(heading).transform_vector(&vector![-j, s * j])
}
