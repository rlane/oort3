// Tutorial: Rotation (solution)
// Destroy the asteroid. The target is in a random
// location given by the "target()" function.
//
// You can get the angle between a vector and the x-axis: vec2(a, b).angle()
// And compare an angle with your ship's heading: angle_diff(heading(), angle)
//
// If angle_diff returns a positive number you need to turn left, or if negative then right.
// The turn function takes a speed argument, where positive speeds result in turning left
// and negative speeds will turn right.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        // Hint: "angle_diff(heading(), (target() - position()).angle())"
        // returns the direction your ship needs to turn to face the target.
        let heading_error = angle_diff(heading(), (target() - position()).angle());
        turn(heading_error);
        fire(0);
    }
}
