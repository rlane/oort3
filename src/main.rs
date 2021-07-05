mod frame_timer;
mod renderer;
mod simulation;

use macroquad::input::KeyCode;
use macroquad::math::vec2;
use macroquad::{audio, camera, color, input, rand, text, window};
use rapier2d_f64::prelude::*;

pub struct Ball {
    body: RigidBodyHandle,
    r: f32,
}

pub const WORLD_SIZE: f32 = 1000.0;

#[macroquad::main("Oort")]
async fn main() {
    let mut sim = simulation::Simulation::new();
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
        renderer::render(camera_target, zoom, &sim, &balls);
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
