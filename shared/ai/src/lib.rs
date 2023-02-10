mod user;

use std::collections::HashMap;

// For compatibility.
pub use oort_api::prelude;

struct ShipWrapper {
    user_ship: user::Ship,
    rng: oort_api::rng_state::RngState,
}

static mut SHIPS: Option<HashMap<i32, ShipWrapper>> = None;

fn ensure_initialized() {
    unsafe {
        if SHIPS.is_none() {
            SHIPS = Some(HashMap::new());
        }
    }
}

#[doc(hidden)]
#[no_mangle]
pub fn export_tick_ship(key: i32) {
    ensure_initialized();
    unsafe {
        oort_api::dbg::reset();
        let ship = SHIPS
            .as_mut()
            .unwrap()
            .entry(key)
            .or_insert_with(|| ShipWrapper {
                user_ship: user::Ship::new(),
                rng: oort_api::rng_state::RngState::new(),
            });
        oort_api::rng_state::set(&mut ship.rng);
        ship.user_ship.tick();
        oort_api::dbg::update();
    }
}

#[doc(hidden)]
#[no_mangle]
pub fn export_delete_ship(key: i32) {
    ensure_initialized();
    unsafe {
        SHIPS.as_mut().unwrap().remove(&key);
    }
}
