use super::radar::plugin::ScanResult;
use super::vec2::Vec2;
use crate::script::rhai::radar;
use crate::ship::{ShipAccessor, ShipAccessorMut, ShipClass, ShipHandle};
use crate::simulation::Simulation;
use rhai::plugin::*;
use rhai::Map;

#[export_module]
pub mod plugin {
    #[derive(Copy, Clone)]
    pub struct ShipApi {
        pub handle: ShipHandle,
        pub sim: *mut Simulation,
    }

    impl ShipApi {
        #[allow(clippy::mut_from_ref)]
        pub fn sim(&self) -> &mut Simulation {
            unsafe { &mut *self.sim }
        }

        pub fn ship(&self) -> ShipAccessor {
            self.sim().ship(self.handle)
        }

        pub fn ship_mut(&self) -> ShipAccessorMut {
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

    pub fn fire_gun(obj: ShipApi) {
        obj.ship_mut().fire_gun(0);
    }

    // Backwards compatibility.
    pub fn fire_weapon(obj: ShipApi) {
        obj.ship_mut().fire_gun(0);
    }

    #[rhai_fn(name = "fire_gun")]
    pub fn fire_gun_with_index(obj: ShipApi, index: i64) {
        obj.ship_mut().fire_gun(index);
    }

    pub fn aim_gun(obj: ShipApi, index: i64, angle: f64) {
        obj.ship_mut().aim_gun(index, angle);
    }

    pub fn launch_missile(obj: ShipApi) {
        obj.ship_mut().launch_missile(0, "".to_string());
    }

    #[rhai_fn(name = "launch_missile")]
    pub fn launch_missile_with_index(obj: ShipApi, index: i64) {
        obj.ship_mut().launch_missile(index, "".to_string());
    }

    #[rhai_fn(name = "launch_missile")]
    pub fn launch_missile_with_index_and_map(obj: ShipApi, index: i64, map: Map) {
        obj.ship_mut()
            .launch_missile(index, rhai::format_map_as_json(&map));
    }

    pub fn explode(obj: ShipApi) {
        obj.ship_mut().explode();
    }

    pub fn class(obj: ShipApi) -> String {
        match obj.ship().data().class {
            ShipClass::Fighter => "fighter".to_string(),
            ShipClass::Frigate => "frigate".to_string(),
            ShipClass::Cruiser => "cruiser".to_string(),
            ShipClass::Asteroid { .. } => "asteroid".to_string(),
            ShipClass::Target => "target".to_string(),
            ShipClass::Missile => "missile".to_string(),
            ShipClass::Torpedo => "torpedo".to_string(),
        }
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
