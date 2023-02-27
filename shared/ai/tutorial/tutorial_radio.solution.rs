// Tutorial: Radio (solution)
// Destroy the enemy ship. Your radar is broken, but a radio signal on channel
// 2 will give you its position and velocity.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        set_radio_channel(2);
        if let Some(msg) = receive() {
            debug!("msg: {msg:?}");
            let contact_position = vec2(msg[0], msg[1]);
            let contact_velocity = vec2(msg[2], msg[3]);
            accelerate(0.01 * (contact_position - position()) - 0.1 * velocity());
            turn_to(lead_target(contact_position, contact_velocity));
            fire(0);
        } else {
            debug!("no message received");
            turn(0.0);
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

