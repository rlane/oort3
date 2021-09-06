use crate::ui::UI;
use std::sync::atomic::{AtomicBool, Ordering};

static PANICKED: AtomicBool = AtomicBool::new(false);

fn has_panicked() -> bool {
    PANICKED.load(Ordering::SeqCst)
}

pub struct Game {
    ui: Option<Box<UI>>,
    request_snapshot: yew::Callback<()>,
}

impl Game {
    pub fn start(&mut self, scenario_name: &str, code: &str) {
        if has_panicked() {
            return;
        }
        self.ui = Some(Box::new(UI::new(
            scenario_name,
            code,
            self.request_snapshot.clone(),
        )));
    }

    pub fn render(&mut self) {
        if has_panicked() {
            return;
        }
        if self.ui.is_some() {
            self.ui.as_mut().unwrap().render();
        }
    }

    pub fn on_snapshot(&mut self, snapshot: oort_simulator::snapshot::Snapshot) {
        if has_panicked() {
            return;
        }
        if self.ui.is_some() {
            self.ui.as_mut().unwrap().on_snapshot(snapshot);
        }
    }

    pub fn on_key_event(&mut self, e: web_sys::KeyboardEvent) {
        if has_panicked() {
            return;
        }
        if self.ui.is_some() {
            self.ui.as_mut().unwrap().on_key_event(e);
        }
    }

    pub fn on_wheel_event(&mut self, e: web_sys::WheelEvent) {
        if has_panicked() {
            return;
        }
        if self.ui.is_some() {
            self.ui.as_mut().unwrap().on_wheel_event(e);
        }
    }

    pub fn finished_background_simulations(&mut self, results: js_sys::Array) {
        if has_panicked() {
            return;
        }
        let mut snapshots = vec![];
        for x in results.iter() {
            let x = js_sys::Uint8Array::from(x);
            snapshots.push(bincode::deserialize(&x.to_vec()).unwrap())
        }
        if self.ui.is_some() {
            self.ui
                .as_mut()
                .unwrap()
                .finished_background_simulations(&snapshots);
        }
    }
}

pub fn create(request_snapshot: yew::Callback<()>) -> Game {
    console_log::init_with_level(log::Level::Info).expect("initializing logging");
    Game {
        ui: None,
        request_snapshot,
    }
}