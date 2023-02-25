use super::prelude::*;

pub struct TutorialDeflection {
    ship_handle: Option<ShipHandle>,
    target_handle: Option<ShipHandle>,
}

impl TutorialDeflection {
    pub fn new() -> Self {
        Self {
            ship_handle: None,
            target_handle: None,
        }
    }
}

impl Scenario for TutorialDeflection {
    fn name(&self) -> String {
        "tutorial_deflection".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 5: Deflection".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        self.ship_handle = Some(ship::create(
            sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            fighter_without_missiles_or_radar(0),
        ));

        let mut rng = new_rng(seed);
        let mut target_data = fighter(1);
        target_data.health *= 2.0;
        let p = Rotation2::new(rng.gen_range(0.0..TAU)).transform_vector(&vector![1000.0, 0.0]);
        let h = rng.gen_range(0.0..std::f64::consts::TAU);
        let v = Rotation2::new(h).transform_vector(&vector![200.0, 0.0]);
        self.target_handle = Some(ship::create(sim, p, v, h, target_data));

        let target_position = sim.ship(self.target_handle.unwrap()).position();
        let target_velocity = sim.ship(self.target_handle.unwrap()).velocity();
        sim.write_target(
            self.ship_handle.unwrap(),
            target_position.vector,
            target_velocity,
        );
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if sim.ships.len() < 2 {
            return;
        }
        {
            let target_position = sim.ship(self.target_handle.unwrap()).position();
            let target_velocity = sim.ship(self.target_handle.unwrap()).velocity();
            sim.write_target(
                self.ship_handle.unwrap(),
                target_position.vector,
                target_velocity,
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tutorial_victory(sim, DEFAULT_TUTORIAL_MAX_TICKS)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![
            builtin("tutorial/tutorial_deflection.initial"),
            builtin("tutorial/tutorial_deflection.enemy"),
        ]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial_deflection.solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial_radar".to_string())
    }

    fn previous_names(&self) -> Vec<String> {
        vec!["tutorial05".into()]
    }
}
