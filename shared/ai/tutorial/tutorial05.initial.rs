// Tutorial 05
// Destroy the enemy ship. Its location is given by the
// "target" function.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        accelerate(0.1 * (target() - position()));
        fire(0);
    }
}
