use super::{buffer_arena, geometry, glutil};
use glutil::VertexAttribBuilder;
use nalgebra::{Matrix4, Point2, Vector2, Vector4};
use oort_simulator::color;
use oort_simulator::simulation::PHYSICS_TICK_LENGTH;
use oort_simulator::snapshot::Snapshot;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject};
use WebGl2RenderingContext as gl;

pub struct BulletRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    projection_loc: WebGlUniformLocation,
    buffer_arena: buffer_arena::BufferArena,
    vao: WebGlVertexArrayObject,
}

pub struct DrawSet {
    projection_matrix: Matrix4<f32>,
    draws: Vec<Draw>,
}

pub struct Draw {
    num_instances: usize,
    vertices_token: buffer_arena::Token,
    num_vertices: usize,
    attribs_token: buffer_arena::Token,
}

struct Attribs {
    color: Vector4<f32>,
    transform: Matrix4<f32>,
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

        let vao = context
            .create_vertex_array()
            .ok_or("failed to create vertex array")?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context: context.clone(),
            program,
            projection_loc,
            buffer_arena: buffer_arena::BufferArena::new(
                "bullet_renderer",
                context,
                gl::ARRAY_BUFFER,
                1024 * 1024,
            )?,
            vao,
        })
    }

    pub fn upload(
        &mut self,
        projection_matrix: &Matrix4<f32>,
        snapshot: &Snapshot,
        base_line_width: f32,
    ) -> DrawSet {
        let vertices = geometry::quad();
        let vertices_token = self.buffer_arena.write(&vertices);

        let mut draws = vec![];
        for bullets in snapshot.bullets.chunks(1000) {
            let mut attribs = vec![];
            attribs.reserve(bullets.len());
            for bullet in bullets.iter() {
                let p: Point2<f32> = bullet.position.cast();
                let v: Vector2<f32> = bullet.velocity.cast();
                let dt = PHYSICS_TICK_LENGTH as f32;
                let mut color = color::from_u32(bullet.color);
                if bullet.ttl < 0.3 {
                    color.w *= bullet.ttl + 0.3;
                }
                attribs.push(Attribs {
                    color,
                    transform: geometry::line_transform(p - 2.0 * v * dt, p, base_line_width),
                });
            }
            draws.push(Draw {
                num_instances: bullets.len(),
                vertices_token: vertices_token.clone(),
                num_vertices: vertices.len(),
                attribs_token: self.buffer_arena.write(&attribs),
            });
        }

        DrawSet {
            projection_matrix: *projection_matrix,
            draws,
        }
    }

    pub fn draw(&mut self, drawset: &DrawSet) {
        self.context.use_program(Some(&self.program));
        self.context.bind_vertex_array(Some(&self.vao));

        // projection
        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.projection_loc),
            false,
            drawset.projection_matrix.data.as_slice(),
        );

        for draw in &drawset.draws {
            // vertex
            VertexAttribBuilder::new(&self.context)
                .data_token(&draw.vertices_token)
                .index(0)
                .size(2)
                .build();

            // attribs
            let vab = VertexAttribBuilder::new(&self.context)
                .data_token(&draw.attribs_token)
                .size(4)
                .divisor(1);
            vab.index(1).offset(offset_of!(Attribs, color)).build();
            vab.index(2).offset(offset_of!(Attribs, transform)).build();
            vab.index(3)
                .offset(offset_of!(Attribs, transform) + 16)
                .build();
            vab.index(4)
                .offset(offset_of!(Attribs, transform) + 32)
                .build();
            vab.index(5)
                .offset(offset_of!(Attribs, transform) + 48)
                .build();

            self.context.draw_arrays_instanced(
                gl::TRIANGLE_STRIP,
                0,
                draw.num_vertices as i32,
                draw.num_instances as i32,
            );
        }

        self.context.bind_vertex_array(None);
    }
}
