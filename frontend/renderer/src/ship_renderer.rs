use super::{buffer_arena, geometry, glutil};
use glutil::VertexAttribBuilder;
use nalgebra::{vector, Matrix4, Vector4};
use oort_simulator::model;
use oort_simulator::ship::ShipClass;
use oort_simulator::snapshot::{ShipSnapshot, Snapshot};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject};
use WebGl2RenderingContext as gl;

pub struct ShipRenderer {
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

        let vao = context
            .create_vertex_array()
            .ok_or("failed to create vertex array")?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context: context.clone(),
            program,
            projection_loc,
            buffer_arena: buffer_arena::BufferArena::new(
                "ship_renderer",
                context,
                gl::ARRAY_BUFFER,
                1024 * 1024,
            )?,
            vao,
        })
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

    pub fn upload(
        &mut self,
        projection_matrix: &Matrix4<f32>,
        snapshot: &Snapshot,
        base_line_width: f32,
        zoom: f32,
        nlips_enabled: bool,
    ) -> DrawSet {
        let mut ships_by_class = std::collections::HashMap::<ShipClass, Vec<ShipSnapshot>>::new();

        for ship in snapshot.ships.iter() {
            ships_by_class.entry(ship.class).or_insert_with(Vec::new);
            ships_by_class
                .get_mut(&ship.class)
                .unwrap()
                .push((*ship).clone());
        }

        let mut draws = vec![];

        let zoom_factor = 2e-3 / zoom;

        for (&class, ships) in ships_by_class.iter() {
            let model = model::load(class);
            let radius: f32 = model
                .iter()
                .max_by_key(|v| v.norm_squared() as i32)
                .unwrap()
                .norm();
            let min_nlips_scale = 4.0f32.max(radius / 20.0);
            let nlips_scale = (2.0 * zoom_factor / radius.log2()).min(50.0);
            for nlips_draw in [false, true] {
                if nlips_draw
                    && (!nlips_enabled
                        || nlips_scale < min_nlips_scale
                        || matches!(class, ShipClass::Asteroid { .. }))
                {
                    continue;
                }
                let scale = if nlips_draw { nlips_scale } else { 1.0 };
                let vertices =
                    geometry::line_loop_mesh(&model::scale(scale, &model), base_line_width);
                let vertices_token = self.buffer_arena.write(&vertices);
                let num_vertices = vertices.len();

                let mut attribs: Vec<Attribs> = vec![];
                attribs.reserve(ships.len());
                for ship in ships.iter() {
                    let p = ship.position.coords.cast::<f32>();
                    let shielded = ship.active_abilities.contains(&oort_api::Ability::Shield);
                    let mut team_color = Self::team_color(ship.team);
                    if nlips_draw {
                        team_color.w *= (nlips_scale / min_nlips_scale - 1.0)
                            .clamp(0.0, 1.0)
                            .powi(4)
                            .clamp(0.0, 0.5);
                    }
                    let color = if shielded {
                        let frac = (snapshot.time as f32 * 30.0).sin() * 0.2 + 0.5;
                        team_color * (1.0 - frac) + Vector4::new(0.0, 0.0, 1.0, 1.0) * frac
                    } else {
                        team_color
                    };
                    attribs.push(Attribs {
                        color,
                        transform: Matrix4::new_translation(&vector![p.x, p.y, 0.0])
                            * Matrix4::from_euler_angles(0.0, 0.0, ship.heading as f32),
                    });
                }
                let attribs_token = self.buffer_arena.write(&attribs);

                draws.push(Draw {
                    num_instances: ships.len(),
                    vertices_token,
                    num_vertices,
                    attribs_token,
                });
            }
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

        for draw in drawset.draws.iter() {
            // vertex
            VertexAttribBuilder::new(&self.context)
                .data_token(&draw.vertices_token)
                .index(0)
                .size(2)
                .build();

            let vab = VertexAttribBuilder::new(&self.context)
                .data_token(&draw.attribs_token)
                .divisor(1);

            // color
            vab.index(1)
                .size(4)
                .offset(offset_of!(Attribs, color))
                .build();

            // transform
            for i in 0..4 {
                vab.index(2 + i)
                    .size(4)
                    .offset(offset_of!(Attribs, transform) + i as usize * 16)
                    .build();
            }

            self.context.draw_arrays_instanced(
                gl::TRIANGLES,
                0,
                draw.num_vertices as i32,
                draw.num_instances as i32,
            );
        }

        self.context.bind_vertex_array(None);
    }
}
