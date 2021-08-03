use crate::simulation::ship::ShipClass;
use crate::simulation::Line;
use nalgebra::{Point2, Vector2};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub time: f64,
    pub ships: Vec<ShipSnapshot>,
    pub debug_lines: Vec<Line>,
    pub scenario_lines: Vec<Line>,
    pub hits: Vec<Vector2<f64>>,
    pub ships_destroyed: Vec<Vector2<f64>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShipSnapshot {
    pub id: u64,
    pub position: Point2<f64>,
    pub heading: f64,
    pub team: i32,
    pub class: ShipClass,
}
