// tutorial07
// Destroy the enemy ships. They now shoot back.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        set_radar_width(TAU / 60.0);
        if let Some(contact) = scan() {
            let dp = contact.position - position();
            let dv = contact.velocity - velocity();
            accelerate(0.1 * dp);
            let predicted_dp = dp + dv * dp.length() / 1000.0;
            turn_to(predicted_dp.angle());
            fire(0);
            set_radar_heading(dp.angle());
        } else {
            set_radar_heading(radar_heading() + radar_width());
        }
    }
}

fn turn_to(target_heading: f64) {
    torque(3.0 * angle_diff(heading(), target_heading) - angular_velocity());
}
