use super::{buffer_arena, glutil};
use crate::simulation::{SimEvents, Simulation};
use nalgebra::{storage::ContiguousStorage, vector, Matrix4, Vector2};
use rand::Rng;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};
use WebGl2RenderingContext as gl;

pub struct ParticleRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    current_time_loc: WebGlUniformLocation,
    projection_matrix: Matrix4<f32>,
    buffer_arena: buffer_arena::BufferArena,
    particles: Vec<Particle>,
    next_particle_index: usize,
}

pub struct Particle {
    position: Vector2<f32>,
    velocity: Vector2<f32>,
    creation_time: f32,
}

impl ParticleRenderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = glutil::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"#version 300 es
uniform mat4 transform;
uniform float current_time;
layout(location = 0) in vec4 position;
layout(location = 1) in vec3 velocity;
layout(location = 2) in float creation_time;
out vec4 varying_color;

void main() {
    float dt = current_time - creation_time;
    float lifetime = 0.5;
    float life_fraction = clamp(dt / lifetime, 0.0, 1.0);
    gl_Position = transform * (position + vec4(velocity, 0.0) * dt);
    varying_color = vec4(1.0, 1.0, 1.0, 1.0 - life_fraction);
    gl_PointSize = (1.0 - life_fraction) * 10.0;
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

        let current_time_loc = context
            .get_uniform_location(&program, "current_time")
            .ok_or("did not find uniform")?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        let mut particles = vec![];
        for _ in 0..100 {
            particles.push(Particle {
                position: vector![0.0, 0.0],
                velocity: vector![0.0, 0.0],
                creation_time: -100.0,
            });
        }

        Ok(Self {
            context: context.clone(),
            program,
            transform_loc,
            current_time_loc,
            projection_matrix: Matrix4::identity(),
            buffer_arena: buffer_arena::BufferArena::new(
                "particle_renderer",
                context,
                gl::ARRAY_BUFFER,
                1024 * 1024,
            )?,
            particles,
            next_particle_index: 0,
        })
    }

    pub fn update_projection_matrix(&mut self, m: &Matrix4<f32>) {
        self.projection_matrix = *m;
    }

    pub fn add_particle(&mut self, particle: Particle) {
        self.particles[self.next_particle_index] = particle;
        self.next_particle_index += 1;
        if self.next_particle_index >= self.particles.len() {
            self.next_particle_index = 0;
        }
    }

    pub fn update(&mut self, current_time: f64, events: &SimEvents) {
        let mut rng = rand::thread_rng();
        for position in events.hits.iter() {
            let s = 400.0;
            self.add_particle(Particle {
                position: vector![position.x as f32, position.y as f32],
                velocity: vector![rng.gen_range(-s..s), rng.gen_range(-s..s)],
                creation_time: current_time as f32,
            });
        }

        for position in events.ships_destroyed.iter() {
            let s = 200.0;
            for _ in 0..10 {
                let v = vector![rng.gen_range(-s..s), rng.gen_range(-s..s)];
                let p = vector![position.x as f32, position.y as f32] + v * rng.gen_range(0.0..0.1);
                self.add_particle(Particle {
                    position: p,
                    velocity: v,
                    creation_time: current_time as f32 + rng.gen_range(-0.1..0.3),
                });
            }
        }
    }

    pub fn draw(&mut self, sim: &Simulation) {
        self.context.use_program(Some(&self.program));

        let current_time = sim.time() as f32;
        self.context
            .uniform1f(Some(&self.current_time_loc), current_time);

        let mut num_instances = 0;
        let stride = 5 * 4;
        let mut data: Vec<f32> = vec![];
        data.reserve(self.particles.len() * 5);
        for particle in self.particles.iter() {
            if particle.creation_time < current_time - 10.0 {
                continue;
            }
            num_instances += 1;
            data.push(particle.position.x);
            data.push(particle.position.y);
            data.push(particle.velocity.x);
            data.push(particle.velocity.y);
            data.push(particle.creation_time);
        }

        if num_instances == 0 {
            return;
        }

        let (buffer, offset) = self.buffer_arena.write(&data);
        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));

        // position
        let position_index = 0;
        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ position_index,
            /*size=*/ 2,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ stride,
            offset as i32,
        );
        self.context.enable_vertex_attrib_array(position_index);

        // velocity
        let velocity_index = 1;
        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ velocity_index,
            /*size=*/ 2,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ stride,
            8 + offset as i32,
        );
        self.context.enable_vertex_attrib_array(velocity_index);

        // creation_time
        let creation_time_index = 2;
        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ creation_time_index,
            /*size=*/ 1,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ stride,
            16 + offset as i32,
        );
        self.context.enable_vertex_attrib_array(creation_time_index);

        // projection
        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            self.projection_matrix.data.as_slice(),
        );

        self.context
            .draw_arrays(gl::POINTS, 0, num_instances as i32);

        self.context.disable_vertex_attrib_array(position_index);
        self.context.disable_vertex_attrib_array(velocity_index);
        self.context
            .disable_vertex_attrib_array(creation_time_index);
    }
}
