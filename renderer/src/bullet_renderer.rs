use super::{buffer_arena, geometry, glutil};
use glutil::VertexAttribBuilder;
use nalgebra::{Matrix4, Point2, Vector2, Vector4};
use oort_simulator::simulation::PHYSICS_TICK_LENGTH;
use oort_simulator::snapshot::Snapshot;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};
use WebGl2RenderingContext as gl;

pub struct BulletRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    projection_loc: WebGlUniformLocation,
    projection_matrix: Matrix4<f32>,
    buffer_arena: buffer_arena::BufferArena,
}

impl BulletRenderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = glutil::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"#version 300 es
uniform mat4 projection;
layout(location = 0) in vec4 vertex;
layout(location = 1) in vec4 color;
layout(location = 2) in mat4 transform;
out vec4 varying_color;

void main() {
    gl_Position = projection * (transform * vertex);
    varying_color = color * clamp(float(gl_VertexID & 2), 0.1, 1.0);
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

        let projection_loc = context
            .get_uniform_location(&program, "projection")
            .ok_or("did not find uniform")?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context: context.clone(),
            program,
            projection_loc,
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

    pub fn draw(&mut self, snapshot: &Snapshot, base_line_width: f32) {
        let num_instances = snapshot.bullets.len();

        if num_instances == 0 {
            return;
        }

        self.context.use_program(Some(&self.program));

        // vertex
        let vertices = geometry::quad();
        VertexAttribBuilder::new(&self.context)
            .data(&mut self.buffer_arena, &vertices)
            .index(0)
            .size(2)
            .build();

        struct BulletAttribs {
            color: Vector4<f32>,
            transform: Matrix4<f32>,
        }

        let mut attribs = vec![];
        attribs.reserve(num_instances);
        for bullet in snapshot.bullets.iter() {
            let p: Point2<f32> = bullet.position.cast();
            let v: Vector2<f32> = bullet.velocity.cast();
            let dt = PHYSICS_TICK_LENGTH as f32;
            let mut color = bullet.color;
            if bullet.ttl < 1.0 {
                color.w *= bullet.ttl;
            }
            attribs.push(BulletAttribs {
                color: color,
                transform: geometry::line_transform(p - v * dt, p + v * dt, base_line_width),
            });
        }

        let vab = VertexAttribBuilder::new(&self.context)
            .data(&mut self.buffer_arena, &attribs)
            .size(4)
            .divisor(1);
        vab.index(1)
            .offset(offset_of!(BulletAttribs, color))
            .build();
        vab.index(2)
            .offset(offset_of!(BulletAttribs, transform))
            .build();
        vab.index(3)
            .offset(offset_of!(BulletAttribs, transform) + 16)
            .build();
        vab.index(4)
            .offset(offset_of!(BulletAttribs, transform) + 32)
            .build();
        vab.index(5)
            .offset(offset_of!(BulletAttribs, transform) + 48)
            .build();

        // projection
        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.projection_loc),
            false,
            self.projection_matrix.data.as_slice(),
        );

        self.context.draw_arrays_instanced(
            gl::TRIANGLE_STRIP,
            0,
            vertices.len() as i32,
            num_instances as i32,
        );

        self.context.vertex_attrib_divisor(1, 0);
        self.context.vertex_attrib_divisor(2, 0);
        self.context.vertex_attrib_divisor(3, 0);
        self.context.vertex_attrib_divisor(4, 0);
        self.context.vertex_attrib_divisor(5, 0);

        self.context.disable_vertex_attrib_array(0);
        self.context.disable_vertex_attrib_array(1);
        self.context.disable_vertex_attrib_array(2);
        self.context.disable_vertex_attrib_array(3);
        self.context.disable_vertex_attrib_array(4);
        self.context.disable_vertex_attrib_array(5);
    }
}
