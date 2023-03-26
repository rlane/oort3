use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        set_radar_heading(position().angle() + PI);
        set_radar_width(TAU / 360.0);
        set_radar_ecm_mode(EcmMode::Noise);
    }
}
