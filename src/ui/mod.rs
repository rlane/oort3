pub mod code_size;
pub mod fps;
pub mod frame_timer;
pub mod telemetry;
pub mod userid;

use crate::{api, renderer, script, simulation};
use log::{debug, error, info};
use nalgebra::{point, vector, Point2};
use rand::Rng;
use simulation::scenario;
use simulation::scenario::Status;
use simulation::snapshot::Snapshot;
use telemetry::Telemetry;
use wasm_bindgen::JsValue;

const MIN_ZOOM: f32 = 5e-5;
const MAX_ZOOM: f32 = 1e-2;
const INITIAL_ZOOM: f32 = 4e-4;

pub struct UI {
    sim: Box<simulation::Simulation>,
    snapshot: Snapshot,
    renderer: renderer::Renderer,
    zoom: f32,
    camera_target: Point2<f32>,
    frame_timer: frame_timer::FrameTimer,
    status: Status,
    quit: bool,
    single_steps: i32,
    paused: bool,
    scenario: Box<dyn scenario::Scenario>,
    keys_down: std::collections::HashSet<String>,
    keys_ignored: std::collections::HashSet<String>,
    status_div: web_sys::Element,
    frame: u64,
    last_render_time: f64,
    physics_time: f64,
    fps: fps::FPS,
    latest_code: String,
    manual_control: bool,
    debug: bool,
}

unsafe impl Send for UI {}

