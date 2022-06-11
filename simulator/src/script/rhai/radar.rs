use super::vec2::Vec2;
use crate::ship::{ShipAccessorMut, ShipHandle};
use crate::simulation::Simulation;
use rhai::plugin::*;
use std::f64::consts::TAU;

#[export_module]
pub mod plugin {
    #[derive(Copy, Clone)]
    pub struct RadarApi {
        pub handle: ShipHandle,
        pub sim: *mut Simulation,
    }

    impl RadarApi {
        #[allow(clippy::mut_from_ref)]
        fn sim(&self) -> &mut Simulation {
            unsafe { &mut *self.sim }
        }

        fn ship_mut(&self) -> ShipAccessorMut {
            self.sim().ship_mut(self.handle)
        }
    }

    pub fn set_heading(obj: RadarApi, heading: f64) {
        if let Some(radar) = obj.ship_mut().data_mut().radar.as_mut() {
            radar.heading = heading;
        }
    }

    pub fn set_width(obj: RadarApi, width: f64) {
        if let Some(radar) = obj.ship_mut().data_mut().radar.as_mut() {
            radar.width = width.clamp(TAU / 360.0, TAU);
        }
    }

    #[derive(Clone)]
    pub struct ScanResult {
        pub found: bool,
        pub class: String,
        pub position: Vec2,
        pub velocity: Vec2,
    }

    pub fn scan(obj: RadarApi) -> ScanResult {
        if let Some(r) = crate::radar::scan(obj.sim(), obj.handle) {
            ScanResult {
                found: true,
                class: r
                    .class
                    .as_ref()
                    .map_or("unknown".to_string(), |c| c.name().to_string()),
                position: r.position,
                velocity: r.velocity,
            }
        } else {
            ScanResult {
                found: false,
                class: "unknown".to_string(),
                position: Vec2::new(0.0, 0.0),
                velocity: Vec2::new(0.0, 0.0),
            }
        }
    }

    #[rhai_fn(get = "found", pure)]
    pub fn get_found(obj: &mut ScanResult) -> bool {
        obj.found
    }

    #[rhai_fn(get = "class", pure)]
    pub fn get_class(obj: &mut ScanResult) -> String {
        obj.class.clone()
    }

    #[rhai_fn(get = "position", pure)]
    pub fn get_position(obj: &mut ScanResult) -> Vec2 {
        obj.position
    }

    #[rhai_fn(get = "velocity", pure)]
    pub fn get_velocity(obj: &mut ScanResult) -> Vec2 {
        obj.velocity
    }
}
