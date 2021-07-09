use crate::ship::ShipClass;
use crate::simulation::{Simulation, WORLD_SIZE};
use macroquad::math::{vec2, Vec2};
use macroquad::{camera, color, math, shapes, window};

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
    let n = 1 + (WORLD_SIZE as f32 / grid_size) as i32;
    for i in -(n / 2)..(n / 2 + 1) {
        shapes::draw_line(
            (i as f32) * grid_size,
            (-WORLD_SIZE as f32) / 2.0,
            (i as f32) * grid_size,
            (WORLD_SIZE as f32) / 2.0,
            1.0,
            color::GREEN,
        );
        shapes::draw_line(
            (-WORLD_SIZE as f32) / 2.0,
            (i as f32) * grid_size,
            (WORLD_SIZE as f32) / 2.0,
            (i as f32) * grid_size,
            1.0,
            color::GREEN,
        );
    }

    {
        let v = -WORLD_SIZE as f32 / 2.0;
        shapes::draw_line(-v, -v, v, -v, 1.0, color::RED);
        shapes::draw_line(-v, v, v, v, 1.0, color::RED);
        shapes::draw_line(-v, -v, -v, v, 1.0, color::RED);
        shapes::draw_line(v, -v, v, v, 1.0, color::RED);
    }

    for &index in sim.bullets.iter() {
        let bullet = sim.bullet(index);
        let body = bullet.body();
        let x = body.position().translation.x as f32;
        let y = body.position().translation.y as f32;
        let vx = body.linvel().x as f32;
        let vy = body.linvel().y as f32;
        let dt = 2.0 / 60.0;
        shapes::draw_line(x, y, x - vx * dt, y - vy * dt, 1.0, color::ORANGE);
    }

    for &index in sim.ships.iter() {
        let ship = sim.ship(index);
        let x = ship.position().x as f32;
        let y = ship.position().y as f32;
        let h = ship.heading() as f32;
        let translation = vec2(x, y);

        match ship.data().class {
            ShipClass::Fighter => draw_model(&crate::model::ship(), translation, h, zoom),
            ShipClass::Asteroid => draw_model(
                &crate::model::asteroid(ship.data().model_variant),
                translation,
                h,
                zoom,
            ),
        }
    }
}

fn draw_model(vertices: &[Vec2], translation: Vec2, heading: f32, zoom: f32) {
    let matrix = math::Mat2::from_angle(heading);
    let new_vertices = vertices
        .iter()
        .map(|&v| translation + matrix.mul_vec2(v))
        .collect::<Vec<_>>();
    let thickness = 3.0 / (zoom * 1000.0);
    let color = color::YELLOW;
    for i in 1..vertices.len() {
        shapes::draw_line(
            new_vertices[i - 1].x,
            new_vertices[i - 1].y,
            new_vertices[i].x,
            new_vertices[i].y,
            thickness,
            color,
        );
    }
    let i = vertices.len() - 1;
    shapes::draw_line(
        new_vertices[i].x,
        new_vertices[i].y,
        new_vertices[0].x,
        new_vertices[0].y,
        thickness,
        color,
    );
}
