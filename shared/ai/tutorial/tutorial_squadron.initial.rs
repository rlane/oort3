// tutorial07
// Destroy the enemy ships. They now shoot back.
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        set_radar_heading(radar_heading() + TAU / 6.0);
        if let Some(contact) = scan() {
            accelerate(0.1 * (contact.position - position()));
            fire(0);
        }
    }
}
