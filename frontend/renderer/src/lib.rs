pub mod blur;
pub mod buffer_arena;
pub mod bullet_renderer;
pub mod flare_renderer;
pub mod geometry;
pub mod glutil;
pub mod grid_renderer;
pub mod line_renderer;
pub mod particle_renderer;
pub mod ship_renderer;
pub mod text_renderer;
pub mod trail_renderer;

#[macro_use]
extern crate memoffset;

use blur::Blur;
use bullet_renderer::BulletRenderer;
use flare_renderer::FlareRenderer;
use grid_renderer::GridRenderer;
use line_renderer::LineRenderer;
use nalgebra::{point, vector, Matrix4, Point2};
use oort_api::Text;
use oort_simulator::simulation::Line;
use oort_simulator::snapshot::Snapshot;
use particle_renderer::ParticleRenderer;
use ship_renderer::ShipRenderer;
use text_renderer::TextRenderer;
use trail_renderer::TrailRenderer;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use WebGl2RenderingContext as gl;

pub struct Renderer {
    canvas: HtmlCanvasElement,
    context: WebGl2RenderingContext,
    grid_renderer: GridRenderer,
    line_renderer: LineRenderer,
    ship_renderer: ShipRenderer,
    bullet_renderer: BulletRenderer,
    particle_renderer: ParticleRenderer,
    trail_renderer: TrailRenderer,
    text_renderer: TextRenderer,
    flare_renderer: FlareRenderer,
    blur: Blur,
    projection_matrix: Matrix4<f32>,
    base_line_width: f32,
    debug: bool,
    picked_ship: Option<u64>,
    blur_enabled: bool,
}

