pub mod buffer_arena;
pub mod bullet_renderer;
pub mod geometry;
pub mod glutil;
pub mod grid_renderer;
pub mod line_renderer;
pub mod particle_renderer;
pub mod ship_renderer;
pub mod trail_renderer;

#[macro_use]
extern crate memoffset;

use bullet_renderer::BulletRenderer;
use grid_renderer::GridRenderer;
use line_renderer::LineRenderer;
use nalgebra::{point, vector, Matrix4, Point2};
use oort_simulator::simulation::Line;
use oort_simulator::snapshot::Snapshot;
use particle_renderer::ParticleRenderer;
use ship_renderer::ShipRenderer;
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
    projection_matrix: Matrix4<f32>,
    base_line_width: f32,
    debug: bool,
    picked_ship: Option<u64>,
}

impl Renderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let context = canvas
            .get_context("webgl2")?
            .expect("Failed to get webgl2 context")
            .dyn_into::<WebGl2RenderingContext>()?;

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
            trail_renderer: TrailRenderer::new(context)?,
            projection_matrix: Matrix4::identity(),
            base_line_width: 1.0,
            debug: false,
            picked_ship: None,
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
        let device_coords = vector![
            x as f32 / self.context.drawing_buffer_width() as f32,
            -y as f32 / self.context.drawing_buffer_height() as f32,
            0.0
        ] * 2.0
            - vector![1.0, -1.0, 0.0];
        let coords = inverse_matrix.transform_vector(&device_coords);
        point![coords.x as f64, coords.y as f64]
    }

    pub fn render(&mut self, camera_target: Point2<f32>, zoom: f32, snapshot: &Snapshot) {
        if self.canvas.client_width() != self.canvas.width() as i32 {
            self.canvas.set_width(self.canvas.client_width() as u32);
        }
        if self.canvas.client_height() != self.canvas.height() as i32 {
            self.canvas.set_height(self.canvas.client_height() as u32);
        }
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(gl::COLOR_BUFFER_BIT);

        let screen_width = self.context.drawing_buffer_width();
        let screen_height = self.context.drawing_buffer_height();
        self.context.viewport(0, 0, screen_width, screen_height);
        self.set_view(zoom, point![camera_target.x, camera_target.y]);

        self.grid_renderer
            .update_projection_matrix(&self.projection_matrix);
        self.line_renderer
            .update_projection_matrix(&self.projection_matrix);
        self.ship_renderer
            .update_projection_matrix(&self.projection_matrix);
        self.bullet_renderer
            .update_projection_matrix(&self.projection_matrix);
        self.particle_renderer
            .update_projection_matrix(&self.projection_matrix);
        self.trail_renderer
            .update_projection_matrix(&self.projection_matrix);

        self.grid_renderer.draw(zoom);
        self.trail_renderer.draw(snapshot.time as f32);
        let mut lines: Vec<Line> = Vec::new();
        if self.debug {
            for (_, debug_lines) in snapshot.debug_lines.iter() {
                lines.extend(debug_lines.iter().cloned());
            }
        } else if let Some(ship) = self.picked_ship {
            if let Some(debug_lines) = snapshot.debug_lines.get(&ship) {
                lines.extend(debug_lines.iter().cloned());
            }
        }
        lines.extend(snapshot.scenario_lines.iter().cloned());
        self.line_renderer.draw(&lines);
        self.bullet_renderer.draw(snapshot, self.base_line_width);
        self.ship_renderer.draw(snapshot, self.base_line_width);
        self.particle_renderer.draw(snapshot);
    }

    pub fn update(&mut self, snapshot: &Snapshot) {
        self.particle_renderer.update(snapshot);
        self.trail_renderer.update(snapshot);
    }
}
