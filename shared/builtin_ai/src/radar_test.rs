use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        set_radar_width(TAU / 60.0);
        Ship {}
    }

    pub fn tick(&mut self) {
        if let Some(contact) = scan() {
            draw_line(
                contact.position,
                contact.position + contact.velocity,
                0xffffff,
            );
        }

        let t = 120;
        if current_tick() <= t {
            /* nop */
        } else if current_tick() % t == 0 {
            set_radar_width(TAU / 60.0);
        } else if current_tick() % (t / 2) == 0 {
            set_radar_width(TAU / 360.0);
        }
    }
}
