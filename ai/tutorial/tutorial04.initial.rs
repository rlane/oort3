// Tutorial 04
// Destroy the asteroid. The target is in a random
// location given by the "target()" function.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        torque(1.0);
        fire_gun(0);
    }
}
