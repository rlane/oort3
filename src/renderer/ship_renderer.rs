use super::{buffer_arena, geometry, glutil, model};
use crate::simulation::ship::ShipClass;
use crate::simulation::snapshot::{ShipSnapshot, Snapshot};
use glutil::VertexAttribBuilder;
use nalgebra::{storage::ContiguousStorage, vector, Matrix4, Point2, Vector4};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};
use WebGl2RenderingContext as gl;

pub struct ShipRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    projection_matrix: Matrix4<f32>,
    buffer_arena: buffer_arena::BufferArena,
}

impl ShipRenderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = glutil::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"#version 300 es
uniform mat4 transform;
layout(location = 0) in vec2 vertex;
layout(location = 1) in vec4 position;
layout(location = 2) in float heading;
layout(location = 3) in vec4 color;
out vec4 varying_color;

// https://gist.github.com/yiwenl/3f804e80d0930e34a0b33359259b556c
vec2 rotate(vec2 v, float a) {
    float s = sin(a);
    float c = cos(a);
    mat2 m = mat2(c, s, -s, c);
    return m * v;
}

void main() {
    gl_Position = transform * (position + vec4(rotate(vertex, heading), 0.0, 0.0));
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
            buffer_arena: buffer_arena::BufferArena::new(
                "ship_renderer",
                context,
                gl::ARRAY_BUFFER,
                1024 * 1024,
            )?,
        })
    }

    pub fn update_projection_matrix(&mut self, m: &Matrix4<f32>) {
        self.projection_matrix = *m;
    }

    pub fn team_color(team: i32) -> Vector4<f32> {
        match team {
            0 => vector![0.99, 0.98, 0.00, 1.00],
            1 => vector![0.99, 0.00, 0.98, 1.00],
            9 => vector![0.40, 0.40, 0.40, 1.00],
            _ => vector![1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn draw(&mut self, snapshot: &Snapshot, line_scale: f32) {
        self.context.use_program(Some(&self.program));

        let mut ships_by_class = std::collections::HashMap::<ShipClass, Vec<ShipSnapshot>>::new();

        for ship in snapshot.ships.iter() {
            ships_by_class.entry(ship.class).or_insert_with(Vec::new);
            ships_by_class
                .get_mut(&ship.class)
                .unwrap()
                .push((*ship).clone());
        }

        struct ShipAttribs {
            position: Point2<f32>,
            heading: f32,
            color: Vector4<f32>,
        }

        for (class, ships) in ships_by_class.iter() {
            // vertex

            let model_vertices = geometry::line_loop_mesh(&model::load(*class), line_scale);
            let num_vertices = model_vertices.len();

            VertexAttribBuilder::new(&self.context)
                .data(&mut self.buffer_arena, &model_vertices)
                .index(0)
                .size(2)
                .build();

            let mut attribs: Vec<ShipAttribs> = vec![];
            attribs.reserve(ships.len());
            for ship in ships.iter() {
                attribs.push(ShipAttribs {
                    position: ship.position.cast(),
                    heading: ship.heading as f32,
                    color: Self::team_color(ship.team),
                });
            }

            let vab = VertexAttribBuilder::new(&self.context)
                .data(&mut self.buffer_arena, &attribs)
                .divisor(1);

            // position
            vab.index(1)
                .size(2)
                .offset(offset_of!(ShipAttribs, position))
                .build();

            // heading
            vab.index(2)
                .size(1)
                .offset(offset_of!(ShipAttribs, heading))
                .build();

            // color
            vab.index(3)
                .size(4)
                .offset(offset_of!(ShipAttribs, color))
                .build();

            // projection

            self.context.uniform_matrix4fv_with_f32_array(
                Some(&self.transform_loc),
                false,
                self.projection_matrix.data.as_slice(),
            );

            let num_instances = ships.len();
            self.context.draw_arrays_instanced(
                gl::TRIANGLES,
                0,
                num_vertices as i32,
                num_instances as i32,
            );

            self.context.vertex_attrib_divisor(1, 0);
            self.context.vertex_attrib_divisor(2, 0);
            self.context.vertex_attrib_divisor(3, 0);

            self.context.disable_vertex_attrib_array(0);
            self.context.disable_vertex_attrib_array(1);
            self.context.disable_vertex_attrib_array(2);
            self.context.disable_vertex_attrib_array(3);
        }
    }
}
