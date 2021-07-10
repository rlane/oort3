use macroquad::{shapes, window};

#[macroquad::main("Oort")]
async fn main() {
    let mut ui = oort::ui::UI::new();

    loop {
        ui.render();

        // HACK required by macroquad.
        shapes::draw_circle(0.0, 0.0, 1.0, macroquad::color::WHITE);

        window::next_frame().await
    }
}
