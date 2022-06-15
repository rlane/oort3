use super::prelude::*;
use oorandom::Rand64;
use std::f64::consts::{PI, TAU};

pub struct NativeShip {
    api: Api,
    rng: Rand64,
    orders: Vec2,
    target_position: Vec2,
    target_velocity: Vec2,
    ticks: i64,
    has_locked: bool,
    no_contact_ticks: i64,
}

impl NativeShip {
    pub fn new(api: Api, orders: String, seed: u64) -> Self {
        let target_position = api.position();
        let target_velocity = Vec2::new(0.0, 0.0);
        let orders = parse_orders(&orders);
        NativeShip {
            api,
            rng: Rand64::new(seed as u128),
            orders,
            target_position,
            target_velocity,
            ticks: 0,
            has_locked: false,
            no_contact_ticks: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.api.class() == ShipClass::Missile {
            self.missile_tick();
        } else if self.api.class() == ShipClass::Torpedo {
            self.torpedo_tick();
        } else {
            self.ship_tick();
        }
    }

    pub fn ship_tick(&mut self) {
        let api = self.api;
        self.ticks += 1;
        if api.class() == ShipClass::Cruiser {
            if self.ticks % 6 == 0 {
                api.set_radar_width(TAU);
            } else {
                api.set_radar_width(TAU / 60.0);
                api.set_radar_heading(TAU * (self.ticks as f64 * 2.0) / 60.0 - api.heading());
            }
        }

        let scan_result = api.scan();
        if let Some(contact) = scan_result.as_ref() {
            let dp = contact.position - api.position();
            let dv = contact.velocity - api.velocity();
            let mut predicted_dp = dp;
            let bullet_speed = 1000.0;
            if dp.dot(&dv) > -0.9 {
                for _ in 0..3 {
                    predicted_dp = dp + dv * predicted_dp.magnitude() / bullet_speed;
                }
            }
            api.set_radar_heading(dp.angle0() - api.heading());
            self.target_position = contact.position;
            self.target_velocity = contact.velocity;

            if api.class() == ShipClass::Fighter {
                if predicted_dp.magnitude() < 5000.0 {
                    api.fire_gun(0);
                }
                api.launch_missile(0, format_orders(contact.position));
            } else if api.class() == ShipClass::Frigate {
                api.fire_gun(0);
                api.aim_gun(
                    1,
                    (predicted_dp - vec2(0.0, 15.0).rotate(api.heading())).angle0() - api.heading(),
                );
                api.fire_gun(1);
                api.aim_gun(
                    2,
                    (predicted_dp - vec2(0.0, -15.0).rotate(api.heading())).angle0()
                        - api.heading(),
                );
                api.fire_gun(2);
                api.launch_missile(0, format_orders(contact.position));
            } else if api.class() == ShipClass::Cruiser {
                if predicted_dp.magnitude() < 5000.0 {
                    api.aim_gun(0, predicted_dp.angle0() - api.heading());
                    api.fire_gun(0);
                }
                for i in 0..2 {
                    api.launch_missile(i, format_orders(contact.position));
                }
                if contact.class == Some(ShipClass::Frigate)
                    || contact.class == Some(ShipClass::Cruiser)
                {
                    api.launch_missile(2, format_orders(contact.position));
                }
                //dbg.draw_diamond(contact.position, 30.0, 0xffff00);
            }
        } else {
            api.set_radar_heading(self.rand(0.0, TAU));
            if (self.target_position - api.position()).magnitude() < 100.0 {
                self.target_position =
                    vec2(self.rand(3500.0, 4500.0), 0.0).rotate(self.rand(0.0, TAU));
                self.target_velocity = vec2(0.0, 0.0);
            }
        }

        let dp = self.target_position - api.position();
        let dist = dp.magnitude();
        let mut bullet_speed = 1000.0;
        if api.class() == ShipClass::Frigate {
            bullet_speed = 4000.0;
        }
        let t = dist / bullet_speed;
        let predicted_dp = dp + t * (self.target_velocity - api.velocity());
        self.turn_to(predicted_dp.angle0(), 0.0);

        if scan_result.is_some() && dist < 1000.0 {
            api.accelerate(-api.velocity().rotate(-api.heading()));
        } else {
            api.accelerate((dp - api.velocity()).rotate(-api.heading()));
        }
    }

    fn missile_tick(&mut self) {
        let api = self.api;
        let acc = 400.0;

        if !self.has_locked {
            self.target_position = self.orders;
            api.set_radar_heading((self.target_position - api.position()).angle0() - api.heading());
            api.set_radar_width(TAU / 32.0);
            //dbg.draw_diamond(target_position, 20.0, 0xff0000);
        }

        let mut contact = api.scan();
        if contact.is_some()
            && api.class() == ShipClass::Torpedo
            && contact.unwrap().class != Some(ShipClass::Frigate)
            && contact.unwrap().class != Some(ShipClass::Cruiser)
        {
            contact = None;
        }
        if contact.is_none() {
            if self.has_locked {
                api.set_radar_heading(self.rand(0.0, TAU));
                api.set_radar_width(TAU / 6.0);
            } else {
                let dp = self.target_position - api.position();
                self.turn_to(dp.angle0(), 0.0);
                let a = dp.rotate(-api.heading()).normalize() * acc;
                api.accelerate(a);
            }
            return;
        }
        self.has_locked = true;
        let contact = contact.unwrap();
        api.set_radar_heading((contact.position - api.position()).angle0() - api.heading());

        let dp = contact.position - api.position();
        let dv = contact.velocity - api.velocity();

        let dist = dp.magnitude();
        let next_dist = (dp + dv / 60.0).magnitude();
        if next_dist < 30.0 || dist < 100.0 && next_dist > dist {
            api.explode();
            return;
        }

        let badv = -(dv - dv.dot(&dp) * dp.normalize() / dp.magnitude());
        let a = (dp - badv * 10.0).rotate(-api.heading()).normalize() * acc;
        api.accelerate(a);
        self.turn_to(a.rotate(api.heading()).angle0(), 0.0);

        /* TODO
        dbg.draw_diamond(contact.position, 20.0, 0xffff00);
        dbg.draw_diamond(api.position() + dp, 5.0, 0xffffff);
        dbg.draw_line(api.position(), api.position() + dp, 0x222222);
        dbg.draw_line(api.position(), api.position() - dv, 0xffffff);
        dbg.draw_line(api.position(), api.position() + badv, 0x222299);
        */
    }

    fn torpedo_tick(&mut self) {
        let api = self.api;
        let mut acc = 1000.0;
        self.target_velocity = api.velocity();

        if self.ticks == 0 {
            self.target_position = self.orders;
        }
        self.ticks += 1;

        let target_heading = (self.target_position - api.position()).angle0();
        api.set_radar_heading(
            target_heading - api.heading()
                + self.rand(-PI, PI) * (self.no_contact_ticks as f64 / 600.0),
        );
        if (self.target_position - api.position()).magnitude() < 200.0 {
            api.set_radar_width(PI * 2.0 / 6.0);
        } else {
            api.set_radar_width(PI * 2.0 / 60.0);
        }

        let mut contact = api.scan();
        if contact.is_some()
            && api.class() == ShipClass::Torpedo
            && contact.unwrap().class != Some(ShipClass::Frigate)
            && contact.unwrap().class != Some(ShipClass::Cruiser)
        {
            contact = None;
        }
        if let Some(contact) = contact {
            self.target_position = contact.position;
            self.target_velocity = contact.velocity;
            self.no_contact_ticks = 0;
        } else {
            self.target_position += self.target_velocity / 60.0;
            self.no_contact_ticks += 1;
        }

        let dp = self.target_position - api.position();
        let dv = self.target_velocity - api.velocity();

        if contact.is_some() {
            let dist = dp.magnitude();
            let next_dist = (dp + dv / 60.0).magnitude();
            if next_dist < 60.0 || dist < 100.0 && next_dist > dist {
                api.explode();
                return;
            }
        } else {
            acc /= 10.0;
        }

        let predicted_position =
            self.target_position + self.target_velocity * (dp.magnitude() / 8000.0);
        let pdp = predicted_position - api.position();

        let badv = -(dv - dv.dot(&dp) * pdp.normalize() / pdp.magnitude());
        let a = (pdp - badv * 10.0).rotate(-api.heading()).normalize() * acc;
        api.accelerate(a);
        self.turn_to(a.rotate(api.heading()).angle0(), 0.0);

        /*
        if no_contact_ticks > 0 {
            dbg.draw_diamond(target_position, 20.0, 0xff0000);
        } else {
            dbg.draw_diamond(contact.position, 20.0, 0xffff00);
            dbg.draw_diamond(api.position() + pdp, 5.0, 0xffffff);
        }

        dbg.draw_line(api.position(), api.position() + dp, 0x222222);
        dbg.draw_line(api.position(), api.position() - dv, 0xffffff);
        dbg.draw_line(api.position(), api.position() + badv, 0x222299);
        */
    }

    fn turn_to(&mut self, target_heading: f64, target_angular_velocity: f64) {
        let api = self.api;
        let mut acc = TAU;
        if api.class() == ShipClass::Frigate {
            acc = TAU / 6.0;
        } else if api.class() == ShipClass::Cruiser {
            acc = TAU / 16.0;
        }
        let dh = angle_diff(api.heading(), target_heading);
        let vh = api.angular_velocity() - target_angular_velocity;
        let t = (vh / acc).abs();
        let pdh = vh * t + 0.5 * -acc * t * t - dh;
        if pdh < 0.0 {
            api.torque(acc);
        } else if pdh > 0.0 {
            api.torque(-acc);
        }
    }

    fn rand(&mut self, low: f64, high: f64) -> f64 {
        self.rng.rand_float() * (high - low) + low
    }
}

fn parse_orders(orders: &str) -> Vec2 {
    if orders.is_empty() {
        return vec2(0.0, 0.0);
    }
    let mut orders = orders.split(' ');
    let x = orders.next().unwrap().parse::<f64>().unwrap();
    let y = orders.next().unwrap().parse::<f64>().unwrap();
    vec2(x, y)
}

fn format_orders(target: Vec2) -> String {
    format!("{} {}", target.x, target.y)
}
