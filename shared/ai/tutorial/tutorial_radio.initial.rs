// Tutorial: Radio
// Destroy the enemy ship. Your radar is broken, but a radio signal on channel
// 2 will give you its position and velocity.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        set_radio_channel(0);
        if let Some(msg) = receive() {
            debug!("msg: {msg:?}");
        } else {
            debug!("no message received");
        }
    }
}
