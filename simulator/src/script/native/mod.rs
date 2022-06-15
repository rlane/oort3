mod reference;

use super::{ShipController, TeamController};
use crate::radar::ScanResult;
use crate::ship::{ShipAccessor, ShipAccessorMut, ShipClass, ShipHandle};
use crate::simulation::Simulation;
use std::f64::consts::TAU;

pub type Vec2 = nalgebra::Vector2<f64>;

pub struct NativeTeamController {}

impl NativeTeamController {
    pub fn create() -> Result<Box<dyn TeamController>, super::Error> {
        log::info!("Creating NativeTeamController");
        Ok(Box::new(NativeTeamController {}))
    }
}

impl TeamController for NativeTeamController {
    fn create_ship_controller(
        &mut self,
        handle: ShipHandle,
        sim: &mut Simulation,
        orders: String,
    ) -> Result<Box<dyn ShipController>, super::Error> {
        let api = Api { handle, sim };
        let native_ship =
            reference::NativeShip::new(api, orders, sim.seed() as u64 ^ u64::from(handle));
        Ok(Box::new(NativeShipController { native_ship }))
    }
}

struct NativeShipController {
    native_ship: reference::NativeShip,
}

impl ShipController for NativeShipController {
    fn tick(&mut self) -> Result<(), super::Error> {
        self.native_ship.tick();
        Ok(())
    }

    fn write_target(&mut self, _target: Vec2) {}
}

#[derive(Copy, Clone)]
pub struct Api {
    pub handle: ShipHandle,
    pub sim: *mut Simulation,
}

impl Api {
    #[allow(clippy::mut_from_ref)]
    fn sim(&self) -> &mut Simulation {
        unsafe { &mut *self.sim }
    }

    fn ship(&self) -> ShipAccessor {
        self.sim().ship(self.handle)
    }

    fn ship_mut(&self) -> ShipAccessorMut {
        self.sim().ship_mut(self.handle)
    }

    // Reads

    pub fn class(&self) -> ShipClass {
        self.ship().data().class
    }

    pub fn position(&self) -> Vec2 {
        self.ship().position().vector
    }

    pub fn velocity(&self) -> Vec2 {
        self.ship().velocity()
    }

    pub fn heading(&self) -> f64 {
        self.ship().heading()
    }

    pub fn angular_velocity(&self) -> f64 {
        self.ship().angular_velocity()
    }

    // Writes

    pub fn accelerate(&self, acc: Vec2) {
        self.ship_mut().accelerate(acc);
    }

    pub fn torque(&self, angular_acceleration: f64) {
        self.ship_mut().torque(angular_acceleration);
    }

    pub fn fire_gun(&self, index: i64) {
        self.ship_mut().fire_gun(index);
    }

    pub fn aim_gun(&self, index: i64, angle: f64) {
        self.ship_mut().aim_gun(index, angle);
    }

    pub fn launch_missile(&self, index: i64, orders: String) {
        self.ship_mut().launch_missile(index, orders);
    }

    pub fn explode(&self) {
        self.ship_mut().explode();
    }

    // Radar

    pub fn set_radar_heading(&self, heading: f64) {
        if let Some(radar) = self.ship_mut().data_mut().radar.as_mut() {
            radar.heading = heading;
        }
    }

    pub fn set_radar_width(&self, width: f64) {
        if let Some(radar) = self.ship_mut().data_mut().radar.as_mut() {
            radar.width = width.clamp(TAU / 360.0, TAU);
        }
    }

    pub fn scan(&self) -> Option<ScanResult> {
        crate::radar::scan(self.sim(), self.handle)
    }
}

pub trait Vec2Extras {
    fn angle0(&self) -> f64;
    fn distance(&self, other: Vec2) -> f64;
    fn rotate(&self, angle: f64) -> Vec2;
}

impl Vec2Extras for Vec2 {
    fn distance(&self, other: Vec2) -> f64 {
        self.metric_distance(&other)
    }

    fn angle0(&self) -> f64 {
        let mut a = self.y.atan2(self.x);
        if a < 0.0 {
            a += std::f64::consts::TAU;
        }
        a
    }

    fn rotate(&self, angle: f64) -> Vec2 {
        nalgebra::Rotation2::new(angle).transform_vector(self)
    }
}

pub fn vec2(x: f64, y: f64) -> Vec2 {
    Vec2::new(x, y)
}

pub fn normalize_angle(a: f64) -> f64 {
    let mut a = a;
    if a.abs() > TAU {
        a %= TAU;
    }
    if a < 0.0 {
        a += TAU;
    }
    a
}

pub fn angle_diff(a: f64, b: f64) -> f64 {
    use std::f64::consts::PI;
    let c = normalize_angle(b - a);
    if c > PI {
        c - TAU
    } else {
        c
    }
}

mod prelude {
    pub use super::{angle_diff, normalize_angle, vec2, Api, Vec2, Vec2Extras};
    pub use crate::radar::ScanResult;
    pub use crate::ship::ShipClass;
}
