use macroquad::{shapes, window};
use nalgebra::point;
use oort::{frame_timer, renderer, simulation};
use std::sync::mpsc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, KeyboardEvent};

#[macroquad::main("Oort")]
async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let status_div = document
        .get_element_by_id("status")
        .expect("should have a status div");
    status_div.set_inner_html("Hello from Rust");

    let canvas = document
        .get_element_by_id("glcanvas")
        .expect("expecting a canvas");

    let (key_tx, key_rx) = mpsc::channel::<KeyboardEvent>();
    let key_callback = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        if key_tx.send(e).is_err() {
            console::log_1(&"Failed to enqueue key".into());
        }
    }) as Box<dyn FnMut(_)>);

    canvas
        .add_event_listener_with_callback("keydown", key_callback.as_ref().unchecked_ref())
        .expect("adding event listener failed");
    canvas
        .add_event_listener_with_callback("keyup", key_callback.as_ref().unchecked_ref())
        .expect("adding event listener failed");
    key_callback.forget();

    let mut sim = simulation::Simulation::new();
    let mut renderer = renderer::Renderer::new();
    let mut zoom = 0.001;
    let mut camera_target = point![0.0, 0.0];
    let mut frame_timer: frame_timer::FrameTimer = Default::default();
    let mut paused = false;
    let mut finished = false;
    let mut single_steps = 0;

    let scenario = oort::scenario::load("asteroid");
    scenario.init(&mut sim);

    let mut keys_down = std::collections::HashSet::<String>::new();
    let mut keys_ignored = std::collections::HashSet::<String>::new();

    loop {
        let mut status_msgs: Vec<String> = Vec::new();

        frame_timer.start("frame");

        while let Ok(e) = key_rx.try_recv() {
            if e.type_() == "keydown" {
                keys_down.insert(e.key());
            } else if e.type_() == "keyup" {
                keys_down.remove(&e.key());
                keys_ignored.remove(&e.key());
            }
        }

        let camera_step = 0.01 / zoom;
        let is_key_down = |key: &str| keys_down.contains(key);
        if is_key_down("w") {
            camera_target.y += camera_step;
        }
        if is_key_down("s") {
            camera_target.y -= camera_step;
        }
        if is_key_down("a") {
            camera_target.x -= camera_step;
        }
        if is_key_down("d") {
            camera_target.x += camera_step;
        }
        if is_key_down("z") {
            zoom *= 0.99;
        }
        if is_key_down("x") {
            zoom *= 1.01;
        }
        if is_key_down("u") && !keys_ignored.contains("u") {
            keys_ignored.insert("u".to_string());
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
        if is_key_down(" ") && !keys_ignored.contains(" ") {
            keys_ignored.insert(" ".to_string());
            paused = !paused;
            single_steps = 0;
        }
        if is_key_down("n") && !keys_ignored.contains("n") {
            keys_ignored.insert("n".to_string());
            paused = true;
            single_steps += 1;
        }

        if !paused {
            if let Some(&ship_handle) = sim.ships.iter().next() {
                let force = 1e4;
                if is_key_down("ArrowUp") {
                    sim.ship_mut(ship_handle).thrust_main(force);
                }
                if is_key_down("ArrowDown") {
                    sim.ship_mut(ship_handle).thrust_main(-force);
                }
                if is_key_down("ArrowLeft") {
                    if is_key_down("Shift") {
                        sim.ship_mut(ship_handle).thrust_lateral(force);
                    } else {
                        sim.ship_mut(ship_handle).thrust_angular(force);
                    }
                }
                if is_key_down("ArrowRight") {
                    if is_key_down("Shift") {
                        sim.ship_mut(ship_handle).thrust_lateral(-force);
                    } else {
                        sim.ship_mut(ship_handle).thrust_angular(-force);
                    }
                }
                if is_key_down("f") {
                    sim.ship_mut(ship_handle).fire_weapon();
                }
                if is_key_down("Shift") && is_key_down("f") {
                    sim.ship_mut(ship_handle).fire_weapon();
                }
                if is_key_down("Shift") && is_key_down("k") {
                    sim.ship_mut(ship_handle).explode();
                }
            }
        }

        if !finished && scenario.tick(&mut sim) == oort::scenario::Status::Finished {
            finished = true;
        }

        if !finished && (!paused || single_steps > 0) {
            frame_timer.start("simulate");
            sim.step();
            frame_timer.end("simulate");
            if single_steps > 0 {
                single_steps -= 1;
            }
        }

        frame_timer.start("render");
        renderer.render(camera_target, zoom, &sim);
        frame_timer.end("render");

        if sim.collided {
            sim.collided = false;
            println!("collided");
        }

        frame_timer.end("frame");

        {
            let (a, b, c) = frame_timer.get("frame");
            status_msgs.push(format!(
                "Frame time: {:.1}/{:.1}/{:.1} ms",
                a * 1e3,
                b * 1e3,
                c * 1e3
            ));
        }

        if paused {
            status_msgs.push("PAUSED".to_string());
        } else if finished {
            status_msgs.push("FINISHED".to_string());
        }

        status_div.set_inner_html(&status_msgs.join("; "));

        // HACK required by macroquad.
        shapes::draw_circle(0.0, 0.0, 1.0, macroquad::color::WHITE);

        window::next_frame().await
    }
}
