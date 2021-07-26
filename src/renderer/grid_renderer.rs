use super::{buffer_arena, glutil};
use nalgebra::{storage::Storage, vector, Matrix4};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlUniformLocation};
use WebGl2RenderingContext as gl;

pub struct GridRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    color_loc: WebGlUniformLocation,
    projection_matrix: Matrix4<f32>,
    buffer: WebGlBuffer,
    coarse_offset: i32,
    coarse_num_vertices: i32,
    fine_offset: i32,
    fine_num_vertices: i32,
}

impl GridRenderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = glutil::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"#version 300 es
uniform mat4 transform;
layout(location = 0) in vec4 vertex;

void main() {
    gl_Position = transform * vertex;
}
    "#,
        )?;
        let frag_shader = glutil::compile_shader(
            &context,
            gl::FRAGMENT_SHADER,
            r#"#version 300 es
precision mediump float;
uniform vec4 color;
out vec4 fragmentColor;
void main() {
    fragmentColor = color;
}
    "#,
        )?;
        let program = glutil::link_program(&context, &vert_shader, &frag_shader)?;

        let transform_loc = context
            .get_uniform_location(&program, "transform")
            .ok_or("did not find uniform")?;

        let color_loc = context
            .get_uniform_location(&program, "color")
            .ok_or("did not find uniform")?;

        let mut buffer_arena = buffer_arena::BufferArena::new(
            "grid_renderer",
            context.clone(),
            gl::ARRAY_BUFFER,
            1024 * 1024,
        )?;

        let coarse_vertex_data = Self::generate_vertices(1e3);
        let coarse_num_vertices = (coarse_vertex_data.len() / 2) as i32;
        let (_, coarse_offset) = buffer_arena.write(&coarse_vertex_data);

        let fine_vertex_data = Self::generate_vertices(1e2);
        let fine_num_vertices = (fine_vertex_data.len() / 2) as i32;
        let (buffer, fine_offset) = buffer_arena.write(&fine_vertex_data);

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context,
            program,
            transform_loc,
            color_loc,
            projection_matrix: Matrix4::identity(),
            buffer,
            coarse_offset: coarse_offset as i32,
            coarse_num_vertices,
            fine_offset: fine_offset as i32,
            fine_num_vertices,
        })
    }

    fn generate_vertices(grid_size: f32) -> Vec<f32> {
        use crate::simulation::WORLD_SIZE;

        let mut vertices = vec![];
        let n = 1 + (WORLD_SIZE as f32 / grid_size) as i32;
        vertices.reserve((2 * (n + 1) * 3) as usize);
        for i in -(n / 2)..(n / 2 + 1) {
            // Vertical
            vertices.push((i as f32) * grid_size);
            vertices.push((-WORLD_SIZE as f32) / 2.0);
            vertices.push((i as f32) * grid_size);
            vertices.push((WORLD_SIZE as f32) / 2.0);

            // Horizontal
            vertices.push((-WORLD_SIZE as f32) / 2.0);
            vertices.push((i as f32) * grid_size);
            vertices.push((WORLD_SIZE as f32) / 2.0);
            vertices.push((i as f32) * grid_size);
        }

        vertices
    }

    pub fn update_projection_matrix(&mut self, m: &Matrix4<f32>) {
        self.projection_matrix = *m;
    }

    pub fn draw(&mut self, zoom: f32) {
        let f = |z: f32| 0.2 * (zoom * z * 10.0).log(10.0).clamp(0.0, 1.0);
        let fine_color = vector![0.0, f(1e3), 0.0, 1.0];
        let coarse_color = vector![0.0, f(1e4), 0.0, 1.0];

        self.context.use_program(Some(&self.program));

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            self.projection_matrix.data.as_slice(),
        );

        self.context.line_width(1.0);

        self.context
            .bind_buffer(gl::ARRAY_BUFFER, Some(&self.buffer));

        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 0,
            /*size=*/ 2,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ 0,
            self.fine_offset,
        );
        self.context.enable_vertex_attrib_array(0);

        self.context
            .uniform4fv_with_f32_array(Some(&self.color_loc), &fine_color.data.as_slice());

        self.context
            .draw_arrays(gl::LINES, 0, self.fine_num_vertices);

        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 0,
            /*size=*/ 2,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ 0,
            self.coarse_offset,
        );
        self.context.enable_vertex_attrib_array(0);

        self.context
            .uniform4fv_with_f32_array(Some(&self.color_loc), &coarse_color.data.as_slice());

        self.context
            .draw_arrays(gl::LINES, 0, self.coarse_num_vertices);

        self.context.disable_vertex_attrib_array(0);
    }
}
