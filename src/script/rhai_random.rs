use oorandom::Rand64;
use rhai::plugin::*;
use std::cell::RefCell;
use std::rc::Rc;

#[export_module]
pub mod random_module {
    #[derive(Clone)]
    pub struct Random {
        rng: Rc<RefCell<Rand64>>,
    }

    pub fn new_rng(seed: i64) -> Random {
        Random {
            rng: Rc::new(RefCell::new(Rand64::new((seed as u64).into()))),
        }
    }

    pub fn next(obj: &mut Random, low: f64, high: f64) -> f64 {
        obj.rng.borrow_mut().rand_float() * (high - low) + low
    }
}
