use oort_api::prelude::*;

pub struct Ship {
    target: Vec2,
    initial_position: Vec2,
    rng: oorandom::Rand64,
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            target: position(),
            initial_position: position(),
            rng: oorandom::Rand64::new(seed()),
        }
    }

    pub fn tick(&mut self) {
        if class() == Class::Missile {
            return self.missile_tick();
        }

        if (self.target - position()).length() < 50.0 {
            self.target = self.initial_position
                + vec2(self.rng.rand_float() * 1000.0, 0.0).rotate(self.rng.rand_float() * TAU);
        }

        accelerate(self.target - position() - velocity());

        if let Some(contact) = scan() {
            turn_to((contact.position - position()).angle(), 0.0);
            if (contact.position - position()).length() < 1000.0 {
                fire(0);
            }
            fire(1);
        } else {
            turn_to((self.target - position()).angle(), 0.0);
        }
    }

    pub fn missile_tick(&mut self) {
        if let Some(contact) = scan() {
            seek(contact.position, contact.velocity);

            let dp = contact.position - position();
            let dv = contact.velocity - velocity();
            if dp.length().min((dp + dv * TICK_LENGTH).length()) < 25.0 {
                explode();
            }

            set_radar_heading((contact.position - position()).angle());
            set_radar_width((10.0 * TAU / dp.length()).clamp(TAU / 30.0, TAU));
        } else {
            accelerate(vec2(100.0, 0.0).rotate(heading()));
            set_radar_width(TAU / 4.0);
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
    turn_to(a.angle(), 0.0);
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
