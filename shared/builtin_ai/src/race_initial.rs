use core::cmp::Ordering;
use oort_api::prelude::*;

pub struct Ship {
    targets: Vec<Vec2>,
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            targets: Vec::new(),
        }
    }

    pub fn tick(&mut self) {
        for target in &self.targets {
            debug!("Target: {}", target);
        }

        set_radio_channel(2);
        match receive() {
            None => {
                debug!("No radio :(");
            }
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
    }
}
