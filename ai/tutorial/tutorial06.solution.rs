// Tutorial 06
// Destroy the enemy ships. Use your radar to find them.
// Hint: Press 'g' in-game to show where your radar is looking.
use crate::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        if let Some(contact) = scan() {
            accelerate(0.1 * (contact.position - position() - velocity()).rotate(-heading()));
            turn_to((contact.position - position()).angle());
            fire_gun(0);
            set_radar_heading((contact.position - position()).angle() - heading());
        } else {
            set_radar_heading(radar_heading() + TAU / 6.0);
        }
    }
}

fn turn_to(target_heading: f64) {
    torque(3.0 * angle_diff(heading(), target_heading) - angular_velocity());
}
