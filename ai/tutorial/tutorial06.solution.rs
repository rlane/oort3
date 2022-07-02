// Tutorial 06
// Destroy the enemy ships. Use your radar to find them.
// Hint: Press 'g' in-game to show where your radar is looking.
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
        if let Some(contact) = scan() {
            accelerate(0.1 * (contact.position - position()).rotate(-heading()));
            turn_to((contact.position - position()).angle());
            fire_gun(0);
            set_radar_heading((contact.position - position()).angle() - heading());
        } else {
            set_radar_heading(self.ticks as f64 * std::f64::consts::TAU / 6.0);
        }
    }
}

fn turn_to(target_heading: f64) {
    torque(3.0 * angle_diff(heading(), target_heading) - angular_velocity());
}
