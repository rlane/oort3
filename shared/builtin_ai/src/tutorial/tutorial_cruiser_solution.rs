// Tutorial: Cruiser (solution)
// Destroy the enemy ships with your Cruiser.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        if class() == Class::Cruiser {
            select_radar(1);
            set_radar_heading(PI);
        }
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
                set_radar_width(TAU / 120.0);
            } else if let Some(msg) = receive() {
                let target_position = vec2(msg[0], msg[1]);
                let dp = target_position - position();
                set_radar_width(TAU / 120.0);
                set_radar_heading(dp.angle());
            } else {
                accelerate(vec2(100.0, 0.0).rotate(heading()));
                set_radar_width(TAU / 32.0);
                set_radar_heading(radar_heading() + radar_width());
            }
        } else {
            for radar in 0..2 {
                select_radar(radar);
                set_radar_width(TAU / 32.0);
                if let Some(contact) = scan() {
                    fire(radar + 1); // Corresponding missile
                    send([contact.position.x, contact.position.y, 0.0, 0.0]);
                    aim(0, lead_target(contact.position, contact.velocity, 2000.0));
                    fire(0);
                }
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

fn lead_target(target_position: Vec2, target_velocity: Vec2, bullet_speed: f64) -> f64 {
    let dp = target_position - position();
    let dv = target_velocity - velocity();
    let predicted_dp = dp + dv * dp.length() / bullet_speed;
    predicted_dp.angle()
}
