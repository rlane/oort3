pub mod buffer_arena;
pub mod bullet_renderer;
pub mod glutil;
pub mod grid_renderer;
pub mod line_renderer;
pub mod model;
pub mod ship_renderer;

use crate::simulation::scenario::Line;
use crate::simulation::Simulation;
use bullet_renderer::BulletRenderer;
use grid_renderer::GridRenderer;
use line_renderer::LineRenderer;
use nalgebra::{point, Matrix4, Point2};
use ship_renderer::ShipRenderer;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use WebGl2RenderingContext as gl;

pub struct Renderer {
    context: WebGl2RenderingContext,
    grid_renderer: GridRenderer,
    line_renderer: LineRenderer,
    ship_renderer: ShipRenderer,
    bullet_renderer: BulletRenderer,
    projection_matrix: Matrix4<f32>,
}

impl Renderer {
    pub fn new() -> Result<Self, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("glcanvas").unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>()?;

        let context = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;

        Ok(Renderer {
            context: context.clone(),
            grid_renderer: GridRenderer::new(context.clone())?,
            line_renderer: LineRenderer::new(context.clone())?,
            ship_renderer: ShipRenderer::new(context.clone())?,
            bullet_renderer: BulletRenderer::new(context)?,
            projection_matrix: Matrix4::identity(),
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
    }

    pub fn render(
        &mut self,
        camera_target: Point2<f32>,
        zoom: f32,
        sim: &Simulation,
        lines: &[Line],
    ) {
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

        self.grid_renderer.draw(zoom);
        self.line_renderer.draw(lines);
        self.bullet_renderer.draw(&sim);
        self.ship_renderer.draw(&sim);
    }
}
