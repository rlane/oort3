use crate::{frame_timer, renderer, scenario, simulation};
use log::info;
use nalgebra::{point, Point2};
use std::sync::mpsc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, KeyboardEvent};

pub struct UI {
    sim: simulation::Simulation,
    renderer: renderer::Renderer,
    zoom: f32,
    camera_target: Point2<f32>,
    frame_timer: frame_timer::FrameTimer,
    finished: bool,
    single_steps: i32,
    paused: bool,
    scenario: Box<dyn scenario::Scenario>,
    keys_down: std::collections::HashSet<String>,
    keys_ignored: std::collections::HashSet<String>,
    status_div: web_sys::Element,
    key_rx: mpsc::Receiver<KeyboardEvent>,
    tick: u64,
}

unsafe impl Send for UI {}

impl UI {
    pub fn new() -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Debug).expect("initializing logging");
        info!("Initializing UI");

        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let status_div = document
            .get_element_by_id("status")
            .expect("should have a status div");
        status_div.set_inner_html("Hello from Rust");

        let canvas = document
            .get_element_by_id("glcanvas")
            .expect("expecting a canvas");

        let (key_tx, key_rx) = mpsc::channel::<KeyboardEvent>();
        let key_callback = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
            if key_tx.send(e).is_err() {
                console::log_1(&"Failed to enqueue key".into());
            }
        }) as Box<dyn FnMut(_)>);

        canvas
            .add_event_listener_with_callback("keydown", key_callback.as_ref().unchecked_ref())
            .expect("adding event listener failed");
        canvas
            .add_event_listener_with_callback("keyup", key_callback.as_ref().unchecked_ref())
            .expect("adding event listener failed");
        key_callback.forget();

        let mut sim = simulation::Simulation::new();
        let renderer = renderer::Renderer::new();
        let zoom = 0.001;
        let camera_target = point![0.0, 0.0];
        let frame_timer: frame_timer::FrameTimer = Default::default();
        let paused = false;
        let finished = false;
        let single_steps = 0;

        let scenario = scenario::load("asteroid");
        scenario.init(&mut sim);

        let keys_down = std::collections::HashSet::<String>::new();
        let keys_ignored = std::collections::HashSet::<String>::new();

        UI {
            sim,
            renderer,
            zoom,
            camera_target,
            frame_timer,
            finished,
            single_steps,
            paused,
            scenario,
            keys_down,
            keys_ignored,
            status_div,
            key_rx,
            tick: 0,
        }
    }

    pub fn render(&mut self) {
        let mut status_msgs: Vec<String> = Vec::new();

        self.frame_timer.start("frame");

        while let Ok(e) = self.key_rx.try_recv() {
            if e.type_() == "keydown" {
                self.keys_down.insert(e.key());
            } else if e.type_() == "keyup" {
                self.keys_down.remove(&e.key());
                self.keys_ignored.remove(&e.key());
            }
        }

        let camera_step = 0.01 / self.zoom;
        if self.keys_down.contains("w") {
            self.camera_target.y += camera_step;
        }
        if self.keys_down.contains("s") {
            self.camera_target.y -= camera_step;
        }
        if self.keys_down.contains("a") {
            self.camera_target.x -= camera_step;
        }
        if self.keys_down.contains("d") {
            self.camera_target.x += camera_step;
        }
        if self.keys_down.contains("z") {
            self.zoom *= 0.99;
        }
        if self.keys_down.contains("x") {
            self.zoom *= 1.01;
        }
        if self.keys_down.contains("u") && !self.keys_ignored.contains("u") {
            self.keys_ignored.insert("u".to_string());
            for name in self.frame_timer.get_names() {
                let (a, b, c) = self.frame_timer.get(name);
                println!("{}: {:.1}/{:.1}/{:.1} ms", name, a, b, c);
            }
            println!(
                "Number of: ships={} bullets={}",
                self.sim.ships.iter().count(),
                self.sim.bullets.iter().count()
            );
        }
        if self.keys_down.contains(" ") && !self.keys_ignored.contains(" ") {
            self.keys_ignored.insert(" ".to_string());
            self.paused = !self.paused;
            self.single_steps = 0;
        }
        if self.keys_down.contains("n") && !self.keys_ignored.contains("n") {
            self.keys_ignored.insert("n".to_string());
            self.paused = true;
            self.single_steps += 1;
        }

        if !self.paused {
            if let Some(&ship_handle) = self.sim.ships.iter().next() {
                let force = 1e4;
                if self.keys_down.contains("ArrowUp") {
                    self.sim.ship_mut(ship_handle).thrust_main(force);
                }
                if self.keys_down.contains("ArrowDown") {
                    self.sim.ship_mut(ship_handle).thrust_main(-force);
                }
                if self.keys_down.contains("ArrowLeft") {
                    if self.keys_down.contains("Shift") {
                        self.sim.ship_mut(ship_handle).thrust_lateral(force);
                    } else {
                        self.sim.ship_mut(ship_handle).thrust_angular(force);
                    }
                }
                if self.keys_down.contains("ArrowRight") {
                    if self.keys_down.contains("Shift") {
                        self.sim.ship_mut(ship_handle).thrust_lateral(-force);
                    } else {
                        self.sim.ship_mut(ship_handle).thrust_angular(-force);
                    }
                }
                if self.keys_down.contains("f") {
                    self.sim.ship_mut(ship_handle).fire_weapon();
                }
                if self.keys_down.contains("Shift") && self.keys_down.contains("f") {
                    self.sim.ship_mut(ship_handle).fire_weapon();
                }
                if self.keys_down.contains("Shift") && self.keys_down.contains("k") {
                    self.sim.ship_mut(ship_handle).explode();
                }
            }
        }

        if !self.finished && self.scenario.tick(&mut self.sim) == scenario::Status::Finished {
            self.finished = true;
        }

        if !self.finished && (!self.paused || self.single_steps > 0) {
            self.frame_timer.start("simulate");
            self.sim.step();
            self.frame_timer.end("simulate");
            if self.single_steps > 0 {
                self.single_steps -= 1;
            }
        }

        self.frame_timer.start("render");
        self.renderer
            .render(self.camera_target, self.zoom, &self.sim);
        self.frame_timer.end("render");

        if self.sim.collided {
            self.sim.collided = false;
            println!("collided");
        }

        self.frame_timer.end("frame");

        {
            let (a, b, c) = self.frame_timer.get("frame");
            status_msgs.push(format!("Frame time: {:.1}/{:.1}/{:.1} ms", a, b, c,));
        }

        if self.paused {
            status_msgs.push("PAUSED".to_string());
        } else if self.finished {
            status_msgs.push("FINISHED".to_string());
        }

        if self.tick % 10 == 0 {
            let status_msg = status_msgs.join("; ");
            self.status_div.set_inner_html(&status_msg);
        }

        self.tick += 1;
    }
}

impl Default for UI {
    fn default() -> Self {
        Self::new()
    }
}
