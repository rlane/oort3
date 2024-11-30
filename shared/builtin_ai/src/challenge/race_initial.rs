// Challenge: Race
// Fly through the three target positions. These are advertised on radio channels 0, 1, and 2.
use oort_api::prelude::*;

pub struct Ship {
    targets: [Vec2; 3],
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            targets: [
                vec2(f64::NAN, f64::NAN),
                vec2(f64::NAN, f64::NAN),
                vec2(f64::NAN, f64::NAN),
            ],
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
    }
}
