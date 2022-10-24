use oort_api::prelude::*;

pub struct Ship {
    target: Vec2,
    rng: oorandom::Rand64,
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            target: position(),
            rng: oorandom::Rand64::new(seed()),
        }
    }

    pub fn tick(&mut self) {
        if (self.target - position()).length() < 50.0 {
            self.target =
                vec2(self.rng.rand_float() * 500.0, 0.0).rotate(self.rng.rand_float() * TAU);
        }

        accelerate(self.target - position() - velocity());
        turn_to((self.target - position()).angle());
        fire(0);
    }
}

fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(3.0 * heading_error);
}
