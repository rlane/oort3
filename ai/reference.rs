use oort_api::prelude::*;

pub struct Ship {
    target_position: Vec2,
    target_velocity: Vec2,
    has_locked: bool,
    last_contact_time: f64,
}

impl Ship {
    pub fn new() -> Ship {
        let target_position = parse_orders(orders());
        let target_velocity = Vec2::new(0.0, 0.0);
        Ship {
            target_position,
            target_velocity,
            has_locked: false,
            last_contact_time: current_time(),
        }
    }

    pub fn tick(&mut self) {
        if class() == Class::Missile {
            self.missile_tick();
        } else if class() == Class::Torpedo {
            self.torpedo_tick();
        } else {
            self.ship_tick();
        }
    }

    pub fn ship_tick(&mut self) {
        if class() == Class::Cruiser {
            if current_tick() % 6 == 0 {
                set_radar_width(TAU);
            } else {
                set_radar_width(TAU / 60.0);
                set_radar_heading(TAU * (current_tick() as f64 * 2.0) / 60.0 - heading());
            }
        }

        let scan_result = scan();
        if let Some(contact) = scan_result.as_ref() {
            let dp = contact.position - position();
            let dv = contact.velocity - velocity();
            let mut predicted_dp = dp;
            let bullet_speed = 1000.0;
            if dp.dot(dv) > -0.9 {
                for _ in 0..3 {
                    predicted_dp = dp + dv * predicted_dp.length() / bullet_speed;
                }
            }
            set_radar_heading(dp.angle() - heading());
            self.target_position = contact.position;
            self.target_velocity = contact.velocity;

            if class() == Class::Fighter {
                if predicted_dp.length() < 5000.0 {
                    fire_gun(0);
                }
                launch_missile(0, make_orders(contact.position));
            } else if class() == Class::Frigate {
                fire_gun(0);
                aim_gun(
                    1,
                    (predicted_dp - vec2(0.0, 15.0).rotate(heading())).angle() - heading(),
                );
                fire_gun(1);
                aim_gun(
                    2,
                    (predicted_dp - vec2(0.0, -15.0).rotate(heading())).angle() - heading(),
                );
                fire_gun(2);
                launch_missile(0, make_orders(contact.position));
            } else if class() == Class::Cruiser {
                if predicted_dp.length() < 5000.0 {
                    aim_gun(0, predicted_dp.angle() - heading());
                    fire_gun(0);
                }
                for i in 0..2 {
                    launch_missile(i, make_orders(contact.position));
                }
                if contact.class == Class::Frigate || contact.class == Class::Cruiser {
                    launch_missile(2, make_orders(contact.position));
                }
                //dbg.draw_diamond(contact.position, 30.0, 0xffff00);
            }
        } else {
            set_radar_heading(rand(0.0, TAU));
            if (self.target_position - position()).length() < 100.0 {
                self.target_position = vec2(rand(3500.0, 4500.0), 0.0).rotate(rand(0.0, TAU));
                self.target_velocity = vec2(0.0, 0.0);
            }
        }

        let dp = self.target_position - position();
        let dist = dp.length();
        let mut bullet_speed = 1000.0;
        if class() == Class::Frigate {
            bullet_speed = 4000.0;
        }
        let t = dist / bullet_speed;
        let predicted_dp = dp + t * (self.target_velocity - velocity());
        self.turn_to(predicted_dp.angle(), 0.0);

        if scan_result.is_some() && dist < 1000.0 {
            accelerate(-velocity());
        } else {
            accelerate(dp - velocity());
        }
    }

