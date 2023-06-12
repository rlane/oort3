// Tutorial: Acceleration 2 (solution)
// Fly through the target circle. The target is in a random
// location given by the "target" function.
//
// You can add vectors together: vec2(a, b) + vec2(c, d) == vec2(a + c, b + d)
// And subtract them: vec2(a, b) - vec2(c, d) == vec2(a - c, b - d)
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        // Hint: "target() - position()" returns a vector pointing towards the target.
        accelerate(target() - position());
    }
}
