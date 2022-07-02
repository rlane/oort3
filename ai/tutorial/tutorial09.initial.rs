// tutorial09
// Destroy the enemy ships with your missiles.
use crate::prelude::*;

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
                torque(20 * (angle_diff(heading(), dp.angle()) - 0.1 * angular_velocity()));
                accelerate((dp + dv).rotate(-heading()));
                if dp.magnitude() < 20 {
                    explode();
                }
            }
        } else {
            launch_missile(0);
        }
    }
}
