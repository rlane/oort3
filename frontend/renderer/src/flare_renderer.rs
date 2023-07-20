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
    scale: Vector2<f32>,
}

fn flare_positions(class: ShipClass) -> Vec<FlarePosition> {
    match class {
        ShipClass::Fighter => {
            let x = 4.0;
            vec![
                FlarePosition {
                    offset: vector![-7.0, 0.0],
                    angle: TAU / 2.0,
                    scale: vector![20.0, 16.0],
                },
                FlarePosition {
                    offset: vector![-7.0, 0.0],
                    angle: 0.0,
                    scale: vector![10.0, 10.0],
                },
                FlarePosition {
                    offset: vector![0.0, x],
                    angle: TAU / 4.0,
                    scale: vector![10.0, 8.0],
                },
                FlarePosition {
                    offset: vector![0.0, -x],
                    angle: -TAU / 4.0,
                    scale: vector![10.0, 8.0],
                },
            ]
        }
        ShipClass::Frigate => {
            let x = -48.0;
            vec![
                FlarePosition {
                    offset: vector![x, 0.0],
                    angle: TAU / 2.0,
                    scale: vector![200.0, 50.0],
                },
                FlarePosition {
                    offset: vector![x, 0.0],
                    angle: 0.0,
                    scale: vector![100.0, 40.0],
                },
                FlarePosition {
                    offset: vector![-24.0, 18.0],
                    angle: TAU / 4.0,
                    scale: vector![10.0, 10.0],
                },
                FlarePosition {
                    offset: vector![24.0, -18.0],
                    angle: -TAU / 4.0,
                    scale: vector![10.0, 10.0],
                },
            ]
        }
        ShipClass::Cruiser => {
            let x = -96.0;
            let y = 48.0;
            vec![
                FlarePosition {
                    offset: vector![x, 0.0],
                    angle: TAU / 2.0,
                    scale: vector![150.0, 80.0],
                },
                FlarePosition {
                    offset: vector![x, 0.0],
                    angle: 0.0,
                    scale: vector![100.0, 80.0],
                },
                FlarePosition {
                    offset: vector![0.0, y],
                    angle: TAU / 4.0,
                    scale: vector![10.0, 10.0],
                },
                FlarePosition {
                    offset: vector![0.0, -y],
                    angle: -TAU / 4.0,
                    scale: vector![10.0, 10.0],
                },
            ]
        }
        ShipClass::Missile => {
            vec![
                FlarePosition {
                    offset: vector![-2.1, 0.0],
                    angle: TAU / 2.0,
                    scale: vector![10.0, 6.0],
                },
                FlarePosition {
                    offset: vector![-2.1, 0.0],
                    angle: 0.0,
                    scale: vector![10.0, 6.0],
                },
                FlarePosition {
                    offset: vector![0.0, 0.0],
                    angle: TAU / 4.0,
                    scale: vector![5.0, 3.0],
                },
                FlarePosition {
                    offset: vector![0.0, 0.0],
                    angle: -TAU / 4.0,
                    scale: vector![5.0, 3.0],
                },
            ]
        }
        ShipClass::Torpedo => {
            vec![
                FlarePosition {
                    offset: vector![-6.4, 0.0],
                    angle: TAU / 2.0,
                    scale: vector![10.0, 8.0],
                },
                FlarePosition {
                    offset: vector![-6.4, 0.0],
                    angle: 0.0,
                    scale: vector![10.0, 8.0],
                },
                FlarePosition {
                    offset: vector![0.0, 1.6],
                    angle: TAU / 4.0,
                    scale: vector![5.0, 3.0],
                },
                FlarePosition {
                    offset: vector![0.0, -1.6],
                    angle: -TAU / 4.0,
                    scale: vector![5.0, 3.0],
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
layout(location = 0) in vec4 vertex;
layout(location = 1) in float id;
layout(location = 2) in mat4 transform;
out vec2 varying_vertex;
out float varying_id;

void main() {
    varying_vertex = vertex.xy;
    varying_id = id;
    gl_Position = projection * (transform * vertex);
}
    "#,
        )?;
        let frag_shader = glutil::compile_shader(
            &context,
            gl::FRAGMENT_SHADER,
            r#"#version 300 es
precision mediump float;
uniform float current_time;
in vec2 varying_vertex;
in float varying_id;
out vec4 fragmentColor;

const float M_PI = 3.14159265358979323846264338327950288;

// https://www.shadertoy.com/view/4sc3D7
// Copyright (C) 2014 by Benjamin 'BeRo' Rosseaux
// http://creativecommons.org/publicdomain/zero/1.0/
vec3 colorTemperatureToRGB(const in float temperature){
  // Values from: http://blenderartists.org/forum/showthread.php?270332-OSL-Goodness&p=2268693&viewfull=1#post2268693   
  mat3 m = (temperature <= 6500.0) ? mat3(vec3(0.0, -2902.1955373783176, -8257.7997278925690),
                                          vec3(0.0, 1669.5803561666639, 2575.2827530017594),
                                          vec3(1.0, 1.3302673723350029, 1.8993753891711275)) : 
                                     mat3(vec3(1745.0425298314172, 1216.6168361476490, -8257.7997278925690),
                                          vec3(-2666.3474220535695, -2173.1012343082230, 2575.2827530017594),
                                          vec3(0.55995389139931482, 0.70381203140554553, 1.8993753891711275)); 
  return mix(clamp(vec3(m[0] / (vec3(clamp(temperature, 1000.0, 40000.0)) + m[1]) + m[2]), vec3(0.0), vec3(1.0)), vec3(1.0), smoothstep(1000.0, 0.0, temperature));
}

// https://www.shadertoy.com/view/4dS3Wd
// By Morgan McGuire @morgan3d, http://graphicscodex.com
// Reuse permitted under the BSD license.
float hash(float p) { p = fract(p * 0.011); p *= p + 7.5; p *= p + p; return fract(p); }
float hash(vec2 p) {vec3 p3 = fract(vec3(p.xyx) * 0.13); p3 += dot(p3, p3.yzx + 3.333); return fract((p3.x + p3.y) * p3.z); }

float noise(vec2 x) {
    vec2 i = floor(x);
    vec2 f = fract(x);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}


float fbm(vec2 x) {
    float v = 0.0;
    float a = 0.5;
    vec2 shift = vec2(100);
    mat2 rot = mat2(cos(0.5), sin(0.5), -sin(0.5), cos(0.50));
    for (int i = 0; i < 5; ++i) {
        v += a * noise(x);
        x = rot * x * 2.0 + shift;
        a *= 0.5;
    }
    return v;
}

void main() {
    vec2 uv = varying_vertex + vec2(0.5, 0.5);
    float bx = cos((1.0 - uv.x) * M_PI * 0.25);
    float by = sin(uv.y * M_PI * 0.5 + M_PI / 4.0);
    float brightness = clamp(pow(bx * by, 10.0), 0.0, 1.0);
    if (brightness < 0.01) {
        discard;
        return;
    }
    float t = current_time + varying_id * 0.01;
    float max_temp = 2000.0 + 10000.0 * fbm(uv - vec2(t * 5.0, sin(t * 10.0)));
    fragmentColor = vec4(
        colorTemperatureToRGB(brightness * max_temp) * vec3(0.8, 0.8, 1.2),
        brightness);
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
            id: f32,
            #[allow(dead_code)]
            pad: [f32; 3],
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
            attribs.reserve(ships.len() * 4);
            for ship in ships.iter() {
                let p = ship.position.coords.cast::<f32>();
                let ship_transform = Matrix4::new_translation(&vector![p.x, p.y, 0.0])
                    * Matrix4::from_euler_angles(0.0, 0.0, ship.heading as f32);
                for flare_position in &flare_positions {
                    let direction =
                        UnitComplex::from_angle(ship.heading as f32 + flare_position.angle)
                            .transform_vector(&vector![1.0, 0.0]);
                    let strength = (-ship.acceleration.cast::<f32>().dot(&direction)).max(0.0);
                    if strength <= 0.0 {
                        continue;
                    }

                    let strength_scale_transform = Matrix4::new_nonuniform_scaling(&vector![
                        -flare_position.scale.x * strength.sqrt(),
                        flare_position.scale.y,
                        1.0
                    ]);
                    let flare_offset_transform = Matrix4::new_translation(&vector![
                        flare_position.offset.x,
                        flare_position.offset.y,
                        0.0
                    ]);

                    let flare_model_transform = Matrix4::new_translation(&vector![-0.5, 0.0, 0.0]);

                    let flare_rotation_transform = Matrix4::from_axis_angle(
                        &Unit::new_normalize(vector![0.0, 0.0, 1.0]),
                        flare_position.angle,
                    );

                    let transform = ship_transform
                        * flare_offset_transform
                        * flare_rotation_transform
                        * strength_scale_transform
                        * flare_model_transform;
                    attribs.push(FlareAttribs {
                        id: (ship.id % 73) as f32,
                        pad: [0.0; 3],
                        transform,
                    });
                }
            }

            if attribs.is_empty() {
                continue;
            }

            let vab = VertexAttribBuilder::new(&self.context)
                .data(&mut self.buffer_arena, &attribs)
                .divisor(1);
            vab.index(1).offset(offset_of!(FlareAttribs, id)).build();
            vab.index(2)
                .offset(offset_of!(FlareAttribs, transform))
                .size(4)
                .build();
            vab.index(3)
                .offset(offset_of!(FlareAttribs, transform) + 16)
                .size(4)
                .build();
            vab.index(4)
                .offset(offset_of!(FlareAttribs, transform) + 32)
                .size(4)
                .build();
            vab.index(5)
                .offset(offset_of!(FlareAttribs, transform) + 48)
                .size(4)
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

            for i in 0..6 {
                self.context.vertex_attrib_divisor(i, 0);
                self.context.disable_vertex_attrib_array(i);
            }
        }
    }
}
