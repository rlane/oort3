// Tutorial 01
// Destroy the asteroid.
use crate::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    // Uncomment me, then press ctrl-Enter to upload the code.
    pub fn tick(&mut self) {
        fire_gun(0);
    }
}
