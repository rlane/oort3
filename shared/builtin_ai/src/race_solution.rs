use oort_api::prelude::*;

pub struct Ship {
    targets: [Vec2; 3],
    current_target_idx: usize,
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            targets: [
                vec2(f64::NAN, f64::NAN),
                vec2(f64::NAN, f64::NAN),
                vec2(f64::NAN, f64::NAN),
            ],
            current_target_idx: 0,
        }
    }

    pub fn tick(&mut self) {
        // read target data from beacon cruiser radio for the first few ticks
        if current_tick() < 4 {
            if let Some(data) = receive() {
                self.targets[get_radio_channel()] = vec2(data[0], data[1]);
            };
            set_radio_channel((current_tick()) as usize);
            return; // note this early return during target acquisition!
        }

        // implement pilot to fly through all targets here
        // access the target locations like so:
        for target in &self.targets {
            debug!("Target: {}", target);
        }

        // accelerate towards target
        // while reducing own velocity (minimize tangential part, avoid orbiting targets and flying out of bounds)
        // also turn towards target and shoot to eliminate asteroids in the way
        let current_target = self.targets[self.current_target_idx];
        let delta_position = current_target - position();
        let delta_heading = angle_diff(heading(), delta_position.angle());

        let desired_velocity = delta_position.normalize() * 300.0;
        let delta_velocity = desired_velocity - velocity();

        if delta_velocity.length() > 10.0 {
            accelerate(100.0 * delta_velocity);
        }

        turn(10.0 * delta_heading);
        fire(0);

        if delta_position.length() < 45.0 {
            self.current_target_idx += 1;
        }
    }
}
