use crate::prelude::*;

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
        debug::text(&format!("target: {:?}", self.target));
        if (self.target - position()).length() < 50.0 {
            self.target =
                vec2(self.rng.rand_float() * 500.0, 0.0).rotate(self.rng.rand_float() * TAU);
        }

        accelerate((self.target - position() - velocity()).rotate(-heading()));
        turn_to((self.target - position()).angle());
    }
}

fn turn_to(target_heading: f64) {
    torque(3.0 * angle_diff(heading(), target_heading) - angular_velocity());
}
