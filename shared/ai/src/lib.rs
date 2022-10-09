mod user;

// For compatibility.
pub use oort_api::prelude;

static mut SHIPS: Vec<Option<(usize, user::Ship)>> = Vec::new();

#[doc(hidden)]
#[no_mangle]
pub fn export_tick_ship(index: u32, generation: u32) {
    let index = index as usize;
    let generation = generation as usize;
    unsafe {
        oort_api::dbg::reset();
        if SHIPS.len() <= index {
            SHIPS.resize_with(index + 1, || None);
        }
        match SHIPS.get_mut(index) {
            Some(Some((gen, ship))) => {
                if *gen != generation {
                    *ship = user::Ship::new();
                    *gen = generation;
                }
                ship.tick();
            }
            Some(x @ None) => {
                let mut ship = user::Ship::new();
                ship.tick();
                *x = Some((generation, ship));
            }
            _ => {}
        }
        oort_api::dbg::update();
    }
}