    fn missile_tick(&mut self) {
        let acc = max_acceleration().x;

        if !self.has_locked {
            set_radar_heading((self.target_position - position()).angle() - heading());
            set_radar_width(TAU / 32.0);
            //dbg.draw_diamond(target_position, 20.0, 0xff0000);
        }

        let mut contact = scan();
        if contact.is_some()
            && class() == Class::Torpedo
            && contact.as_ref().unwrap().class != Class::Frigate
            && contact.as_ref().unwrap().class != Class::Cruiser
        {
            contact = None;
        }
        if contact.is_none() {
            if self.has_locked {
                set_radar_heading(rand(0.0, TAU));
                set_radar_width(TAU / 6.0);
            } else {
                let dp = self.target_position - position();
                self.turn_to(dp.angle(), 0.0);
                accelerate(dp.normalize() * acc);
            }
            return;
        }
        self.has_locked = true;
        let contact = contact.unwrap();
        set_radar_heading((contact.position - position()).angle() - heading());

        let dp = contact.position - position();
        let dv = contact.velocity - velocity();

        let dist = dp.length();
        let next_dist = (dp + dv / 60.0).length();
        if next_dist < 30.0 || dist < 100.0 && next_dist > dist {
            explode();
            return;
        }

        let badv = -(dv - dv.dot(dp) * dp.normalize() / dp.length());
        let a = (dp - badv * 10.0).normalize() * acc;
        accelerate(a);
        self.turn_to(a.angle(), 0.0);

        /* TODO
        dbg.draw_diamond(contact.position, 20.0, 0xffff00);
        dbg.draw_diamond(position() + dp, 5.0, 0xffffff);
        dbg.draw_line(position(), position() + dp, 0x222222);
        dbg.draw_line(position(), position() - dv, 0xffffff);
        dbg.draw_line(position(), position() + badv, 0x222299);
        */
    }

    fn torpedo_tick(&mut self) {
        let mut acc = max_acceleration().x;
        self.target_velocity = velocity();

        let target_heading = (self.target_position - position()).angle();
        set_radar_heading(
            target_heading - heading()
                + rand(-PI, PI) * ((current_time() - self.last_contact_time) / 10.0),
        );
        if (self.target_position - position()).length() < 200.0 {
            set_radar_width(PI * 2.0 / 6.0);
        } else {
            set_radar_width(PI * 2.0 / 60.0);
        }

        let mut contact = scan();
        if contact.is_some()
            && class() == Class::Torpedo
            && contact.as_ref().unwrap().class != Class::Frigate
            && contact.as_ref().unwrap().class != Class::Cruiser
        {
            contact = None;
        }
        if let Some(contact) = &contact {
            self.target_position = contact.position;
            self.target_velocity = contact.velocity;
            self.last_contact_time = current_time();
        } else {
            self.target_position += self.target_velocity / 60.0;
        }

        let dp = self.target_position - position();
        let dv = self.target_velocity - velocity();

        if contact.is_some() {
            let dist = dp.length();
            let next_dist = (dp + dv / 60.0).length();
            if next_dist < 60.0 || dist < 100.0 && next_dist > dist {
                explode();
                return;
            }
        } else {
            acc /= 2.0;
        }

        let predicted_position =
            self.target_position + self.target_velocity * (dp.length() / 8000.0);
        let pdp = predicted_position - position();

        let badv = -(dv - dv.dot(dp) * pdp.normalize() / pdp.length());
        let a = (pdp - badv * 10.0).normalize() * acc;
        accelerate(a);
        self.turn_to(a.angle(), 0.0);

        /*
        if no_contact_ticks > 0 {
            dbg.draw_diamond(target_position, 20.0, 0xff0000);
        } else {
            dbg.draw_diamond(contact.position, 20.0, 0xffff00);
            dbg.draw_diamond(position() + pdp, 5.0, 0xffffff);
        }

        dbg.draw_line(position(), position() + dp, 0x222222);
        dbg.draw_line(position(), position() - dv, 0xffffff);
        dbg.draw_line(position(), position() + badv, 0x222299);
        */
    }

    fn turn_to(&mut self, target_heading: f64, target_angular_velocity: f64) {
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
}

const SCALE: f64 = 1e6;
const BIAS: f64 = SCALE / 2.0;

fn parse_orders(o: f64) -> Vec2 {
    if o == 0.0 {
        vec2(0.0, 0.0)
    } else {
        vec2(o % SCALE - BIAS, (o / SCALE).round() - BIAS)
    }
}

fn make_orders(o: Vec2) -> f64 {
    (o.x.round() + BIAS) + (o.y.round() + BIAS) * SCALE
}
