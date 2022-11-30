use super::{buffer_arena, glutil};
use crate::geometry;
use image::io::Reader as ImageReader;
use image::EncodableLayout;
use nalgebra::Matrix4;
use oort_api::Text;
use oort_simulator::debug::convert_color;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlTexture, WebGlUniformLocation};
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
    projection_matrix: Matrix4<f32>,
    buffer_arena: buffer_arena::BufferArena,
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
layout(location = 2) in vec4 color;
layout(location = 3) in vec2 base_texcoord;
layout(location = 4) in vec2 extent_texcoord;
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

        assert_eq!(context.get_error(), gl::NO_ERROR);

        Ok(Self {
            context: context.clone(),
            program,
            transform_loc,
            glyph_size_loc,
            sampler_loc,
            texture,
            projection_matrix: Matrix4::identity(),
            buffer_arena: buffer_arena::BufferArena::new(
                "text_renderer",
                context,
                gl::ARRAY_BUFFER,
                1024 * 1024,
            )?,
        })
    }

    pub fn update_projection_matrix(&mut self, m: &Matrix4<f32>) {
        self.projection_matrix = *m;
    }

    pub fn draw(&mut self, texts: &[Text], zoom: f32) {
        if texts.is_empty() {
            return;
        }

        let quad_vertices = geometry::triquad();
        let num_glyphs: usize = texts.iter().map(|x| x.length as usize).sum();
        let glyph_size = (1.0 / 100.0) / zoom;
        let glyph_width = 1.0 / FONT_COLS as f32;
        let glyph_height = 1.0 / FONT_ROWS as f32;
        let pixel_width = 1.0 / (FONT_COLS * FONT_GLYPH_SIZE) as f32;
        let pixel_height = 1.0 / (FONT_ROWS * FONT_GLYPH_SIZE) as f32;

        let mut positions: Vec<f32> = vec![];
        positions.reserve(2 * num_glyphs);
        let mut colors: Vec<f32> = vec![];
        colors.reserve(4 * num_glyphs);
        let mut base_texcoords: Vec<f32> = vec![];
        base_texcoords.reserve(2 * num_glyphs);
        let mut extent_texcoords: Vec<f32> = vec![];
        extent_texcoords.reserve(2 * num_glyphs);
        for text in texts {
            let mut x = text.x as f32;
            let color = convert_color(text.color);
            for i in 0..text.length {
                let idx = (text.text[i as usize] as usize - 32).clamp(0, FONT_ROWS * FONT_COLS - 1);
                let row = FONT_ROWS - idx / FONT_COLS - 1;
                let col = idx % FONT_COLS;

                positions.push(x);
                positions.push(text.y as f32);

                colors.extend_from_slice(color.as_slice());

                base_texcoords.push(col as f32 * glyph_width);
                base_texcoords.push(row as f32 * glyph_height + pixel_height);

                extent_texcoords.push(glyph_width - pixel_width);
                extent_texcoords.push(glyph_height - pixel_height);

                x += glyph_size * 1.1;
            }
        }

        self.context.use_program(Some(&self.program));

        let (buffer, vertices_offset) = self.buffer_arena.write(&quad_vertices);
        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));
        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 0,
            /*size=*/ 2,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ 0,
            vertices_offset as i32,
        );
        self.context.enable_vertex_attrib_array(0);

        let (buffer, positions_offset) = self.buffer_arena.write(&positions);
        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));
        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 1,
            /*size=*/ 2,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ 0,
            positions_offset as i32,
        );
        self.context.enable_vertex_attrib_array(1);
        self.context.vertex_attrib_divisor(1, 1);

        let (buffer, colors_offset) = self.buffer_arena.write(&colors);
        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));
        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 2,
            /*size=*/ 4,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ 0,
            colors_offset as i32,
        );
        self.context.enable_vertex_attrib_array(2);
        self.context.vertex_attrib_divisor(2, 1);

        let (buffer, base_texcoords_offset) = self.buffer_arena.write(&base_texcoords);
        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));
        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 3,
            /*size=*/ 2,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ 0,
            base_texcoords_offset as i32,
        );
        self.context.enable_vertex_attrib_array(3);
        self.context.vertex_attrib_divisor(3, 1);

        let (buffer, extent_texcoords_offset) = self.buffer_arena.write(&extent_texcoords);
        self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));
        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ 4,
            /*size=*/ 2,
            /*type_=*/ gl::FLOAT,
            /*normalized=*/ false,
            /*stride=*/ 0,
            extent_texcoords_offset as i32,
        );
        self.context.enable_vertex_attrib_array(4);
        self.context.vertex_attrib_divisor(4, 1);

        self.context
            .uniform1f(Some(&self.glyph_size_loc), glyph_size);

        self.context.uniform1i(Some(&self.sampler_loc), 0);

        self.context.uniform_matrix4fv_with_f32_array(
            Some(&self.transform_loc),
            false,
            self.projection_matrix.data.as_slice(),
        );

        self.context.active_texture(gl::TEXTURE0);
        self.context
            .bind_texture(gl::TEXTURE_2D, Some(&self.texture));

        self.context.draw_arrays_instanced(
            gl::TRIANGLES,
            0,
            quad_vertices.len() as i32,
            num_glyphs as i32,
        );

        self.context.bind_texture(gl::TEXTURE_2D, None);

        self.context.vertex_attrib_divisor(1, 0);
        self.context.vertex_attrib_divisor(2, 0);
        self.context.vertex_attrib_divisor(3, 0);
        self.context.vertex_attrib_divisor(4, 0);

        self.context.disable_vertex_attrib_array(0);
        self.context.disable_vertex_attrib_array(1);
        self.context.disable_vertex_attrib_array(2);
        self.context.disable_vertex_attrib_array(3);
        self.context.disable_vertex_attrib_array(4);
    }
}
