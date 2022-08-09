mod user;

// For compatibility.
pub use oort_api::prelude;

static mut USER_STATE: Option<user::Ship> = None;

#[doc(hidden)]
#[no_mangle]
pub fn export_tick() {
    unsafe {
        oort_api::dbg::reset();
        if USER_STATE.is_none() {
            USER_STATE = Some(user::Ship::new());
        }
        USER_STATE.as_mut().unwrap().tick();
        oort_api::dbg::update();
    }
}
