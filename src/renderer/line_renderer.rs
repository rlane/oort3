use super::{buffer_arena, glutil};
use crate::simulation::scenario::Line;
use nalgebra::{storage::Storage, Matrix4};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};
use WebGl2RenderingContext as gl;

pub struct LineRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    projection_matrix: Matrix4<f32>,
    buffer_arena: buffer_arena::BufferArena,
}

impl LineRenderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = glutil::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"#version 300 es
uniform mat4 transform;
layout(location = 0) in vec4 vertex;
layout(location = 1) in vec4 color;
out vec4 varying_color;
void main() {
    gl_Position = transform * vertex;
    varying_color = color;
}
    "#,
        )?;
        let frag_shader = glutil::compile_shader(
            &context,
            gl::FRAGMENT_SHADER,
            r#"#version 300 es
precision mediump float;
in vec4 varying_color;
out vec4 fragmentColor;
void main() {
    fragmentColor = varying_color;
}
    "#,
        )?;
        let program = glutil::link_program(&context, &vert_shader, &frag_shader)?;

        let transform_loc = context
            .get_uniform_location(&program, "transform")
            .ok_or("did not find uniform")?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context: context.clone(),
            program,
            transform_loc,
            projection_matrix: Matrix4::identity(),
            buffer_arena: buffer_arena::BufferArena::new(context, gl::ARRAY_BUFFER, 1024 * 1024)?,
        })
    }

    pub fn update_projection_matrix(&mut self, m: &Matrix4<f32>) {
        self.projection_matrix = *m;
    }

    pub fn draw(&mut self, lines: &[Line]) {
        if lines.is_empty() {
            return;
        }

        let mut vertices: Vec<f32> = vec![];
        vertices.reserve(4 * lines.len());
        let mut colors: Vec<f32> = vec![];
        colors.reserve(8 * lines.len());
        for line in lines {
            vertices.push(line.a.x as f32);
            vertices.push(line.a.y as f32);
            vertices.push(line.b.x as f32);
            vertices.push(line.b.y as f32);

            colors.push(line.color[0]);
            colors.push(line.color[1]);
            colors.push(line.color[2]);
            colors.push(line.color[3]);

            colors.push(line.color[0]);
            colors.push(line.color[1]);
            colors.push(line.color[2]);
            colors.push(line.color[3]);
        }

        self.context.use_program(Some(&self.program));

        let (buffer, vertices_offset) = self.buffer_arena.write(&vertices);
        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));

        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 0,
            /*size=*/ 2,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ 0,
            vertices_offset as i32,
        );
        self.context.enable_vertex_attrib_array(0);

        let (buffer, colors_offset) = self.buffer_arena.write(&colors);
        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));

        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 1,
            /*size=*/ 4,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ 0,
            colors_offset as i32,
        );
        self.context.enable_vertex_attrib_array(1);

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            self.projection_matrix.data.as_slice(),
        );

        self.context.line_width(1.0);

        self.context
            .draw_arrays(gl::LINES, 0, (vertices.len() / 2) as i32);

        self.context.disable_vertex_attrib_array(0);
        self.context.disable_vertex_attrib_array(1);
    }
}
