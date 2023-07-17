use std::f32::consts::TAU;

use super::{buffer_arena, geometry, glutil};
use glutil::VertexAttribBuilder;
use nalgebra::{vector, Matrix4, Unit, UnitComplex, Vector2, Vector4};
use oort_simulator::ship::ShipClass;
use oort_simulator::snapshot::{ShipSnapshot, Snapshot};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};
use WebGl2RenderingContext as gl;

struct FlarePosition {
    offset: Vector2<f32>,
    angle: f32,
    scale: f32,
}

fn flare_positions(class: ShipClass) -> Vec<FlarePosition> {
    match class {
        ShipClass::Fighter => {
            let x = 4.0;
            vec![
                FlarePosition {
                    offset: vector![-7.0, 0.0],
                    angle: TAU / 2.0,
                    scale: 1.0,
                },
                FlarePosition {
                    offset: vector![0.0, x],
                    angle: TAU / 4.0,
                    scale: 1.0,
                },
                FlarePosition {
                    offset: vector![0.0, -x],
                    angle: -TAU / 4.0,
                    scale: 1.0,
                },
                FlarePosition {
                    offset: vector![0.0, -x],
                    angle: 0.0,
                    scale: 0.5,
                },
                FlarePosition {
                    offset: vector![0.0, x],
                    angle: 0.0,
                    scale: 0.5,
                },
            ]
        }
        _ => vec![],
    }
}

pub struct FlareRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    projection_loc: WebGlUniformLocation,
    current_time_loc: WebGlUniformLocation,
    projection_matrix: Matrix4<f32>,
    buffer_arena: buffer_arena::BufferArena,
}

impl FlareRenderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = glutil::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"#version 300 es
uniform mat4 projection;
uniform float current_time;
layout(location = 0) in vec4 vertex;
layout(location = 1) in vec4 color;
layout(location = 2) in mat4 transform;
out vec4 varying_color;

void main() {
    gl_Position = projection * (transform * vertex);
    varying_color = color * ((sin(current_time * 10.0) * 0.5 + 1.0) * 0.1 + 0.9);
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

        let current_time_loc = context
            .get_uniform_location(&program, "current_time")
            .ok_or("did not find uniform")?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context: context.clone(),
            program,
            projection_loc,
            current_time_loc,
            projection_matrix: Matrix4::identity(),
            buffer_arena: buffer_arena::BufferArena::new(
                "flare_renderer",
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

    pub fn draw(&mut self, snapshot: &Snapshot) {
        self.context.use_program(Some(&self.program));

        let mut ships_by_class = std::collections::HashMap::<ShipClass, Vec<ShipSnapshot>>::new();

        for ship in snapshot.ships.iter() {
            ships_by_class.entry(ship.class).or_insert_with(Vec::new);
            ships_by_class
                .get_mut(&ship.class)
                .unwrap()
                .push((*ship).clone());
        }

        struct FlareAttribs {
            color: Vector4<f32>,
            transform: Matrix4<f32>,
        }

        for (class, ships) in ships_by_class.iter() {
            let flare_positions = flare_positions(*class);
            if flare_positions.is_empty() {
                continue;
            }

            // vertex
            let vertices = geometry::quad();
            VertexAttribBuilder::new(&self.context)
                .data(&mut self.buffer_arena, &vertices)
                .index(0)
                .size(2)
                .build();

            let mut attribs: Vec<FlareAttribs> = vec![];
            attribs.reserve(ships.len());
            for ship in ships.iter() {
                let p = ship.position.coords.cast::<f32>();
                let ship_transform = Matrix4::new_translation(&vector![p.x, p.y, 0.0])
                    * Matrix4::from_euler_angles(0.0, 0.0, ship.heading as f32);
                for flare_position in &flare_positions {
                    let direction =
                        UnitComplex::from_angle(ship.heading as f32 + flare_position.angle)
                            .transform_vector(&vector![1.0, 0.0]);
                    let strength = (-ship.acceleration.cast::<f32>().dot(&direction)).max(0.0);

                    let strength_scale_transform =
                        Matrix4::new_nonuniform_scaling(&vector![strength, 1.0, 1.0]);
                    let flare_offset_transform = Matrix4::new_translation(&vector![
                        flare_position.offset.x,
                        flare_position.offset.y,
                        0.0
                    ]);
                    let flare_scale_transform =
                        Matrix4::new_nonuniform_scaling(&vector![-1.0, 0.3, 1.0])
                            * flare_position.scale;

                    let flare_model_transform = Matrix4::new_translation(&vector![-0.5, 0.0, 0.0]);

                    let flare_rotation_transform = Matrix4::from_axis_angle(
                        &Unit::new_normalize(vector![0.0, 0.0, 1.0]),
                        flare_position.angle,
                    );

                    let transform = ship_transform
                        * flare_offset_transform
                        * flare_rotation_transform
                        * strength_scale_transform
                        * flare_scale_transform
                        * flare_model_transform;
                    attribs.push(FlareAttribs {
                        color: Self::team_color(ship.team),
                        transform,
                    });
                }
            }

            let vab = VertexAttribBuilder::new(&self.context)
                .data(&mut self.buffer_arena, &attribs)
                .size(4)
                .divisor(1);
            vab.index(1).offset(offset_of!(FlareAttribs, color)).build();
            vab.index(2)
                .offset(offset_of!(FlareAttribs, transform))
                .build();
            vab.index(3)
                .offset(offset_of!(FlareAttribs, transform) + 16)
                .build();
            vab.index(4)
                .offset(offset_of!(FlareAttribs, transform) + 32)
                .build();
            vab.index(5)
                .offset(offset_of!(FlareAttribs, transform) + 48)
                .build();

            // projection
            self.context.uniform_matrix4fv_with_f32_array(
                Some(&self.projection_loc),
                false,
                self.projection_matrix.data.as_slice(),
            );

            // current_time
            self.context
                .uniform1f(Some(&self.current_time_loc), snapshot.time as f32);

            let num_instances = attribs.len();
            self.context.draw_arrays_instanced(
                gl::TRIANGLE_STRIP,
                0,
                vertices.len() as i32,
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
