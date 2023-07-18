use super::{buffer_arena, geometry, glutil};
use glutil::VertexAttribBuilder;
use nalgebra::vector;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlFramebuffer, WebGlProgram, WebGlTexture};
use WebGl2RenderingContext as gl;

pub const TEXTURE_WIDTH: i32 = 2048;
pub const TEXTURE_HEIGHT: i32 = 2048;
pub const REDUCTION: i32 = 2;

pub struct Blur {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    buffer_arena: buffer_arena::BufferArena,
    texture: WebGlTexture,
    fb: WebGlFramebuffer,
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
precision mediump float;
in vec2 v_texcoord;
uniform sampler2D u_texture;
out vec4 fragmentColor;
void main() {
    vec2 iResolution = vec2(2048.0, 2048.0);
    float Pi = 6.28318530718; // Pi*2

    // GAUSSIAN BLUR SETTINGS {{{
    float Directions = 16.0; // BLUR DIRECTIONS (Default 16.0 - More is better but slower)
    float Quality = 16.0; // BLUR QUALITY (Default 4.0 - More is better but slower)
    float Size = 8.0; // BLUR SIZE (Radius)
    // GAUSSIAN BLUR SETTINGS }}}

    vec2 Radius = Size/iResolution.xy;

    // Normalized pixel coordinates (from 0 to 1)
    vec2 uv = gl_FragCoord.xy/iResolution.xy;
    // Pixel colour
    vec4 orig_color = texture(u_texture, v_texcoord);
    vec4 blurred_color = orig_color;

    // Blur calculations
    for( float d=0.0; d<Pi; d+=Pi/Directions)
    {
        for(float i=1.0/Quality; i<=1.0; i+=1.0/Quality)
        {
            blurred_color += texture( u_texture, v_texcoord+vec2(cos(d),sin(d))*Radius*i);
        }
    }

    // Output to screen
    blurred_color /= Quality * Directions / 1.5;


    fragmentColor = blurred_color;
}
    "#,
        )?;
        let program = glutil::link_program(&context, &vert_shader, &frag_shader)?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

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
                TEXTURE_WIDTH,
                TEXTURE_HEIGHT,
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

        Ok(Self {
            context: context.clone(),
            program,
            buffer_arena: buffer_arena::BufferArena::new(
                "blur",
                context,
                gl::ARRAY_BUFFER,
                1024 * 1024,
            )?,
            texture,
            fb,
        })
    }

    // Set up to render to the blur texture
    pub fn start(&mut self) {
        self.context
            .bind_framebuffer(gl::FRAMEBUFFER, Some(&self.fb));
        let screen_width = self.context.drawing_buffer_width();
        let screen_height = self.context.drawing_buffer_height();
        self.context
            .viewport(0, 0, screen_width / REDUCTION, screen_height / REDUCTION);
    }

    // Render blurred texture to screen
    pub fn finish(&mut self) {
        self.context.bind_framebuffer(gl::FRAMEBUFFER, None);

        let screen_width = self.context.drawing_buffer_width();
        let screen_height = self.context.drawing_buffer_height();
        self.context.viewport(0, 0, screen_width, screen_height);
        self.context
            .bind_texture(gl::TEXTURE_2D, Some(&self.texture));
        self.draw();
        self.context.bind_texture(gl::TEXTURE_2D, None);
    }

    fn draw(&mut self) {
        self.context.use_program(Some(&self.program));

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
            screen_width / (REDUCTION * TEXTURE_WIDTH) as f32,
            screen_height / (REDUCTION * TEXTURE_HEIGHT) as f32
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
