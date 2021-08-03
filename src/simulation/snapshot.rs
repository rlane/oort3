use crate::simulation::Line;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub debug_lines: Vec<Line>,
    pub scenario_lines: Vec<Line>,
}
