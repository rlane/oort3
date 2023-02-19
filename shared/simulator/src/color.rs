use nalgebra::{vector, Vector4};

pub fn to_u32(c: Vector4<f32>) -> u32 {
    let convert = |x| (x * 255.0) as u32;
    convert(c.x) << 24 | convert(c.y) << 16 | convert(c.z) << 8 | convert(c.w)
}

pub fn from_u32(c: u32) -> Vector4<f32> {
    let extract_color = |k: i64| -> f32 { (((c >> (k * 8)) & 0xff) as f32) / 255.0 };
    vector![
        extract_color(3),
        extract_color(2),
        extract_color(1),
        extract_color(0)
    ]
}

pub fn from_u24(c: u32) -> Vector4<f32> {
    let extract_color = |k: i64| -> f32 { (((c >> (k * 8)) & 0xff) as f32) / 255.0 };
    vector![extract_color(2), extract_color(1), extract_color(0), 1.0]
}
