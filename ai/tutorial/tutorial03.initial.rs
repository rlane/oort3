// Tutorial 03
// Fly through the target circle. The target is in a random
// location given by the "target" function.
// Hint: Look for a "position" function in the documentation, linked at
// the top of the screen.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        accelerate(vec2(100.0, 0.0));
    }
}
