pub mod rhai;

use self::rhai::RhaiShipController;
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use nalgebra::Vector2;

pub trait ShipController {
    fn upload_code(&mut self, code: &str);
    fn start(&mut self);
    fn tick(&mut self);
    fn write_target(&mut self, target: Vector2<f64>);
}

pub fn new_ship_controller(handle: ShipHandle, sim: *mut Simulation) -> Box<dyn ShipController> {
    Box::new(RhaiShipController::new(handle, sim))
}
