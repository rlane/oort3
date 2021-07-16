use super::{buffer_arena, glutil, model};
use crate::simulation::ship::{ShipClass, ShipHandle};
use crate::simulation::Simulation;
use nalgebra::{storage::Storage, vector, Matrix4};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};
use WebGl2RenderingContext as gl;

pub struct ShipRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    color_loc: WebGlUniformLocation,
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

// https://gist.github.com/yiwenl/3f804e80d0930e34a0b33359259b556c
vec2 rotate(vec2 v, float a) {
    float s = sin(a);
    float c = cos(a);
    mat2 m = mat2(c, s, -s, c);
    return m * v;
}

void main() {
    gl_Position = transform * (position + vec4(rotate(vertex, heading), 0.0, 0.0));
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
            buffer_arena: buffer_arena::BufferArena::new(context, gl::ARRAY_BUFFER, 1024 * 1024)?,
        })
    }

    pub fn update_projection_matrix(&mut self, m: &Matrix4<f32>) {
        self.projection_matrix = *m;
    }

    pub fn draw(&mut self, sim: &Simulation) {
        let thickness = 2.0;
        let color = vector![0.99, 0.98, 0.00, 1.00];

        self.context.use_program(Some(&self.program));
        self.context.line_width(thickness);

        self.context.uniform4f(
            Some(&self.color_loc),
            color[0],
            color[1],
            color[2],
            color[3],
        );

        let mut ships_by_class = std::collections::HashMap::<ShipClass, Vec<ShipHandle>>::new();
        ships_by_class.insert(ShipClass::Fighter, vec![]);
        ships_by_class.insert(ShipClass::Asteroid, vec![]);

        for &handle in sim.ships.iter() {
            let ship = sim.ship(handle);
            ships_by_class
                .get_mut(&ship.data().class)
                .unwrap()
                .push(handle);
        }

        for (class, handles) in ships_by_class.iter() {
            if handles.is_empty() {
                continue;
            }

            // vertex

            let model_vertices = match class {
                ShipClass::Fighter => model::ship(),
                ShipClass::Asteroid => model::asteroid(),
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

            self.context.disable_vertex_attrib_array(0);
            self.context.disable_vertex_attrib_array(1);
            self.context.disable_vertex_attrib_array(2);
        }
    }
}
