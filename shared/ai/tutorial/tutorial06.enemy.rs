use oort_api::prelude::*;

pub struct Ship {
    target: Vec2,
}

impl Ship {
    pub fn new() -> Ship {
        Ship { target: position() }
    }

    pub fn tick(&mut self) {
        if (self.target - position()).length() < 50.0 {
            self.target = vec2(rand(0.0, 500.0), 0.0).rotate(rand(0.0, TAU));
        }

        accelerate(self.target - position() - velocity());
        turn_to((self.target - position()).angle());
    }
}

fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(3.0 * heading_error);
}
