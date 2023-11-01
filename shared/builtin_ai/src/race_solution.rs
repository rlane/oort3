use core::cmp::Ordering;
use oort_api::prelude::*;

pub struct Ship {
    targets: Vec<Vec2>,
    current_target_idx: usize,
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            targets: Vec::new(),
            current_target_idx: 0,
        }
    }

    pub fn tick(&mut self) {
        // print targets for clarity
        for target in &self.targets {
            debug!("Target: {}", target);
        }

        // read target positions from dummy enemy ships
        set_radio_channel(2);
        match receive() {
            None => {}
            Some(data) => {
                let p = vec2(data[0], data[1]);
                if self
                    .targets
                    .iter()
                    .all(|v| v.x.total_cmp(&p.x) != Ordering::Equal)
                {
                    self.targets.push(p);
                }
            }
        }

        if self.targets.len() < 3 {
            return;
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
