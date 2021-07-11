pub struct FPS {
    frame_time_moving_average: f64,
    last_frame_start_time: f64,
}

impl FPS {
    pub fn new() -> Self {
        FPS {
            frame_time_moving_average: 0.0,
            last_frame_start_time: 0.0,
        }
    }

    pub fn start_frame(&mut self, now: f64) {
        if self.last_frame_start_time == 0.0 {
            // no-op
        } else if self.frame_time_moving_average == 0.0 {
            self.frame_time_moving_average = now - self.last_frame_start_time;
        } else {
            let weight = 0.01;
            self.frame_time_moving_average = weight * (now - self.last_frame_start_time)
                + (1.0 - weight) * self.frame_time_moving_average;
        }
        self.last_frame_start_time = now;
    }

    pub fn fps(&self) -> f64 {
        if self.frame_time_moving_average == 0.0 {
            0.0
        } else {
            1e3 / self.frame_time_moving_average
        }
    }
}

impl Default for FPS {
    fn default() -> Self {
        Self::new()
    }
}
