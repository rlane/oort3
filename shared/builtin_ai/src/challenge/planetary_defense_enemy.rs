use oort_api::prelude::*;

pub struct Ship {
    target: Vec2,
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            target: vec2(rand(-1.0, 1.0) * 10e3, -25000.0),
        }
    }

    pub fn tick(&mut self) {
        let dp = self.target - position();
        draw_line(position(), self.target, 0x880000);
        let err = velocity().normalize() - dp.normalize();
        let acc =
            dp.normalize() * max_forward_acceleration() - err * 10.0 * max_lateral_acceleration();
        activate_ability(Ability::Boost);
        turn(angle_diff(heading(), acc.angle()));
        accelerate(acc);
    }
}
