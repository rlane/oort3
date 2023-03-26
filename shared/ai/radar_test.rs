use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        let t = 120;
        if current_tick() < t * 2 {
            return;
        }
        if current_tick() % t == 0 {
            set_radar_heading(radar_heading() + TAU / 10.0);
            set_radar_width(TAU / 60.0);
        } else if current_tick() % (t / 2) == 0 {
            set_radar_width(TAU / 360.0);
        }

        if let Some(contact) = scan() {
            draw_line(
                contact.position,
                contact.position + contact.velocity,
                0xffffff,
            );
        }
    }
}
