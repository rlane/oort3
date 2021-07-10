use nalgebra::{point, vector, Matrix3, Point2, Vector4};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlProgram, WebGlRenderingContext, WebGlShader};

pub struct WebGlRenderer {
    context: WebGlRenderingContext,
    program: WebGlProgram,
    perspective_matrix: Matrix3<f32>,
}

impl WebGlRenderer {
    pub fn new() -> Result<Self, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("glcanvas").unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>()?;

        let context = canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()?;

        let vert_shader = compile_shader(
            &context,
            WebGlRenderingContext::VERTEX_SHADER,
            r#"
        attribute vec4 position;
        void main() {
            gl_Position = position;
        }
    "#,
        )?;
        let frag_shader = compile_shader(
            &context,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"
        precision mediump float;
        uniform vec4 color;
        void main() {
            gl_FragColor = color;
        }
    "#,
        )?;
        let program = link_program(&context, &vert_shader, &frag_shader)?;

        let scale = 1.0 / 1000.0;
        let center = point![0.0, 0.0];
        Ok(WebGlRenderer {
            context,
            program,
            perspective_matrix: Matrix3::new_nonuniform_scaling_wrt_point(
                &vector![scale, scale],
                &center,
            ),
        })
    }

    pub fn set_perspective(&mut self, zoom: f32, center: Point2<f32>) {
        let screen_width = self.context.drawing_buffer_width() as f32;
        let screen_height = self.context.drawing_buffer_height() as f32;
        let scale = vector![zoom, zoom * screen_width / screen_height];
        self.perspective_matrix = Matrix3::new_nonuniform_scaling_wrt_point(&scale, &center);
    }

    pub fn clear(&mut self) {
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
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
        let p1 = self.perspective_matrix.transform_point(&point![x1, y1]);
        let p2 = self.perspective_matrix.transform_point(&point![x2, y2]);
        let vertices: [f32; 6] = [p1.x, p1.y, 0.0, p2.x, p2.y, 0.0];

        let maybe_buffer = self.context.create_buffer();
        if maybe_buffer == None {
            // Lost GL context.
            return;
        }
        let buffer = maybe_buffer.unwrap();
        self.context
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let vert_array = js_sys::Float32Array::view(&vertices);

            self.context.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        self.context.vertex_attrib_pointer_with_i32(
            0,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        self.context.enable_vertex_attrib_array(0);

        let color_loc = self
            .context
            .get_uniform_location(&self.program, "color")
            .expect("missing color uniform");
        self.context
            .uniform4f(Some(&color_loc), color[0], color[1], color[2], color[3]);

        self.context.line_width(thickness);

        self.context
            .draw_arrays(WebGlRenderingContext::LINES, 0, (vertices.len() / 3) as i32);
    }
}

pub fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
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
    context: &WebGlRenderingContext,
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
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
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
