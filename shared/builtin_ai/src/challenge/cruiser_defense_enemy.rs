use oort_api::prelude::*;

pub struct Ship {
    target: Vec2,
    enemy_cruiser_position: Vec2,
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            target: position(),
            enemy_cruiser_position: vec2(0.0, 0.0),
        }
    }

    pub fn tick(&mut self) {
        if class() == Class::Missile {
            return self.missile_tick();
        }

        {
            let dp = self.enemy_cruiser_position - position();
            let angle = dp.angle();
            let dst = self.enemy_cruiser_position
                + vec2(15e3, 0.0).rotate(angle + TAU / 2.0 + (TAU / 128.0) * current_time().sin());
            draw_line(position(), dst, 0xffffff);
            seek(dst, vec2(0.0, 0.0));
        }

        if let Some(contact) = scan() {
            set_radar_width(TAU / 32.0);
            if contact.class == Class::Cruiser {
                self.enemy_cruiser_position = contact.position;
                send([
                    self.enemy_cruiser_position.x,
                    self.enemy_cruiser_position.y,
                    0.0,
                    0.0,
                ]);
                fire(1);
                turn_to((contact.position - position()).angle());
                set_radar_heading((contact.position - position()).angle());
            } else if [Class::Fighter, Class::Missile, Class::Torpedo].contains(&contact.class) {
                if (contact.position - position()).length() < 10e3 {
                    turn_to((contact.position - position()).angle());
                    set_radar_heading((contact.position - position()).angle());
                    fire(0);
                } else {
                    set_radar_heading((self.enemy_cruiser_position - position()).angle());
                }
            }
        } else {
            turn_to((self.target - position()).angle());
            set_radar_heading(radar_heading() + radar_width());
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
            if dp.length() < 1e3 {
                turn_to(dp.angle());
            }

            set_radar_heading((contact.position - position()).angle());
            set_radar_width((10.0 * TAU / dp.length()).clamp(TAU / 30.0, TAU));
        } else if rand(0.0, 1.0) < 0.1 {
            if let Some(msg) = receive() {
                let p = vec2(msg[0], msg[1]);
                self.enemy_cruiser_position = p;
                set_radar_heading((p - position()).angle());
                set_radar_width(TAU / 32.0);
            }
        } else {
            accelerate(vec2(100.0, 0.0).rotate(heading()));
            set_radar_width(TAU / 4.0);
            set_radar_heading(heading());
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
    let a = vec2(
        if fuel() < 500.0 { 0.0 } else { 100.0 },
        N * closing_speed * los_rate,
    )
    .rotate(los);
    let a = vec2(400.0, 0.0).rotate(a.angle());
    accelerate(a);
    turn_to(a.angle());
}

fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(10.0 * heading_error);
}
