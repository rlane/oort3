use crate::ship::ShipClass;
use nalgebra::{vector, Rotation2, Vector2};
use oorandom::Rand32;

fn scale(scale: f32, vertices: &[Vector2<f32>]) -> Vec<Vector2<f32>> {
    vertices
        .iter()
        .map(|&v| vector![scale * v.x, scale * v.y])
        .collect::<Vec<_>>()
}

pub fn fighter() -> Vec<Vector2<f32>> {
    scale(
        10.0,
        &[vector![-0.7, -0.71], vector![1.0, 0.0], vector![-0.7, 0.71]],
    )
}

pub fn frigate() -> Vec<Vector2<f32>> {
    scale(
        30.0,
        &[
            vector![-0.8, -0.4],
            vector![-0.8, 0.4],
            vector![0.0, 0.2],
            vector![0.0, 0.4],
            vector![0.95, 0.2],
            vector![0.95, -0.2],
            vector![0.0, -0.4],
            vector![0.0, -0.2],
        ],
    )
}

pub fn cruiser() -> Vec<Vector2<f32>> {
    scale(
        40.0,
        &[
            // back left
            vector![-0.8, -0.4],
            // back right
            vector![-0.8, 0.4],
            // right missile battery
            vector![-0.5, 0.4],
            vector![-0.5, 0.5],
            vector![0.5, 0.5],
            vector![0.5, 0.4],
            // front right
            vector![0.8, 0.4],
            // front
            vector![1.1, 0.2],
            vector![1.1, -0.2],
            // front left
            vector![0.8, -0.4],
            // left missile battery
            vector![0.5, -0.4],
            vector![0.5, -0.5],
            vector![-0.5, -0.5],
            vector![-0.5, -0.4],
        ],
    )
}

pub fn asteroid(variant: i32) -> Vec<Vector2<f32>> {
    let n = 7;
    let mut rng = Rand32::new(variant as u64);
    let mut vertices = vec![];
    for i in 0..n {
        let r = rng.rand_float();
        let rotation = Rotation2::new(i as f32 * 2.0 * std::f32::consts::PI / n as f32);
        vertices.push(rotation.transform_vector(&vector![r, 0.0]));
    }
    scale(50.0, &vertices)
}

pub fn target() -> Vec<Vector2<f32>> {
    let n = 20;
    let mut vertices = vec![];
    for i in 0..n {
        let rotation = Rotation2::new(i as f32 * 2.0 * std::f32::consts::PI / n as f32);
        vertices.push(rotation.transform_vector(&vector![1.0, 0.0]));
    }
    scale(10.0, &vertices)
}

pub fn missile() -> Vec<Vector2<f32>> {
    scale(
        3.0,
        &[
            vector![-0.7, -0.71],
            vector![0.0, 0.0],
            vector![1.0, 0.0],
            vector![0.0, 0.0],
            vector![-0.7, 0.71],
            vector![0.0, 0.0],
        ],
    )
}

pub fn load(class: ShipClass) -> Vec<Vector2<f32>> {
    match class {
        ShipClass::Fighter => fighter(),
        ShipClass::Frigate => frigate(),
        ShipClass::Cruiser => cruiser(),
        ShipClass::Asteroid { variant } => asteroid(variant),
        ShipClass::Target => target(),
        ShipClass::Missile => missile(),
    }
}
