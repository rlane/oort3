#[no_mangle]
pub static mut SYSTEM_STATE: [f64; 2] = [
    42.0,
    21.0
];

pub fn write_system_state(index: usize, value: f64) {
    unsafe {
        SYSTEM_STATE[index] = value;
    }
}

mod api {
    use super::write_system_state;

    pub fn accelerate(x: f64, y: f64) {
        write_system_state(0, x );
        write_system_state(1, y );
    }
}

#[no_mangle]
pub fn export_tick() {
    tick();
}

// User code

pub fn tick() {
    api::accelerate(1.0, 2.0);
}
