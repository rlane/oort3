use super::{buffer_arena, glutil};
use crate::geometry;
use glutil::VertexAttribBuilder;
use image::io::Reader as ImageReader;
use image::EncodableLayout;
use nalgebra::{point, vector, Matrix4, Vector2, Vector4};
use oort_api::Text;
use oort_simulator::color;
use wasm_bindgen::prelude::*;
use web_sys::{
    WebGl2RenderingContext, WebGlProgram, WebGlTexture, WebGlUniformLocation,
    WebGlVertexArrayObject,
};
use WebGl2RenderingContext as gl;

const FONT_PNG: &[u8] = include_bytes!("../../../assets/null_terminator.png");
const FONT_ROWS: usize = 12;
const FONT_COLS: usize = 8;
const FONT_GLYPH_SIZE: usize = 8;

pub struct TextRenderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
    transform_loc: WebGlUniformLocation,
    glyph_size_loc: WebGlUniformLocation,
    sampler_loc: WebGlUniformLocation,
    texture: WebGlTexture,
    buffer_arena: buffer_arena::BufferArena,
    vao: WebGlVertexArrayObject,
}

pub struct DrawSet {
    pixel_projection_matrix: Matrix4<f32>,
    num_instances: usize,
    vertices_token: buffer_arena::Token,
    num_vertices: usize,
    attribs_token: buffer_arena::Token,
    screen_glyph_size: f32,
}

struct Attribs {
    position: Vector2<f32>,
    base_texcoord: Vector2<f32>,
    extent_texcoord: Vector2<f32>,
    #[allow(dead_code)]
    pad: [f32; 2],
    color: Vector4<f32>,
}

