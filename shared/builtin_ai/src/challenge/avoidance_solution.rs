// Challenge: Avoidance
// Reach the position advertised on radio channel 1. Beware of the hostile frigate.
use oort_api::prelude::{maths_rs::num::Base, *};

pub struct Ship {
    target: Option<Vec2>
}

impl Ship {
    pub fn new() -> Ship {
        Ship { target: None }
    }

    pub fn tick(&mut self) {
        if self.target.is_none() {
            set_radio_channel(1);
            if let Some([x, y, _, _]) = receive() {
                self.target = Some(Vec2::new(x, y));
            }
        }
        if let Some(target) = self.target {
            let dp = target - position();
            let w = if dp.length() < 10000. {
                1.0
            } else {
                0.12
            };
            let a = w * target.angle() + (1. - w) * position().angle();
            let d = w * target.length() + (1. - w) * position().length();
            let rmax = world_size() * 0.45 / f64::max(a.sin().abs(), a.cos().abs());
            let d = d.min(rmax);
            let t = Vec2::new(d, 0.0).rotate(a);
            draw_line(position(), t, 0xff0000);
            draw_line(t, target, 0xff0000);
            draw_line(Vec2::zero(), target, 0x00ff00);
            draw_line(Vec2::zero(), t, 0x00ff00);
            draw_line(Vec2::zero(), position(), 0x00ff00);

            let max_fwd = max_forward_acceleration();
            let fwd = (t - position()).normalize();
            let n = Vec2::new(-fwd.y, fwd.x);
            let acc_n = (-velocity().dot(n) * 100.).clamp(-max_fwd, max_fwd);
            let acc_fwd = if velocity().length() < 1000.0 {
                f64::sqrt(max_fwd.powi(2) - acc_n.powi(2)).min(50.)
            } else {
                0.0
            };
            let acc = fwd * acc_fwd + n * acc_n;
            accelerate(acc);
            turn_to(acc.angle());
        }
    }
}

/// Turns towards the given heading.
fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(10.0 * heading_error);
}
