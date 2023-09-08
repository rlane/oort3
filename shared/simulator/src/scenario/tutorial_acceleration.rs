use super::{draw_ngon, prelude::*};

pub struct TutorialAcceleration {
    hit_target: bool,
}

impl TutorialAcceleration {
    const TARGET: Vector2<f64> = vector![250.0, 0.0];

    pub fn new() -> Self {
        Self { hit_target: false }
    }
}

impl Scenario for TutorialAcceleration {
    fn name(&self) -> String {
        "tutorial_acceleration".into()
    }

    fn human_name(&self) -> String {
        "Tutorial 2: Acceleration".into()
    }

    fn init(&mut self, sim: &mut Simulation, _seed: u32) {
        let handle = ship::create(
            sim,
            vector![-250.0, 0.0],
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
        draw_ngon(&mut lines, n, center, r, color);

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
        vec![builtin("tutorial/tutorial_acceleration_initial")]
    }

    fn solution(&self) -> Code {
        builtin("tutorial/tutorial_acceleration_solution")
    }

    fn next_scenario(&self) -> Option<String> {
        Some("tutorial_acceleration2".to_string())
    }

    fn previous_names(&self) -> Vec<String> {
        vec!["tutorial02".into()]
    }
}
