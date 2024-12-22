pub mod fps;
pub mod frame_timer;
pub mod setting;

use log::{debug, info};
use nalgebra::{point, vector, Point2, Vector2};
use oort_renderer::Renderer;
use oort_simulator::model;
use oort_simulator::scenario::Status;
use oort_simulator::simulation::{self, PHYSICS_TICK_LENGTH};
use oort_simulator::snapshot::{self, ShipSnapshot, Snapshot};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use web_sys::{Element, HtmlCanvasElement};
use yew::NodeRef;

use crate::editor_window::{is_mac, EditorAction};

const ZOOM_SPEED: f32 = 0.02;
const MIN_ZOOM: f32 = 5e-6;
const MAX_ZOOM: f32 = 5e-3;
const INITIAL_ZOOM: f32 = 1e-3;
const SNAPSHOT_PRELOAD: usize = 5;
const MAX_SNAPSHOT_REQUESTS_IN_FLIGHT: usize = 10;

pub struct UI {
    version: String,
    seed: u32,
    snapshot: Option<Snapshot>,
    uninterpolated_snapshot: Option<Snapshot>,
    snapshots: Vec<Snapshot>,
    renderer: Renderer,
    canvas: HtmlCanvasElement,
    zoom: f32,
    camera_focus: Point2<f32>,
    camera_offset: Vector2<f32>,
    frame_timer: frame_timer::FrameTimer,
    status: Status,
    quit: bool,
    steps_forward: i32,
    steps_backward: i32,
    paused: bool,
    slowmo: bool,
    keys_down: std::collections::HashSet<String>,
    keys_pressed: std::collections::HashSet<String>,
    frame: u64,
    start_time: instant::Instant,
    last_render_time: instant::Instant,
    /// Time difference between the first and current frame
    physics_time: std::time::Duration,
    fps: fps::FPS,
    debug: bool,
    last_status_msg: String,
    snapshot_requests_in_flight: usize,
    nonce: u32,
    request_snapshot: yew::Callback<()>,
    picked_ship_id: Option<u64>,
    chasing_ship_id: Option<u64>,
    status_ref: NodeRef,
    picked_ref: NodeRef,
    touches: HashMap<i32, Touch>,
    drag_start: Option<Point2<i32>>,
    needs_render: bool,
    on_editor_action: yew::Callback<EditorAction>,
}

unsafe impl Send for UI {}

