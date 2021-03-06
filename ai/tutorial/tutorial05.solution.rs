// Tutorial 05
// Destroy the enemy ship. Its location is given by the
// "target" function.
use crate::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        accelerate(0.1 * (target() - position() - velocity()).rotate(-heading()));
        turn_to((target() - position()).angle());
        fire_gun(0);
    }
}

fn turn_to(target_heading: f64) {
    torque(3.0 * angle_diff(heading(), target_heading) - angular_velocity());
}
