use super::{buffer_arena, model, webgl};
use crate::simulation::ship::ShipClass;
use crate::simulation::Simulation;
use nalgebra::{
    storage::Storage, vector, Matrix4, Rotation2, Rotation3, Translation2, Translation3, Vector2,
    Vector3,
};
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
        let vert_shader = webgl::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"
        uniform mat4 transform;
        attribute vec4 position;
        void main() {
            gl_Position = transform * position;
        }
    "#,
        )?;
        let frag_shader = webgl::compile_shader(
            &context,
            gl::FRAGMENT_SHADER,
            r#"
        precision mediump float;
        uniform vec4 color;
        void main() {
            gl_FragColor = color;
        }
    "#,
        )?;
        let program = webgl::link_program(&context, &vert_shader, &frag_shader)?;

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
            buffer_arena: buffer_arena::BufferArena::new(context, 1024 * 1024)?,
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

        for &index in sim.ships.iter() {
            let ship = sim.ship(index);
            let x = ship.position().x as f32;
            let y = ship.position().y as f32;
            let h = ship.heading() as f32;
            let translation = Translation2::new(x, y);
            let rotation = Rotation2::new(h);

            match ship.data().class {
                ShipClass::Fighter => self.draw_model(&model::ship(), translation, rotation),
                ShipClass::Asteroid => self.draw_model(
                    &model::asteroid(ship.data().model_variant),
                    translation,
                    rotation,
                ),
            }
        }
    }

    fn draw_model(
        &mut self,
        vertices: &[Vector2<f32>],
        translation: Translation2<f32>,
        rotation: Rotation2<f32>,
    ) {
        let translation = Translation3::new(translation.x, translation.y, 0.0);
        let rotation = Rotation3::from_axis_angle(&Vector3::z_axis(), rotation.angle());
        let mvp_matrix =
            self.projection_matrix * translation.to_homogeneous() * rotation.to_homogeneous();

        let mut new_vertices = vec![];
        new_vertices.reserve(vertices.len() * 3);
        for v in vertices {
            new_vertices.push(v[0]);
            new_vertices.push(v[1]);
            new_vertices.push(0.0);
        }

        let (buffer, offset) = self.buffer_arena.write(&new_vertices);
        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));

        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 0,
            /*size=*/ 3,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ 0,
            offset as i32,
        );
        self.context.enable_vertex_attrib_array(0);

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            mvp_matrix.data.as_slice(),
        );

        self.context
            .draw_arrays(gl::LINE_LOOP, 0, (new_vertices.len() / 3) as i32);
    }
}
