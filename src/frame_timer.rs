use macroquad::time;

#[derive(Default)]
pub struct FrameTimer {
    frame_start_time: f64,
    ewma: f64,
    frame_times: Vec<f64>,
    frame_start_times: Vec<f64>,
}

impl FrameTimer {
    pub fn start_frame(self: &mut FrameTimer) {
        self.frame_start_time = time::get_time();
    }

    pub fn end_frame(self: &mut FrameTimer) {
        let frame_time = time::get_time() - self.frame_start_time;
        self.ewma = self.ewma * 0.9 + frame_time * 0.1;
        self.frame_times.push(frame_time);
        if self.frame_times.len() > 120 {
            self.frame_times.remove(0);
        }
        self.frame_start_times.push(self.frame_start_time);
        if self.frame_start_times.len() > 120 {
            self.frame_start_times.remove(0);
        }
    }

    pub fn get_moving_average(self: &FrameTimer) -> f64 {
        return self.ewma;
    }

    pub fn get_recent_max(self: &FrameTimer) -> f64 {
        return self
            .frame_times
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| f64::max(a, b));
    }

    pub fn get_fps(self: &FrameTimer) -> usize {
        let past = time::get_time() - 1.0;
        return self.frame_start_times.iter().filter(|&&x| x > past).count();
    }
}