impl Renderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let context = canvas
            .get_context("webgl2")?
            .expect("Failed to get webgl2 context")
            .dyn_into::<WebGl2RenderingContext>()?;

        let extensions = context
            .get_supported_extensions()
            .expect("getting extensions")
            .to_vec();
        let extensions = extensions
            .iter()
            .map(|s| s.as_string().unwrap())
            .collect::<Vec<_>>();
        log::debug!("Supported extensions: {:?}", extensions);

        context.enable(gl::BLEND);
        context.blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        Ok(Renderer {
            canvas,
            context: context.clone(),
            grid_renderer: GridRenderer::new(context.clone())?,
            line_renderer: LineRenderer::new(context.clone())?,
            ship_renderer: ShipRenderer::new(context.clone())?,
            bullet_renderer: BulletRenderer::new(context.clone())?,
            particle_renderer: ParticleRenderer::new(context.clone())?,
            trail_renderer: TrailRenderer::new(context.clone())?,
            text_renderer: TextRenderer::new(context.clone())?,
            flare_renderer: FlareRenderer::new(context.clone())?,
            blur: Blur::new(context)?,
            projection_matrix: Matrix4::identity(),
            base_line_width: 1.0,
            debug: false,
            picked_ship: None,
            blur_enabled: true,
        })
    }

    pub fn set_view(&mut self, zoom: f32, center: Point2<f32>) {
        let screen_width = self.context.drawing_buffer_width() as f32;
        let screen_height = self.context.drawing_buffer_height() as f32;
        let view_width = 1.0 / zoom;
        let view_height = view_width * (screen_height / screen_width);
        let left = center.x - view_width / 2.0;
        let right = center.x + view_width / 2.0;
        let bottom = center.y - view_height / 2.0;
        let top = center.y + view_height / 2.0;
        let znear = -1.0;
        let zfar = 1.0;
        self.projection_matrix = Matrix4::new_orthographic(left, right, bottom, top, znear, zfar);

        let pixel_size = (self.unproject(1, 0) - self.unproject(0, 0)).x as f32;
        let zoom_factor = 2e-3 / zoom;
        self.base_line_width =
            (zoom_factor - 0.01 * zoom_factor * zoom_factor).clamp(pixel_size, 3.0 * pixel_size);
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn set_picked_ship(&mut self, id: Option<u64>) {
        self.picked_ship = id;
    }

    pub fn unproject(&self, x: i32, y: i32) -> Point2<f64> {
        let inverse_matrix = self.projection_matrix.try_inverse().unwrap();
        let dpr = gloo_utils::window().device_pixel_ratio() as f32;
        let device_coords = vector![
            dpr * x as f32 / (self.context.drawing_buffer_width() as f32),
            dpr * -y as f32 / (self.context.drawing_buffer_height() as f32),
            0.0
        ] * 2.0
            - vector![1.0, -1.0, 0.0];
        let coords = inverse_matrix.transform_vector(&device_coords);
        point![coords.x as f64, coords.y as f64]
    }

    pub fn render(&mut self, camera_target: Point2<f32>, zoom: f32, snapshot: &Snapshot) {
        let dpr = gloo_utils::window().device_pixel_ratio();
        let new_width = (self.canvas.client_width() as f64 * dpr) as u32;
        let new_height = (self.canvas.client_height() as f64 * dpr) as u32;
        if new_width != self.canvas.width() || new_height != self.canvas.height() {
            log::info!(
                "Client size: {}x{}",
                self.canvas.client_width(),
                self.canvas.client_height()
            );
            log::info!("Device pixel ratio: {}", dpr);
            log::info!("Resizing canvas to {}x{}", new_width, new_height);
        }
        if new_width != self.canvas.width() {
            self.canvas.set_width(new_width);
        }
        if new_height != self.canvas.height() {
            self.canvas.set_height(new_height);
        }

        let screen_width = self.context.drawing_buffer_width();
        let screen_height = self.context.drawing_buffer_height();
        self.set_view(zoom, point![camera_target.x, camera_target.y]);

        self.grid_renderer
            .update_projection_matrix(&self.projection_matrix);
        self.trail_renderer
            .update_projection_matrix(&self.projection_matrix);

        let ship_drawset =
            self.ship_renderer
                .upload(&self.projection_matrix, snapshot, self.base_line_width);
        let bullet_drawset =
            self.bullet_renderer
                .upload(&self.projection_matrix, snapshot, self.base_line_width);
        let particle_drawset = self
            .particle_renderer
            .upload(&self.projection_matrix, snapshot);
        let flare_drawset = self
            .flare_renderer
            .upload(&self.projection_matrix, snapshot);

        let text_drawset = {
            let mut texts: Vec<Text> = Vec::new();
            if self.debug {
                for (_, drawn_text) in snapshot.drawn_text.iter() {
                    texts.extend(drawn_text.iter().cloned());
                }
            } else if let Some(ship) = self.picked_ship {
                if let Some(drawn_text) = snapshot.drawn_text.get(&ship) {
                    texts.extend(drawn_text.iter().cloned());
                }
            }
            self.text_renderer.upload(&self.projection_matrix, &texts)
        };

        let line_drawset = {
            let mut lines: Vec<Line> = Vec::new();
            if self.debug {
                for (_, debug_lines) in snapshot.debug_lines.iter() {
                    lines.extend(debug_lines.iter().cloned());
                }
            } else if let Some(ship) = self.picked_ship {
                for (ship2, debug_lines) in snapshot.debug_lines.iter() {
                    if ship == *ship2 {
                        lines.extend(debug_lines.iter().cloned());
                    }
                }
            }
            lines.extend(snapshot.scenario_lines.iter().cloned());
            self.line_renderer.upload(&self.projection_matrix, &lines)
        };

        self.context.viewport(0, 0, screen_width, screen_height);

        if self.blur_enabled {
            let blur_bullet_drawset = self.bullet_renderer.upload(
                &self.projection_matrix,
                snapshot,
                self.base_line_width * 2.0,
            );

            self.blur.start();
            // Render to blur source texture
            self.context.clear_color(0.0, 0.0, 0.0, 0.0);
            self.context.clear(gl::COLOR_BUFFER_BIT);
            self.trail_renderer.draw(snapshot.time as f32, 2.0);
            self.flare_renderer.draw(&flare_drawset);
            self.bullet_renderer.draw(&blur_bullet_drawset);
            self.particle_renderer
                .draw(&particle_drawset, 10.0 * self.base_line_width);
            self.ship_renderer.draw(&ship_drawset);
            self.blur.finish();
        }

        {
            // Render non-blurred graphics
            self.context.clear_color(0.0, 0.0, 0.0, 0.0);
            self.context.clear(gl::COLOR_BUFFER_BIT);
            self.grid_renderer
                .draw(zoom, camera_target, snapshot.world_size);
            if self.blur_enabled {
                self.blur.draw();
            }
            self.trail_renderer.draw(snapshot.time as f32, 2.0);
            self.flare_renderer.draw(&flare_drawset);
            self.bullet_renderer.draw(&bullet_drawset);
            self.particle_renderer
                .draw(&particle_drawset, 5.0 * self.base_line_width);
            self.line_renderer.draw(&line_drawset);
            self.ship_renderer.draw(&ship_drawset);
            self.text_renderer.draw(&text_drawset);
        }
    }

    pub fn update(&mut self, snapshot: &Snapshot) {
        self.particle_renderer.update(snapshot);
        self.trail_renderer.update(snapshot);
    }

    pub fn set_blur(&mut self, blur: bool) {
        self.blur_enabled = blur;
    }

    pub fn get_blur(&self) -> bool {
        self.blur_enabled
    }
}
