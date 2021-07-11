use crate::buffer_arena;
use nalgebra::{
    point, storage::Storage, Matrix4, Point2, Rotation2, Rotation3, Translation2, Translation3,
    Vector2, Vector3, Vector4,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation,
};
use WebGl2RenderingContext as gl;

pub struct WebGlRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    color_loc: WebGlUniformLocation,
    projection_matrix: Matrix4<f32>,
    buffer_arena: buffer_arena::BufferArena,
}

impl WebGlRenderer {
    pub fn new() -> Result<Self, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("glcanvas").unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>()?;

        let context = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;

        let vert_shader = compile_shader(
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
        let frag_shader = compile_shader(
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
        let program = link_program(&context, &vert_shader, &frag_shader)?;

        let transform_loc = context
            .get_uniform_location(&program, "transform")
            .ok_or("did not find uniform")?;

        let color_loc = context
            .get_uniform_location(&program, "color")
            .ok_or("did not find uniform")?;

        Ok(WebGlRenderer {
            context: context.clone(),
            program,
            transform_loc,
            color_loc,
            projection_matrix: Matrix4::identity(),
            buffer_arena: buffer_arena::BufferArena::new(context, 1024 * 1024)?,
        })
    }

    pub fn set_view(&mut self, zoom: f32, center: Point2<f32>) {
        let screen_width = self.context.drawing_buffer_width() as f32;
        let screen_height = self.context.drawing_buffer_height() as f32;
        let view_width = 1.0 / zoom;
        let view_height = view_width * (screen_height / screen_width);
        let left = center.x - view_width / 2.0;
        let right = center.x + view_width / 2.0;
        let bottom = center.y - view_height / 2.0;
        let top = center.y + view_height / 2.0;
        let znear = -1.0;
        let zfar = 1.0;
        self.projection_matrix = Matrix4::new_orthographic(left, right, bottom, top, znear, zfar);
    }

    pub fn update_viewport(&mut self) {
        let screen_width = self.context.drawing_buffer_width();
        let screen_height = self.context.drawing_buffer_height();
        self.context.viewport(0, 0, screen_width, screen_height);
    }

    pub fn clear(&mut self) {
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(gl::COLOR_BUFFER_BIT);
    }

    pub fn draw_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        thickness: f32,
        color: Vector4<f32>,
    ) {
        self.context.use_program(Some(&self.program));
        let p1 = self.projection_matrix.transform_point(&point![x1, y1, 0.0]);
        let p2 = self.projection_matrix.transform_point(&point![x2, y2, 0.0]);
        let vertices: [f32; 6] = [p1.x, p1.y, 0.0, p2.x, p2.y, 0.0];

        let (buffer, offset) = self.buffer_arena.write(&vertices);
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

        self.context.uniform4f(
            Some(&self.color_loc),
            color[0],
            color[1],
            color[2],
            color[3],
        );

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            &[
                1.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 1.0,
            ],
        );

        self.context.line_width(thickness);

        self.context
            .draw_arrays(gl::LINES, 0, (vertices.len() / 3) as i32);
    }

    pub fn draw_line_loop(
        &mut self,
        vertices: &[Vector2<f32>],
        translation: Translation2<f32>,
        rotation: Rotation2<f32>,
        thickness: f32,
        color: Vector4<f32>,
    ) {
        self.context.use_program(Some(&self.program));
        let translation = Translation3::new(translation.x, translation.y, 0.0);
        let rotation = Rotation3::from_axis_angle(&Vector3::z_axis(), rotation.angle());

        let mvp_matrix =
            self.projection_matrix * translation.to_homogeneous() * rotation.to_homogeneous();

        let mut new_vertices = vec![];
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

        self.context.uniform4f(
            Some(&self.color_loc),
            color[0],
            color[1],
            color[2],
            color[3],
        );

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            mvp_matrix.data.as_slice(),
        );

        self.context.line_width(thickness);

        self.context
            .draw_arrays(gl::LINE_LOOP, 0, (new_vertices.len() / 3) as i32);
    }

    pub fn draw_grid(&mut self, grid_size: f32, color: Vector4<f32>) {
        use crate::simulation::WORLD_SIZE;

        let mut vertices = vec![];
        let n = 1 + (WORLD_SIZE as f32 / grid_size) as i32;
        for i in -(n / 2)..(n / 2 + 1) {
            // Vertical
            vertices.push((i as f32) * grid_size);
            vertices.push((-WORLD_SIZE as f32) / 2.0);
            vertices.push(0.0);
            vertices.push((i as f32) * grid_size);
            vertices.push((WORLD_SIZE as f32) / 2.0);
            vertices.push(0.0);

            // Horizontal
            vertices.push((-WORLD_SIZE as f32) / 2.0);
            vertices.push((i as f32) * grid_size);
            vertices.push(0.0);
            vertices.push((WORLD_SIZE as f32) / 2.0);
            vertices.push((i as f32) * grid_size);
            vertices.push(0.0);
        }

        self.context.use_program(Some(&self.program));

        let (buffer, offset) = self.buffer_arena.write(&vertices);
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

        self.context.uniform4f(
            Some(&self.color_loc),
            color[0],
            color[1],
            color[2],
            color[3],
        );

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            self.projection_matrix.data.as_slice(),
        );

        self.context.line_width(1.0);

        self.context
            .draw_arrays(gl::LINES, 0, (vertices.len() / 3) as i32);
    }

    pub fn flush(&mut self) {
        self.context.flush();
    }
}

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, gl::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, gl::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
