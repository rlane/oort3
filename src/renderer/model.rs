use nalgebra::{vector, Rotation2, Vector2};
use oorandom::Rand32;

fn scale(scale: f32, vertices: &[Vector2<f32>]) -> Vec<Vector2<f32>> {
    vertices
        .iter()
        .map(|&v| vector![scale * v.x, scale * v.y])
        .collect::<Vec<_>>()
}

pub fn ship() -> Vec<Vector2<f32>> {
    scale(
        10.0,
        &[vector![-0.7, -0.71], vector![1.0, 0.0], vector![-0.7, 0.71]],
    )
}

pub fn asteroid() -> Vec<Vector2<f32>> {
    let n = 7;
    let mut rng = Rand32::new(1);
    let mut vertices = vec![];
    for i in 0..n {
        let r = rng.rand_float();
        let rotation = Rotation2::new(i as f32 * 2.0 * std::f32::consts::PI / n as f32);
        vertices.push(rotation.transform_vector(&vector![r, 0.0]));
    }
    scale(50.0, &vertices)
}
