pub mod fps;
pub mod frame_timer;

use log::{debug, info};
use nalgebra::{point, vector, Point2};
use oort_renderer::Renderer;
use oort_simulator::scenario::Status;
use oort_simulator::ship::ShipClass;
use oort_simulator::simulation::{self, PHYSICS_TICK_LENGTH};
use oort_simulator::snapshot::{self, ShipSnapshot, Snapshot};
use std::collections::{HashMap, VecDeque};
use web_sys::{Element, HtmlCanvasElement};
use yew::NodeRef;

const ZOOM_SPEED: f32 = 0.02;
const MIN_ZOOM: f32 = 5e-6;
const MAX_ZOOM: f32 = 1e-2;
const INITIAL_ZOOM: f32 = 4e-4;
const SNAPSHOT_PRELOAD: usize = 5;
const MAX_SNAPSHOT_REQUESTS_IN_FLIGHT: usize = 10;

pub struct UI {
    version: String,
    snapshot: Option<Snapshot>,
    pending_snapshots: VecDeque<Snapshot>,
    renderer: Renderer,
    canvas: HtmlCanvasElement,
    zoom: f32,
    camera_target: Point2<f32>,
    frame_timer: frame_timer::FrameTimer,
    status: Status,
    quit: bool,
    single_steps: i32,
    paused: bool,
    keys_down: std::collections::HashSet<String>,
    keys_ignored: std::collections::HashSet<String>,
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
    status_ref: NodeRef,
    picked_ref: NodeRef,
    touches: HashMap<i32, Touch>,
    drag_start: Option<Point2<i32>>,
    needs_render: bool,
}

unsafe impl Send for UI {}

