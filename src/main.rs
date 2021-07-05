mod frame_timer;

use macroquad::input::KeyCode;
use macroquad::math::{vec2, Vec2};
use macroquad::{audio, camera, color, input, rand, shapes, text, window};

struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    r: f32,
}

const WORLD_SIZE: f32 = 1000.0;

#[macroquad::main("Oort")]
async fn main() {
    let mut balls = vec![];
    let collision_sound = audio::load_sound("assets/collision.wav").await.unwrap();
    let mut zoom = 0.001;
    let mut camera_target = vec2(0.0, 0.0);
    let mut frame_timer: frame_timer::FrameTimer = Default::default();

    for _ in 0..100 {
        let r = rand::gen_range(10.0, 20.0);
        let s = 10.0;
        balls.push(Ball {
            x: rand::gen_range(r - WORLD_SIZE / 2.0, WORLD_SIZE / 2.0 - r),
            y: rand::gen_range(r - WORLD_SIZE / 2.0, WORLD_SIZE / 2.0 - r),
            vx: rand::gen_range(-s, s),
            vy: rand::gen_range(-s, s),
            r: r,
        });
    }

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
        simulate(&mut balls, collision_sound);
        frame_timer.end("simulate");

        frame_timer.start("render");
        render(camera_target, zoom, &balls);
        frame_timer.end("render");

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

fn simulate(balls: &mut Vec<Ball>, collision_sound: audio::Sound) {
    for ball in balls.iter_mut() {
        ball.x += ball.vx;
        ball.y += ball.vy;

        if ball.x - ball.r < -WORLD_SIZE / 2.0 || ball.x + ball.r > WORLD_SIZE / 2.0 {
            ball.vx *= -1.0;
        }

        if ball.y - ball.r < -WORLD_SIZE / 2.0 || ball.y + ball.r > WORLD_SIZE / 2.0 {
            ball.vy *= -1.0;
        }
    }

    let n = balls.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let dist_squared =
                (balls[i].x - balls[j].x).powf(2.0) + (balls[i].y - balls[j].y).powf(2.0);
            if dist_squared < (balls[i].r + balls[j].r).powf(2.0) {
                balls[i].vx *= -1.0;
                balls[i].vy *= -1.0;
                balls[j].vx *= -1.0;
                balls[j].vy *= -1.0;
                audio::play_sound_once(collision_sound);
            }
        }
    }
}

fn render(camera_target: Vec2, zoom: f32, balls: &[Ball]) {
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
        shapes::draw_circle(ball.x, ball.y, ball.r, color::YELLOW);
    }
}
