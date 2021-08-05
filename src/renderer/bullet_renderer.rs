use super::{buffer_arena, glutil};
use crate::simulation::snapshot::Snapshot;
use crate::simulation::PHYSICS_TICK_LENGTH;
use glutil::VertexAttribBuilder;
use nalgebra::{storage::ContiguousStorage, vector, Matrix4, Point2};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};
use WebGl2RenderingContext as gl;

pub struct BulletRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    color_loc: WebGlUniformLocation,
    projection_matrix: Matrix4<f32>,
    buffer_arena: buffer_arena::BufferArena,
}

impl BulletRenderer {
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

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context: context.clone(),
            program,
            transform_loc,
            color_loc,
            projection_matrix: Matrix4::identity(),
            buffer_arena: buffer_arena::BufferArena::new(
                "bullet_renderer",
                context,
                gl::ARRAY_BUFFER,
                1024 * 1024,
            )?,
        })
    }

    pub fn update_projection_matrix(&mut self, m: &Matrix4<f32>) {
        self.projection_matrix = *m;
    }

    pub fn draw(&mut self, snapshot: &Snapshot) {
        let thickness = 1.0;
        let color = vector![1.00, 0.63, 0.00, 1.00];
        let num_instances = snapshot.bullets.len();
        let num_vertices = 2;

        if num_instances == 0 {
            return;
        }

        self.context.use_program(Some(&self.program));
        self.context.line_width(thickness);

        self.context.uniform4f(
            Some(&self.color_loc),
            color[0],
            color[1],
            color[2],
            color[3],
        );

        // vertex

        let mut vertex_data: Vec<Point2<f32>> = vec![];
        vertex_data.reserve(num_instances * num_vertices);
        for bullet in snapshot.bullets.iter() {
            let p = bullet.position.cast();
            let v = bullet.velocity.cast();
            let dt = PHYSICS_TICK_LENGTH as f32;
            let p1 = p - v * dt;
            let p2 = p + v * dt;
            vertex_data.push(p1);
            vertex_data.push(p2);
        }

        VertexAttribBuilder::new(&self.context)
            .data(&mut self.buffer_arena, &vertex_data)
            .index(0)
            .size(2)
            .build();

        // projection

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            self.projection_matrix.data.as_slice(),
        );

        self.context
            .draw_arrays(gl::LINES, 0, (num_vertices * num_instances) as i32);

        self.context.disable_vertex_attrib_array(0);
    }
}
