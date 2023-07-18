use super::{buffer_arena, geometry, glutil};
use glutil::VertexAttribBuilder;
use nalgebra::vector;
use wasm_bindgen::prelude::*;
use web_sys::{
    WebGl2RenderingContext, WebGlFramebuffer, WebGlProgram, WebGlTexture, WebGlUniformLocation,
};
use WebGl2RenderingContext as gl;

pub const TEXTURE_SIZE: i32 = 2048;
pub const REDUCTION: i32 = 1;

#[derive(PartialEq)]
enum Direction {
    Horizontal,
    Vertical,
}

pub struct Blur {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    buffer_arena: buffer_arena::BufferArena,
    textures: Vec<WebGlTexture>,
    fbs: Vec<WebGlFramebuffer>,
    resolution_loc: WebGlUniformLocation,
    radius_loc: WebGlUniformLocation,
    dir_loc: WebGlUniformLocation,
}

impl Blur {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = glutil::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"#version 300 es
layout(location = 0) in vec4 vertex;
layout(location = 1) in vec4 texcoord;
out vec2 v_texcoord;

void main() {
    gl_Position = vec4(vertex.xyz * 2.0, vertex.w);
    v_texcoord = texcoord.xy;
}
    "#,
        )?;
        let frag_shader = glutil::compile_shader(
            &context,
            gl::FRAGMENT_SHADER,
            r#"#version 300 es
// https://github.com/mattdesl/lwjgl-basics/wiki/ShaderLesson5
precision mediump float;

uniform sampler2D tex;
uniform float resolution;
uniform float radius;
uniform vec2 dir;

in vec2 v_texcoord;
out vec4 fragmentColor;

void main() {
    //this will be our RGBA sum
    vec4 sum = vec4(0.0);

    //our original texcoord for this fragment
    vec2 tc = v_texcoord;

    //the amount to blur, i.e. how far off center to sample from 
    //1.0 -> blur by one pixel
    //2.0 -> blur by two pixels, etc.
    float blur = radius/resolution;

    //the direction of our blur
    //(1.0, 0.0) -> x-axis blur
    //(0.0, 1.0) -> y-axis blur
    float hstep = dir.x;
    float vstep = dir.y;

    //apply blurring, using a 9-tap filter with predefined gaussian weights

    sum += texture(tex, vec2(tc.x - 4.0*blur*hstep, tc.y - 4.0*blur*vstep)) * 0.0162162162;
    sum += texture(tex, vec2(tc.x - 3.0*blur*hstep, tc.y - 3.0*blur*vstep)) * 0.0540540541;
    sum += texture(tex, vec2(tc.x - 2.0*blur*hstep, tc.y - 2.0*blur*vstep)) * 0.1216216216;
    sum += texture(tex, vec2(tc.x - 1.0*blur*hstep, tc.y - 1.0*blur*vstep)) * 0.1945945946;

    sum += texture(tex, vec2(tc.x, tc.y)) * 0.2270270270;

    sum += texture(tex, vec2(tc.x + 1.0*blur*hstep, tc.y + 1.0*blur*vstep)) * 0.1945945946;
    sum += texture(tex, vec2(tc.x + 2.0*blur*hstep, tc.y + 2.0*blur*vstep)) * 0.1216216216;
    sum += texture(tex, vec2(tc.x + 3.0*blur*hstep, tc.y + 3.0*blur*vstep)) * 0.0540540541;
    sum += texture(tex, vec2(tc.x + 4.0*blur*hstep, tc.y + 4.0*blur*vstep)) * 0.0162162162;

    sum.a *= 2.0;
    fragmentColor = sum;
}"#,
        )?;
        let program = glutil::link_program(&context, &vert_shader, &frag_shader)?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        let mut textures = vec![];
        let mut fbs = vec![];

        for _ in [0, 1] {
            let texture = context.create_texture().unwrap();
            context.bind_texture(gl::TEXTURE_2D, Some(&texture));
            let level = 0;
            let internal_format = gl::RGBA;
            let border = 0;
            let format = gl::RGBA;
            let typ = gl::UNSIGNED_BYTE;
            context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
                gl::TEXTURE_2D,
                level,
                internal_format as i32,
                TEXTURE_SIZE,
                TEXTURE_SIZE,
                border,
                format,
                typ,
                None,
            )
            .unwrap();
            // set the filtering so we don't need mips
            context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            context.bind_texture(gl::TEXTURE_2D, None);

            let fb = context.create_framebuffer().unwrap();
            context.bind_framebuffer(gl::FRAMEBUFFER, Some(&fb));

            // attach the texture as the first color attachment
            let attachment_point = gl::COLOR_ATTACHMENT0;
            context.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                attachment_point,
                gl::TEXTURE_2D,
                Some(&texture),
                level,
            );

            assert_eq!(
                context.check_framebuffer_status(gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE
            );

            context.bind_framebuffer(gl::FRAMEBUFFER, None);

            textures.push(texture);
            fbs.push(fb);
        }

        let resolution_loc = context
            .get_uniform_location(&program, "resolution")
            .ok_or("did not find uniform")?;
        let radius_loc = context
            .get_uniform_location(&program, "radius")
            .ok_or("did not find uniform")?;
        let dir_loc = context
            .get_uniform_location(&program, "dir")
            .ok_or("did not find uniform")?;

        Ok(Self {
            context: context.clone(),
            program,
            buffer_arena: buffer_arena::BufferArena::new(
                "blur",
                context,
                gl::ARRAY_BUFFER,
                1024 * 1024,
            )?,
            textures,
            fbs,
            resolution_loc,
            radius_loc,
            dir_loc,
        })
    }

    // Set up to render to the blur texture
    pub fn start(&mut self) {
        self.context
            .bind_framebuffer(gl::FRAMEBUFFER, Some(&self.fbs[0]));
        let screen_width = self.context.drawing_buffer_width();
        let screen_height = self.context.drawing_buffer_height();
        self.context
            .viewport(0, 0, screen_width / REDUCTION, screen_height / REDUCTION);
    }

    // Render blurred texture to screen
    pub fn finish(&mut self) {
        // Horizontal pass into fbs[1]
        self.context
            .bind_framebuffer(gl::FRAMEBUFFER, Some(&self.fbs[1]));
        self.context.clear_color(0.0, 0.0, 0.0, 0.0);
        self.context.clear(gl::COLOR_BUFFER_BIT);
        self.context
            .bind_texture(gl::TEXTURE_2D, Some(&self.textures[0]));
        self.draw(Direction::Horizontal);

        // Vertical pass into screen
        self.context.bind_framebuffer(gl::FRAMEBUFFER, None);
        let screen_width = self.context.drawing_buffer_width();
        let screen_height = self.context.drawing_buffer_height();
        self.context.viewport(0, 0, screen_width, screen_height);
        self.context
            .bind_texture(gl::TEXTURE_2D, Some(&self.textures[1]));
        self.draw(Direction::Vertical);
        self.context.bind_texture(gl::TEXTURE_2D, None);
    }

    fn draw(&mut self, direction: Direction) {
        self.context.use_program(Some(&self.program));

        self.context.uniform1f(Some(&self.radius_loc), 1.0);
        self.context
            .uniform1f(Some(&self.resolution_loc), TEXTURE_SIZE as f32);

        if direction == Direction::Vertical {
            self.context.uniform2f(Some(&self.dir_loc), 0.0, 1.0);
        } else {
            self.context.uniform2f(Some(&self.dir_loc), 1.0, 0.0);
        }

        // vertex
        let vertices = geometry::quad();
        VertexAttribBuilder::new(&self.context)
            .data(&mut self.buffer_arena, &vertices)
            .index(0)
            .size(2)
            .build();

        // texcoord
        let mut texcoord = geometry::unit_quad();
        let screen_width = self.context.drawing_buffer_width() as f32;
        let screen_height = self.context.drawing_buffer_height() as f32;
        let scale = vector![
            screen_width / (REDUCTION * TEXTURE_SIZE) as f32,
            screen_height / (REDUCTION * TEXTURE_SIZE) as f32
        ];
        for point in texcoord.iter_mut() {
            point.x *= scale.x;
            point.y *= scale.y;
        }
        VertexAttribBuilder::new(&self.context)
            .data(&mut self.buffer_arena, &texcoord)
            .index(1)
            .size(2)
            .build();

        self.context
            .draw_arrays(gl::TRIANGLE_STRIP, 0, vertices.len() as i32);

        self.context.disable_vertex_attrib_array(0);
        self.context.disable_vertex_attrib_array(1);
    }
}
