// Tutorial: Deflection (solution)
// Destroy the enemy ship. Its position is given by the "target" function and velocity by the
// "target_velocity" function.
//
// Hint: p = p₀ + v₀t + ½at² (the third equation of kinematics)
// Learn more about the physics of Oort in the wiki:
// https://github.com/rlane/oort3/wiki/The-Physics-of-Oort
//
// p.s. You can change your username by clicking on it at the top of the page.
use oort_api::prelude::*;

const BULLET_SPEED: f64 = 1000.0; // m/s

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        turn_to(lead_target(target(), target_velocity()));
        fire(0);
    }
}

fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(10.0 * heading_error);
}

fn lead_target(target_position: Vec2, target_velocity: Vec2) -> f64 {
    let dp = target_position - position();
    let dv = target_velocity - velocity();
    let predicted_dp = dp + dv * dp.length() / BULLET_SPEED;
    predicted_dp.angle()
}
