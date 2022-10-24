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
        let dp = target() - position();
        let predicted_dp = dp - velocity() * dp.length() / 1000.0;
        accelerate(0.1 * (dp - velocity()));
        turn_to(predicted_dp.angle());
        fire(0);
    }
}

fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(3.0 * heading_error);
}
