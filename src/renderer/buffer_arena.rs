use log::debug;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};
use WebGl2RenderingContext as gl;

pub struct BufferArena {
    context: WebGl2RenderingContext,
    buffer_size: u32,
    active_buffer: WebGlBuffer,
    standby_buffer: WebGlBuffer,
    offset: u32,
    target: u32,
}

impl BufferArena {
    pub fn new(
        context: WebGl2RenderingContext,
        target: u32,
        buffer_size: u32,
    ) -> Result<Self, String> {
        let active_buffer = context.create_buffer().ok_or("failed to create buffer")?;
        let standby_buffer = context.create_buffer().ok_or("failed to create buffer")?;
        Ok(BufferArena {
            context,
            buffer_size,
            active_buffer,
            standby_buffer,
            offset: buffer_size,
            target,
        })
    }

    pub fn write(&mut self, data: &[f32]) -> (WebGlBuffer, u32) {
        assert!(!data.is_empty());
        let data_length = (data.len() * 4) as u32;
        if (self.buffer_size - self.offset) < data_length {
            std::mem::swap(&mut self.active_buffer, &mut self.standby_buffer);
            self.context
                .bind_buffer(self.target, Some(&self.active_buffer));
            self.context.buffer_data_with_i32(
                self.target,
                self.buffer_size as i32,
                gl::DYNAMIC_DRAW,
            );
            self.offset = 0;
            debug!("Allocated new buffer len={}", self.buffer_size);
        }
        let offset = self.offset;
        self.context
            .bind_buffer(self.target, Some(&self.active_buffer));
        unsafe {
            // Note that `Float32Array::view` is somewhat dangerous (hence the
            // `unsafe`!). This is creating a raw view into our module's
            // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
            // (aka do a memory allocation in Rust) it'll cause the buffer to change,
            // causing the `Float32Array` to be invalid.
            //
            // As a result, after `Float32Array::view` we have to be very careful not to
            // do any memory allocations before it's dropped.
            let view = js_sys::Float32Array::view(data);
            self.context.buffer_sub_data_with_i32_and_array_buffer_view(
                /*target=*/ self.target,
                /*offset=*/ offset as i32,
                /*src_data=*/ &view,
            );
        }
        self.offset += data_length;
        (self.active_buffer.clone(), offset)
    }
}
