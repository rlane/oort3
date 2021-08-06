use nalgebra::{point, vector, Matrix4, Point2, Unit};

pub fn quad() -> [Point2<f32>; 4] {
    let x = 0.5;
    [point![-x, -x], point![-x, x], point![x, -x], point![x, x]]
}

pub fn line_transform(p1: Point2<f32>, p2: Point2<f32>, width: f32) -> Matrix4<f32> {
    let axis = Unit::new_normalize(vector![0.0, 0.0, 1.0]);
    let dp = p2 - p1;
    let p = p1 + dp * 0.5;
    let angle = dp.y.atan2(dp.x);
    Matrix4::from_axis_angle(&axis, angle)
        .prepend_nonuniform_scaling(&vector![dp.magnitude(), width, 1.0])
        .append_translation(&vector![p.x, p.y, 0.0])
}
