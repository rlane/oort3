use oort_api::prelude::*;

pub struct Ship {
    course_correction_time: f64,
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            course_correction_time: current_time() + rand(0.0, 10.0),
        }
    }

    pub fn tick(&mut self) {
        if current_time() < self.course_correction_time {
            return;
        }
        let target = vec2(0.0, -17500.0);
        let dp = target - position();
        let err = velocity().normalize() - dp.normalize();
        let mut acc =
            dp.normalize() * max_forward_acceleration() - err * 10.0 * max_lateral_acceleration();
        if velocity().length() > 2000.0 {
            acc -= velocity();
        }
        turn(angle_diff(heading(), acc.angle()));
        accelerate(acc);
    }
}
