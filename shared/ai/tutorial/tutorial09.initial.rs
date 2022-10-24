// tutorial09
// Destroy the enemy ship with your missiles.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        if class() == Class::Missile {
            if let Some(contact) = scan() {
                let dp = contact.position - position();
                let dv = contact.velocity - velocity();
                turn_to(dp.angle());
                accelerate(dp + dv);
                if dp.length() < 20.0 {
                    explode();
                }
            }
        } else {
            fire(1);
        }
    }
}

fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(10.0 * heading_error);
}