impl UI {
    pub fn new(scenario_name: &str, mut code: &str) -> Self {
        info!("Loading scenario {}", scenario_name);
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let status_div = document
            .get_element_by_id("status")
            .expect("should have a status div");
        status_div.set_inner_html("Hello from Rust");

        let mut sim = Box::new(simulation::Simulation::new());
        let renderer = renderer::Renderer::new().expect("Failed to create renderer");
        let zoom = INITIAL_ZOOM;
        let camera_target = point![0.0, 0.0];
        let frame_timer: frame_timer::FrameTimer = Default::default();
        let mut paused = false;
        let single_steps = 0;
        let seed: u64 = rand::thread_rng().gen();

        let mut scenario = scenario::load(scenario_name);

        let keys_down = std::collections::HashSet::<String>::new();
        let keys_ignored = std::collections::HashSet::<String>::new();

        let userid = userid::get_userid();
        log::info!("userid {}", &userid);
        log::info!("username {}", &userid::get_username(&userid));

        if !code.is_empty() {
            let storage = window
                .local_storage()
                .expect("failed to get local storage")
                .unwrap();
            if let Err(msg) = storage.set_item(&format!("/code/{}", scenario.name()), code) {
                error!("Failed to save code: {:?}", msg);
            }
        } else {
            code = "fn tick() {}";
        }

        sim.upload_code(/*team=*/ 0, code);
        scenario.init(&mut sim, seed);
        let snapshot = sim.snapshot();
        let latest_code = code.to_string();
        if !sim.events().errors.is_empty() {
            paused = true;
        }

        let ui = UI {
            sim,
            snapshot,
            renderer,
            zoom,
            camera_target,
            frame_timer,
            status: Status::Running,
            quit: false,
            single_steps,
            paused,
            scenario,
            keys_down,
            keys_ignored,
            status_div,
            frame: 0,
            last_render_time: instant::now(),
            physics_time: instant::now(),
            fps: fps::FPS::new(),
            latest_code,
            manual_control: false,
            debug: false,
        };
        ui.display_errors(&ui.sim.events().errors);
        ui
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
        if self.keys_down.contains("z") && self.zoom > MIN_ZOOM {
            self.zoom *= 0.99;
        }
        if self.keys_down.contains("x") && self.zoom < MAX_ZOOM {
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
        if self.keys_down.contains("g") && !self.keys_ignored.contains("g") {
            self.keys_ignored.insert("g".to_string());
            self.debug = !self.debug;
            self.renderer.set_debug(self.debug);
        }
        if self.keys_down.contains("q") {
            self.status_div.set_text_content(Some("Exited"));
            self.quit = true;
        }
        if self.keys_down.contains("m") {
            self.manual_control = true;
        }

        if self.manual_control && !self.paused {
            if let Some(&ship_handle) = self.sim.ships.iter().next() {
                let linear_acc = 100.0;
                let angular_acc = 1.0;
                if self.keys_down.contains("ArrowUp") {
                    self.sim
                        .ship_mut(ship_handle)
                        .accelerate(vector![linear_acc, 0.0]);
                }
                if self.keys_down.contains("ArrowDown") {
                    self.sim
                        .ship_mut(ship_handle)
                        .accelerate(vector![-linear_acc, 0.0]);
                }
                if self.keys_down.contains("ArrowLeft") {
                    if self.keys_down.contains("Shift") {
                        self.sim
                            .ship_mut(ship_handle)
                            .accelerate(vector![0.0, linear_acc]);
                    } else {
                        self.sim.ship_mut(ship_handle).torque(angular_acc);
                    }
                }
                if self.keys_down.contains("ArrowRight") {
                    if self.keys_down.contains("Shift") {
                        self.sim
                            .ship_mut(ship_handle)
                            .accelerate(vector![0.0, -linear_acc]);
                    } else {
                        self.sim.ship_mut(ship_handle).torque(-angular_acc);
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

        if self.status == Status::Running {
            self.status = self.scenario.status(&self.sim);
            if self.status == Status::Finished {
                if !self.manual_control && !self.sim.cheats {
                    telemetry::send(Telemetry::FinishScenario {
                        scenario_name: self.scenario.name(),
                        code: self.latest_code.to_string(),
                        ticks: self.sim.tick(),
                        code_size: code_size::calculate(&self.latest_code),
                    });
                }
                self.display_finished_screen();
            }
        }

        if self.status == Status::Running && (!self.paused || self.single_steps > 0) {
            let dt = simulation::PHYSICS_TICK_LENGTH * 1e3;
            self.physics_time = self.physics_time.max(now - dt * 2.0);
            if self.single_steps > 0 || self.physics_time + dt < now {
                self.scenario.tick(&mut self.sim);
                self.sim.step();
                self.physics_time += dt;
                if !self.sim.events().errors.is_empty() {
                    self.display_errors(&self.sim.events().errors);
                    self.paused = true;
                }
                self.snapshot = self.sim.snapshot();
                self.snapshot.scenario_lines = self.scenario.lines();
                self.renderer.update(&self.snapshot);
            }
            if self.single_steps > 0 {
                self.single_steps -= 1;
            }
        }

        self.renderer
            .render(self.camera_target, self.zoom, &self.snapshot);

        if self.manual_control {
            status_msgs.push("MANUAL".to_string());
        }

        if self.sim.cheats {
            status_msgs.push("CHEATS".to_string());
        }

        if self.paused {
            status_msgs.push("PAUSED".to_string());
        } else if self.status == Status::Finished {
            status_msgs.push("FINISHED".to_string());
        } else if self.status == Status::Failed {
            status_msgs.push("FAILED".to_string());
        }

        if self.frame % 10 == 0 {
            status_msgs.push(format!("{:.0} fps", self.fps.fps()));
            {
                let (a, b, c) = self.frame_timer.get_latency();
                status_msgs.push(format!("{:.1}/{:.1}/{:.1} ms", a, b, c,));
            }
            let status_msg = status_msgs.join("; ");
            self.status_div.set_text_content(Some(&status_msg));
        }

        if self.frame == 600 {
            info!(
                "Average frame time after {} frames: {:.1} ms",
                self.frame,
                self.frame_timer.get_average()
            );
        }

        self.frame += 1;

        self.frame_timer.end(instant::now());
    }

    pub fn on_key_event(&mut self, e: web_sys::KeyboardEvent) {
        if e.type_() == "keydown" {
            self.keys_down.insert(e.key());
        } else if e.type_() == "keyup" {
            self.keys_down.remove(&e.key());
            self.keys_ignored.remove(&e.key());
        }
    }

    pub fn on_wheel_event(&mut self, e: web_sys::WheelEvent) {
        let amount = e.delta_y();
        self.zoom *= (1.0 - amount.signum() * 0.01).powf(amount.abs() / 30.0) as f32;
        self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);

        // Move camera target to keep cursor in the same location.
        let zoom_target = self
            .renderer
            .unproject(e.offset_x() as i32, e.offset_y() as i32);
        self.renderer.set_view(self.zoom, self.camera_target);
        let new_zoom_target = self
            .renderer
            .unproject(e.offset_x() as i32, e.offset_y() as i32);
        let diff = new_zoom_target - zoom_target;
        self.camera_target -= vector![diff.x as f32, diff.y as f32];
    }

    pub fn get_initial_code(&self) -> String {
        let window = web_sys::window().expect("no global `window` exists");
        let storage = window
            .local_storage()
            .expect("failed to get local storage")
            .unwrap();
        match storage.get_item(&format!("/code/{}", self.scenario.name())) {
            Ok(Some(code)) => code,
            Ok(None) => {
                info!("No saved code, using starter code");
                self.scenario.initial_code()
            }
            Err(msg) => {
                error!("Failed to load code: {:?}", msg);
                self.scenario.initial_code()
            }
        }
    }

    pub fn display_finished_screen(&self) {
        api::display_mission_complete_overlay(
            &self.scenario.name(),
            self.sim.time(),
            code_size::calculate(&self.latest_code),
            &self
                .scenario
                .next_scenario()
                .unwrap_or_else(|| "".to_string()),
        );
    }

    pub fn display_errors(&self, errors: &[script::Error]) {
        api::display_errors(JsValue::from_serde(errors).unwrap());
    }
}

impl Default for UI {
    fn default() -> Self {
        Self::new("asteroid", "fn tick() {}")
    }
}
