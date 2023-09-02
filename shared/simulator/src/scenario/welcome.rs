use crate::ship::{ShipClass, ShipData};

use super::prelude::*;
use rand::seq::SliceRandom;

pub struct Welcome {
    rng: Option<SeededRng>,
}

impl Welcome {
    pub fn new() -> Self {
        Self {
            rng: None as Option<SeededRng>,
        }
    }
}

impl Scenario for Welcome {
    fn name(&self) -> String {
        "welcome".into()
    }

    fn human_name(&self) -> String {
        "Welcome".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        self.rng = Some(new_rng(seed));
        let rng = self.rng.as_mut().unwrap();

        let ship_datas = &[fighter(0), frigate(0), cruiser(0)];
        let ship_data = rng.sample(rand::distributions::Slice::new(ship_datas).unwrap());
        ship::create(
            sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship_data.clone(),
        );

        // HACK: avoid automatically picking the player ship and showing debug lines.
        ship::create(
            sim,
            vector![2000.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ShipData {
                team: 0,
                ..asteroid(1)
            },
        );
    }

    fn tick(&mut self, sim: &mut Simulation) {
        let rng = self.rng.as_mut().unwrap();
        let asteroid_variants = [1, 6, 14];
        let num_asteroids = sim
            .ships
            .iter()
            .filter(|s| matches!(sim.ship(**s).data().class, ShipClass::Asteroid { .. }))
            .count();
        for _ in num_asteroids..20 {
            let p = Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                .transform_point(&point![rng.gen_range(500.0..2000.0), 0.0]);
            ship::create(
                sim,
                vector![p.x, p.y],
                vector![rng.gen_range(-30.0..30.0), rng.gen_range(-30.0..30.0)],
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                asteroid(*asteroid_variants.choose(rng).unwrap()),
            );
        }
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![Code::None, Code::None]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn lines(&self) -> Vec<Line> {
        let x = 500.0;
        vec![Line {
            a: point![x, x],
            b: point![x, x],
            color: vector![0.0, 0.0, 0.0, 0.0],
        }]
    }
}
