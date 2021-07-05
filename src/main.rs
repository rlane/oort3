mod frame_timer;

use macroquad::input::KeyCode;
use macroquad::math::{vec2, Vec2};
use macroquad::{audio, camera, color, input, rand, shapes, text, window};
use rapier2d_f64::prelude::*;

struct Ball {
    body: RigidBodyHandle,
    r: f32,
}

const WORLD_SIZE: f32 = 1000.0;

#[macroquad::main("Oort")]
async fn main() {
    let mut sim = Simulation::new();
    let mut balls: Vec<Ball> = vec![];
    let collision_sound = audio::load_sound("assets/collision.wav").await.unwrap();
    let mut zoom = 0.001;
    let mut camera_target = vec2(0.0, 0.0);
    let mut frame_timer: frame_timer::FrameTimer = Default::default();

    for _ in 0..100 {
        let s = 500.0;
        let r = rand::gen_range(10.0, 20.0);
        let x = rand::gen_range(r - WORLD_SIZE / 2.0, WORLD_SIZE / 2.0 - r);
        let y = rand::gen_range(r - WORLD_SIZE / 2.0, WORLD_SIZE / 2.0 - r);
        let vx = rand::gen_range(-s, s);
        let vy = rand::gen_range(-s, s);
        let rigid_body = RigidBodyBuilder::new_dynamic()
            .translation(vector![x.into(), y.into()])
            .linvel(vector![vx.into(), vy.into()])
            .build();
        let handle = sim.bodies.insert(rigid_body);
        let collider = ColliderBuilder::ball(r.into())
            .restitution(1.0)
            .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
            .build();
        sim.colliders
            .insert_with_parent(collider, handle, &mut sim.bodies);
        balls.push(Ball { body: handle, r: r });
    }

    let mut make_edge = |x: f32, y: f32, a: f32| {
        let edge_length = WORLD_SIZE;
        let edge_width = 1.0;
        let rigid_body = RigidBodyBuilder::new_static()
            .translation(vector![x.into(), y.into()])
            .rotation(a.into())
            .build();
        let handle = sim.bodies.insert(rigid_body);
        let collider = ColliderBuilder::cuboid(edge_length as f64, edge_width)
            .restitution(1.0)
            .build();
        sim.colliders
            .insert_with_parent(collider, handle, &mut sim.bodies);
    };
    make_edge(0.0, WORLD_SIZE / 2.0, 0.0);
    make_edge(0.0, -WORLD_SIZE / 2.0, 0.0);
    make_edge(WORLD_SIZE / 2.0, 0.0, std::f32::consts::PI / 2.0);
    make_edge(-WORLD_SIZE / 2.0, 0.0, std::f32::consts::PI / 2.0);

    loop {
        frame_timer.start("frame");

        let camera_step = 0.01 / zoom;
        if input::is_key_down(KeyCode::W) {
            camera_target.y += camera_step;
        }
        if input::is_key_down(KeyCode::S) {
            camera_target.y -= camera_step;
        }
        if input::is_key_down(KeyCode::A) {
            camera_target.x -= camera_step;
        }
        if input::is_key_down(KeyCode::D) {
            camera_target.x += camera_step;
        }
        if input::is_key_down(KeyCode::Z) {
            zoom *= 0.99;
        }
        if input::is_key_down(KeyCode::X) {
            zoom *= 1.01;
        }
        if input::is_key_down(KeyCode::Q) | input::is_key_down(KeyCode::Escape) {
            break;
        }
        if input::is_key_pressed(KeyCode::U) {
            for name in frame_timer.get_names() {
                let (a, b, c) = frame_timer.get(name);
                println!("{}: {:.1}/{:.1}/{:.1} ms", name, a * 1e3, b * 1e3, c * 1e3);
            }
        }

        frame_timer.start("simulate");
        sim.step();
        frame_timer.end("simulate");

        frame_timer.start("render");
        render(camera_target, zoom, &sim, &balls);
        frame_timer.end("render");

        if sim.collision_event_handler.collision.load() {
            sim.collision_event_handler.collision.store(false);
            audio::play_sound_once(collision_sound);
        }

        frame_timer.end("frame");

        camera::set_default_camera();
        {
            let (a, b, c) = frame_timer.get("frame");
            text::draw_text(
                format!(
                    "Frame time: {:.1}/{:.1}/{:.1} ms",
                    a * 1e3,
                    b * 1e3,
                    c * 1e3
                )
                .as_str(),
                window::screen_width() - 400.0,
                20.0,
                32.0,
                color::WHITE,
            );
        }

        window::next_frame().await
    }

    camera::set_default_camera();
    text::draw_text(
        format!("Game over").as_str(),
        window::screen_width() / 2.0,
        window::screen_height() / 2.0,
        100.0,
        color::RED,
    );
}

struct Simulation {
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: JointSet,
    pub collision_event_handler: CollisionEventHandler,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
}

impl Simulation {
    pub fn new() -> Simulation {
        return Simulation {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: JointSet::new(),
            collision_event_handler: CollisionEventHandler::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
        };
    }

    pub fn step(self: &mut Simulation) {
        let gravity = vector![0.0, 0.0];
        let physics_hooks = ();

        self.physics_pipeline.step(
            &gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &mut self.ccd_solver,
            &physics_hooks,
            &self.collision_event_handler,
        );
    }
}

struct CollisionEventHandler {
    collision: crossbeam::atomic::AtomicCell<bool>,
}

impl CollisionEventHandler {
    pub fn new() -> CollisionEventHandler {
        return CollisionEventHandler {
            collision: crossbeam::atomic::AtomicCell::new(false),
        };
    }
}

impl EventHandler for CollisionEventHandler {
    fn handle_intersection_event(&self, _event: IntersectionEvent) {}

    fn handle_contact_event(&self, event: ContactEvent, _contact_pair: &ContactPair) {
        match event {
            ContactEvent::Started(_, _) => {
                self.collision.store(true);
                //println!("Collision: {:?}", event);
            }
            _ => {}
        }
    }
}

fn render(camera_target: Vec2, zoom: f32, sim: &Simulation, balls: &[Ball]) {
    window::clear_background(color::BLACK);

    camera::set_camera(&camera::Camera2D {
        zoom: vec2(
            zoom,
            zoom * window::screen_width() / window::screen_height(),
        ),
        target: camera_target,
        ..Default::default()
    });

    let grid_size = 100.0;
    let n = 1 + (WORLD_SIZE / grid_size) as i32;
    for i in -(n / 2)..(n / 2 + 1) {
        shapes::draw_line(
            (i as f32) * grid_size,
            -WORLD_SIZE / 2.0,
            (i as f32) * grid_size,
            WORLD_SIZE / 2.0,
            1.0,
            color::GREEN,
        );
        shapes::draw_line(
            -WORLD_SIZE / 2.0,
            (i as f32) * grid_size,
            WORLD_SIZE / 2.0,
            (i as f32) * grid_size,
            1.0,
            color::GREEN,
        );
    }

    {
        let v = -WORLD_SIZE / 2.0;
        shapes::draw_line(-v, -v, v, -v, 1.0, color::RED);
        shapes::draw_line(-v, v, v, v, 1.0, color::RED);
        shapes::draw_line(-v, -v, -v, v, 1.0, color::RED);
        shapes::draw_line(v, -v, v, v, 1.0, color::RED);
    }

    for ball in balls {
        let body = sim.bodies.get(ball.body).unwrap();
        shapes::draw_circle(
            body.position().translation.x as f32,
            body.position().translation.y as f32,
            ball.r,
            color::YELLOW,
        );
    }
}
