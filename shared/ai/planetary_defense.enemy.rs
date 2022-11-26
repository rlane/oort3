use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        let target = vec2(0.0, -7500.0);
        let dp = target - position();
        let err = velocity().normalize() - dp.normalize();
        let acc =
            dp.normalize() * max_forward_acceleration() - err * 10.0 * max_lateral_acceleration();
        turn(angle_diff(heading(), acc.angle()));
        accelerate(acc);
    }
}
