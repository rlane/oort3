use prelude::*;

pub fn tick() {
    accelerate(vec2(100.0, 20.0));
    torque(1.0);
    aim_gun(0, std::f64::consts::TAU / 4.0);
    fire_gun(0);
    launch_missile(0);
}
