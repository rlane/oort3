use macroquad::{window, color, shapes};
use rand::Rng;

struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    r: f32,
}

#[macroquad::main("Oort")]
async fn main() {
    let mut balls = vec![];
    let mut rng = rand::thread_rng();

    for _ in 0..10 {
        let r = rng.gen_range(10.0..20.0);
        let s = 10.0;
        balls.push(Ball {
            x: rng.gen_range(r .. (window::screen_width() - r)),
            y: rng.gen_range(r .. (window::screen_height() - r)),
            vx: rng.gen_range(-s..s),
            vy: rng.gen_range(-s..s),
            r: r,
        });
    }

    loop {
        window::clear_background(color::BLACK);

        let grid_size = 100.0;
        let n = 1 + (window::screen_width() / grid_size) as i32;
        for i in 0..n {
            shapes::draw_line(
                (i as f32) * grid_size,
                0.0,
                (i as f32) * grid_size,
                window::screen_height(),
                1.0,
                color::GREEN,
            );
            shapes::draw_line(
                0.0,
                (i as f32) * grid_size,
                window::screen_width(),
                (i as f32) * grid_size,
                1.0,
                color::GREEN,
            );
        }

        for ball in &mut balls {
            ball.x += ball.vx;
            ball.y += ball.vy;

            if ball.x < 0.0 || ball.x > window::screen_width() {
                ball.vx *= -1.0;
            }

            if ball.y < 0.0 || ball.y > window::screen_height() {
                ball.vy *= -1.0;
            }

            shapes::draw_circle(ball.x, ball.y, ball.r, color::YELLOW);
        }

        window::next_frame().await
    }
}
