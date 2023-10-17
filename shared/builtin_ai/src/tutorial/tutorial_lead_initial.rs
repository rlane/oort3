// Tutorial: Lead
// Destroy the enemy ship. Its position is given by the "target" function and velocity by the
// "target_velocity" function. Your ship is not able to accelerate in this scenario.
//
// This is where the game becomes challenging! You'll need to lead the target
// by firing towards where the target will be by the time the bullet gets there.
//
// Hint1: target() + target_velocity() * t gives the position of the target after t seconds.
// Hint2: function tick() is called 60 times per second. You can use TICK_LENGTH to get the time duration per tick.
//
// You can scale a vector by a number: vec2(a, b) * c == vec2(a * c, b * c)
//
// p.s. You can change your username by clicking on it at the top of the page.
use oort_api::prelude::*;

const BULLET_SPEED: f64 = 1000.0; // m/s

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        draw_line(position(), target(), 0x00ff00);
        let dp = target() - position();
        debug!("distance to target: {}", dp.length());
        debug!("time to target: {}", dp.length() / BULLET_SPEED);
        turn(1.0);
        fire(0);
    }
}
