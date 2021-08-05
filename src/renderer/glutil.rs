use super::buffer_arena::BufferArena;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader};
use WebGl2RenderingContext as gl;

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

#[derive(Clone)]
pub struct VertexAttribBuilder {
    context: WebGl2RenderingContext,
    buffer: Option<WebGlBuffer>,
    base: u32,
    indx: u32,
    size: i32,
    type_: u32,
    normalized: bool,
    stride: i32,
    offset: i32,
    divisor: u32,
}

impl VertexAttribBuilder {
    pub fn new(context: &WebGl2RenderingContext) -> Self {
        Self {
            context: context.clone(),
            buffer: None,
            base: 0,
            indx: 0,
            size: 1,
            type_: gl::FLOAT,
            normalized: false,
            stride: 0,
            offset: 0,
            divisor: 0,
        }
    }

    pub fn index(&self, indx: u32) -> Self {
        VertexAttribBuilder {
            indx,
            ..self.clone()
        }
    }

    pub fn size(&self, size: i32) -> Self {
        VertexAttribBuilder {
            size,
            ..self.clone()
        }
    }

    pub fn datatype(&self, type_: u32) -> Self {
        VertexAttribBuilder {
            type_,
            ..self.clone()
        }
    }

    pub fn normalized(&self, normalized: bool) -> Self {
        VertexAttribBuilder {
            normalized,
            ..self.clone()
        }
    }

    pub fn stride(&self, stride: i32) -> Self {
        VertexAttribBuilder {
            stride,
            ..self.clone()
        }
    }

    pub fn offset(&self, offset: usize) -> Self {
        VertexAttribBuilder {
            offset: offset as i32,
            ..self.clone()
        }
    }

    pub fn divisor(&self, divisor: u32) -> Self {
        VertexAttribBuilder {
            divisor,
            ..self.clone()
        }
    }

    pub fn data<T>(&self, arena: &mut BufferArena, data: &[T]) -> Self {
        let (buffer, base) = arena.write(data);
        VertexAttribBuilder {
            buffer: Some(buffer),
            base,
            stride: std::mem::size_of::<T>() as i32,
            ..self.clone()
        }
    }

    pub fn build(&self) {
        assert!(self.buffer.is_some());
        self.context
            .bind_buffer(gl::ARRAY_BUFFER, Some(self.buffer.as_ref().unwrap()));
        self.context.vertex_attrib_pointer_with_i32(
            /*indx=*/ self.indx,
            /*size=*/ self.size,
            /*type_=*/ self.type_,
            /*normalized=*/ self.normalized,
            /*stride=*/ self.stride,
            self.base as i32 + self.offset as i32,
        );
        self.context.vertex_attrib_divisor(self.indx, self.divisor);
        self.context.enable_vertex_attrib_array(self.indx);
    }
}
