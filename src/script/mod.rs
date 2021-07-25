pub mod rhai;
pub mod rhai_random;

use self::rhai::RhaiShipController;
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub line: usize,
    pub msg: String,
}

pub trait ShipController {
    fn upload_code(&mut self, code: &str) -> Result<(), Error>;
    fn start(&mut self) -> Result<(), Error>;
    fn tick(&mut self) -> Result<(), Error>;
    fn write_target(&mut self, target: Vector2<f64>);
}

pub fn new_ship_controller(handle: ShipHandle, sim: *mut Simulation) -> Box<dyn ShipController> {
    Box::new(RhaiShipController::new(handle, sim))
}
