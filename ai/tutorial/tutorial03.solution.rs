// Tutorial 03
// Fly through the target circle. The target is in a random
// location given by the "target" function.
// Hint: Look for a "position" function in the documentation, linked at
// the top of the screen.
use crate::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        let dp = target() - position();
        accelerate(dp.rotate(-heading()).normalize() * 30.0);
    }
}
