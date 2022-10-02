// tutorial07
// Destroy the enemy ships. They now shoot back.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        if let Some(contact) = scan() {
            accelerate(0.1 * (contact.position - position()));
            turn_to((contact.position - position()).angle());
            fire_gun(0);
            set_radar_heading((contact.position - position()).angle());
        } else {
            set_radar_heading(radar_heading() + TAU / 6.0);
        }
    }
}

fn turn_to(target_heading: f64) {
    torque(3.0 * angle_diff(heading(), target_heading) - angular_velocity());
}
