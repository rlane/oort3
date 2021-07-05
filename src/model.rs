use macroquad::math::{vec2, Vec2};

fn scale(scale: f32, vertices: &[Vec2]) -> Vec<Vec2> {
    vertices
        .iter()
        .map(|&v| vec2(scale * v.x, scale * v.y))
        .collect::<Vec<_>>()
}

pub fn ship() -> Vec<Vec2> {
    scale(10.0, &[vec2(-0.7, -0.71), vec2(1.0, 0.0), vec2(-0.7, 0.71)])
}
