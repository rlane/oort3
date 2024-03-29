// Tutorial: Search
// Destroy the enemy ship. It is initially outside of your radar range.
// Hint: The set_radar_width() function can be used to create a tighter radar
// beam that's effective at longer distances.
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
