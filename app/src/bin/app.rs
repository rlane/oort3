//use js_sys::Array;
//use wasm_bindgen::{prelude::*, JsCast};

//#[global_allocator]
//static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

//use oort_app;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("initializing logging");
    log::info!("starting");
    oort_app::run_app().unwrap();
}
