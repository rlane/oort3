use log::info;
use rhai::{Engine, INT};

fn log(x: INT) {
    info!("Script logged: {}", x);
}

pub fn eval(code: &str) {
    let mut engine = Engine::new();

    engine.register_fn("log", log);

    engine.consume(code).expect("engine.consume failed");
}
