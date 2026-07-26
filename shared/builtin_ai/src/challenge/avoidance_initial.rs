// Challenge: Avoidance
// Reach the position advertised on radio channel 1. Beware of the hostile frigate.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        set_radio_channel(1);
        if let Some([x, y, _, _]) = receive() {
            let target = Vec2::new(x, y);
            let dp = target - position();
            let acc = dp.normalize() * 10000.;
            accelerate(acc);
            turn_to(dp.angle());
        }
    }
}

/// Turns towards the given heading.
fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(10.0 * heading_error);
}
