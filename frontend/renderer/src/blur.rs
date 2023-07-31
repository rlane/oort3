use super::{buffer_arena, geometry, glutil};
use glutil::VertexAttribBuilder;
use nalgebra::vector;
use wasm_bindgen::prelude::*;
use web_sys::{
    WebGl2RenderingContext, WebGlFramebuffer, WebGlProgram, WebGlTexture, WebGlUniformLocation,
};
use WebGl2RenderingContext as gl;

pub const TEXTURE_SIZE: i32 = 1024;

pub struct Blur {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    buffer_arena: buffer_arena::BufferArena,
    resolution_loc: WebGlUniformLocation,
    radius_loc: WebGlUniformLocation,
    active_framebuffers: Framebuffers,
    standby_framebuffers: Framebuffers,
}

pub struct Framebuffers {
    renderbuffer_fb: WebGlFramebuffer,
    texture_fb: WebGlFramebuffer,
    texture: WebGlTexture,
}

impl Blur {
    const REDUCTION: i32 = 4;

    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = glutil::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"#version 300 es
layout(location = 0) in vec4 vertex;
layout(location = 1) in vec4 texcoord;
out vec2 v_texcoord;

void main() {
    gl_Position = vertex;
    v_texcoord = texcoord.xy;
}
    "#,
        )?;
        let frag_shader = glutil::compile_shader(
            &context,
            gl::FRAGMENT_SHADER,
            r#"#version 300 es
// https://github.com/mattdesl/lwjgl-basics/wiki/ShaderLesson5
// https://webgl-shaders.com/shaders/frag-blur.glsl
precision mediump float;

uniform sampler2D tex;
uniform float resolution;
uniform float radius;

in vec2 v_texcoord;
out vec4 fragmentColor;

void main() {
    float step = radius/resolution;

    vec4 sum = vec4(0.0);
    if (radius < 0.0) {
    sum += texture(tex, v_texcoord + step * vec2(0, 0));
    } else {
    sum += (1.0 / 256.0) * texture(tex, v_texcoord + step * vec2(-2, -2));
    sum += (4.0 / 256.0) * texture(tex, v_texcoord + step * vec2(-2, -1));
    sum += (6.0 / 256.0) * texture(tex, v_texcoord + step * vec2(-2, 0));
    sum += (4.0 / 256.0) * texture(tex, v_texcoord + step * vec2(-2, 1));
    sum += (1.0 / 256.0) * texture(tex, v_texcoord + step * vec2(-2, 2));
    sum += (4.0 / 256.0) * texture(tex, v_texcoord + step * vec2(-1, -2));
    sum += (16.0 / 256.0) * texture(tex, v_texcoord + step * vec2(-1, -1));
    sum += (24.0 / 256.0) * texture(tex, v_texcoord + step * vec2(-1, 0));
    sum += (16.0 / 256.0) * texture(tex, v_texcoord + step * vec2(-1, 1));
    sum += (4.0 / 256.0) * texture(tex, v_texcoord + step * vec2(-1, 2));
    sum += (6.0 / 256.0) * texture(tex, v_texcoord + step * vec2(0, -2));
    sum += (24.0 / 256.0) * texture(tex, v_texcoord + step * vec2(0, -1));
    sum += (36.0 / 256.0) * texture(tex, v_texcoord + step * vec2(0, 0));
    sum += (24.0 / 256.0) * texture(tex, v_texcoord + step * vec2(0, 1));
    sum += (6.0 / 256.0) * texture(tex, v_texcoord + step * vec2(0, 2));
    sum += (4.0 / 256.0) * texture(tex, v_texcoord + step * vec2(1, -2));
    sum += (16.0 / 256.0) * texture(tex, v_texcoord + step * vec2(1, -1));
    sum += (24.0 / 256.0) * texture(tex, v_texcoord + step * vec2(1, 0));
    sum += (16.0 / 256.0) * texture(tex, v_texcoord + step * vec2(1, 1));
    sum += (4.0 / 256.0) * texture(tex, v_texcoord + step * vec2(1, 2));
    sum += (1.0 / 256.0) * texture(tex, v_texcoord + step * vec2(2, -2));
    sum += (4.0 / 256.0) * texture(tex, v_texcoord + step * vec2(2, -1));
    sum += (6.0 / 256.0) * texture(tex, v_texcoord + step * vec2(2, 0));
    sum += (4.0 / 256.0) * texture(tex, v_texcoord + step * vec2(2, 1));
    sum += (1.0 / 256.0) * texture(tex, v_texcoord + step * vec2(2, 2));
    }

    sum.a *= 2.0;
    fragmentColor = sum;
}"#,
        )?;
        let program = glutil::link_program(&context, &vert_shader, &frag_shader)?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        let resolution_loc = context
            .get_uniform_location(&program, "resolution")
            .ok_or("did not find uniform")?;
        let radius_loc = context
            .get_uniform_location(&program, "radius")
            .ok_or("did not find uniform")?;

        Ok(Self {
            context: context.clone(),
            program,
            buffer_arena: buffer_arena::BufferArena::new(
                "blur",
                context.clone(),
                gl::ARRAY_BUFFER,
                1024 * 1024,
            )?,
            active_framebuffers: Self::create_framebuffers(context.clone()),
            standby_framebuffers: Self::create_framebuffers(context),
            resolution_loc,
            radius_loc,
        })
    }

    pub fn create_framebuffers(context: WebGl2RenderingContext) -> Framebuffers {
        // Multisample framebuffer
        let renderbuffer_fb = {
            let max_samples = context
                .get_parameter(gl::MAX_SAMPLES)
                .unwrap()
                .as_f64()
                .unwrap() as i32;
            let renderbuffer = context.create_renderbuffer().unwrap();
            context.bind_renderbuffer(gl::RENDERBUFFER, Some(&renderbuffer));
            context.renderbuffer_storage_multisample(
                gl::RENDERBUFFER,
                max_samples,
                gl::RGBA8,
                TEXTURE_SIZE,
                TEXTURE_SIZE,
            );

            let fb = context.create_framebuffer().unwrap();
            context.bind_framebuffer(gl::FRAMEBUFFER, Some(&fb));
            context.framebuffer_renderbuffer(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::RENDERBUFFER,
                Some(&renderbuffer),
            );

            assert_eq!(
                context.check_framebuffer_status(gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE
            );

            context.bind_framebuffer(gl::FRAMEBUFFER, None);
            fb
        };

        // Texture-backed framebuffer
        let (texture_fb, texture) = {
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
            context.tex_parameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
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

            (fb, texture)
        };

        Framebuffers {
            renderbuffer_fb,
            texture_fb,
            texture,
        }
    }

    // Set up to render to renderbuffer
    pub fn start(&mut self) {
        std::mem::swap(
            &mut self.active_framebuffers,
            &mut self.standby_framebuffers,
        );
        let screen_width = self.context.drawing_buffer_width();
        let screen_height = self.context.drawing_buffer_height();
        self.context.viewport(
            0,
            0,
            screen_width / Self::REDUCTION,
            screen_height / Self::REDUCTION,
        );
        self.context.bind_framebuffer(
            gl::FRAMEBUFFER,
            Some(&self.active_framebuffers.renderbuffer_fb),
        );
    }

    // Stop rendering to renderbuffer
    pub fn finish(&mut self) {
        let screen_width = self.context.drawing_buffer_width();
        let screen_height = self.context.drawing_buffer_height();
        self.context.viewport(0, 0, screen_width, screen_height);

        self.context.bind_framebuffer(gl::FRAMEBUFFER, None);
        self.context.bind_framebuffer(
            gl::READ_FRAMEBUFFER,
            Some(&self.active_framebuffers.renderbuffer_fb),
        );
        self.context.bind_framebuffer(
            gl::DRAW_FRAMEBUFFER,
            Some(&self.active_framebuffers.texture_fb),
        );

        self.context.blit_framebuffer(
            0,
            0,
            screen_width / Self::REDUCTION,
            screen_height / Self::REDUCTION,
            0,
            0,
            screen_width / Self::REDUCTION,
            screen_height / Self::REDUCTION,
            gl::COLOR_BUFFER_BIT,
            gl::LINEAR,
        );

        self.context.bind_framebuffer(gl::READ_FRAMEBUFFER, None);
        self.context.bind_framebuffer(gl::DRAW_FRAMEBUFFER, None);

        self.context
            .bind_texture(gl::TEXTURE_2D, Some(&self.active_framebuffers.texture));
        self.context.generate_mipmap(gl::TEXTURE_2D);
        self.context.bind_texture(gl::TEXTURE_2D, None);
    }

    // Draw blurred texture to screen
    pub fn draw(&mut self) {
        self.context.blend_func(gl::ONE, gl::ONE);

        self.context.bind_framebuffer(gl::FRAMEBUFFER, None);
        self.context
            .bind_texture(gl::TEXTURE_2D, Some(&self.active_framebuffers.texture));
        self.blur_once(2.0);
        self.context.bind_texture(gl::TEXTURE_2D, None);

        self.context
            .blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    fn blur_once(&mut self, radius: f32) {
        self.context.use_program(Some(&self.program));

        self.context
            .uniform1f(Some(&self.radius_loc), radius / Self::REDUCTION as f32);
        self.context
            .uniform1f(Some(&self.resolution_loc), TEXTURE_SIZE as f32);

        // vertex
        let vertices = geometry::clip_quad();
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
            screen_width / (Self::REDUCTION * TEXTURE_SIZE) as f32,
            screen_height / (Self::REDUCTION * TEXTURE_SIZE) as f32
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
