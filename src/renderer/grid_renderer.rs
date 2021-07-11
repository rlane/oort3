use super::{buffer_arena, webgl};
use nalgebra::{storage::Storage, vector, Matrix4};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};
use WebGl2RenderingContext as gl;

pub struct GridRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    color_loc: WebGlUniformLocation,
    projection_matrix: Matrix4<f32>,
    buffer_arena: buffer_arena::BufferArena,
}

impl GridRenderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = webgl::compile_shader(
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
        let frag_shader = webgl::compile_shader(
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
        let program = webgl::link_program(&context, &vert_shader, &frag_shader)?;

        let transform_loc = context
            .get_uniform_location(&program, "transform")
            .ok_or("did not find uniform")?;

        let color_loc = context
            .get_uniform_location(&program, "color")
            .ok_or("did not find uniform")?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context: context.clone(),
            program,
            transform_loc,
            color_loc,
            projection_matrix: Matrix4::identity(),
            buffer_arena: buffer_arena::BufferArena::new(context, gl::ARRAY_BUFFER, 1024 * 1024)?,
        })
    }

    pub fn update_projection_matrix(&mut self, m: &Matrix4<f32>) {
        self.projection_matrix = *m;
    }

    pub fn draw(&mut self) {
        use crate::simulation::WORLD_SIZE;
        let grid_size = 100.0;
        let color = vector![0.0, 1.0, 0.0, 1.0];

        let mut vertices = vec![];
        let n = 1 + (WORLD_SIZE as f32 / grid_size) as i32;
        vertices.reserve((2 * (n + 1) * 3) as usize);
        for i in -(n / 2)..(n / 2 + 1) {
            // Vertical
            vertices.push((i as f32) * grid_size);
            vertices.push((-WORLD_SIZE as f32) / 2.0);
            vertices.push(0.0);
            vertices.push((i as f32) * grid_size);
            vertices.push((WORLD_SIZE as f32) / 2.0);
            vertices.push(0.0);

            // Horizontal
            vertices.push((-WORLD_SIZE as f32) / 2.0);
            vertices.push((i as f32) * grid_size);
            vertices.push(0.0);
            vertices.push((WORLD_SIZE as f32) / 2.0);
            vertices.push((i as f32) * grid_size);
            vertices.push(0.0);
        }

        self.context.use_program(Some(&self.program));

        let (buffer, offset) = self.buffer_arena.write(&vertices);
        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));

        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 0,
            /*size=*/ 3,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ 0,
            offset as i32,
        );
        self.context.enable_vertex_attrib_array(0);

        self.context.uniform4f(
            Some(&self.color_loc),
            color[0],
            color[1],
            color[2],
            color[3],
        );

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            self.projection_matrix.data.as_slice(),
        );

        self.context.line_width(1.0);

        self.context
            .draw_arrays(gl::LINES, 0, (vertices.len() / 3) as i32);

        self.context.disable_vertex_attrib_array(0);
    }
}
