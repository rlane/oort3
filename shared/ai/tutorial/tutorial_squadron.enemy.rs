use oort_api::prelude::*;

const BULLET_SPEED: f64 = 1000.0;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        set_radio_channel(1);
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
            set_radar_width(TAU / 60.0);
            if let Some(contact) = scan() {
                let dp = contact.position - position();
                fire(1);
                send([
                    contact.position.x,
                    contact.position.y,
                    contact.velocity.x,
                    contact.velocity.y,
                ]);
                set_radar_heading(dp.angle());
                seek(contact.position, contact.velocity);

                if dp.length() < 8000.0 {
                    let angle = lead_target(contact.position, contact.velocity);
                    turn_to(angle);
                    if angle_diff(heading(), angle).abs() < 0.1 {
                        fire(0);
                    }
                }
            } else {
                set_radar_heading(radar_heading() + radar_width());
                seek(vec2(0.0, 0.0), vec2(0.0, 0.0));
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

fn lead_target(target_position: Vec2, target_velocity: Vec2) -> f64 {
    let dp = target_position - position();
    let dv = target_velocity - velocity();
    let predicted_dp = dp + dv * dp.length() / BULLET_SPEED;
    predicted_dp.angle()
}
