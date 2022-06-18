use crate::prelude::*;
use std::f64::consts::TAU;

pub struct Ship {
    ticks: u64,
}

impl Ship {
    pub fn new() -> Ship {
        Ship { ticks: 0 }
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
        accelerate(vec2(100.0, 20.0));
        torque((self.ticks as f64 * 1e-2).sin());
        launch_missile(0);
        set_radar_width(TAU / 4.0);
        if let Some(contact) = scan() {
            aim_gun(0, 0.0);
            fire_gun(0);
        }
    }
}
