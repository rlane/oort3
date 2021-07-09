use macroquad::math::{vec2, Mat2, Vec2};
use oorandom::Rand32;

fn scale(scale: f32, vertices: &[Vec2]) -> Vec<Vec2> {
    vertices
        .iter()
        .map(|&v| vec2(scale * v.x, scale * v.y))
        .collect::<Vec<_>>()
}

pub fn ship() -> Vec<Vec2> {
    scale(10.0, &[vec2(-0.7, -0.71), vec2(1.0, 0.0), vec2(-0.7, 0.71)])
}

pub fn asteroid() -> Vec<Vec2> {
    let n = 7;
    let mut rng = Rand32::new(1);
    let mut vertices = vec![];
    for i in 0..n {
        let r = rng.rand_float();
        let m = Mat2::from_angle(i as f32 * 2.0 * std::f32::consts::PI / n as f32);
        vertices.push(m.mul_vec2(vec2(r, 0.0)));
    }
    scale(50.0, &vertices)
}
