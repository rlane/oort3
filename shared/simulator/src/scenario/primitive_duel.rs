use super::prelude::*;

pub struct PrimitiveDuel {
    ship0: Option<ShipHandle>,
    ship1: Option<ShipHandle>,
}

impl PrimitiveDuel {
    pub fn new() -> Self {
        Self {
            ship0: None,
            ship1: None,
        }
    }
}

impl Scenario for PrimitiveDuel {
    fn name(&self) -> String {
        "primitive_duel".into()
    }

    fn human_name(&self) -> String {
        "Primitive Duel".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        let placements = place_teams(&mut rng, self.world_size());

        for (team, placement) in placements.into_iter().enumerate() {
            let Placement { position, heading } = placement;
            let handle = ship::create(
                sim,
                position,
                vector![0.0, 0.0],
                heading,
                fighter_without_missiles_or_radar(team as i32),
            );
            if team == 0 {
                self.ship0 = Some(handle);
            } else {
                self.ship1 = Some(handle);
            }
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![
            builtin("tutorial/tutorial_deflection_initial"),
            builtin("tutorial/tutorial_deflection_solution"),
        ]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }

    fn is_tournament(&self) -> bool {
        true
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if !sim.ships.contains(self.ship0.unwrap()) || !sim.ships.contains(self.ship1.unwrap()) {
            return;
        }

        let ship0_position = sim.ship(self.ship0.unwrap()).position().vector;
        let ship0_velocity = sim.ship(self.ship0.unwrap()).velocity();
        let ship1_position = sim.ship(self.ship1.unwrap()).position().vector;
        let ship1_velocity = sim.ship(self.ship1.unwrap()).velocity();

        sim.write_target(self.ship0.unwrap(), ship1_position, ship1_velocity);
        sim.write_target(self.ship1.unwrap(), ship0_position, ship0_velocity);
    }
}
