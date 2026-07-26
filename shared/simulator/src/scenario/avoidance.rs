use super::prelude::*;
use crate::ship::ShipClass;

pub struct Avoidance {
    hit_target: bool,
    target: Option<Point2<f64>>,
    beacon_ship_handle: Option<ShipHandle>
}
impl Avoidance {
    pub fn new() -> Self {
        Self {
            hit_target: false,
            target: None,
            beacon_ship_handle: None
        }
    }
}

impl Scenario for Avoidance {
    fn name(&self) -> String {
        "avoidance".into()
    }

    fn human_name(&self) -> String {
        "Avoidance".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);
        let placements = place_teams(&mut rng, self.world_size());

        ship::create(
            sim,
            placements[0].position,
            vector![0.0, 0.0],
            placements[0].heading,
            fighter(0)
        );

        let target = placements[1].position;
        self.target = Some(target.into());

        self.beacon_ship_handle = Some(ship::create(
            sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            beacon(2),
        ));

        let s = self.world_size() * 0.1;
        let position = vector![
            rng.random_range(-s..s),
            rng.random_range(-s..s)
        ];
        ship::create(
            sim,
            position,
            vector![0.0, 0.0],
            rng.random_range(0.0..TAU),
            frigate(1)
        );
    }

    fn tick(&mut self, sim: &mut Simulation) {
        let ship = sim.ships.iter()
            .map(|&handle| sim.ship(handle))
            .filter(|ship| ship.data().team == 0)
            .filter(|ship| ship.data().class == ShipClass::Fighter)
            .next();
        let target = self.target.unwrap();
        if let Some(ship) = ship {
            if (ship.position().vector - target.coords).magnitude() < 50.0 {
                self.hit_target = true;
            }
        }

        let beacon = self.beacon_ship_handle.unwrap();
        if sim.ships.contains(beacon) {
            let mut beacon = sim.ship_mut(beacon);
            let radio = &mut beacon.data_mut().radios[0];
            radio.set_channel(1);
            radio.set_sent(Some([target.x, target.y, 0.0, 0.0]));
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
                a: center + Rotation2::new(angle_a).transform_vector(&vector![r, 0.0]),
                b: center + Rotation2::new(angle_b).transform_vector(&vector![r, 0.0]),
                color,
            });
        }
        lines
    }

    fn status(&self, sim: &Simulation) -> Status {
        let alive = sim.ships.iter()
            .map(|&handle| sim.ship(handle))
            .filter(|ship| ship.data().team == 0)
            .filter(|ship| ship.data().class == ShipClass::Fighter)
            .next()
            .is_some();
        if !alive {
            return Status::Failed;
        }
        if self.hit_target {
            return Status::Victory { team: 0 };
        }
        if sim.tick() >= 9999 {
            Status::Draw
        } else {
            Status::Running
        }
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![builtin("challenge/avoidance_initial"), reference_ai(), empty_ai()]
    }

    fn solution(&self) -> Code {
        builtin("challenge/avoidance_solution")
    }
}
