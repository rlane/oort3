use super::prelude::*;
use crate::{bullet, simulation};

pub struct TestScenario {}

impl Scenario for TestScenario {
    fn name(&self) -> String {
        "test".into()
    }

    fn init(&mut self, _sim: &mut Simulation, _seed: u32) {}

    fn world_size(&self) -> f64 {
        simulation::MAX_WORLD_SIZE
    }
}

pub struct BasicScenario {}

impl Scenario for BasicScenario {
    fn name(&self) -> String {
        "basic".into()
    }

    fn init(&mut self, sim: &mut Simulation, _seed: u32) {
        ship::create(
            sim,
            vector![-100.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            fighter(0),
        );
        ship::create(
            sim,
            vector![100.0, 0.0],
            vector![0.0, 0.0],
            std::f64::consts::PI,
            fighter(1),
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }
}

pub struct MissileTest {
    target: Option<ShipHandle>,
    rng: SeededRng,
    current_iteration: i64,
    tick_in_iteration: i64,
    acc: Vector2<f64>,
}

impl MissileTest {
    const MAX_ITERATIONS: i64 = 10;
    const MAX_ACCELERATION: f64 = 60.0;

    pub fn new() -> Self {
        Self {
            target: None,
            rng: new_rng(0),
            current_iteration: 0,
            tick_in_iteration: 0,
            acc: vector![0.0, 0.0],
        }
    }
}

impl Scenario for MissileTest {
    fn name(&self) -> String {
        "missile_test".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        log::info!("Running MissileTest iteration {}", self.current_iteration);
        let mut missile_data = missile(0);
        missile_data.ttl = None;

        self.rng = new_rng((seed % 1000) * 1000 + self.current_iteration as u32);
        let d = 4000.0;
        let target_p: Vector2<f64> = vector![self.rng.gen_range(-d..d), self.rng.gen_range(-d..d)];
        let s = 500.0;
        let target_v: Vector2<f64> = vector![self.rng.gen_range(-s..s), self.rng.gen_range(-s..s)];

        if let Some(radar) = missile_data.radar.as_mut() {
            radar.heading = target_p.angle(&vector![0.0, 0.0]);
            radar.width = TAU / 128.0;
        }

        ship::create(
            sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            target_p.y.atan2(target_p.x),
            missile_data,
        );
        let mut target_data = target(1);
        target_data.max_forward_acceleration = Self::MAX_ACCELERATION;
        target_data.max_backward_acceleration = Self::MAX_ACCELERATION;
        target_data.max_lateral_acceleration = Self::MAX_ACCELERATION;
        target_data.radar_cross_section = 1e6;
        self.target = Some(ship::create(
            sim,
            vector![target_p.x, target_p.y],
            vector![target_v.x, target_v.y],
            0.0,
            target_data,
        ));
    }

    fn tick(&mut self, sim: &mut Simulation) {
        let target = self.target.unwrap();
        if !sim.ships.contains(target) && self.current_iteration < MissileTest::MAX_ITERATIONS {
            self.current_iteration += 1;
            self.tick_in_iteration = 0;
            while !sim.bullets.is_empty() {
                bullet::tick(sim);
            }
            self.init(sim, 0);
        } else if sim.ships.contains(target) {
            if (self.tick_in_iteration % 60) == 0 {
                self.acc = Rotation2::new(self.rng.gen_range(0.0..std::f64::consts::TAU))
                    .transform_vector(&vector![Self::MAX_ACCELERATION, 0.0]);
            }
            sim.ship_mut(target).accelerate(self.acc);
        }
        self.tick_in_iteration += 1;
    }

    fn status(&self, sim: &Simulation) -> Status {
        if self.tick_in_iteration > 2000 {
            Status::Failed
        } else if sim.ships.contains(self.target.unwrap())
            || self.current_iteration < MissileTest::MAX_ITERATIONS
        {
            Status::Running
        } else {
            Status::Victory { team: 0 }
        }
    }

    fn solution(&self) -> Code {
        builtin("missile")
    }
}

pub struct FrigateVsCruiser {}

impl FrigateVsCruiser {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for FrigateVsCruiser {
    fn name(&self) -> String {
        "frigate_vs_cruiser".into()
    }

    fn init(&mut self, sim: &mut Simulation, _seed: u32) {
        ship::create(
            sim,
            vector![-1000.0, -500.0],
            vector![0.0, 0.0],
            0.0,
            frigate(0),
        );
        ship::create(
            sim,
            vector![1000.0, 500.0],
            vector![0.0, 0.0],
            std::f64::consts::PI,
            cruiser(1),
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![reference_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }
}

pub struct CruiserVsFrigate {}

impl CruiserVsFrigate {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scenario for CruiserVsFrigate {
    fn name(&self) -> String {
        "cruiser_vs_frigate".into()
    }

    fn init(&mut self, sim: &mut Simulation, _seed: u32) {
        ship::create(
            sim,
            vector![-1000.0, -500.0],
            vector![0.0, 0.0],
            0.0,
            cruiser(0),
        );
        ship::create(
            sim,
            vector![1000.0, 500.0],
            vector![0.0, 0.0],
            std::f64::consts::PI,
            frigate(1),
        );
    }

    fn status(&self, sim: &Simulation) -> Status {
        check_tournament_victory(sim)
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![reference_ai(), reference_ai()]
    }

    fn solution(&self) -> Code {
        reference_ai()
    }
}

pub struct FrigatePointDefense {}

impl Scenario for FrigatePointDefense {
    fn name(&self) -> String {
        "frigate_point_defense".into()
    }

    fn init(&mut self, sim: &mut Simulation, seed: u32) {
        let mut rng = new_rng(seed);

        let mut data = frigate(0);
        data.missile_launchers.clear();
        ship::create(sim, vector![0.0, 0.0], vector![0.0, 0.0], 0.0, data);

        for i in 1..10 {
            let distance = (i as f64) * 1000.0;
            let angle = rng.gen_range(0.0..TAU);
            let position = Rotation2::new(angle) * vector![distance, 0.0];
            let velocity = Rotation2::new(angle) * vector![0.0, rng.gen_range(-2000.0..2000.0)];
            let mut data = missile(1);
            data.ttl = None;
            ship::create(sim, position, velocity, angle + PI, data);
        }
    }

    fn status(&self, _sim: &Simulation) -> Status {
        Status::Running
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![empty_ai(), reference_ai()]
    }
}

pub struct RadarTest {}

impl Scenario for RadarTest {
    fn name(&self) -> String {
        "radar_test".into()
    }

    fn init(&mut self, sim: &mut Simulation, _seed: u32) {
        for (j, class) in [missile, torpedo, fighter, frigate, cruiser]
            .iter()
            .enumerate()
        {
            let y = -2.5e3 + j as f64 * 1000.0;
            let mut data = class(0);
            data.ttl = None;
            ship::create(sim, vector![-50e3, y], vector![0.0, 0.0], 0.0, data);
            ship::create(sim, vector![-40e3, y], vector![0.0, 0.0], PI, fighter(1));
        }
    }

    fn status(&self, _sim: &Simulation) -> Status {
        Status::Running
    }

    fn initial_code(&self) -> Vec<Code> {
        vec![builtin("radar_test"), builtin("radar_test.enemy")]
    }

    fn world_size(&self) -> f64 {
        200e3
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if sim.tick() > 120 && sim.tick() % 120 == 0 {
            let handles: Vec<_> = sim.ships.iter().cloned().collect();
            for handle in handles {
                if sim.ship(handle).data().team == 1 {
                    let mut ship = sim.ship_mut(handle);
                    let body = ship.body();
                    if body.translation().x < 35e3 {
                        body.set_translation(body.translation() + vector![10e3, 0.0], true);
                    } else {
                        body.set_translation(vector![-40e3, body.translation().y], true);
                    }
                }
            }
        }
    }
}
