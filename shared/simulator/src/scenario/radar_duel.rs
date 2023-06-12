use super::prelude::*;

pub struct RadarDuel {
    ship0: Option<ShipHandle>,
    ship1: Option<ShipHandle>,
}

impl RadarDuel {
    pub fn new() -> Self {
        Self {
            ship0: None,
            ship1: None,
        }
    }
}

impl Scenario for RadarDuel {
    fn name(&self) -> String {
        "radar_duel".into()
    }

    fn human_name(&self) -> String {
        "Radar Duel".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        let placements = place_teams(&mut rng, self.world_size());

        for (team, placement) in placements.into_iter().enumerate() {
            let Placement { position, heading } = placement;
            ship::create(
                sim,
                position,
                vector![0.0, 0.0],
                heading,
                fighter_without_missiles(team as i32),
            );
        }
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![
            builtin("tutorial/tutorial_radar_initial"),
            builtin("tutorial/tutorial_radar_solution"),
        ]
    }

    fn solution(&self) -> Code {
        Code::None
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
