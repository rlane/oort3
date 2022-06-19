use crate::prelude::*;
use std::f64::consts::TAU;

pub struct Ship {
    ticks: u64,
    rng: oorandom::Rand64,
}

impl Ship {
    pub fn new() -> Ship {
        set_radar_width(TAU / 4.0);
        Ship {
            ticks: 0,
            rng: oorandom::Rand64::new(seed()),
        }
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
        accelerate(vec2(100.0, 20.0));
        torque((self.ticks as f64 * 1e-2).sin());
        launch_missile(0);
        if let Some(contact) = scan() {
            fire_gun(0);
        }
        let radar_heading = self.rng.rand_float() * TAU;
        set_radar_heading(radar_heading);
        aim_gun(0, radar_heading);
    }
}
