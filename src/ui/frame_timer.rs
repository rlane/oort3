const SHORT_HISTORY_LENGTH: usize = 60;
const LONG_HISTORY_LENGTH: usize = 300;

#[derive(Default)]
pub struct FrameTimer {
    start_time: f64,
    elapsed_times: Vec<f64>,
    total: f64,
    count: i32,
}

impl FrameTimer {
    pub fn start(&mut self, now: f64) {
        self.start_time = now;
    }

    pub fn end(&mut self, now: f64) {
        let elapsed = now - self.start_time;
        self.elapsed_times.push(elapsed);
        if self.elapsed_times.len() > LONG_HISTORY_LENGTH {
            self.elapsed_times.remove(0);
        }
        self.total += elapsed;
        self.count += 1;
    }

    // Returns worst latency in (last frame, short history, long history).
    pub fn get_latency(&self) -> (f64, f64, f64) {
        let v = &self.elapsed_times;
        if v.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        return (
            v[0],
            v[..std::cmp::min(v.len(), SHORT_HISTORY_LENGTH)]
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| f64::max(a, b)),
            v.iter().fold(f64::NEG_INFINITY, |a, &b| f64::max(a, b)),
        );
    }

    pub fn get_average(&mut self) -> f64 {
        if self.count > 0 {
            self.total / (self.count as f64)
        } else {
            0.0
        }
    }
}
