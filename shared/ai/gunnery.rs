use oort_api::prelude::*;

const TRACK_RADAR_WIDTH: f64 = TAU / 360.0;
const SEARCH_RADAR_WIDTH: f64 = TAU / 120.0;
const RELOAD_TICKS: i64 = 60;
const TARGET_VELOCITY_EWMA_WEIGHT: f64 = 1.0 / 8.0;

pub struct Ship {
    last_target_heading: f64,
    ticks_since_fired: i64,
    target_velocity_ewma: Vec2,
}

impl Ship {
    pub fn new() -> Ship {
        set_radar_width(SEARCH_RADAR_WIDTH);
        set_radar_heading(TAU / 8.0 + heading());
        Ship {
            last_target_heading: 0.0,
            ticks_since_fired: RELOAD_TICKS,
            target_velocity_ewma: vec2(0.0, 0.0),
        }
    }

    pub fn tick(&mut self) {
        self.ticks_since_fired += 1;
        if let Some(contact) = scan() {
            let bullet_speed = 4000.0;
            let bullet_offset = 40.0;

            self.target_velocity_ewma = (1.0 - TARGET_VELOCITY_EWMA_WEIGHT)
                * self.target_velocity_ewma
                + TARGET_VELOCITY_EWMA_WEIGHT * contact.velocity;

            let dp = contact.position - position();
            let dv = self.target_velocity_ewma - velocity();
            let mut predicted_dp = dp;
            let mut iterations = 0;
            for _ in 0..100 {
                iterations += 1;
                let dist = predicted_dp.length() - bullet_offset;
                let t = dist / bullet_speed;
                let new_predicted_dp = dp + t * dv;
                let delta = predicted_dp.distance(new_predicted_dp);
                predicted_dp = new_predicted_dp;
                if delta < 1e-3 {
                    break;
                }
            }
            debug!("prediction iterations: {}", iterations);

            let target_heading = predicted_dp.angle();
            let target_angular_velocity = (target_heading - self.last_target_heading) * 60.0;
            turn_to(target_heading, target_angular_velocity);
            self.last_target_heading = target_heading;

            let error = vec2(predicted_dp.length(), 0.0)
                .rotate(heading())
                .distance(predicted_dp);
            debug!("error = {}", error);
            debug!("ticks since fired = {}", self.ticks_since_fired);
            if error <= 5.0
                && radar_width() <= TRACK_RADAR_WIDTH
                && self.ticks_since_fired >= RELOAD_TICKS
            {
                debug!("shot");
                fire(0);
                if current_time() < 10.0 {
                    set_radar_width(SEARCH_RADAR_WIDTH);
                    set_radar_heading(radar_heading() - SEARCH_RADAR_WIDTH);
                }
                self.ticks_since_fired = 0;
            } else {
                let next_tick_dp = dp + dv / 60.0;
                set_radar_heading(next_tick_dp.angle() - angular_velocity() * TICK_LENGTH);
                set_radar_width((radar_width() / 2.0).max(TRACK_RADAR_WIDTH));
            }
        } else {
            set_radar_width(SEARCH_RADAR_WIDTH);
            set_radar_heading(radar_heading() - SEARCH_RADAR_WIDTH);
            if angle_diff(radar_heading(), 0.0) > TAU / 8.0 {
                set_radar_heading(TAU / 8.0);
            }
        }
    }
}

fn turn_to(target_heading: f64, target_angular_velocity: f64) {
    let acc = max_angular_acceleration();
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

