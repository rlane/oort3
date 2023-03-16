use super::buffer_arena::BufferArena;
use super::{buffer_arena, geometry, glutil};
use glutil::VertexAttribBuilder;
use nalgebra::{point, vector, Matrix4, Point2, Vector2};
use oort_simulator::simulation;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};
use WebGl2RenderingContext as gl;

pub struct GridRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    pitch_loc: WebGlUniformLocation,
    offset_loc: WebGlUniformLocation,
    half_world_size_loc: WebGlUniformLocation,
    pixel_size_loc: WebGlUniformLocation,
    color_loc: WebGlUniformLocation,
    buffer_arena: BufferArena,
    projection_matrix: Matrix4<f32>,
    pixel_size: f32,
    bottom_left: Vector2<f32>,
    top_right: Vector2<f32>,
}

impl GridRenderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = glutil::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"#version 300 es
uniform mat4 transform;
layout(location = 0) in vec4 vertex;
layout(location = 1) in vec2 world_coords;
out vec2 varying_world_coords;

void main() {
    gl_Position = transform * vertex;
    varying_world_coords = world_coords;
}
    "#,
        )?;
        let frag_shader = glutil::compile_shader(
            &context,
            gl::FRAGMENT_SHADER,
            r#"#version 300 es
precision mediump float;
uniform vec2 pitch;
uniform vec2 offset;
uniform float half_world_size;
uniform float pixel_size;
uniform vec4 color;
in vec2 varying_world_coords;
out vec4 fragmentColor;

void main() {
    vec2 coord = gl_FragCoord.xy + floor(offset);
    if ((mod(coord.x, pitch[0]) < 1. ||
        mod(coord.y, pitch[1]) < 1.) &&
        varying_world_coords.x <= half_world_size + 2.0 * pixel_size &&
        varying_world_coords.x > -half_world_size &&
        varying_world_coords.y <= half_world_size + 2.0 * pixel_size &&
        varying_world_coords.y > -half_world_size) {
        fragmentColor = color;
    } else {
        discard;
    }
}
    "#,
        )?;
        let program = glutil::link_program(&context, &vert_shader, &frag_shader)?;

        let transform_loc = context
            .get_uniform_location(&program, "transform")
            .ok_or("did not find uniform")?;

        let pitch_loc = context
            .get_uniform_location(&program, "pitch")
            .ok_or("did not find uniform")?;

        let offset_loc = context
            .get_uniform_location(&program, "offset")
            .ok_or("did not find uniform")?;

        let half_world_size_loc = context
            .get_uniform_location(&program, "half_world_size")
            .ok_or("did not find uniform")?;

        let pixel_size_loc = context
            .get_uniform_location(&program, "pixel_size")
            .ok_or("did not find uniform")?;

        let color_loc = context
            .get_uniform_location(&program, "color")
            .ok_or("did not find uniform")?;

        let buffer_arena = buffer_arena::BufferArena::new(
            "grid_renderer",
            context.clone(),
            gl::ARRAY_BUFFER,
            1024 * 1024,
        )?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context,
            program,
            transform_loc,
            pitch_loc,
            offset_loc,
            half_world_size_loc,
            pixel_size_loc,
            color_loc,
            buffer_arena,
            projection_matrix: Matrix4::identity(),
            pixel_size: 1.0,
            bottom_left: vector![0.0, 0.0],
            top_right: vector![0.0, 0.0],
        })
    }

    pub fn update_projection_matrix(&mut self, projection_matrix: &Matrix4<f32>) {
        self.projection_matrix = *projection_matrix;
        let screen_height = self.context.drawing_buffer_height() as f32;
        let screen_width = self.context.drawing_buffer_width() as f32;
        self.pixel_size = (self.unproject(1, 0) - self.unproject(0, 0)).x as f32;
        self.bottom_left = self.unproject(0, screen_height as i32).coords.cast::<f32>();
        self.top_right = self.unproject(screen_width as i32, 0).coords.cast::<f32>();
    }

    pub fn draw(&mut self, zoom: f32, _camera_target: Point2<f32>) {
        self.context.use_program(Some(&self.program));

        let vertices = geometry::quad();
        VertexAttribBuilder::new(&self.context)
            .data(&mut self.buffer_arena, &vertices)
            .index(0)
            .size(2)
            .build();

        let world_coords = {
            let bl = self.bottom_left;
            let tr = self.top_right;
            vec![bl, vector![bl.x, tr.y], vector![tr.x, bl.y], tr]
        };
        VertexAttribBuilder::new(&self.context)
            .data(&mut self.buffer_arena, &world_coords)
            .index(1)
            .size(2)
            .build();

        let transform = nalgebra::Matrix4::new_scaling(2.0);
        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            transform.data.as_slice(),
        );

        self.context.uniform1f(
            Some(&self.half_world_size_loc),
            0.5 * simulation::WORLD_SIZE as f32,
        );

        self.context
            .uniform1f(Some(&self.pixel_size_loc), self.pixel_size);

        for scale in [1e2, 1e3] {
            let green = 0.2 * (zoom * scale * 100.0).log(10.0).clamp(0.0, 1.0);
            let color = vector![0.0, green, 0.0, 1.0];
            let pitch = vector![scale, scale] / self.pixel_size;
            let offset = self.bottom_left / self.pixel_size;

            self.context
                .uniform2fv_with_f32_array(Some(&self.pitch_loc), pitch.data.as_slice());
            self.context
                .uniform2fv_with_f32_array(Some(&self.offset_loc), offset.data.as_slice());
            self.context
                .uniform4fv_with_f32_array(Some(&self.color_loc), color.data.as_slice());

            self.context
                .draw_arrays(gl::TRIANGLE_STRIP, 0, vertices.len() as i32);
        }

        self.context.disable_vertex_attrib_array(0);
        self.context.disable_vertex_attrib_array(1);
    }

    pub fn unproject(&self, x: i32, y: i32) -> Point2<f64> {
        let inverse_matrix = self.projection_matrix.try_inverse().unwrap();
        let device_coords = point![
            x as f32 / self.context.drawing_buffer_width() as f32,
            -y as f32 / self.context.drawing_buffer_height() as f32,
            0.0
        ] * 2.0
            - vector![1.0, -1.0, 0.0];
        inverse_matrix.transform_point(&device_coords).cast().xy()
    }
}
