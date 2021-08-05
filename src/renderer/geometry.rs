use nalgebra::{point, Point2};

pub fn quad() -> [Point2<f32>; 4] {
    let x = 0.5;
    [point![-x, -x], point![-x, x], point![x, -x], point![x, x]]
}
