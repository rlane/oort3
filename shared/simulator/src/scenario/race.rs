use super::prelude::*;

pub struct Target {
    hit: bool,
    position: Point2<f64>,
}

pub struct Race {
    targets: Vec<Target>,
}

impl Race {
    pub fn new() -> Self {
        Self {
            targets: Vec::new()
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
                position: point![rng.gen_range(-self.world_size()..self.world_size()),
                rng.gen_range(-self.world_size()..self.world_size())] / 2.0,
            });
        }

        ship::create(
            sim,
            Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                .transform_vector(&vector![rng.gen_range(100.0..200.0), 0.0]),
            vector![0.0, 0.0],
            0.0,
            fighter_without_missiles_or_radar(0),
        );

        for _ in 0..200 {
            let p = point![rng.gen_range(-self.world_size()..self.world_size()),
                rng.gen_range(-self.world_size()..self.world_size())] / 2.0;
            ship::create(
                sim,
                vector![p.x, p.y],
                vector![rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0)],
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                asteroid(rng.gen_range(0.0..1000.0) as i32),
            );
        }
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if let Some(&handle) = sim.ships.iter().next() {
            let ship = sim.ship(handle);

            for target in &mut self.targets {
                let dx = target.position.x - ship.position().x;
                let dy = target.position.y - ship.position().y;
                if (dx * dx + dy * dy) < 2500.0 {
                    target.hit = true;
                }
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

    fn world_size(&self) -> f64 {
        10e3
    }

    fn status(&self, _: &Simulation) -> Status {
        if self.targets.iter().all(|t| t.hit) {
            Status::Victory { team: 0 }
        } else {
            Status::Running
        }
    }
}
