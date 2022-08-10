pub mod fps;
pub mod frame_timer;

use log::{debug, info};
use nalgebra::{point, vector, Point2};
use oort_renderer::Renderer;
use oort_simulator::scenario::Status;
use oort_simulator::simulation;
use oort_simulator::snapshot::{self, ShipSnapshot, Snapshot};
use std::collections::VecDeque;

const MIN_ZOOM: f32 = 5e-5;
const MAX_ZOOM: f32 = 1e-2;
const INITIAL_ZOOM: f32 = 4e-4;
const SNAPSHOT_PRELOAD: usize = 5;
const MAX_SNAPSHOT_REQUESTS_IN_FLIGHT: usize = 10;

pub struct UI {
    version: String,
    snapshot: Option<Snapshot>,
    pending_snapshots: VecDeque<Snapshot>,
    renderer: Renderer,
    zoom: f32,
    camera_target: Point2<f32>,
    frame_timer: frame_timer::FrameTimer,
    status: Status,
    quit: bool,
    single_steps: i32,
    paused: bool,
    keys_down: std::collections::HashSet<String>,
    keys_ignored: std::collections::HashSet<String>,
    status_div: web_sys::Element,
    frame: u64,
    last_render_time: f64,
    physics_time: f64,
    fps: fps::FPS,
    debug: bool,
    last_status_msg: String,
    snapshot_requests_in_flight: usize,
    nonce: u32,
    request_snapshot: yew::Callback<()>,
    picked_ship_id: Option<u64>,
    picked_text: web_sys::Element,
}

unsafe impl Send for UI {}

