use super::prelude::*;
use crate::bullet::{self, BulletData};
use crate::color;

pub struct StressScenario {}

impl Scenario for StressScenario {
    fn name(&self) -> String {
        "stress".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        add_walls(sim);
        ship::create(sim, vector![0.0, 0.0], vector![0.0, 0.0], 0.0, fighter(0));

        let bound = (WORLD_SIZE / 2.0) * 0.9;
        for team in [0, 1] {
            for _ in 0..100 {
                ship::create(
                    sim,
                    vector![rng.gen_range(-bound..bound), rng.gen_range(-bound..bound)],
                    vector![rng.gen_range(-30.0..30.0), rng.gen_range(-30.0..30.0)],
                    rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                    fighter(team),
                );
            }
        }
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![reference_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        builtin("reference")
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }
}

pub struct AsteroidStressScenario {}

impl Scenario for AsteroidStressScenario {
    fn name(&self) -> String {
        "asteroid-stress".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        add_walls(sim);
        ship::create(sim, vector![0.0, 0.0], vector![0.0, 0.0], 0.0, fighter(0));

        let bound = (WORLD_SIZE / 2.0) * 0.9;
        for _ in 0..1000 {
            ship::create(
                sim,
                vector![rng.gen_range(-bound..bound), rng.gen_range(-bound..bound)],
                vector![rng.gen_range(-30.0..30.0), rng.gen_range(-30.0..30.0)],
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                asteroid(rng.gen_range(0..30)),
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS)
    }
}

pub struct BulletStressScenario {}

impl Scenario for BulletStressScenario {
    fn name(&self) -> String {
        "bullet-stress".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        add_walls(sim);
        ship::create(sim, vector![0.0, 0.0], vector![0.0, 0.0], 0.0, fighter(0));

        let bound = (WORLD_SIZE / 2.0) * 0.9;
        for _ in 0..1000 {
            let s = 1000.0;
            bullet::create(
                sim,
                vector![rng.gen_range(-bound..bound), rng.gen_range(-bound..bound)],
                vector![rng.gen_range(-s..s), rng.gen_range(-s..s)],
                BulletData {
                    mass: 0.1,
                    team: 0,
                    color: color::to_u32(vector![1.00, 0.63, 0.00, 0.30]),
                    ttl: 100.0,
                },
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        if sim.bullets.is_empty() {
            Status::Victory { team: 0 }
        } else {
            Status::Running
        }
    }
}

pub struct MissileStressScenario {}

impl Scenario for MissileStressScenario {
    fn name(&self) -> String {
        "missile-stress".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        if seed != 0 {
            log::warn!("Ignoring nonzero seed {}", seed);
        }
        let mut rng = new_rng(0);
        add_walls(sim);

        let bound = (WORLD_SIZE / 2.0) * 0.9;
        for i in 0..100 {
            ship::create(
                sim,
                vector![rng.gen_range(-bound..bound), rng.gen_range(-bound..bound)],
                vector![rng.gen_range(-30.0..30.0), rng.gen_range(-30.0..30.0)],
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                missile(i % 2),
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        if sim.ships.len() < 50 {
            Status::Victory { team: 0 }
        } else {
            Status::Running
        }
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![reference_ai(), reference_ai()]
    }
}
