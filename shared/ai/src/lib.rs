mod user;

use std::collections::HashMap;

// For compatibility.
pub use oort_api::prelude;

struct ShipWrapper {
    user_ship: user::Ship,
    rng: oort_api::rng_state::RngState,
}

static mut SHIPS: Option<HashMap<i32, ShipWrapper>> = None;

#[doc(hidden)]
#[no_mangle]
fn export_initialize() {
    unsafe {
        SHIPS = Some(HashMap::new());
    }
}

#[doc(hidden)]
#[no_mangle]
pub fn export_tick_ship(key: i32) {
    oort_api::dbg::reset();
    unsafe {
        let ship = SHIPS.as_mut().unwrap().entry(key).or_insert_with(|| {
            let rng = oort_api::rng_state::RngState::new();
            oort_api::rng_state::set(rng.clone());
            ShipWrapper {
                user_ship: user::Ship::new(),
                rng,
            }
        });
        oort_api::rng_state::set(ship.rng.clone());
        ship.user_ship.tick();
        oort_api::dbg::update();
        ship.rng = oort_api::rng_state::get().clone();
    }
}

#[doc(hidden)]
#[no_mangle]
pub fn export_delete_ship(key: i32) {
    unsafe {
        SHIPS.as_mut().unwrap().remove(&key);
    }
}
