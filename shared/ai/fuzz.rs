use oort_api::prelude::*;
use oort_api::SystemState;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        draw_triangle(vec2(gen(), gen()), gen(), 0xffffff);
        for i in 0..(SystemState::MaxSize as u8) {
            if i == SystemState::Explode as u8 {
                continue;
            }
            oort_api::sys::write_system_state(unsafe { std::mem::transmute(i) }, gen());
        }
    }
}

fn gen() -> f64 {
    let r = rand(0.0, 1.0);
    if r < 0.1 {
        let vals = &[
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MIN,
            f64::MIN_POSITIVE,
            f64::MAX,
            1.0e-308_f64,
        ];
        vals[rand(0.0, vals.len() as f64) as usize]
    } else {
        rand(f64::MIN, f64::MAX)
    }
}
