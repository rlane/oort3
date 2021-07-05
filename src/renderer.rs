use crate::simulation::{Simulation, WORLD_SIZE};
use macroquad::math::{vec2, Vec2};
use macroquad::{camera, color, shapes, window};

pub fn render(camera_target: Vec2, zoom: f32, sim: &Simulation) {
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

    for ball in &sim.balls {
        let body = sim.bodies.get(ball.body).unwrap();
        shapes::draw_circle(
            body.position().translation.x as f32,
            body.position().translation.y as f32,
            ball.r,
            color::YELLOW,
        );
    }
}
