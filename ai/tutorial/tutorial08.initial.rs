// tutorial08
// Destroy the enemy ships. They are initially outside of your radar range.
use crate::prelude::*;

pub struct Ship {
    ticks: i64,
}

impl Ship {
    pub fn new() -> Ship {
        Ship { ticks: 0 }
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
        set_radar_heading(self.ticks as f64 * std::f64::consts::TAU / 6.0);
        if let Some(contact) = scan() {
            accelerate(0.1 * (contact.position - position()));
            fire_gun(0);
        }
    }
}
