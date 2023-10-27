use nalgebra::ComplexField;
use oort_api::{Class, ClassStats};
use rapier2d_f64::prelude::RigidBody;

use super::prelude::*;
use crate::ship::{ShipClass, ShipData};
use crate::simulation::PHYSICS_TICK_LENGTH;

const PLANET_MASS: f64 = 1.5e19;
const G: f64 = 6.674e-11;

pub struct Orbit {}

impl Orbit {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for Orbit {
    fn name(&self) -> String {
        "orbit".into()
    }

    fn human_name(&self) -> String {
        "Orbit".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let flip = seed % 2 == 1;
        let seed = seed / 2;
        let mut rng = new_rng(seed);
        for team in 0..2 {
            let t = team as f64 * 2.0 - 1.0;
            let t = if flip { -t } else { t };
            let r = rng.gen_range(11e3..20e3);
            let s = ComplexField::sqrt(G * PLANET_MASS / r);
            ship::create(
                sim,
                vector![t * r, 0.0],
                vector![0.0, s],
                0.5 * std::f64::consts::PI,
                frigate(team),
            );
        }

        ship::create(
            sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ShipData {
                class: ShipClass::Planet,
                team: 2,
                health: 1e9,
                base_stats: ClassStats {
                    mass: PLANET_MASS,
                    ..Class::Unknown.default_stats()
                },
                radar_cross_section: 1e6,
                ..Default::default()
            },
        );
    }

    fn tick(&mut self, sim: &mut Simulation) {
        let apply_gravity = |body: &mut RigidBody| {
            let r = body.translation().norm();
            let g = G * PLANET_MASS / (r * r);
            let acc = body.translation().normalize() * -g;
            let impulse = acc * body.mass() * PHYSICS_TICK_LENGTH;
            body.apply_impulse(impulse, true);
        };

        let handles = sim.ships.iter().cloned().collect::<Vec<_>>();
        for handle in handles {
            let mut ship = sim.ship_mut(handle);
            if ship.data().team == 2 {
                continue;
            }
            apply_gravity(ship.body());
        }

        let handles = sim.bullets.iter().cloned().collect::<Vec<_>>();
        for handle in handles {
            let body = sim.bodies.get_mut(handle.into()).unwrap();
            apply_gravity(body);
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
        80e3
    }
}
