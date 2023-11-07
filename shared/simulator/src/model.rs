use crate::ship::ShipClass;
use nalgebra::{vector, Rotation2, Vector2};
use oorandom::Rand32;

pub fn scale(scale: f32, vertices: &[Vector2<f32>]) -> Vec<Vector2<f32>> {
    vertices
        .iter()
        .map(|&v| vector![scale * v.x, scale * v.y])
        .collect::<Vec<_>>()
}

pub fn offset(offset: Vector2<f32>, vertices: &[Vector2<f32>]) -> Vec<Vector2<f32>> {
    vertices
        .iter()
        .map(|&v| vector![offset.x + v.x, offset.y + v.y])
        .collect::<Vec<_>>()
}

pub fn fighter() -> Vec<Vector2<f32>> {
    offset(
        vector![1.3333334, 0.0],
        &scale(
            10.0,
            &[vector![-0.7, -0.71], vector![1.0, 0.0], vector![-0.7, 0.71]],
        ),
    )
}

pub fn frigate() -> Vec<Vector2<f32>> {
    offset(
        vector![0.76033056, 0.0],
        &scale(
            60.0,
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
        ),
    )
}

pub fn cruiser() -> Vec<Vector2<f32>> {
    offset(
        vector![-12.350365, 0.0],
        &scale(
            120.0,
            &[
                // back left
                vector![-0.8, -0.3],
                // back right
                vector![-0.8, 0.3],
                // right missile battery
                vector![-0.5, 0.3],
                vector![-0.5, 0.4],
                vector![0.5, 0.4],
                vector![0.5, 0.3],
                // front right
                vector![0.8, 0.3],
                // front
                vector![1.1, 0.2],
                vector![1.1, -0.2],
                // front left
                vector![0.8, -0.3],
                // left missile battery
                vector![0.5, -0.3],
                vector![0.5, -0.4],
                vector![-0.5, -0.4],
                vector![-0.5, -0.3],
            ],
        ),
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

pub fn big_asteroid(variant: i32) -> Vec<Vector2<f32>> {
    let n = 17;
    let mut rng = Rand32::new(variant as u64 ^ 4983247321098);
    let mut vertices = vec![];
    for i in 0..n {
        let r = rng.rand_float() * 0.5 + 0.5;
        let rotation = Rotation2::new(i as f32 * 2.0 * std::f32::consts::PI / n as f32);
        vertices.push(rotation.transform_vector(&vector![r, 0.0]));
    }
    scale(500.0, &vertices)
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
    offset(
        vector![0.4, 0.0],
        &scale(
            3.0,
            &[
                vector![-0.7, -0.71],
                vector![0.0, 0.0],
                vector![1.0, 0.0],
                vector![0.0, 0.0],
                vector![-0.7, 0.71],
                vector![0.0, 0.0],
            ],
        ),
    )
}

pub fn torpedo() -> Vec<Vector2<f32>> {
    offset(
        vector![-0.61714286, 0.0],
        &scale(
            8.0,
            &[
                // back left
                vector![-0.8, -0.2],
                // back right
                vector![-0.8, 0.2],
                // front right
                vector![0.8, 0.2],
                // front
                vector![1.1, 0.0],
                // front left
                vector![0.8, -0.2],
            ],
        ),
    )
}

pub fn planet() -> Vec<Vector2<f32>> {
    let n = 100;
    let mut vertices = vec![];
    for i in 0..n {
        let rotation = Rotation2::new(i as f32 * 2.0 * std::f32::consts::PI / n as f32);
        vertices.push(rotation.transform_vector(&vector![1.0, 0.0]));
    }
    scale(2000.0, &vertices)
}

pub fn big_planet() -> Vec<Vector2<f32>> {
    let n = 100;
    let mut vertices = vec![];
    for i in 0..n {
        let rotation = Rotation2::new(i as f32 * 2.0 * std::f32::consts::PI / n as f32);
        vertices.push(rotation.transform_vector(&vector![1.0, 0.0]));
    }
    scale(10000.0, &vertices)
}

pub fn load(class: ShipClass) -> Vec<Vector2<f32>> {
    match class {
        ShipClass::Fighter => fighter(),
        ShipClass::Frigate => frigate(),
        ShipClass::Cruiser => cruiser(),
        ShipClass::Asteroid { variant } => asteroid(variant),
        ShipClass::BigAsteroid { variant } => big_asteroid(variant),
        ShipClass::Target => target(),
        ShipClass::Missile => missile(),
        ShipClass::Torpedo => torpedo(),
        ShipClass::Planet => big_planet(),
    }
}

pub fn radius(class: ShipClass) -> f32 {
    load(class)
        .iter()
        .map(|&v| v.norm())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
}
