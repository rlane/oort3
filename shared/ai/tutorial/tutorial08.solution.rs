// tutorial08
// Destroy the enemy ships. They are initially outside of your radar range.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        set_radar_width(TAU / 60.0);
        if let Some(contact) = scan() {
            accelerate(0.01 * (contact.position - position() - velocity()));

            let dp = contact.position - position();
            let predicted_dp = dp + (dp.length() / 1000.0) * (contact.velocity - velocity());
            let predicted_dp =
                dp + (predicted_dp.length() / 1000.0) * (contact.velocity - velocity());

            turn_to(predicted_dp.angle(), 0.0);
            fire(0);
            set_radar_heading((contact.position - position()).angle());
        } else {
            torque(-angular_velocity());
            set_radar_heading(radar_heading() + TAU / 60.0);
        }
    }
}

fn turn_to(target_heading: f64, target_angular_velocity: f64) {
    let acc = max_angular_acceleration();
    let dh = angle_diff(heading(), target_heading);
    let vh = angular_velocity() - target_angular_velocity;
    let t = (vh / acc).abs();
    let pdh = vh * t + 0.5 * -acc * t * t - dh;
    if pdh < 0.0 {
        torque(acc);
    } else if pdh > 0.0 {
        torque(-acc);
    }
}
