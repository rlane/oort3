use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use crate::geometry;

use super::{buffer_arena, glutil};
use glutil::VertexAttribBuilder;
use nalgebra::{vector, Matrix4, Vector2, Vector4};
use oort_simulator::snapshot::Snapshot;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject};
use WebGl2RenderingContext as gl;

const MAX_PARTICLES: usize = 100;
const EMPTY_PARTICLE_CREATION_TIME: f32 = -100.0;

pub struct ParticleRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    current_time_loc: WebGlUniformLocation,
    scale_loc: WebGlUniformLocation,
    buffer_arena: buffer_arena::BufferArena,
    particles: Vec<Particle>,
    /// HashSet consisting of the time of the snapshot and the particle's index
    seen_particles: HashSet<Particle>,
    next_particle_index: usize,
    max_particles_seen: usize,
    vao: WebGlVertexArrayObject,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct Particle {
    position: Vector2<f32>,
    velocity: Vector2<f32>,
    color: Vector4<f32>,
    lifetime: f32,
    creation_time: f32,
}

impl PartialEq for Particle {
    fn eq(&self, other: &Particle) -> bool {
        let positions = self.position;
        let other_positions = other.position;
        if !positions.eq(&other_positions) {
            return false;
        }

        let velocity = self.velocity;
        let other_velocity = other.velocity;
        if !velocity.eq(&other_velocity) {
            return false;
        }

        let color = self.color;
        let other_color = other.color;
        if !color.eq(&other_color) {
            return false;
        }

        if self.lifetime - other.lifetime != 0.0 {
            return false;
        }

        if self.creation_time - other.creation_time != 0.0 {
            return false;
        }

        true
    }
}

impl Eq for Particle {}
impl Hash for Particle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let position = self.position;
        let velocity = self.velocity;
        let color = self.color;
        for position in &position {
            state.write(&position.to_be_bytes());
        }

        for velocity in &velocity {
            state.write(&velocity.to_be_bytes());
        }

        for color in &color {
            state.write(&color.to_be_bytes());
        }

        state.write(&self.lifetime.to_be_bytes());
        state.write(&self.creation_time.to_be_bytes());
    }
}

#[derive(Debug)]
pub struct DrawSet {
    projection_matrix: Matrix4<f32>,
    vertices_token: buffer_arena::Token,
    num_vertices: usize,
    attribs_token: buffer_arena::Token,
    num_instances: usize,
    current_time: f32,
}

impl DrawSet {
    pub fn len(&self) -> usize {
        self.num_instances
    }
}

impl ParticleRenderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = glutil::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"#version 300 es
uniform mat4 transform;
uniform float current_time;
uniform float scale;
layout(location = 0) in vec4 vertex;
layout(location = 1) in vec2 position;
layout(location = 2) in vec2 velocity;
layout(location = 3) in vec4 color;
layout(location = 4) in float lifetime;
layout(location = 5) in float creation_time;
out vec4 varying_color;

