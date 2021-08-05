use instant::Instant;
use log::{debug, info};
use web_sys::{WebGl2RenderingContext, WebGlBuffer};
use WebGl2RenderingContext as gl;

pub struct BufferArena {
    name: String,
    context: WebGl2RenderingContext,
    buffer_size: u32,
    active_buffer: WebGlBuffer,
    standby_buffer: WebGlBuffer,
    offset: u32,
    target: u32,
    fill_count: u32,
    creation_time: Instant,
}

impl BufferArena {
    pub fn new(
        name: &str,
        context: WebGl2RenderingContext,
        target: u32,
        buffer_size: u32,
    ) -> Result<Self, String> {
        let active_buffer = context.create_buffer().ok_or("failed to create buffer")?;
        let standby_buffer = context.create_buffer().ok_or("failed to create buffer")?;
        context.bind_buffer(target, Some(&active_buffer));
        context.buffer_data_with_i32(target, buffer_size as i32, gl::DYNAMIC_DRAW);
        let now = Instant::now();
        Ok(BufferArena {
            name: name.to_string(),
            context,
            buffer_size,
            active_buffer,
            standby_buffer,
            offset: 0,
            target,
            fill_count: 0,
            creation_time: now,
        })
    }

    pub fn write<T>(&mut self, data: &[T]) -> (WebGlBuffer, u32) {
        let buf = unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * std::mem::size_of::<T>(),
            )
        };
        self.write_buf(buf)
    }

    pub fn write_buf(&mut self, data: &[u8]) -> (WebGlBuffer, u32) {
        assert!(!data.is_empty());
        let data_length = data.len() as u32;
        if (self.buffer_size - self.offset) < data_length {
            std::mem::swap(&mut self.active_buffer, &mut self.standby_buffer);
            self.context
                .bind_buffer(self.target, Some(&self.active_buffer));
            self.context.buffer_data_with_i32(
                self.target,
                self.buffer_size as i32,
                gl::DYNAMIC_DRAW,
            );
            self.fill_count += 1;
            debug!(
                "Allocated new buffer name={} len={} fill_count={}",
                &self.name, self.buffer_size, self.fill_count
            );
            if self.fill_count == 1 || self.fill_count == 10 || self.fill_count == 100 {
                let elapsed = self.creation_time.elapsed().as_secs_f64();
                info!(
                    "Filled {} rendering buffer {} time(s) in {:.2}s, {:.2} MB/s",
                    &self.name,
                    self.fill_count,
                    elapsed,
                    (self.fill_count * self.buffer_size) as f64 / (elapsed * 1e6)
                );
            }
            self.offset = 0;
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
            let view = js_sys::Uint8Array::view(data);
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
