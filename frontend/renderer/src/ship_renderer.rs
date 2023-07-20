use super::{buffer_arena, geometry, glutil};
use glutil::VertexAttribBuilder;
use nalgebra::{vector, Matrix4, Vector4};
use oort_simulator::model;
use oort_simulator::ship::ShipClass;
use oort_simulator::snapshot::{ShipSnapshot, Snapshot};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};
use WebGl2RenderingContext as gl;

pub struct ShipRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    projection_loc: WebGlUniformLocation,
    projection_matrix: Matrix4<f32>,
    buffer_arena: buffer_arena::BufferArena,
}

impl ShipRenderer {
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
            2 => vector![0.13, 0.50, 0.73, 1.00],
            9 => vector![0.40, 0.40, 0.40, 1.00],
            _ => vector![1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn draw(&mut self, snapshot: &Snapshot, base_line_width: f32) {
        self.context.use_program(Some(&self.program));

        struct ShipBatch {
            num_instances: usize,
            vertices_token: buffer_arena::Token,
            num_vertices: usize,
            attribs_token: buffer_arena::Token,
        }

        struct ShipAttribs {
            color: Vector4<f32>,
            transform: Matrix4<f32>,
        }

        let mut ships_by_class = std::collections::HashMap::<ShipClass, Vec<ShipSnapshot>>::new();

        for ship in snapshot.ships.iter() {
            ships_by_class.entry(ship.class).or_insert_with(Vec::new);
            ships_by_class
                .get_mut(&ship.class)
                .unwrap()
                .push((*ship).clone());
        }

        let mut batches = vec![];
        for (&class, ships) in ships_by_class.iter() {
            let model_vertices = geometry::line_loop_mesh(&model::load(class), base_line_width);
            let vertices_token = self.buffer_arena.write(&model_vertices);
            let num_vertices = model_vertices.len();

            let mut attribs: Vec<ShipAttribs> = vec![];
            attribs.reserve(ships.len());
            for ship in ships.iter() {
                let p = ship.position.coords.cast::<f32>();
                let shielded = ship.active_abilities.contains(&oort_api::Ability::Shield);
                let team_color = Self::team_color(ship.team);
                let color = if shielded {
                    let frac = (snapshot.time as f32 * 30.0).sin() * 0.2 + 0.5;
                    team_color * (1.0 - frac) + Vector4::new(0.0, 0.0, 1.0, 1.0) * frac
                } else {
                    team_color
                };
                attribs.push(ShipAttribs {
                    color,
                    transform: Matrix4::new_translation(&vector![p.x, p.y, 0.0])
                        * Matrix4::from_euler_angles(0.0, 0.0, ship.heading as f32),
                });
            }
            let attribs_token = self.buffer_arena.write(&attribs);

            batches.push(ShipBatch {
                num_instances: ships.len(),
                vertices_token,
                num_vertices,
                attribs_token,
            });
        }

        for batch in batches.iter() {
            // vertex
            VertexAttribBuilder::new(&self.context)
                .data_token(&batch.vertices_token)
                .index(0)
                .size(2)
                .build();

            let vab = VertexAttribBuilder::new(&self.context)
                .data_token(&batch.attribs_token)
                .divisor(1);

            // color
            vab.index(1)
                .size(4)
                .offset(offset_of!(ShipAttribs, color))
                .build();

            // transform
            for i in 0..4 {
                vab.index(2 + i)
                    .size(4)
                    .offset(offset_of!(ShipAttribs, transform) + i as usize * 16)
                    .build();
            }

            // projection
            self.context.uniform_matrix4fv_with_f32_array(
                Some(&self.projection_loc),
                false,
                self.projection_matrix.data.as_slice(),
            );

            self.context.draw_arrays_instanced(
                gl::TRIANGLES,
                0,
                batch.num_vertices as i32,
                batch.num_instances as i32,
            );
        }

        for i in 0..5 {
            self.context.vertex_attrib_divisor(i, 0);
            self.context.disable_vertex_attrib_array(i);
        }
    }
}
