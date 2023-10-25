mod user;

// For compatibility.
pub use oort_api::prelude;

use std::sync::Once;
use user::Ship;

static START: Once = Once::new();
static mut SHIP: Option<Ship> = None;

#[doc(hidden)]
#[no_mangle]
unsafe fn export_initialize() {}

#[doc(hidden)]
#[no_mangle]
pub unsafe fn export_tick_ship(_key: i32) {
    START.call_once(|| {
        oort_api::panic::install();
        oort_api::rng_state::set(oort_api::rng_state::RngState::new());
    });
    oort_api::dbg::reset();
    oort_api::panic::reset();
    unsafe {
        let ship = SHIP.get_or_insert_with(Ship::new);
        ship.tick();
        oort_api::dbg::update();
    }
}

#[doc(hidden)]
#[no_mangle]
pub unsafe fn export_delete_ship(_key: i32) {}
