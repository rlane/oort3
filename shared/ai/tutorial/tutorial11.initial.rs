// tutorial11
// Destroy the enemy ships with your Cruiser.
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
                torque(20.0 * (angle_diff(heading(), dp.angle()) - 0.1 * angular_velocity()));
                accelerate(dp + dv);
                if dp.length() < 20.0 {
                    explode();
                }
            }
        } else {
            aim_gun(0, 0.0);
            fire_gun(0);
            launch_missile(0);
            launch_missile(1);
        }
    }
}
