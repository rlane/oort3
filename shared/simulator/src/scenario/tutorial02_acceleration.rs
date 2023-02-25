use super::prelude::*;

pub struct Tutorial02 {
    hit_target: bool,
}

impl Tutorial02 {
    const TARGET: Vector2<f64> = vector![1000.0, 0.0];

    pub fn new() -> Self {
        Self { hit_target: false }
    }
}

impl Scenario for Tutorial02 {
    fn name(&self) -> String {
        "tutorial02".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 2: Acceleration".into()
    }

    fn init(&mut self, sim: &mut Simulation, _seed: u32) {
        add_walls(sim);
        let handle = ship::create(
            sim,
            vector![-1000.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            fighter_without_missiles_or_radar(0),
        );
        sim.write_target(handle, Self::TARGET, vector![0.0, 0.0]);
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if let Some(&handle) = sim.ships.iter().next() {
            let ship = sim.ship(handle);
            if (ship.position().vector - Self::TARGET).magnitude() < 50.0 {
                self.hit_target = true;
            }
        }
    }

    fn lines(&self) -> Vec<Line> {
        let mut lines = vec![];
        let center: Point2<f64> = Self::TARGET.into();
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

    fn initial_code(&self) -> Vec<Code> {
        vec![builtin("tutorial/tutorial02.initial")]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial02.solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial03".to_string())
    }
}
