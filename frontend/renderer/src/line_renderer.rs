use super::{buffer_arena, glutil};
use glutil::VertexAttribBuilder;
use nalgebra::{vector, Matrix4, Vector4};
use oort_simulator::simulation::Line;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject};
use WebGl2RenderingContext as gl;

pub struct LineRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    buffer_arena: buffer_arena::BufferArena,
    vao: WebGlVertexArrayObject,
}

pub struct DrawSet {
    projection_matrix: Matrix4<f32>,
    draws: Vec<Draw>,
}

pub struct Draw {
    num_vertices: usize,
    attribs_token: buffer_arena::Token,
}

struct Attribs {
    vertex: Vector4<f32>,
    color: Vector4<f32>,
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

        let vao = context
            .create_vertex_array()
            .ok_or("failed to create vertex array")?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context: context.clone(),
            program,
            transform_loc,
            buffer_arena: buffer_arena::BufferArena::new(
                "line_renderer",
                context,
                gl::ARRAY_BUFFER,
                1024 * 1024,
            )?,
            vao,
        })
    }

    pub fn upload(&mut self, projection_matrix: &Matrix4<f32>, lines: &[Line]) -> DrawSet {
        let mut draws = vec![];
        for lines in lines.chunks(1000) {
            let mut attribs = vec![];
            attribs.reserve(2 * lines.len());
            for line in lines {
                for position in [line.a, line.b] {
                    let p = position.coords.cast();
                    let mut color = line.color;
                    color.w *= 0.5; // Will be drawn twice
                    attribs.push(Attribs {
                        vertex: vector![p.x, p.y, 0.0, 1.0],
                        color,
                    });
                }
            }

            let attribs_token = self.buffer_arena.write(&attribs);
            draws.push(Draw {
                num_vertices: attribs.len(),
                attribs_token,
            });
        }
        DrawSet {
            projection_matrix: *projection_matrix,
            draws,
        }
    }

    pub fn draw(&mut self, drawset: &DrawSet) {
        if drawset.draws.is_empty() {
            return;
        }

        self.context.use_program(Some(&self.program));
        self.context.bind_vertex_array(Some(&self.vao));

        let mut line_width = 1.0;

        for _ in 0..2 {
            self.context.line_width(line_width);

            self.context.uniform_matrix4fv_with_f32_array(
                Some(&self.transform_loc),
                false,
                drawset.projection_matrix.data.as_slice(),
            );

            for draw in &drawset.draws {
                let vab = VertexAttribBuilder::new(&self.context).data_token(&draw.attribs_token);
                vab.index(0)
                    .size(4)
                    .offset(offset_of!(Attribs, vertex))
                    .build();
                vab.index(1)
                    .size(4)
                    .offset(offset_of!(Attribs, color))
                    .build();

                self.context
                    .draw_arrays(gl::LINES, 0, draw.num_vertices as i32);

                line_width *= 2.0;
            }
        }

        self.context.bind_vertex_array(None);
    }
}
