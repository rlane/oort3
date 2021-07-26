use super::radar::plugin::ScanResult;
use super::vec2::Vec2;
use crate::script::rhai::radar;
use crate::simulation::ship::{ShipAccessor, ShipAccessorMut, ShipHandle};
use crate::simulation::Simulation;
use rhai::plugin::*;

#[export_module]
pub mod plugin {
    #[derive(Copy, Clone)]
    pub struct ShipApi {
        pub handle: ShipHandle,
        pub sim: *mut Simulation,
    }

    impl ShipApi {
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
    }

    pub fn position(obj: ShipApi) -> Vec2 {
        obj.ship().position().vector
    }

    pub fn velocity(obj: ShipApi) -> Vec2 {
        obj.ship().velocity()
    }

    pub fn heading(obj: ShipApi) -> f64 {
        obj.ship().heading()
    }

    pub fn angular_velocity(obj: ShipApi) -> f64 {
        obj.ship().angular_velocity()
    }

    pub fn accelerate(obj: ShipApi, acceleration: Vec2) {
        obj.ship_mut().accelerate(acceleration);
    }

    pub fn torque(obj: ShipApi, angular_acceleration: f64) {
        obj.ship_mut().torque(angular_acceleration);
    }

    pub fn fire_weapon(obj: ShipApi) {
        obj.ship_mut().fire_weapon(0);
    }

    pub fn fire_weapon_with_index(obj: ShipApi, index: i64) {
        obj.ship_mut().fire_weapon(index);
    }

    pub fn explode(obj: ShipApi) {
        obj.ship_mut().explode();
    }

    // Backwards compatibility.
    pub fn scan(obj: ShipApi) -> ScanResult {
        let radar = radar::plugin::RadarApi {
            sim: obj.sim,
            handle: obj.handle,
        };
        radar::plugin::scan(radar)
    }
}
