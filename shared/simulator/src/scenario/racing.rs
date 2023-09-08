use std::ops::RangeInclusive;

use super::{draw_ngon, prelude::*};

pub struct Checkpoint {
    pos: Point2<f64>,
    radius: f64,
}
pub struct Race {
    players: usize,
    player_ships: Vec<ShipHandle>,
    num_checkpoints_range: RangeInclusive<usize>,
    obstacle_density: f64,
    checkpoints: Vec<Checkpoint>,
    next_checkpoints: Vec<usize>,
    beacon: Option<ShipHandle>,
}

impl Race {
    pub fn new() -> Self {
        Self {
            players: 1,
            player_ships: vec![],
            obstacle_density: 0.0,
            checkpoints: vec![],
            next_checkpoints: vec![],
            num_checkpoints_range: 10..=15,
            beacon: None,
        }
    }
}

impl Scenario for Race {
    fn name(&self) -> String {
        "race".into()
    }

    fn human_name(&self) -> String {
        "Race: Time Attack".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        for idx in 0..rng.gen_range(self.num_checkpoints_range.clone()) {
            let pos = point![
                rng.gen_range(-1000.0..1000.0),
                rng.gen_range(-1000.0..1000.0)
            ];
            let radius = rng.gen_range(30.0..70.0);
            self.checkpoints.push(Checkpoint { pos, radius })
        }

        let goal_pos = self.checkpoints.last().unwrap().pos;
        let first_pos = self.checkpoints.first().unwrap().pos;

        for team in 0..self.players {
            let handle = ship::create(
                sim,
                goal_pos.coords,
                vector![0.0, 0.0],
                goal_pos.coords.angle(&first_pos.coords),
                fighter_without_missiles_or_radar(team as i32),
            );
            self.next_checkpoints.push(0);
            self.player_ships.push(handle);
            //sim.ship_mut(handle)
        }

        {
            let mut data = target_asteroid(4);
            data.radios.push(ship::radio());
            self.beacon = Some(ship::create(
                sim,
                vector![250.0, 0.0],
                vector![0.0, 0.0],
                0.1,
                data,
            ));
        }
    }

    fn tick(&mut self, sim: &mut Simulation) {
        let mut ships = self.player_ships.iter();
        for handle in &self.player_ships {
            let ship = sim.ship(*handle);
            let team = ship.data().team;

            let next_checkpoint = self.next_checkpoints[team as usize];
            if let Some(checkpoint) = self.checkpoints.get(next_checkpoint) {
                if (ship.position().vector - checkpoint.pos.coords).magnitude() < checkpoint.radius
                {
                    self.next_checkpoints[team as usize] += 1;
                }
            }
        }

        if let Some(handle) = self.beacon {
            sim.ship_mut(handle).radio_mut(0).unwrap().sent = Some([1.0, 2.0, 3.0, 4.0]);
        }
    }

    fn lines(&self) -> Vec<Line> {
        let mut lines = vec![];
        let color = vector![1.0, 0.0, 0.0, 1.0];
        for checkpoint in &self.checkpoints {
            let center: Point2<f64> = checkpoint.pos;
            let n = 20;
            let r = checkpoint.radius;

            draw_ngon(&mut lines, n, center, r, color);
        }

        for cs in self.checkpoints.windows(2) {
            lines.push(Line {
                a: cs[0].pos,
                b: cs[1].pos,
                color,
            });
        }

        lines
    }

    fn status(&self, _: &Simulation) -> Status {
        let winner = self
            .next_checkpoints
            .iter()
            .position(|next_checkpoint| *next_checkpoint == self.checkpoints.len());

        match winner {
            Some(team) => Status::Victory { team: team as i32 },
            None => Status::Running,
        }
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![builtin("tutorial/tutorial_acceleration2_initial")]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial_acceleration2_solution")
    }
}
