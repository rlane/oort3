use nalgebra::{point, vector, Matrix4, Point2, Unit, Vector2};

pub fn quad() -> [Point2<f32>; 4] {
    let x = 0.5;
    [point![-x, -x], point![-x, x], point![x, -x], point![x, x]]
}

pub fn unit_quad() -> [Point2<f32>; 4] {
    [
        point![0.0, 0.0],
        point![0.0, 1.0],
        point![1.0, 0.0],
        point![1.0, 1.0],
    ]
}

pub fn clip_quad() -> [Point2<f32>; 4] {
    [
        point![-1.0, -1.0],
        point![-1.0, 1.0],
        point![1.0, -1.0],
        point![1.0, 1.0],
    ]
}

pub fn triquad() -> [Point2<f32>; 6] {
    let x = 0.5;
    [
        point![-x, -x],
        point![-x, x],
        point![x, -x],
        point![-x, x],
        point![x, -x],
        point![x, x],
    ]
}

// Use gl::TRIANGLE_FAN
pub fn hexagon() -> [Point2<f32>; 8] {
    [
        point![0.0, 0.0],
        point![0.5, 0.0],
        point![0.25, 0.433],
        point![-0.25, 0.433],
        point![-0.5, 0.0],
        point![-0.25, -0.433],
        point![0.25, -0.433],
        point![0.5, 0.0],
    ]
}

pub fn line_transform(p1: Point2<f32>, p2: Point2<f32>, width: f32) -> Matrix4<f32> {
    let axis = Unit::new_normalize(vector![0.0, 0.0, 1.0]);
    let dp = p2 - p1;
    let p = p1 + dp * 0.5;
    let angle = dp.y.atan2(dp.x);
    Matrix4::from_axis_angle(&axis, angle)
        .prepend_nonuniform_scaling(&vector![dp.magnitude().max(width), width, 1.0])
        .append_translation(&vector![p.x, p.y, 0.0])
}

pub fn line_loop_mesh(vertices: &[Vector2<f32>], width: f32) -> Vec<Vector2<f32>> {
    let mut result = vec![];
    let mut prev = vertices[vertices.len() - 1];
    for &v in vertices {
        let q = (prev - v).normalize() * width * 0.5;
        let u = vector![q.y, -q.x];
        let v0 = prev + u + q;
        let v1 = prev - u + q;
        let v2 = v + u - q;
        let v3 = v - u - q;
        result.extend_from_slice(&[v0, v1, v3, v2, v3, v0]);
        prev = v;
    }
    result
}
