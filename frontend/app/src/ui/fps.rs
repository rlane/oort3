const BUFFER_SIZE: usize = 32;

pub struct FPS {
    last_frame_start_time: f64,
    frames: usize,
    sum: f32,
    buffer: [f32; BUFFER_SIZE],
}

impl FPS {
    pub fn new() -> Self {
        FPS {
            last_frame_start_time: 0.0,
            frames: 0,
            sum: 0.0,
            buffer: [0.0; BUFFER_SIZE],
        }
    }

    pub fn start_frame(&mut self, now: f64) {
        if self.last_frame_start_time == 0.0 {
            // no-op
        } else {
            let elapsed = now - self.last_frame_start_time;
            self.sum -= self.buffer[self.frames % BUFFER_SIZE];
            self.buffer[self.frames % BUFFER_SIZE] = elapsed as f32;
            self.sum += elapsed as f32;
            self.frames += 1;
        }
        self.last_frame_start_time = now;
    }

    pub fn fps(&self) -> f64 {
        let frame_duration = self.sum / BUFFER_SIZE.min(self.frames) as f32;
        1e3 / frame_duration as f64
    }
}

impl Default for FPS {
    fn default() -> Self {
        Self::new()
    }
}
