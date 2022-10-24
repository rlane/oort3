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
            accelerate(0.1 * dp);
            turn_to(lead_target(contact.position, contact.velocity));
            fire(0);
            set_radar_heading(dp.angle());
        } else {
            set_radar_heading(radar_heading() + radar_width());
        }
        if current_time() < 3.0 {
            turn_to(TAU / 4.0);
            accelerate(vec2(0.0, 200.0));
        }
    }
}

fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(10.0 * heading_error);
}

fn lead_target(target_position: Vec2, target_velocity: Vec2) -> f64 {
    let dp = target_position - position();
    let dv = target_velocity - velocity();
    let predicted_dp = dp + dv * dp.length() / 1000.0;
    predicted_dp.angle()
}
