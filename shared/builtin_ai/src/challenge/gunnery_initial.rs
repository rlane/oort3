// Challenge: Gunnery
// Destroy the targets. Your frigate can't accelerate and only has its main gun.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        fire(0);
    }
}
