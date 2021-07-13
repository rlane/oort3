pub mod rhai;

use self::rhai::RhaiShipController;
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;

pub trait ShipController {
    fn upload_code(&mut self, code: &str);
    fn start(&mut self);
    fn tick(&mut self);
}

pub fn new_ship_controller(handle: ShipHandle, sim: *mut Simulation) -> Box<dyn ShipController> {
    Box::new(RhaiShipController::new(handle, sim))
}
