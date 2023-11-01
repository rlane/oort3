use oort_api::prelude::*;

pub struct Ship {
    initial_position: Vec2,
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            initial_position: position(),
        }
    }

    pub fn tick(&mut self) {
        set_radio_channel(rand(0.0, 10.0) as usize);
        send([self.initial_position.x, self.initial_position.y, 0.0, 0.0]);
    }
}
