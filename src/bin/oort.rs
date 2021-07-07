use macroquad::input::KeyCode;
use macroquad::math::vec2;
use macroquad::{audio, camera, color, input, text, window};
use oort::{frame_timer, renderer, simulation};

#[macroquad::main("Oort")]
async fn main() {
    let mut sim = simulation::Simulation::new();
    let collision_sound = audio::load_sound("assets/collision.wav").await.unwrap();
    let mut zoom = 0.001;
    let mut camera_target = vec2(0.0, 0.0);
    let mut frame_timer: frame_timer::FrameTimer = Default::default();
    let mut paused = false;
    let mut single_steps = 0;

    let ship0 = oort::ship::create(&mut sim, -100.0, 0.0, 0.0, 0.0, 0.0);
    oort::ship::create(&mut sim, 100.0, 0.0, 0.0, 0.0, std::f64::consts::PI);

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
            println!(
                "Number of: ships={} bullets={}",
                sim.ships.iter().count(),
                sim.bullets.iter().count()
            );
        }
        if input::is_key_pressed(KeyCode::Space) {
            paused = !paused;
            single_steps = 0;
        }
        if input::is_key_pressed(KeyCode::N) {
            paused = true;
            single_steps += 1;
        }

        if !paused {
            let ship_handle = ship0;
            let force = 1e4;
            if input::is_key_down(KeyCode::Up) {
                sim.ship_mut(ship_handle).thrust_main(force);
            }
            if input::is_key_down(KeyCode::Down) {
                sim.ship_mut(ship_handle).thrust_main(-force);
            }
            if input::is_key_down(KeyCode::Left) {
                if input::is_key_down(KeyCode::LeftShift) {
                    sim.ship_mut(ship_handle).thrust_lateral(force);
                } else {
                    sim.ship_mut(ship_handle).thrust_angular(force);
                }
            }
            if input::is_key_down(KeyCode::Right) {
                if input::is_key_down(KeyCode::LeftShift) {
                    sim.ship_mut(ship_handle).thrust_lateral(-force);
                } else {
                    sim.ship_mut(ship_handle).thrust_angular(-force);
                }
            }
            if input::is_key_pressed(KeyCode::F) {
                sim.ship_mut(ship_handle).fire_weapon();
            }
            if input::is_key_down(KeyCode::LeftShift) && input::is_key_down(KeyCode::F) {
                sim.ship_mut(ship_handle).fire_weapon();
            }
        }

        if !paused || single_steps > 0 {
            frame_timer.start("simulate");
            sim.step();
            frame_timer.end("simulate");
            if single_steps > 0 {
                single_steps -= 1;
            }
        }

        frame_timer.start("render");
        renderer::render(camera_target, zoom, &sim);
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

        if paused {
            text::draw_text(
                "PAUSED",
                window::screen_width() / 2.0 - 96.0,
                window::screen_height() - 30.0,
                32.0,
                color::WHITE,
            );
        }

        window::next_frame().await
    }

    camera::set_default_camera();
    text::draw_text(
        "Game over",
        window::screen_width() / 2.0,
        window::screen_height() / 2.0,
        100.0,
        color::RED,
    );
}
