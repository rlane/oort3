use super::prelude::*;

pub struct Target {
    hit: bool,
    position: Point2<f64>,
}

pub struct Race {
    targets: Vec<Target>,
    beacon_ship_handle: Option<ShipHandle>,
    player_ship_handle: Option<ShipHandle>,
}

impl Race {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            beacon_ship_handle: None,
            player_ship_handle: None,
        }
    }
}

impl Scenario for Race {
    fn name(&self) -> String {
        "race".into()
    }

    fn human_name(&self) -> String {
        "Asteroid Race".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);

        for _ in 0..3 {
            self.targets.push(Target {
                hit: false,
                position: point![
                    rng.gen_range(-self.world_size()..self.world_size()),
                    rng.gen_range(-self.world_size()..self.world_size())
                ] / 3.0,
            });
        }

        self.beacon_ship_handle = Some(ship::create(
            sim,
            vector![self.world_size() / 2.2, self.world_size() / 2.2],
            vector![0.0, 0.0],
            0.0,
            beacon(1),
        ));

        self.player_ship_handle = Some(ship::create(
            sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            fighter(0),
        ));

        let mut big_asteroid_positions: Vec<Vector2<f64>> = vec![];
        for i in 0..10 {
            let p = vector![
                rng.gen_range(-self.world_size()..self.world_size()),
                rng.gen_range(-self.world_size()..self.world_size())
            ] / 2.0;
            if big_asteroid_positions
                .iter()
                .any(|&p2| (p - p2).norm() < 1200.0)
                || p.norm() < 2000.0
            {
                continue;
            }
            big_asteroid_positions.push(p);
            ship::create(
                sim,
                p,
                vector![rng.gen_range(-3.0..3.0), rng.gen_range(-3.0..3.0)],
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                big_asteroid(i),
            );
        }

        for i in 0..100 {
            let p = vector![
                rng.gen_range(-self.world_size()..self.world_size()),
                rng.gen_range(-self.world_size()..self.world_size())
            ] / 2.0;
            if big_asteroid_positions
                .iter()
                .any(|&p2| (p - p2).norm() < 700.0)
            {
                continue;
            }
            ship::create(
                sim,
                p,
                vector![rng.gen_range(-30.0..30.0), rng.gen_range(-30.0..30.0)],
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                asteroid(i),
            );
        }
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if sim.ships.contains(self.player_ship_handle.unwrap()) {
            let player_ship = sim.ship(self.player_ship_handle.unwrap());

            for target in &mut self.targets {
                let dx = target.position.x - player_ship.position().x;
                let dy = target.position.y - player_ship.position().y;
                if (dx * dx + dy * dy).sqrt() < 50.0 {
                    target.hit = true;
                }
            }
        }

        if sim.ships.contains(self.beacon_ship_handle.unwrap()) {
            for (i, (radio, target)) in sim
                .ship_mut(self.beacon_ship_handle.unwrap())
                .data_mut()
                .radios
                .iter_mut()
                .zip(self.targets.iter())
                .enumerate()
            {
                radio.set_channel(i);
                radio.set_sent(Some([target.position.x, target.position.y, 0.0, 0.0]));
            }
        }
    }

    fn lines(&self) -> Vec<Line> {
        let mut lines = vec![];
        let n = 20;
        let r = 50.0;

        for target in &self.targets {
            let center = target.position;
            let color = if target.hit {
                vector![0.0, 1.0, 0.0, 1.0]
            } else {
                vector![1.0, 0.0, 0.0, 1.0]
            };
            for i in 0..n {
                let frac = (i as f64) / (n as f64);
                let angle_a = std::f64::consts::TAU * frac;
                let angle_b = std::f64::consts::TAU * (frac + 1.0 / n as f64);
                lines.push(Line {
                    a: center + vector![r * angle_a.cos(), r * angle_a.sin()],
                    b: center + vector![r * angle_b.cos(), r * angle_b.sin()],
                    color,
                });
            }
        }

        lines
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![builtin("challenge/race_initial"), empty_ai()]
    }

    fn solution(&self) -> Code {
        builtin("challenge/race_solution")
    }
    fn world_size(&self) -> f64 {
        10e3
    }

    fn status(&self, sim: &Simulation) -> Status {
        if self.targets.iter().all(|t| t.hit) {
            Status::Victory { team: 0 }
        } else if !sim.ships.contains(self.player_ship_handle.unwrap()) {
            Status::Failed
        } else {
            Status::Running
        }
    }
}