void main() {
    float dt = current_time - creation_time;
    float life_fraction = clamp(dt / lifetime, 0.0, 1.0);
    float size = dt >= 0.0 ? (1.0 - life_fraction) * scale : 0.0;
    vec2 p = vertex.xy * size + position + velocity * dt;
    gl_Position = transform * vec4(p, 0.0, 1.0);
    varying_color = vec4(color.x, color.y, color.z, color.w * (1.0 - life_fraction));
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

        let scale_loc = context
            .get_uniform_location(&program, "scale")
            .ok_or("did not find uniform")?;

        let vao = context
            .create_vertex_array()
            .ok_or("failed to create vertex array")?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        let mut particles = vec![];
        for _ in 0..MAX_PARTICLES {
            particles.push(Particle {
                position: vector![0.0, 0.0],
                velocity: vector![0.0, 0.0],
                color: vector![0.0, 0.0, 0.0, 0.0],
                lifetime: 1.0,
                creation_time: EMPTY_PARTICLE_CREATION_TIME,
            });
        }

        Ok(Self {
            context: context.clone(),
            program,
            transform_loc,
            current_time_loc,
            scale_loc,
            buffer_arena: buffer_arena::BufferArena::new(
                "particle_renderer",
                context,
                gl::ARRAY_BUFFER,
                1024 * 1024,
            )?,
            particles,
            seen_particles: HashSet::new(),
            next_particle_index: 0,
            max_particles_seen: MAX_PARTICLES,
            vao,
        })
    }

    pub fn add_particle(&mut self, particle: Particle) {
        // Don't duplicate particles
        if self.seen_particles.get(&particle).is_some() {
            return;
        }

        // If a particle is being overwritten, remove it from the seen index
        self.seen_particles
            .remove(&self.particles[self.next_particle_index]);

        self.particles[self.next_particle_index] = particle;
        self.next_particle_index += 1;
        if self.next_particle_index >= self.particles.len() {
            self.next_particle_index = 0;
        }
        self.seen_particles.insert(particle);
    }

    pub fn update(&mut self, snapshot: &Snapshot) {
        if snapshot.particles.len() > self.max_particles_seen {
            self.max_particles_seen = snapshot.particles.len();
            log::info!("Saw {} particles in snapshot", self.max_particles_seen);
        }
        for particle in snapshot.particles.iter() {
            self.add_particle(Particle {
                position: vector![particle.position.x as f32, particle.position.y as f32],
                velocity: vector![particle.velocity.x as f32, particle.velocity.y as f32],
                color: particle.color,
                lifetime: particle.lifetime,
                creation_time: snapshot.time as f32,
            });
        }
    }

    pub fn upload(&mut self, projection_matrix: &Matrix4<f32>, snapshot: &Snapshot) -> DrawSet {
        let current_time = snapshot.time as f32;

        let vertices = geometry::hexagon();

        let attribs: Vec<Particle> = self
            .particles
            .iter()
            .filter(|x| x.creation_time >= current_time - 10.0)
            .cloned()
            .collect();

        let creation_times: Vec<f32> = attribs.iter().map(|p| p.creation_time).collect();

        DrawSet {
            projection_matrix: *projection_matrix,
            vertices_token: self.buffer_arena.write(&vertices),
            num_vertices: vertices.len(),
            attribs_token: self.buffer_arena.write(attribs.as_slice()),
            num_instances: attribs.len(),
            current_time,
        }
    }

    pub fn draw(&mut self, drawset: &DrawSet, scale: f32) {
        if drawset.num_instances == 0 {
            return;
        }

        self.context.use_program(Some(&self.program));
        self.context.bind_vertex_array(Some(&self.vao));

        self.context
            .uniform1f(Some(&self.current_time_loc), drawset.current_time);

        self.context.uniform1f(Some(&self.scale_loc), scale);

        // vertices
        VertexAttribBuilder::new(&self.context)
            .data_token(&drawset.vertices_token)
            .index(0)
            .size(2)
            .build();

        // attribs
        let vab = VertexAttribBuilder::new(&self.context)
            .data_token(&drawset.attribs_token)
            .divisor(1);
        vab.index(1)
            .size(2)
            .offset(offset_of!(Particle, position))
            .build();
        vab.index(2)
            .size(2)
            .offset(offset_of!(Particle, velocity))
            .build();
        vab.index(3)
            .size(4)
            .offset(offset_of!(Particle, color))
            .build();
        vab.index(4)
            .size(1)
            .offset(offset_of!(Particle, lifetime))
            .build();
        vab.index(5)
            .size(1)
            .offset(offset_of!(Particle, creation_time))
            .build();

        // projection
        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            drawset.projection_matrix.data.as_slice(),
        );

        self.context.draw_arrays_instanced(
            gl::TRIANGLE_FAN,
            0,
            drawset.num_vertices as i32,
            drawset.num_instances as i32,
        );

        self.context.bind_vertex_array(None);
    }
}