impl UI {
    pub fn new(
        request_snapshot: yew::Callback<()>,
        nonce: u32,
        version: String,
        canvas_ref: NodeRef,
        status_ref: NodeRef,
        picked_ref: NodeRef,
    ) -> Self {
        if let Some(elem) = status_ref.cast::<Element>() {
            elem.set_text_content(Some("LOADING..."));
        }

        let canvas = canvas_ref
            .cast::<HtmlCanvasElement>()
            .expect("canvas element");
        let mut renderer = Renderer::new(canvas.clone()).expect("Failed to create renderer");
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
            canvas,
            zoom,
            camera_target,
            frame_timer,
            status: Status::Running,
            quit: false,
            single_steps,
            paused,
            keys_down,
            keys_ignored,
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
            status_ref,
            picked_ref,
            touches: HashMap::new(),
            drag_start: None,
            needs_render: true,
        }
    }

    pub fn render(&mut self) {
        if self.quit {
            return;
        }
        self.needs_render = false;

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
            self.zoom /= 1.0 + ZOOM_SPEED;
        }
        if self.keys_down.contains("x") && self.zoom < MAX_ZOOM {
            self.zoom *= 1.0 + ZOOM_SPEED;
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
            self.set_status_message("EXITED");
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
            Status::Draw => {
                status_msgs.push("DRAW".to_string());
            }
            _ if self.paused => {
                status_msgs.push("PAUSED".to_string());
            }
            _ => {}
        }

        if self.pending_snapshots.len() <= 1 && !fast_forward {
            status_msgs.push("SLOW SIM".to_owned());
        }

        if self.debug {
            if let Some(snapshot) = self.snapshot.as_ref() {
                status_msgs.push(format!(
                    "TICK {}",
                    (snapshot.time / PHYSICS_TICK_LENGTH).round() as i64
                ));
            }
        }

        if self.frame % 10 == 0 || self.paused || self.status != Status::Running {
            status_msgs.push(format!("{:.0} fps", self.fps.fps()));
            if self.debug {
                let (a, b, c) = self.frame_timer.get_latency();
                status_msgs.push(format!("UI {a:.1}/{b:.1}/{c:.1} ms",));
                if let Some(snapshot) = self.snapshot.as_ref() {
                    status_msgs.push(format!("PHYS {:.1} ms", snapshot.timing.physics * 1e3));
                    status_msgs.push(format!("SCRIPT {:.1} ms", snapshot.timing.script * 1e3));
                }
                status_msgs.push(format!("SNAP {}", self.pending_snapshots.len()));
            }
            status_msgs.push(self.version.clone());
            let status_msg = status_msgs.join("; ");
            if status_msg != self.last_status_msg {
                self.set_status_message(&status_msg);
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

        self.needs_render = true;
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

        if first_snapshot {
            // Zoom out to show all ships.
            let maxdist = snapshot
                .ships
                .iter()
                .map(|ship| nalgebra::distance(&ship.position, &point![0.0, 0.0]))
                .fold(0.0, |a, b| if a > b { a } else { b });
            let cornerdist = nalgebra::distance(&point![0.0, 0.0], &self.renderer.unproject(0, 0));
            self.zoom = (self.zoom * cornerdist as f32 / (2.0 * maxdist as f32))
                .clamp(MIN_ZOOM, INITIAL_ZOOM);

            // Pick player ship if there's only one.
            let own_ships: Vec<_> = snapshot
                .ships
                .iter()
                .filter(|ship| ship.team == 0)
                .collect();
            if own_ships.len() == 1 {
                self.picked_ship_id = Some(own_ships[0].id);
            }
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
        self.needs_render = true;
    }

    pub fn on_wheel_event(&mut self, e: web_sys::WheelEvent) {
        let amount = e.delta_y();
        self.zoom *= (1.0 - amount.signum() as f32 * ZOOM_SPEED).powf(amount.abs() as f32 / 30.0);
        self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);

        // Move camera target to keep cursor in the same location.
        let zoom_target = self.renderer.unproject(e.offset_x(), e.offset_y());
        self.renderer.set_view(self.zoom, self.camera_target);
        let new_zoom_target = self.renderer.unproject(e.offset_x(), e.offset_y());
        let diff = new_zoom_target - zoom_target;
        self.camera_target -= vector![diff.x as f32, diff.y as f32];

        self.needs_render = true;
    }

    fn ship_pick_radius(class: ShipClass) -> f64 {
        match class {
            ShipClass::Planet => 2000.0,
            _ => 60.0,
        }
    }

    pub fn on_pointer_event(&mut self, e: web_sys::PointerEvent) {
        let bounds = self.canvas.get_bounding_client_rect();
        let canvas_position = point![
            e.client_x() - bounds.x() as i32,
            e.client_y() - bounds.y() as i32
        ];
        let world_position = self
            .renderer
            .unproject(canvas_position.x, canvas_position.y)
            + vector![self.camera_target.x as f64, self.camera_target.y as f64];

        log::debug!(
            "PointerEvent: pointer_id={} pointer_type={} buttons={} canvas={:?} world={:?}",
            e.pointer_id(),
            e.pointer_type(),
            e.buttons(),
            canvas_position,
            world_position
        );

        if e.buttons() == 0 {
            self.touches.remove(&e.pointer_id());
            if let Some(start_canvas_position) = std::mem::take(&mut self.drag_start) {
                if (canvas_position - start_canvas_position)
                    .cast::<f64>()
                    .magnitude()
                    < 10.0
                {
                    let extra_radius = (self.renderer.unproject(10, 0)
                        - self.renderer.unproject(0, 0))
                    .magnitude();
                    self.picked_ship_id = self.snapshot.as_ref().and_then(|snapshot| {
                        snapshot
                            .ships
                            .iter()
                            .filter(|ship| {
                                nalgebra::distance(&ship.position, &world_position)
                                    < Self::ship_pick_radius(ship.class) + extra_radius
                            })
                            .min_by_key(|ship| {
                                nalgebra::distance(&ship.position, &world_position) as i64
                            })
                            .map(|ship| ship.id)
                    });
                    self.update_picked();
                    self.needs_render = true;
                }
            }
            return;
        }

        if let Some(touch) = self.touches.get_mut(&e.pointer_id()) {
            let diff = (touch.world_position - world_position).cast();
            self.camera_target += diff;
            self.renderer.set_view(self.zoom, self.camera_target);
        } else {
            self.touches
                .insert(e.pointer_id(), Touch { world_position });
        }

        if self.drag_start.is_none() {
            self.drag_start = Some(canvas_position);
        }

        self.needs_render = true;
    }

    pub fn on_blur_event(&mut self, _e: web_sys::FocusEvent) {
        self.keys_down.clear();
        self.needs_render = true;
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
                ..
            } = ship;
            let debug_text = self
                .snapshot
                .as_ref()
                .and_then(|s| s.debug_text.get(&self.picked_ship_id.unwrap()))
                .cloned()
                .unwrap_or_default();
            if let Some(elem) = self.picked_ref.cast::<Element>() {
                elem.set_text_content(Some(&format!(
                    "{class:?}\nTeam: {team:?}\nHealth: {health:.0}\n{debug_text}"
                )));
            }
        } else if let Some(elem) = self.picked_ref.cast::<Element>() {
            elem.set_text_content(Some(""));
        }
        self.renderer.set_picked_ship(self.picked_ship_id);
    }

    pub fn set_status_message(&self, text: &str) {
        if let Some(elem) = self.status_ref.cast::<Element>() {
            elem.set_text_content(Some(text));
        }
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    pub fn needs_render(&self) -> bool {
        self.needs_render
            || !(self.paused || self.status != Status::Running)
            || !self.keys_down.is_empty()
    }
}

#[derive(Debug)]
struct Touch {
    world_position: Point2<f64>,
}
