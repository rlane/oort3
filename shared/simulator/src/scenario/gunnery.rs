use super::prelude::*;

pub struct GunneryScenario {}

impl Scenario for GunneryScenario {
    fn name(&self) -> String {
        "gunnery".into()
    }

    fn human_name(&self) -> String {
        "Gunnery".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut ship_data = frigate(0);
        ship_data.guns.pop();
        ship_data.guns.pop();
        ship_data.missile_launchers.pop();
        ship_data.acceleration = vector![0.0, 0.0];
        ship::create(
            sim,
            vector![-9000.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship_data,
        );
        let mut rng = new_rng(seed);
        for _ in 0..4 {
            ship::create(
                sim,
                vector![
                    9000.0 + rng.gen_range(-500.0..500.0),
                    -9000.0 + rng.gen_range(-500.0..500.0)
                ],
                vector![
                    0.0 + rng.gen_range(-10.0..10.0),
                    700.0 + rng.gen_range(-300.0..600.0)
                ],
                std::f64::consts::PI,
                target(1),
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS * 2)
    }

    fn solution(&self) -> Code {
        builtin("gunnery")
    }

    fn tick(&mut self, sim: &mut Simulation) {
        let handles = sim.ships.iter().cloned().collect::<Vec<_>>();
        for handle in handles {
            let mut ship = sim.ship_mut(handle);
            if ship.readonly().position().y > self.world_size() * 0.49 {
                let new_position =
                    vector![ship.readonly().position().x, -ship.readonly().position().y];
                ship.body().set_translation(new_position, false);
            }
        }
    }
}