impl UI {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        request_snapshot: yew::Callback<()>,
        on_editor_action: yew::Callback<EditorAction>,
        seed: u32,
        nonce: u32,
        version: String,
        canvas_ref: NodeRef,
        status_ref: NodeRef,
        picked_ref: NodeRef,
        paused: bool,
    ) -> Self {
        if let Some(elem) = status_ref.cast::<Element>() {
            elem.set_text_content(Some("LOADING..."));
        }

        let canvas = canvas_ref
            .cast::<HtmlCanvasElement>()
            .expect("canvas element");
        let mut renderer = Renderer::new(canvas.clone()).expect("Failed to create renderer");
        let zoom = INITIAL_ZOOM;
        let camera_focus = point![0.0, 0.0];
        let camera_offset = vector![0.0, 0.0];
        renderer.set_view(zoom, camera_focus + camera_offset);
        let frame_timer: frame_timer::FrameTimer = Default::default();

        let keys_down = std::collections::HashSet::<String>::new();
        let keys_pressed = std::collections::HashSet::<String>::new();

        let debug = setting::read("debug", false);
        renderer.set_debug(debug);
        renderer.set_blur(setting::read("blur", true));
        renderer.set_nlips(setting::read("nlips", false));

        UI {
            version,
            seed,
            snapshot: None,
            uninterpolated_snapshot: None,
            snapshots: Vec::new(),
            renderer,
            canvas,
            zoom,
            camera_focus,
            camera_offset,
            frame_timer,
            status: Status::Running,
            quit: false,
            steps_forward: 0,
            steps_backward: 0,
            paused,
            slowmo: false,
            keys_down,
            keys_pressed,
            frame: 0,
            start_time: instant::Instant::now(),
            last_render_time: instant::Instant::now(),
            physics_time: std::time::Duration::ZERO,
            fps: fps::FPS::new(),
            debug,
            last_status_msg: "".to_owned(),
            snapshot_requests_in_flight: 0,
            nonce,
            request_snapshot,
            picked_ship_id: None,
            chasing_ship_id: None,
            status_ref,
            picked_ref,
            touches: HashMap::new(),
            drag_start: None,
            needs_render: true,
            on_editor_action,
        }
    }

    pub fn render(&mut self) {
        if self.quit {
            return;
        }
        self.needs_render = false;

        let now = instant::Instant::now();
        let elapsed = now - self.last_render_time;
        self.last_render_time = now;
        if elapsed.as_millis() > 20 {
            debug!("Late render: {:.1} ms", elapsed.as_millis());
        }
        self.fps
            .start_frame((now - self.start_time).as_millis() as f64);
        self.frame_timer
            .start((now - self.start_time).as_millis() as f64);

        let mut status_msgs: Vec<String> = Vec::new();
        let was_paused = self.paused;
        let was_slowmo = self.slowmo;

        let camera_step = 0.01 / self.zoom;
        if self.keys_down.contains("KeyW") {
            self.camera_offset.y += camera_step;
        }
        if self.keys_down.contains("KeyS") {
            self.camera_offset.y -= camera_step;
        }
        if self.keys_down.contains("KeyA") {
            self.camera_offset.x -= camera_step;
        }
        if self.keys_down.contains("KeyD") {
            self.camera_offset.x += camera_step;
        }
        if self.keys_down.contains("KeyZ") && self.zoom > MIN_ZOOM {
            self.zoom /= 1.0 + ZOOM_SPEED;
        }
        if self.keys_down.contains("KeyX") && self.zoom < MAX_ZOOM {
            self.zoom *= 1.0 + ZOOM_SPEED;
        }
        if self.keys_pressed.contains("Space") {
            self.paused = !self.paused;
            self.steps_forward = 0;
            self.steps_backward = 0;
        }
        if self.keys_pressed.contains("KeyN") {
            self.paused = true;
            self.steps_forward += 1;
        } else if self.keys_pressed.contains("KeyP") {
            self.paused = true;
            self.steps_backward += 1;
        }

        if is_mac() {
            if self.keys_pressed.contains("Meta-Enter") {
                self.on_editor_action.emit(EditorAction::Execute)
            }
            if self.keys_pressed.contains("Meta-Shift-Enter") {
                self.on_editor_action.emit(EditorAction::Replay)
            }
            if self.keys_pressed.contains("Meta-Alt-Enter") {
                self.on_editor_action.emit(EditorAction::ReplayPaused)
            }
        } else {
            if self.keys_pressed.contains("Ctrl-Enter") {
                self.on_editor_action.emit(EditorAction::Execute)
            }
            if self.keys_pressed.contains("Ctrl-Shift-Enter") {
                self.on_editor_action.emit(EditorAction::Replay)
            }
            if self.keys_pressed.contains("Ctrl-Alt-Enter") {
                self.on_editor_action.emit(EditorAction::ReplayPaused)
            }
        }

        if self.keys_pressed.contains("KeyG") {
            self.debug = !self.debug;
            self.renderer.set_debug(self.debug);
            setting::write("debug", &self.debug);
        }
        if self.keys_pressed.contains("KeyQ") {
            self.set_status_message("EXITED");
            self.quit = true;
        }
        let fast_forward = self.keys_down.contains("KeyF");
        self.slowmo = self.keys_down.contains("KeyM");
        if self.keys_pressed.contains("KeyB") {
            self.renderer.set_blur(!self.renderer.get_blur());
            setting::write("blur", &self.renderer.get_blur());
        }
        if self.keys_pressed.contains("KeyV") {
            self.renderer.set_nlips(!self.renderer.get_nlips());
            setting::write("nlips", &self.renderer.get_nlips());
        }
        if self.keys_pressed.contains("KeyC") {
            self.chasing_ship_id = match self.chasing_ship_id {
                Some(_) => None,
                None => self.picked_ship_id,
            };
            if self.chasing_ship_id.is_some() {
                self.camera_offset = vector![0.0, 0.0];
            }
        }

        if !self.paused && !self.slowmo {
            self.physics_time += elapsed;
        }

        if self.status == Status::Running
            && (!self.paused
                || self.steps_forward > 0
                || self.steps_backward > 0
                || fast_forward
                || self.slowmo
                || self.snapshot.is_none())
        {
            let dt = std::time::Duration::from_secs_f64(simulation::PHYSICS_TICK_LENGTH);
            if fast_forward {
                for _ in 0..10 {
                    self.physics_time += dt;
                    self.update_snapshot(true);
                }
            } else if self.steps_forward > 0 {
                self.physics_time += dt;
                self.update_snapshot(false);
            } else if self.steps_backward > 0 {
                // Avoiding overflow
                self.physics_time = if self.physics_time < dt {
                    Duration::from_secs(0)
                } else {
                    self.physics_time - dt
                };

                self.update_snapshot(false);
                self.steps_backward -= 1;
            } else if self.slowmo {
                self.physics_time += dt / 10;
                self.update_snapshot(true);
            } else {
                self.update_snapshot(true);
            }
            if self.steps_forward > 0 {
                self.steps_forward -= 1;
            }
        } else if self.paused != was_paused || self.slowmo != was_slowmo {
            self.update_snapshot(false);
        }

        if self.snapshot.is_some() {
            let chasing_ship = self.chasing_ship_id.and_then(|id| {
                self.snapshot
                    .as_ref()
                    .unwrap()
                    .ships
                    .iter()
                    .find(|s| s.id == id)
            });
            if let Some(s) = chasing_ship {
                self.camera_focus = s.position.cast();
            } else {
                self.chasing_ship_id = None;
            }
            self.renderer.render(
                self.camera_target(),
                self.zoom,
                self.snapshot.as_ref().unwrap(),
            );

            if self.snapshot.as_ref().unwrap().cheats {
                status_msgs.push("CHEATS".to_string());
            }
        }

        match self.status {
            Status::Victory { team: 0 } => {
                status_msgs.push(format!(
                    "VICTORY in {:.3}s",
                    self.snapshot.as_ref().unwrap().time
                ));
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

        if self.snapshots.len() <= 1 && !fast_forward {
            status_msgs.push("SLOW SIM".to_owned());
        }

        if self.debug {
            status_msgs.push(format!("SEED {}", self.seed));
            if let Some(snapshot) = self.snapshot.as_ref() {
                status_msgs.push(format!(
                    "TICK {}",
                    (snapshot.time / PHYSICS_TICK_LENGTH).round() as i64
                ));
            }
        }

        if self.frame % 10 == 0 || self.paused || self.status != Status::Running {
            if self.status == Status::Running {
                status_msgs.push(format!("{:.0} fps", self.fps.fps()));
            }
            if self.debug {
                let (a, b, c) = self.frame_timer.get_latency();
                status_msgs.push(format!("UI {a:.1}/{b:.1}/{c:.1} ms",));
                if let Some(snapshot) = self.snapshot.as_ref() {
                    status_msgs.push(format!("SIM {:.1} ms", snapshot.timing.total() * 1e3));
                }
                status_msgs.push(format!("SNAP {}", self.snapshots.len()));
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

        self.frame_timer
            .end((instant::Instant::now() - self.start_time).as_millis() as f64);

        self.keys_pressed.clear();
    }

    pub fn on_snapshot(&mut self, snapshot: Snapshot) {
        if snapshot.nonce != self.nonce {
            return;
        }

        self.snapshots.push(snapshot);
        if self.snapshot_requests_in_flight > 0 {
            self.snapshot_requests_in_flight -= 1;
        }

        self.needs_render = true;
    }

    /// Sets the displayed snapshot by index
    ///
    /// The time of the first snapshot is expected to be at 0 at
    /// the time this was written. This will need to be updated if
    /// that expectation changes.
    ///
    /// Snapshots are evenly spaced, so an index can easily be converted into a time
    /// and vice versa
    pub fn set_snapshot_index(&mut self, index: usize) {
        self.physics_time = Duration::from_secs_f64((index as f64) * PHYSICS_TICK_LENGTH);
        self.paused = true;
        self.update_snapshot(false);
        self.needs_render = true;
        let _ = self.canvas.focus();
    }

    pub fn update_snapshot(&mut self, interpolate: bool) {
        let snapshot_index =
            (self.physics_time.as_secs_f64() / PHYSICS_TICK_LENGTH).round() as usize;

        // Reaching the end of the preloaded snapshots, need to preload more
        if SNAPSHOT_PRELOAD + 1 + snapshot_index > self.snapshots.len()
            && self.snapshot_requests_in_flight < MAX_SNAPSHOT_REQUESTS_IN_FLIGHT
        {
            self.request_snapshot.emit(());
            self.request_snapshot.emit(());
            self.snapshot_requests_in_flight += 2;
        }

        if snapshot_index < self.snapshots.len()
            && std::time::Duration::from_secs_f64(self.snapshots[snapshot_index].time)
                <= self.physics_time
        {
            let first_snapshot = self.snapshot.is_none();

            self.snapshot = self.snapshots.get(snapshot_index).cloned();
            self.uninterpolated_snapshot = self.snapshot.clone();

            let snapshot = self.snapshot.as_mut().unwrap();

            if first_snapshot {
                // Set zoom to show all ships.
                let mut points = snapshot
                    .ships
                    .iter()
                    .map(|ship| ship.position)
                    .collect::<Vec<_>>();
                points.extend(
                    snapshot
                        .scenario_lines
                        .iter()
                        .flat_map(|line| [line.a, line.b]),
                );
                let max_dim = points
                    .iter()
                    .map(|p| p.x.abs().max(p.y.abs()))
                    .fold(0.0, |a: f64, b| a.max(b));
                let top_left = self.renderer.unproject(0, 0);
                let view_dim = top_left.x.abs().max(top_left.y.abs());
                self.zoom = (0.8 * self.zoom * view_dim as f32 / max_dim as f32)
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
        }

        if let Some(snapshot) = self.snapshot.as_mut() {
            // Time of the snapshot
            let t = std::time::Duration::from_secs_f64(snapshot.time);

            // Find the difference between current time and snapshot time, with a max value of 16
            // TODO: Why 16? The tick length is 16.66 miliseconds. Maybe that's it?
            let mut delta = if t <= self.physics_time {
                (self.physics_time - t).min(Duration::from_millis(16))
            } else {
                (t - self.physics_time).min(Duration::from_millis(16))
            };

            // TODO: Unsure what this is for
            // TODO: More magic numbers
            if delta > Duration::from_millis(3) {
                delta -= Duration::from_millis(1);
            }

            // Set the new current time to the time of the snapshot + the delta
            // Could just be unchanged if the delta is less than 4
            self.physics_time = if t <= self.physics_time {
                t + delta
            } else {
                t - delta
            };

            if interpolate {
                // Alters snapshot to make it appear as it would at `physics_time`
                snapshot::interpolate(snapshot, delta.as_secs_f64());
            } else if snapshot.time != self.uninterpolated_snapshot.as_ref().unwrap().time {
                // TODO: Do we need to do this? Does this ever happen?
                // uninterpolated_snapshot is a copy of snapshot
                *snapshot = self.uninterpolated_snapshot.as_ref().unwrap().clone();
            }

            self.renderer.update(snapshot);

            snapshot.particles.clear();
        }
    }

    pub fn on_key_event(&mut self, e: web_sys::KeyboardEvent) {
        if e.type_() == "keydown" {
            self.keys_down.insert(e.code());

            let mut pressed_key: String = e.code().to_string();
            if e.shift_key() {
                pressed_key.insert_str(0, "Shift-");
            }
            if e.meta_key() {
                pressed_key.insert_str(0, "Meta-");
            }
            if e.alt_key() {
                pressed_key.insert_str(0, "Alt-");
            }
            if e.ctrl_key() {
                pressed_key.insert_str(0, "Ctrl-");
            }
            self.keys_pressed.insert(pressed_key);
        } else if e.type_() == "keyup" {
            self.keys_down.remove(&e.code());
        }
        self.needs_render = true;
    }

    pub fn on_wheel_event(&mut self, e: web_sys::WheelEvent) {
        let amount = e.delta_y();
        self.zoom *= (1.0 - amount.signum() as f32 * ZOOM_SPEED).powf(amount.abs() as f32 / 30.0);
        self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);

        // Move camera target to keep cursor in the same location.
        let zoom_target = self.renderer.unproject(e.offset_x(), e.offset_y());
        self.renderer.set_view(self.zoom, self.camera_target());
        let new_zoom_target = self.renderer.unproject(e.offset_x(), e.offset_y());
        let diff = new_zoom_target - zoom_target;
        self.camera_offset -= vector![diff.x as f32, diff.y as f32];

        self.needs_render = true;
        let _ = self.canvas.focus();
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
            + vector![self.camera_target().x as f64, self.camera_target().y as f64];
        let world_camera_offset = self
            .renderer
            .unproject(canvas_position.x, canvas_position.y)
            + vector![self.camera_offset.x as f64, self.camera_offset.y as f64];

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
                    let classes = self
                        .snapshot
                        .as_ref()
                        .map(|snapshot| {
                            snapshot
                                .ships
                                .iter()
                                .map(|ship| ship.class)
                                .collect::<HashSet<_>>()
                        })
                        .unwrap_or_default();
                    let radiuses = classes
                        .iter()
                        .map(|&class| (class, model::radius(class) as f64 + extra_radius))
                        .collect::<HashMap<_, _>>();
                    self.picked_ship_id = self.snapshot.as_ref().and_then(|snapshot| {
                        snapshot
                            .ships
                            .iter()
                            .filter(|ship| {
                                nalgebra::distance(&ship.position, &world_position)
                                    < radiuses[&ship.class]
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
            let diff = (touch.world_camera_offset - world_camera_offset).cast();
            self.camera_offset += diff;
            self.renderer.set_view(self.zoom, self.camera_target());
        } else {
            self.touches.insert(
                e.pointer_id(),
                Touch {
                    world_camera_offset,
                },
            );
        }

        if self.drag_start.is_none() {
            self.drag_start = Some(canvas_position);
        }

        self.needs_render = true;
        let _ = self.canvas.focus();
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

    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    pub fn snapshot_index(&self) -> usize {
        (self.snapshot.as_ref().map_or(0.0, |s| s.time) / PHYSICS_TICK_LENGTH).round() as usize
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
                fuel,
                active_abilities,
                ..
            } = ship;
            let debug_text = self
                .snapshot
                .as_ref()
                .and_then(|s| s.debug_text.get(&self.picked_ship_id.unwrap()))
                .cloned()
                .unwrap_or_default();
            if let Some(elem) = self.picked_ref.cast::<Element>() {
                let fuel_text = if let Some(fuel) = fuel {
                    format!("Fuel: {:.0}\n", fuel)
                } else {
                    "".to_string()
                };
                let active_abilities_text = if !active_abilities.is_empty() {
                    format!(
                        "Active abilities: {}\n",
                        active_abilities
                            .iter()
                            .map(|ability| format!("{:?}", ability))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                } else {
                    "".to_string()
                };
                elem.set_text_content(Some(&format!(
                    "{class:?}\nTeam: {team:?}\nHealth: {health:.0}\n{fuel_text}{active_abilities_text}{debug_text}"
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
            || !self.keys_pressed.is_empty()
    }

    pub fn camera_target(&self) -> Point2<f32> {
        self.camera_focus + self.camera_offset
    }
}

#[derive(Debug)]
struct Touch {
    world_camera_offset: Point2<f64>,
}
