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
        set_radar_width(TAU / 60.0);
        if let Some(contact) = scan() {
            accelerate(0.1 * (contact.position - position()).rotate(-heading()));

            let dp = contact.position - position();
            let predicted_dp = dp + (dp.length() / 1000.0) * (contact.velocity - velocity());
            let predicted_dp =
                dp + (predicted_dp.length() / 1000.0) * (contact.velocity - velocity());

            turn_to(predicted_dp.angle());
            fire_gun(0);
            set_radar_heading((contact.position - position()).angle() - heading());
        } else {
            set_radar_heading(self.ticks as f64 * std::f64::consts::TAU / 60.0);
        }
    }
}

fn turn_to(target_heading: f64) {
    torque(10.0 * angle_diff(heading(), target_heading) - angular_velocity());
}
