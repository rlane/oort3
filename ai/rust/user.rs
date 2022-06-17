use prelude::*;
use std::f64::consts::TAU;

pub fn tick() {
    accelerate(vec2(100.0, 20.0));
    torque(1.0);
    launch_missile(0);
    set_radar_width(TAU / 4.0);
    if let Some(contact) = scan() {
        aim_gun(0, 0.0);
        fire_gun(0);
    }
}
