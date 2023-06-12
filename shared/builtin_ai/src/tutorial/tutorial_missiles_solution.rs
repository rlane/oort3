// Tutorial: Missiles (solution)
// Destroy the enemy ship with your missiles.
// Hint: https://en.wikipedia.org/wiki/Proportional_navigation
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        if class() == Class::Missile {
            if let Some(contact) = scan() {
                seek(contact.position, contact.velocity);

                let dp = contact.position - position();
                let dv = contact.velocity - velocity();
                if dp.length().min((dp + dv * TICK_LENGTH).length()) < 25.0 {
                    explode();
                }

                set_radar_heading((contact.position - position()).angle());
                set_radar_width((10.0 * TAU / dp.length()).clamp(TAU / 30.0, TAU));
            } else if let Some(msg) = receive() {
                let target_position = vec2(msg[0], msg[1]);
                let target_velocity = vec2(msg[2], msg[3]);
                seek(target_position, target_velocity);
                set_radar_heading((target_position - position()).angle());
                set_radar_width(TAU / 360.0);
            } else {
                accelerate(vec2(100.0, 0.0).rotate(heading()));
                set_radar_width(TAU / 4.0);
            }
        } else {
            set_radar_width(TAU / 32.0);
            if let Some(contact) = scan() {
                fire(1);
                send([
                    contact.position.x,
                    contact.position.y,
                    contact.velocity.x,
                    contact.velocity.y,
                ]);
                set_radar_heading(contact.position.angle());
                turn_to(contact.position.angle());
            } else {
                set_radar_heading(radar_heading() + TAU / 32.0);
            }
        }
    }
}

pub fn seek(p: Vec2, v: Vec2) {
    let dp = p - position();
    let dv = v - velocity();
    let closing_speed = -(dp.y * dv.y - dp.x * dv.x).abs() / dp.length();
    let los = dp.angle();
    let los_rate = (dp.y * dv.x - dp.x * dv.y) / (dp.length() * dp.length());

    const N: f64 = 4.0;
    let a = vec2(100.0, N * closing_speed * los_rate).rotate(los);
    let a = vec2(400.0, 0.0).rotate(a.angle());
    accelerate(a);
    turn_to(a.angle());
}

fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(10.0 * heading_error);
}
