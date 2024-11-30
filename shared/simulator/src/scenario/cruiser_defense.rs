use nalgebra::UnitComplex;

use super::check_victory_with_filter;
use super::prelude::*;
use crate::ship::ShipClass;

pub struct CruiserDefense {
    rng: SeededRng,
}

impl CruiserDefense {
    pub fn new() -> Self {
        Self { rng: new_rng(0) }
    }
}

impl Scenario for CruiserDefense {
    fn name(&self) -> String {
        "cruiser_defense".into()
    }

    fn human_name(&self) -> String {
        "Cruiser Defense".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        self.rng = new_rng(seed);

        {
            let team = 0;
            let center = vector![0.0, 0.0];
            let heading = TAU / 4.0;
            ship::create(sim, center, vector![0.0, 0.0], heading, cruiser(team));
            for i in [-1.0, 0.0, 1.0] {
                ship::create(
                    sim,
                    center + vector![i * 100.0, 500.0],
                    vector![0.0, 0.0],
                    heading,
                    fighter(team),
                );
            }
        }

        {
            let team = 1;
            for _ in 0..3 {
                let angle = self.rng.gen_range(0.0..TAU);
                let center = UnitComplex::new(angle).transform_point(&point![15e3, 0.0]);
                let heading = angle + TAU / 2.0;
                for i in 0..3 {
                    ship::create(
                        sim,
                        center.coords
                            + i as f64
                                * UnitComplex::new(angle + TAU / 4.0)
                                    .transform_vector(&vector![100.0, 0.0]),
                        vector![0.0, 0.0],
                        heading,
                        fighter(team),
                    );
                }
            }
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_victory_with_filter(sim, TOURNAMENT_MAX_TICKS, |ship| {
            ship.data().team == 0
                && [ShipClass::Frigate, ShipClass::Cruiser].contains(&ship.data().class)
                || ship.data().team == 1 && [ShipClass::Fighter].contains(&ship.data().class)
        })
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![
            builtin("challenge/cruiser_defense_initial"),
            builtin("tutorial/tutorial_cruiser_enemy"),
        ]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn score_time(&self, sim: &Simulation) -> f64 {
        let initial_health = cruiser(0).health;
        if let Some(&cruiser) = sim
            .ships
            .iter()
            .find(|&&handle| sim.ship(handle).data().class == ShipClass::Cruiser)
        {
            sim.time() + (1.0 - sim.ship(cruiser).data().health / initial_health) * 60.0
        } else {
            1e6
        }
    }
}
