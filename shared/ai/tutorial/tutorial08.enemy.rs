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
        if (self.target - position()).length() < 50.0 {
            self.target = self.initial_position
                + vec2(self.rng.rand_float() * 500.0, 0.0).rotate(self.rng.rand_float() * TAU);
        }

        accelerate(self.target - position() - velocity());

        if let Some(contact) = scan() {
            turn_to((contact.position - position()).angle());
            if (contact.position - position()).length() < 1000.0 {
                fire_gun(0);
            }
        } else {
            turn_to((self.target - position()).angle());
        }
    }
}

fn turn_to(target_heading: f64) {
    torque(3.0 * angle_diff(heading(), target_heading) - angular_velocity());
}
