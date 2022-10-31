// Tutorial 05
// Destroy the enemy ship. Its position is given by the "target" function and velocity by the
// "target_velocity" function.
use oort_api::prelude::*;

const BULLET_SPEED: f64 = 1000.0;

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
