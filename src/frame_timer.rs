use macroquad::time;

const SHORT_HISTORY_LENGTH: usize = 60;
const LONG_HISTORY_LENGTH: usize = 300;

#[derive(Default)]
pub struct FrameTimer {
    start_times: std::collections::HashMap<String, f64>,
    elapsed_times: std::collections::HashMap<String, Vec<f64>>,
    names: Vec<String>,
}

impl FrameTimer {
    pub fn start(self: &mut FrameTimer, name: &str) {
        self.start_times.insert(name.to_string(), time::get_time());
        if !self.elapsed_times.contains_key(name) {
            self.elapsed_times.insert(name.to_string(), Vec::new());
            self.names.push(name.to_string());
        }
    }

    pub fn end(self: &mut FrameTimer, name: &str) {
        let now = time::get_time();
        let start_time = *self.start_times.get(name).unwrap_or(&now);
        let elapsed = now - start_time;
        let v = self.elapsed_times.get_mut(name).unwrap();
        v.push(elapsed);
        if v.len() > LONG_HISTORY_LENGTH {
            v.remove(0);
        }
    }

    // Returns worst latency in (last frame, short history, long history).
    pub fn get(self: &FrameTimer, name: &str) -> (f64, f64, f64) {
        let v = self.elapsed_times.get(name).unwrap();
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

    pub fn get_names(self: &FrameTimer) -> &[String] {
        &self.names[..]
    }
}
