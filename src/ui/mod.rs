pub mod fps;
pub mod frame_timer;

use crate::{renderer, simulation};
use log::{debug, info};
use nalgebra::{point, Point2};
use simulation::scenario;
use std::sync::mpsc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, KeyboardEvent};

pub struct UI {
    sim: Box<simulation::Simulation>,
    renderer: renderer::Renderer,
    zoom: f32,
    camera_target: Point2<f32>,
    frame_timer: frame_timer::FrameTimer,
    finished: bool,
    quit: bool,
    single_steps: i32,
    paused: bool,
    scenario: Box<dyn scenario::Scenario>,
    keys_down: std::collections::HashSet<String>,
    keys_ignored: std::collections::HashSet<String>,
    status_div: web_sys::Element,
    key_rx: mpsc::Receiver<KeyboardEvent>,
    tick: u64,
    last_render_time: f64,
    physics_time: f64,
    fps: fps::FPS,
}

unsafe impl Send for UI {}

impl UI {
    pub fn new(scenario_name: &str) -> Self {
        info!("Loading scenario {}", scenario_name);
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

        let mut sim = Box::new(simulation::Simulation::new());
        let renderer = renderer::Renderer::new().expect("Failed to create renderer");
        let zoom = 0.001;
        let camera_target = point![0.0, 0.0];
        let frame_timer: frame_timer::FrameTimer = Default::default();
        let paused = false;
        let finished = false;
        let single_steps = 0;

        let scenario = scenario::load(scenario_name);
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
            quit: false,
            single_steps,
            paused,
            scenario,
            keys_down,
            keys_ignored,
            status_div,
            key_rx,
            tick: 0,
            last_render_time: instant::now(),
            physics_time: instant::now(),
            fps: fps::FPS::new(),
        }
    }

    pub fn render(&mut self) {
        if self.quit {
            return;
        }

        let now = instant::now();
        if now - self.last_render_time > 20.0 {
            debug!("Late render: {:.1} ms", now - self.last_render_time);
        }
        self.last_render_time = now;
        self.fps.start_frame(now);
        self.frame_timer.start(now);

        let mut status_msgs: Vec<String> = Vec::new();

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
        if self.keys_down.contains("q") {
            self.status_div.set_text_content(Some("Exited"));
            self.quit = true;
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
                    self.sim.ship_mut(ship_handle).fire_weapon(0);
                }
                if self.keys_down.contains("Shift") && self.keys_down.contains("f") {
                    self.sim.ship_mut(ship_handle).fire_weapon(0);
                }
                if self.keys_down.contains("Shift") && self.keys_down.contains("k") {
                    self.sim.ship_mut(ship_handle).explode();
                }
            }
        }

        if self.paused {
            self.physics_time = now;
        }

        if !self.finished && self.scenario.tick(&mut self.sim) == scenario::Status::Finished {
            self.finished = true;
        }

        if !self.finished && (!self.paused || self.single_steps > 0) {
            let dt = simulation::PHYSICS_TICK_LENGTH * 1e3;
            self.physics_time = self.physics_time.max(now - dt * 2.0);
            if self.single_steps > 0 || self.physics_time + dt < now {
                self.sim.step();
                self.physics_time += dt;
            }
            if self.single_steps > 0 {
                self.single_steps -= 1;
            }
        }

        self.renderer.render(
            self.camera_target,
            self.zoom,
            &self.sim,
            &self.scenario.lines(),
        );

        if self.sim.collided {
            self.sim.collided = false;
            println!("collided");
        }

        if self.paused {
            status_msgs.push("PAUSED".to_string());
        } else if self.finished {
            status_msgs.push("FINISHED".to_string());
        }

        if self.tick % 10 == 0 {
            status_msgs.push(format!("{:.0} fps", self.fps.fps()));
            {
                let (a, b, c) = self.frame_timer.get_latency();
                status_msgs.push(format!("{:.1}/{:.1}/{:.1} ms", a, b, c,));
            }
            let status_msg = status_msgs.join("; ");
            self.status_div.set_text_content(Some(&status_msg));
        }

        if self.tick == 600 {
            info!(
                "Average frame time after {} ticks: {:.1} ms",
                self.tick,
                self.frame_timer.get_average()
            );
        }

        self.tick += 1;

        self.frame_timer.end(instant::now());
    }

    pub fn upload_code(&mut self, code: &str) {
        self.sim.upload_code(code);
    }

    pub fn get_initial_code(&self) -> String {
        self.scenario.initial_code()
    }
}

impl Default for UI {
    fn default() -> Self {
        Self::new("asteroid")
    }
}