impl TextRenderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader = glutil::compile_shader(
            &context,
            gl::VERTEX_SHADER,
            r#"#version 300 es
uniform mat4 transform;
uniform float glyph_size;
layout(location = 0) in vec2 vertex;
layout(location = 1) in vec2 position;
layout(location = 2) in vec2 base_texcoord;
layout(location = 3) in vec2 extent_texcoord;
layout(location = 4) in vec4 color;
out vec2 varying_texcoord;
out vec4 varying_color;
void main() {
    vec2 v = vertex + vec2(0.5, 0.5);
    gl_Position = transform * vec4(position + (vec2(0.0, -1.0) + v) * glyph_size, 0.0, 1.0);
    varying_texcoord = base_texcoord + v * extent_texcoord;
    varying_color = color;
}
    "#,
        )?;
        let frag_shader = glutil::compile_shader(
            &context,
            gl::FRAGMENT_SHADER,
            r#"#version 300 es
precision mediump float;
uniform sampler2D sampler;
in vec2 varying_texcoord;
in vec4 varying_color;
out vec4 fragmentColor;
void main() {
    fragmentColor = texture(sampler, varying_texcoord) * varying_color;
}
    "#,
        )?;
        let program = glutil::link_program(&context, &vert_shader, &frag_shader)?;

        let transform_loc = context
            .get_uniform_location(&program, "transform")
            .ok_or("did not find uniform")?;

        let glyph_size_loc = context
            .get_uniform_location(&program, "glyph_size")
            .ok_or("did not find uniform")?;

        let sampler_loc = context
            .get_uniform_location(&program, "sampler")
            .ok_or("did not find uniform")?;

        let texture;
        {
            let font = ImageReader::new(std::io::Cursor::new(FONT_PNG))
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            let font = font.flipv();
            let rgba8 = font.to_rgba8();
            let pixels = rgba8.as_bytes();

            texture = context.create_texture().unwrap();
            context.bind_texture(gl::TEXTURE_2D, Some(&texture));

            context
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    font.width() as i32,
                    font.height() as i32,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    Some(pixels),
                )
                .unwrap();

            context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            context.bind_texture(gl::TEXTURE_2D, None);
        }

        let vao = context
            .create_vertex_array()
            .ok_or("failed to create vertex array")?;

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context: context.clone(),
            program,
            transform_loc,
            glyph_size_loc,
            sampler_loc,
            texture,
            buffer_arena: buffer_arena::BufferArena::new(
                "text_renderer",
                context,
                gl::ARRAY_BUFFER,
                1024 * 1024,
            )?,
            vao,
        })
    }

    pub fn upload(&mut self, world_projection_matrix: &Matrix4<f32>, texts: &[Text]) -> DrawSet {
        let screen_width = self.context.drawing_buffer_width() as f32;
        let screen_height = self.context.drawing_buffer_height() as f32;

        let pixel_projection_matrix = {
            let left = 0.0;
            let right = screen_width;
            let bottom = 0.0;
            let top = screen_height;
            let znear = -1.0;
            let zfar = 1.0;
            Matrix4::new_orthographic(left, right, bottom, top, znear, zfar)
        };

        let quad_vertices = geometry::triquad();
        let vertices_token = self.buffer_arena.write(&quad_vertices);

        let num_glyphs: usize = texts.iter().map(|x| x.length as usize).sum();
        let scale = 2.0;
        let screen_glyph_size = (FONT_GLYPH_SIZE - 1) as f32 * scale;
        let font_glyph_width = 1.0 / FONT_COLS as f32;
        let font_glyph_height = 1.0 / FONT_ROWS as f32;
        let font_pixel_width = 1.0 / (FONT_COLS * FONT_GLYPH_SIZE) as f32;
        let font_pixel_height = 1.0 / (FONT_ROWS * FONT_GLYPH_SIZE) as f32;

        let mut attribs = vec![];
        attribs.reserve(num_glyphs);
        for text in texts {
            let worldpos = vector![text.x as f32, text.y as f32];
            let projected =
                world_projection_matrix.transform_point(&point![worldpos.x, worldpos.y, 0.0]);
            let projected_pixels = vector![
                (projected.x + 1.0) * screen_width / 2.0,
                (projected.y + 1.0) * screen_height / 2.0
            ];
            let mut pos = vector![projected_pixels.x.floor(), projected_pixels.y.floor()];
            let color = color::from_u24(text.color);
            for i in 0..text.length {
                let idx = (text.text[i as usize] as usize - 32).clamp(0, FONT_ROWS * FONT_COLS - 1);
                let row = FONT_ROWS - idx / FONT_COLS - 1;
                let col = idx % FONT_COLS;

                let base_texcoord = vector![
                    col as f32 * font_glyph_width,
                    row as f32 * font_glyph_height + font_pixel_height
                ];
                let extent_texcoord = vector![
                    font_glyph_width - font_pixel_width,
                    font_glyph_height - font_pixel_height
                ];

                attribs.push(Attribs {
                    position: pos,
                    base_texcoord,
                    extent_texcoord,
                    pad: [0.0; 2],
                    color,
                });

                pos.x += (FONT_GLYPH_SIZE as f32 + 1.0) * scale;
            }
        }
        let attribs_token = self.buffer_arena.write(&attribs);

        DrawSet {
            pixel_projection_matrix,
            num_instances: attribs.len(),
            vertices_token,
            num_vertices: quad_vertices.len(),
            attribs_token,
            screen_glyph_size,
        }
    }

    pub fn draw(&mut self, drawset: &DrawSet) {
        if drawset.num_instances == 0 {
            return;
        }

        self.context.use_program(Some(&self.program));
        self.context.bind_vertex_array(Some(&self.vao));

        // vertex
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
            .offset(offset_of!(Attribs, position))
            .build();
        vab.index(2)
            .size(2)
            .offset(offset_of!(Attribs, base_texcoord))
            .build();
        vab.index(3)
            .size(2)
            .offset(offset_of!(Attribs, extent_texcoord))
            .build();
        vab.index(4)
            .size(4)
            .offset(offset_of!(Attribs, color))
            .build();

        self.context
            .uniform1f(Some(&self.glyph_size_loc), drawset.screen_glyph_size);

        self.context.uniform1i(Some(&self.sampler_loc), 0);

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            drawset.pixel_projection_matrix.data.as_slice(),
        );

        self.context.active_texture(gl::TEXTURE0);
        self.context
            .bind_texture(gl::TEXTURE_2D, Some(&self.texture));

        self.context.draw_arrays_instanced(
            gl::TRIANGLES,
            0,
            drawset.num_vertices as i32,
            drawset.num_instances as i32,
        );

        self.context.bind_texture(gl::TEXTURE_2D, None);

        self.context.bind_vertex_array(None);
    }
}
