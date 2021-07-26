use super::radar::plugin::ScanResult;
use super::vec2::Vec2;
use crate::script::rhai::radar;
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use rhai::plugin::*;

#[export_module]
pub mod plugin {
    #[derive(Copy, Clone)]
    pub struct Ship {
        pub handle: ShipHandle,
        pub sim: *mut Simulation,
    }

    impl Ship {
        #[allow(clippy::mut_from_ref)]
        fn sim(&self) -> &mut Simulation {
            unsafe { &mut *self.sim }
        }
    }

    pub fn position(ship: Ship) -> Vec2 {
        ship.sim().ship(ship.handle).position().vector
    }

    pub fn velocity(ship: Ship) -> Vec2 {
        ship.sim().ship(ship.handle).velocity()
    }

    pub fn heading(ship: Ship) -> f64 {
        ship.sim().ship(ship.handle).heading()
    }

    pub fn angular_velocity(ship: Ship) -> f64 {
        ship.sim().ship(ship.handle).angular_velocity()
    }

    pub fn accelerate(ship: Ship, acceleration: Vec2) {
        ship.sim().ship_mut(ship.handle).accelerate(acceleration);
    }

    pub fn torque(ship: Ship, acceleration: f64) {
        ship.sim().ship_mut(ship.handle).torque(acceleration);
    }

    pub fn fire_weapon(ship: Ship) {
        ship.sim().ship_mut(ship.handle).fire_weapon(0);
    }

    pub fn fire_weapon_with_index(ship: Ship, index: i64) {
        ship.sim().ship_mut(ship.handle).fire_weapon(index);
    }

    pub fn explode(ship: Ship) {
        ship.sim().ship_mut(ship.handle).explode();
    }

    // Backwards compatibility.
    pub fn scan(ship: Ship) -> ScanResult {
        let radar = radar::plugin::Radar {
            sim: ship.sim,
            handle: ship.handle,
        };
        radar::plugin::scan(radar)
    }
}
