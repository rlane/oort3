use crate::scenario::Status;
use crate::ship::ShipClass;
use crate::simulation::Line;
use crate::vm;
use nalgebra::{Point2, Vector2, Vector4};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Snapshot {
    pub nonce: u32,
    pub time: f64,
    pub status: Status,
    pub ships: Vec<ShipSnapshot>,
    pub bullets: Vec<BulletSnapshot>,
    pub scenario_lines: Vec<Line>,
    pub hits: Vec<Vector2<f64>>,
    pub ships_destroyed: Vec<Vector2<f64>>,
    pub errors: Vec<vm::Error>,
    pub cheats: bool,
    pub debug_lines: BTreeMap<u64, Vec<Line>>,
    pub debug_text: BTreeMap<u64, String>,
    pub timing: Timing,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShipSnapshot {
    pub id: u64,
    pub position: Point2<f64>,
    pub velocity: Vector2<f64>,
    pub heading: f64,
    pub angular_velocity: f64,
    pub team: i32,
    pub class: ShipClass,
    pub health: f64,
    pub energy: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BulletSnapshot {
    pub position: Point2<f64>,
    pub velocity: Vector2<f64>,
    pub color: Vector4<f32>,
    pub ttl: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Timing {
    pub physics: f64,
    pub script: f64,
}

impl Timing {
    pub fn total(&self) -> f64 {
        self.physics + self.script
    }
}

impl std::ops::Add for Timing {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            physics: self.physics + other.physics,
            script: self.script + other.script,
        }
    }
}

impl std::ops::AddAssign for Timing {
    fn add_assign(&mut self, other: Self) {
        *self = self.clone() + other;
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
