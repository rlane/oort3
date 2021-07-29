use super::{buffer_arena, glutil, model};
use crate::simulation::ship::{ShipClass, ShipHandle};
use crate::simulation::Simulation;
use nalgebra::{storage::Storage, vector, Matrix4, Vector4};
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

    fn team_color(team: i32) -> Vector4<f32> {
        match team {
            0 => vector![0.99, 0.98, 0.00, 1.00],
            1 => vector![0.99, 0.00, 0.98, 1.00],
            9 => vector![0.40, 0.40, 0.40, 1.00],
            _ => vector![1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn draw(&mut self, sim: &Simulation) {
        let thickness = 2.0;

        self.context.use_program(Some(&self.program));
        self.context.line_width(thickness);

        let mut ships_by_class = std::collections::HashMap::<ShipClass, Vec<ShipHandle>>::new();

        for &handle in sim.ships.iter() {
            let ship = sim.ship(handle);
            let class = &ship.data().class;
            if !ships_by_class.contains_key(class) {
                ships_by_class.insert(*class, vec![]);
            }
            ships_by_class.get_mut(class).unwrap().push(handle);
        }

        for (class, handles) in ships_by_class.iter() {
            // vertex

            let model_vertices = match class {
                ShipClass::Fighter => model::ship(),
                ShipClass::Asteroid { variant } => model::asteroid(*variant),
                ShipClass::Target => model::target(),
                ShipClass::Missile => model::missile(),
            };

            let num_vertices = model_vertices.len();
            let mut vertex_data: Vec<f32> = vec![];
            vertex_data.reserve(model_vertices.len() * 2);
            for v in model_vertices {
                vertex_data.push(v[0]);
                vertex_data.push(v[1]);
            }

            let (buffer, offset) = self.buffer_arena.write(&vertex_data);
            self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));

            self.context.vertex_attrib_pointer_with_i32(
                /*indx=*/ 0,
                /*size=*/ 2,
                /*type_=*/ gl::FLOAT,
                /*normalized=*/ false,
                /*stride=*/ 0,
                offset as i32,
            );
            self.context.enable_vertex_attrib_array(0);

            // position

            let mut position_data: Vec<f32> = vec![];
            position_data.reserve(handles.len() * 2);
            for &handle in handles {
                let ship = sim.ship(handle);
                position_data.push(ship.position().x as f32);
                position_data.push(ship.position().y as f32);
            }

            let (buffer, offset) = self.buffer_arena.write(&position_data);
            self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));

            self.context.vertex_attrib_pointer_with_i32(
                /*indx=*/ 1,
                /*size=*/ 2,
                /*type_=*/ gl::FLOAT,
                /*normalized=*/ false,
                /*stride=*/ 0,
                offset as i32,
            );
            self.context.vertex_attrib_divisor(1, 1);
            self.context.enable_vertex_attrib_array(1);

            // heading

            let mut heading_data: Vec<f32> = vec![];
            heading_data.reserve(handles.len());
            for &handle in handles {
                let ship = sim.ship(handle);
                heading_data.push(ship.heading() as f32);
            }

            let (buffer, offset) = self.buffer_arena.write(&heading_data);
            self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));

            self.context.vertex_attrib_pointer_with_i32(
                /*indx=*/ 2,
                /*size=*/ 1,
                /*type_=*/ gl::FLOAT,
                /*normalized=*/ false,
                /*stride=*/ 0,
                offset as i32,
            );
            self.context.vertex_attrib_divisor(2, 1);
            self.context.enable_vertex_attrib_array(2);

            // color

            let mut color_data: Vec<f32> = vec![];
            color_data.reserve(handles.len());
            for &handle in handles {
                let ship = sim.ship(handle);
                let color = Self::team_color(ship.data().team);
                color_data.push(color.x);
                color_data.push(color.y);
                color_data.push(color.z);
                color_data.push(color.w);
            }

            let (buffer, offset) = self.buffer_arena.write(&color_data);
            self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));

            self.context.vertex_attrib_pointer_with_i32(
                /*indx=*/ 3,
                /*size=*/ 4,
                /*type_=*/ gl::FLOAT,
                /*normalized=*/ false,
                /*stride=*/ 0,
                offset as i32,
            );
            self.context.vertex_attrib_divisor(3, 1);
            self.context.enable_vertex_attrib_array(3);

            // projection

            self.context.uniform_matrix4fv_with_f32_array(
                Some(&self.transform_loc),
                false,
                self.projection_matrix.data.as_slice(),
            );

            let num_instances = handles.len();
            self.context.draw_arrays_instanced(
                gl::LINE_LOOP,
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