impl UI {
    pub fn new(request_snapshot: yew::Callback<()>, nonce: u32, version: String) -> Self {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let status_div = document
            .get_element_by_id("status")
            .expect("should have a status div");
        status_div.set_inner_html("LOADING...");
        let picked_text = document
            .get_element_by_id("picked_text")
            .expect("should have a picked_text element");

        let mut renderer = Renderer::new().expect("Failed to create renderer");
        let zoom = INITIAL_ZOOM;
        let camera_target = point![0.0, 0.0];
        renderer.set_view(zoom, point![camera_target.x, camera_target.y]);
        let frame_timer: frame_timer::FrameTimer = Default::default();
        let paused = false;
        let single_steps = 0;

        let keys_down = std::collections::HashSet::<String>::new();
        let keys_ignored = std::collections::HashSet::<String>::new();

        UI {
            version,
            snapshot: None,
            pending_snapshots: VecDeque::new(),
            renderer,
            zoom,
            camera_target,
            frame_timer,
            status: Status::Running,
            quit: false,
            single_steps,
            paused,
            keys_down,
            keys_ignored,
            status_div,
            frame: 0,
            last_render_time: instant::now(),
            physics_time: instant::now(),
            fps: fps::FPS::new(),
            debug: false,
            last_status_msg: "".to_owned(),
            snapshot_requests_in_flight: 0,
            nonce,
            request_snapshot,
            picked_ship_id: None,
            picked_text,
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
        let fast_forward = self.keys_down.contains("f");

        if self.paused {
            self.physics_time = now;
        }

        if self.status == Status::Running && (!self.paused || self.single_steps > 0 || fast_forward)
        {
            let dt = simulation::PHYSICS_TICK_LENGTH;
            self.physics_time = self.physics_time.max(now - dt * 2.0);
            if fast_forward {
                for _ in 0..10 {
                    self.physics_time += dt;
                    self.update_snapshot();
                }
            } else if self.single_steps > 0 || self.physics_time + dt < now {
                self.physics_time += dt;
                self.update_snapshot();
            } else if self.snapshot.is_some() {
                snapshot::interpolate(
                    self.snapshot.as_mut().unwrap(),
                    (now - self.last_render_time) / 1e3,
                );
            }
            if self.single_steps > 0 {
                self.single_steps -= 1;
            }
        }

        if self.snapshot.is_some() {
            self.renderer.render(
                self.camera_target,
                self.zoom,
                self.snapshot.as_ref().unwrap(),
            );

            if self.snapshot.as_ref().unwrap().cheats {
                status_msgs.push("CHEATS".to_string());
            }
        }

        match self.status {
            Status::Victory { team: 0 } => {
                status_msgs.push("VICTORY".to_string());
            }
            Status::Victory { .. } | Status::Failed => {
                status_msgs.push("DEFEAT".to_string());
            }
            _ if self.paused => {
                status_msgs.push("PAUSED".to_string());
            }
            _ => {}
        }

        if self.pending_snapshots.len() <= 1 && !fast_forward {
            status_msgs.push("SLOW SIM".to_owned());
        }

        if self.frame % 10 == 0 {
            status_msgs.push(format!("{:.0} fps", self.fps.fps()));
            if self.debug {
                let (a, b, c) = self.frame_timer.get_latency();
                status_msgs.push(format!("UI {:.1}/{:.1}/{:.1} ms", a, b, c,));
                if let Some(snapshot) = self.snapshot.as_ref() {
                    status_msgs.push(format!("PHYS {:.1} ms", snapshot.timing.physics * 1e3));
                    status_msgs.push(format!("SCRIPT {:.1} ms", snapshot.timing.script * 1e3));
                }
                status_msgs.push(format!("SNAP {}", self.pending_snapshots.len()));
            }
            status_msgs.push(self.version.clone());
            let status_msg = status_msgs.join("; ");
            if status_msg != self.last_status_msg {
                self.status_div.set_text_content(Some(&status_msg));
                self.last_status_msg = status_msg;
            }

            self.update_picked();
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
        self.last_render_time = now;
    }

    pub fn on_snapshot(&mut self, snapshot: Snapshot) {
        if snapshot.nonce != self.nonce {
            return;
        }

        self.pending_snapshots.push_back(snapshot);
        if self.snapshot_requests_in_flight > 0 {
            self.snapshot_requests_in_flight -= 1;
        }
    }

    pub fn update_snapshot(&mut self) {
        if self.pending_snapshots.len() < SNAPSHOT_PRELOAD
            && self.snapshot_requests_in_flight < MAX_SNAPSHOT_REQUESTS_IN_FLIGHT
        {
            self.request_snapshot.emit(());
            self.request_snapshot.emit(());
            self.snapshot_requests_in_flight += 2;
        }

        if self.pending_snapshots.is_empty() || self.pending_snapshots[0].time > self.physics_time {
            return;
        }

        let first_snapshot = self.snapshot.is_none();

        self.snapshot = self.pending_snapshots.pop_front();
        let snapshot = self.snapshot.as_ref().unwrap();

        // Zoom out to show all ships.
        if first_snapshot {
            let maxdist = snapshot
                .ships
                .iter()
                .map(|ship| nalgebra::distance(&ship.position, &point![0.0, 0.0]))
                .fold(0.0, |a, b| if a > b { a } else { b });
            let cornerdist = nalgebra::distance(&point![0.0, 0.0], &self.renderer.unproject(0, 0));
            self.zoom = (self.zoom * cornerdist as f32 / (1.5 * maxdist as f32))
                .clamp(MIN_ZOOM, INITIAL_ZOOM);
        }

        if !snapshot.errors.is_empty() {
            self.paused = true;
        }

        self.status = snapshot.status;

        self.renderer.update(snapshot);
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

    pub fn on_mouse_event(&mut self, e: web_sys::MouseEvent) {
        let target = self
            .renderer
            .unproject(e.offset_x() as i32, e.offset_y() as i32)
            + vector![self.camera_target.x as f64, self.camera_target.y as f64];
        self.picked_ship_id = self.snapshot.as_ref().and_then(|snapshot| {
            snapshot
                .ships
                .iter()
                .filter(|ship| nalgebra::distance(&ship.position, &target) < 60.0)
                .min_by_key(|ship| nalgebra::distance(&ship.position, &target) as i64)
                .map(|ship| ship.id)
        });
        self.update_picked();
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn snapshot(&self) -> Option<Snapshot> {
        self.snapshot.clone()
    }

    pub fn update_picked(&mut self) {
        if let Some(ship) = self.picked_ship_id.and_then(|id| {
            self.snapshot
                .as_ref()
                .and_then(|s| s.ships.iter().find(|ship| ship.id == id))
        }) {
            let ShipSnapshot {
                class,
                team,
                health,
                energy,
                ..
            } = ship;
            let energy = energy / 1e6;
            let debug_text = self
                .snapshot
                .as_ref()
                .and_then(|s| s.debug_text.get(&self.picked_ship_id.unwrap()))
                .cloned()
                .unwrap_or_default();
            self.picked_text.set_text_content(Some(&format!(
                "{class:?}\nTeam: {team:?}\nHealth: {health:.0} Energy: {energy:.0} MJ\n{debug_text}"
            )));
        } else {
            self.picked_text.set_text_content(Some(""));
        }
        self.renderer.set_picked_ship(self.picked_ship_id);
    }
}
