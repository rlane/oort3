use crate::scenario::Status;
use crate::ship::ShipClass;
use crate::simulation::{Line, Particle};
use crate::vm;
use nalgebra::{Point2, Vector2};
use oort_api::{Ability, Text};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Snapshot {
    pub nonce: u32,
    pub time: f64,
    pub score_time: f64,
    pub status: Status,
    pub ships: Vec<ShipSnapshot>,
    pub bullets: Vec<BulletSnapshot>,
    pub scenario_lines: Vec<Line>,
    pub particles: Vec<Particle>,
    pub errors: Vec<vm::Error>,
    pub cheats: bool,
    pub debug_lines: Vec<(u64, Vec<Line>)>,
    pub debug_text: BTreeMap<u64, String>,
    pub drawn_text: BTreeMap<u64, Vec<Text>>,
    pub timing: Timing,
    pub world_size: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShipSnapshot {
    pub id: u64,
    pub position: Point2<f64>,
    pub velocity: Vector2<f64>,
    pub acceleration: Vector2<f64>,
    pub heading: f64,
    pub angular_velocity: f64,
    pub team: i32,
    pub class: ShipClass,
    pub health: f64,
    pub fuel: Option<f64>,
    pub active_abilities: Vec<Ability>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BulletSnapshot {
    pub position: Point2<f64>,
    pub velocity: Vector2<f64>,
    pub color: u32,
    pub ttl: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Timing {
    pub physics: f64,
    pub collision: f64,
    pub radar: f64,
    pub radio: f64,
    pub vm: f64,
    pub ship: f64,
    pub bullet: f64,
    pub scenario: f64,
}

impl Timing {
    pub fn total(&self) -> f64 {
        self.physics
            + self.collision
            + self.radar
            + self.radio
            + self.vm
            + self.ship
            + self.bullet
            + self.scenario
    }
}

impl std::ops::Add for Timing {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            physics: self.physics + other.physics,
            collision: self.collision + other.collision,
            radar: self.radar + other.radar,
            radio: self.radio + other.radio,
            vm: self.vm + other.vm,
            ship: self.ship + other.ship,
            bullet: self.bullet + other.bullet,
            scenario: self.scenario + other.scenario,
        }
    }
}

impl std::ops::AddAssign for Timing {
    fn add_assign(&mut self, other: Self) {
        *self = self.clone() + other;
    }
}

impl std::ops::Mul<f64> for Timing {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Self {
            physics: self.physics * other,
            collision: self.collision * other,
            radar: self.radar * other,
            radio: self.radio * other,
            vm: self.vm * other,
            ship: self.ship * other,
            bullet: self.bullet * other,
            scenario: self.scenario * other,
        }
    }
}

pub fn interpolate(snapshot: &mut Snapshot, dt: f64) {
    snapshot.time += dt;

    for ship in snapshot.ships.iter_mut() {
        ship.position += ship.velocity * dt;
        ship.heading += ship.angular_velocity * dt;
    }

    for bullet in snapshot.bullets.iter_mut() {
        bullet.position += bullet.velocity * dt;
    }
}
