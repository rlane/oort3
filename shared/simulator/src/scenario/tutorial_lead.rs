use super::prelude::*;

pub struct TutorialLead {
    ship_handle: Option<ShipHandle>,
    target_handle: Option<ShipHandle>,
}

impl TutorialLead {
    pub fn new() -> Self {
        Self {
            ship_handle: None,
            target_handle: None,
        }
    }
}

impl Scenario for TutorialLead {
    fn name(&self) -> String {
        "tutorial_lead".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 5: Lead".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut data = fighter_without_missiles_or_radar(0);
        data.fuel = Some(0.0);
        self.ship_handle = Some(ship::create(
            sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            data,
        ));

        let d = 2000.0;
        let l = 2000.0;
        let mut rng = new_rng(seed);
        let target_data = fighter(1);
        let direction = Rotation2::new(rng.gen_range(0.0..TAU));
        let p = direction.transform_vector(&vector![d, -l]);
        let h = direction.angle() + PI / 2.0;
        let v = Rotation2::new(h).transform_vector(&vector![400.0, 0.0]);
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
        vec![builtin("tutorial/tutorial_lead_initial"), builtin("empty")]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial_lead_solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial_deflection".to_string())
    }
}
