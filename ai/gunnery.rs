use crate::prelude::*;

pub struct Ship {
    last_target_heading: f64,
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            last_target_heading: 0.0,
        }
    }

    pub fn tick(&mut self) {
        if let Some(contact) = scan() {
            let bullet_speed = 4000.0;
            let bullet_offset = 40.0;

            let dp = contact.position - position();
            let dv = contact.velocity - velocity();
            let mut predicted_dp = dp;
            for _ in 0..10 {
                let dist = predicted_dp.length() - bullet_offset;
                let t = dist / bullet_speed;
                predicted_dp = dp + t * dv;
            }

            let target_heading = predicted_dp.angle();
            let target_angular_velocity = (target_heading - self.last_target_heading) * 60.0;
            turn_to(target_heading, target_angular_velocity);
            self.last_target_heading = target_heading;

            let error = vec2(predicted_dp.length(), 0.0)
                .rotate(heading())
                .distance(predicted_dp);
            debug!("error = {}", error);
            if error <= 10.0 {
                fire_gun(0);
            }

            set_radar_width(TAU / 128.0);
            let next_tick_dp = dp + dv / 60.0;
            set_radar_heading(next_tick_dp.angle() - heading());
        } else {
            set_radar_width(TAU / 32.0);
            set_radar_heading(radar_heading() + TAU / 32.0);
        }
    }
}

fn turn_to(target_heading: f64, target_angular_velocity: f64) {
    let acc = TAU / 8.0;
    let dh = angle_diff(heading(), target_heading);
    let vh = angular_velocity() - target_angular_velocity;
    let t = (vh / acc).abs();
    let pdh = vh * t + 0.5 * -acc * t * t - dh;
    if pdh < 0.0 {
        torque(acc);
    } else if pdh > 0.0 {
        torque(-acc);
    }
}
