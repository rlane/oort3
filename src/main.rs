use macroquad::prelude::*;

#[macroquad::main("Oort")]
async fn main() {
    loop {
        clear_background(BLACK);

        let grid_size = 100.0;
        let n = 1 + (screen_width() / grid_size) as i32;
        for i in 0..n {
            draw_line(
                (i as f32) * grid_size,
                0.0,
                (i as f32) * grid_size,
                screen_height(),
                1.0,
                GREEN,
            );
            draw_line(
                0.0,
                (i as f32) * grid_size,
                screen_width(),
                (i as f32) * grid_size,
                1.0,
                GREEN,
            );
        }

        //draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        //draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        //draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        //draw_text("HELLO", 20.0, 20.0, 20.0, DARKGRAY);

        next_frame().await
    }
}
