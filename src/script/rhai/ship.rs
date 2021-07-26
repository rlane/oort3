use super::vec2::Vec2;
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use nalgebra::{vector, Point2};
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

    #[derive(Copy, Clone)]
    pub struct ScanResult {
        pub found: bool,
        pub position: Vec2,
        pub velocity: Vec2,
    }

    #[rhai_fn(get = "found", pure)]
    pub fn get_found(obj: &mut ScanResult) -> bool {
        obj.found
    }

    #[rhai_fn(get = "position", pure)]
    pub fn get_position(obj: &mut ScanResult) -> Vec2 {
        obj.position
    }

    #[rhai_fn(get = "velocity", pure)]
    pub fn get_velocity(obj: &mut ScanResult) -> Vec2 {
        obj.velocity
    }

    pub fn scan(ship: Ship) -> ScanResult {
        let sim = ship.sim();
        let own_team = sim.ship(ship.handle).data().team;
        let own_position: Point2<f64> = sim.ship(ship.handle).position().vector.into();
        let mut result = ScanResult {
            found: false,
            position: vector![0.0, 0.0],
            velocity: vector![0.0, 0.0],
        };
        let mut best_distance = 0.0;
        for &other in sim.ships.iter() {
            if sim.ship(other).data().team == own_team {
                continue;
            }
            let other_position: Point2<f64> = sim.ship(other).position().vector.into();
            let distance = nalgebra::distance(&own_position, &other_position);
            if !result.found || distance < best_distance {
                result = ScanResult {
                    found: true,
                    position: other_position.coords,
                    velocity: sim.ship(other).velocity(),
                };
                best_distance = distance;
            }
        }
        result
    }
}
