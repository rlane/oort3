pub mod rhai;

use self::rhai::RhaiTeamController;
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    pub line: usize,
    pub msg: String,
}

pub trait TeamController {
    fn create_ship_controller(
        &mut self,
        handle: ShipHandle,
        sim: *mut Simulation,
    ) -> Result<Box<dyn ShipController>, Error>;
}

pub trait ShipController {
    fn tick(&mut self) -> Result<(), Error>;
    fn write_target(&mut self, target: Vector2<f64>);
}

pub fn new_team_controller(code: &str) -> Result<Box<dyn TeamController>, Error> {
    RhaiTeamController::create(code)
}
