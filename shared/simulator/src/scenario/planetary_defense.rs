use super::prelude::*;
use crate::ship::{ShipClass, ShipData};
use crate::simulation::PHYSICS_TICK_LENGTH;

pub struct PlanetaryDefense {
    rng: SeededRng,
}

impl PlanetaryDefense {
    const PLANET_HEALTH: f64 = 1.5e5;
    const SPAWN_DURATION: f64 = 60.0;

    pub fn new() -> Self {
        Self { rng: new_rng(0) }
    }
}

impl Scenario for PlanetaryDefense {
    fn name(&self) -> String {
        "planetary_defense".into()
    }

    fn human_name(&self) -> String {
        "Planetary Defense".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        self.rng = new_rng(seed);

        {
            let team = 0;
            let center = point![0.0, -3000.0];
            let heading = TAU / 4.0;
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

            ship::create(
                sim,
                vector![0.0, -sim.world_size() / 2.0 + 2500.0],
                vector![0.0, 0.0],
                0.0,
                ShipData {
                    class: ShipClass::Planet,
                    team: 2,
                    health: Self::PLANET_HEALTH,
                    mass: 20e6,
                    radar_cross_section: 50.0,
                    ..Default::default()
                },
            );
        }
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if sim.time() < Self::SPAWN_DURATION {
            let bound = (sim.world_size() / 2.0) * 0.9;
            if self
                .rng
                .gen_bool(PHYSICS_TICK_LENGTH * (sim.time() / Self::SPAWN_DURATION) * 2.0)
            {
                let mut ship_data = if self.rng.gen_bool(0.1) {
                    torpedo(1)
                } else {
                    missile(1)
                };
                ship_data.ttl = None;
                ship::create(
                    sim,
                    vector![self.rng.gen_range(-bound..bound), sim.world_size() / 2.0 - 30.0],
                    vector![
                        self.rng.gen_range(-30.0..30.0),
                        self.rng.gen_range(-1500.0..-500.0)
                    ],
                    -TAU / 4.0,
                    ship_data,
                );
            }
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        let planet_alive = sim
            .ships
            .iter()
            .any(|&handle| sim.ship(handle).data().class == ShipClass::Planet);
        let enemy_alive = sim
            .ships
            .iter()
            .any(|&handle| sim.ship(handle).data().team == 1);
        if !planet_alive {
            Status::Victory { team: 1 }
        } else if sim.time() > Self::SPAWN_DURATION && !enemy_alive {
            Status::Victory { team: 0 }
        } else {
            Status::Running
        }
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), builtin("planetary_defense.enemy")]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }

    fn score_time(&self, sim: &Simulation) -> f64 {
        if let Some(&planet) = sim
            .ships
            .iter()
            .find(|&&handle| sim.ship(handle).data().class == ShipClass::Planet)
        {
            sim.time() + (1.0 - sim.ship(planet).data().health / Self::PLANET_HEALTH) * 60.0
        } else {
            1e6
        }
    }
}
