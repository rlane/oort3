use oort_api::prelude::*;

pub struct Ship {
    target: Vec2,
    initial_position: Vec2,
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            target: position(),
            initial_position: position(),
        }
    }

    pub fn tick(&mut self) {
        if (self.target - position()).length() < 50.0 {
            self.target =
                self.initial_position + vec2(rand(0.0, 1000.0), 0.0).rotate(rand(0.0, TAU));
        }

        accelerate(self.target - position() - velocity());
        turn_to((self.target - position()).angle());
    }
}

fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(10.0 * heading_error);
}
