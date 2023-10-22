use nalgebra::Translation2;
use super::prelude::*;

pub struct Race {
    hit_target: bool,
    target: Option<Point2<f64>>,
}

impl Race {
    pub fn new() -> Self {
        Self {
            hit_target: false,
            target: None,
        }
    }
}

impl Scenario for Race {
    fn name(&self) -> String {
        "race".into()
    }

    fn human_name(&self) -> String {
        "Race".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        self.target = Some(
            Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                .transform_point(&point![rng.gen_range(4500.0..5000.0), 0.0]),
        );
        let handle = ship::create(
            sim,
            Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                .transform_vector(&vector![rng.gen_range(100.0..200.0), 0.0]),
            vector![0.0, 0.0],
            0.0,
            fighter_without_missiles_or_radar(0),
        );

        for _ in 0..200 {
            let p = Translation2::new(self.target.unwrap().x, self.target.unwrap().y)
                .transform_point(&Rotation2::new(rng.gen_range(0.0..std::f64::consts::TAU))
                    .transform_point(&point![rng.gen_range(100.0..3000.0), 0.0]));
            ship::create(
                sim,
                vector![p.x, p.y],
                vector![rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0)],
                rng.gen_range(0.0..(2.0 * std::f64::consts::PI)),
                asteroid(rng.gen_range(0.0..1000.0) as i32),
            );
        }

        sim.write_target(handle, self.target.unwrap().coords, vector![0.0, 0.0]);
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if let Some(&handle) = sim.ships.iter().next() {
            let ship = sim.ship(handle);
            if (ship.position().vector - self.target.unwrap().coords).magnitude() < 50.0 {
                self.hit_target = true;
            }
        }
    }

    fn lines(&self) -> Vec<Line> {
        let mut lines = vec![];
        let center: Point2<f64> = self.target.unwrap();
        let n = 20;
        let r = 50.0;
        let color = if self.hit_target {
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
        lines
    }

    fn status(&self, _: &Simulation) -> Status {
        if self.hit_target {
            Status::Victory { team: 0 }
        } else {
            Status::Running
        }
    }
}
