// Tutorial: Search (solution)
// Destroy the enemy ship. It is initially outside of your radar range.
// Hint: The set_radar_width() function can be used to create a tighter radar
// beam that's effective at longer distances.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        set_radar_heading(rand(0.0, TAU));
        Ship {}
    }

    pub fn tick(&mut self) {
        set_radar_width(TAU / 120.0);
        if let Some(contact) = scan() {
            accelerate(0.01 * (contact.position - position()) - 0.1 * velocity());
            turn_to(lead_target(contact.position, contact.velocity));
            fire(0);
            set_radar_heading((contact.position - position()).angle());
        } else {
            turn(0.0);
            set_radar_heading(radar_heading() + radar_width());
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
