use oort_api::prelude::*;

const SPEED: f64 = 200.0;

pub struct Ship {
    target: Vec2,
}

impl Ship {
    pub fn new() -> Ship {
        Ship { target: position() }
    }

    pub fn tick(&mut self) {
        if (self.target - position()).length() < 200.0 {
            self.target = vec2(1000.0, 0.0).rotate(rand(0.0, TAU));
        }

        draw_line(position(), self.target, 0xffffff);

        let target_velocity = (self.target - position()).normalize() * SPEED;
        accelerate((target_velocity - velocity()) * 1e6);
        turn_to((target_velocity - velocity()).angle());
    }
}

fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(3.0 * heading_error);
}
