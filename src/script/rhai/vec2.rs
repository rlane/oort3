use nalgebra::Rotation2;
use rhai::plugin::*;

pub type Vec2 = nalgebra::Vector2<f64>;

#[export_module]
pub mod plugin {
    #[rhai_fn(name = "vec2")]
    pub fn vec2ff(x: f64, y: f64) -> Vec2 {
        Vec2::new(x, y)
    }

    #[rhai_fn(name = "vec2")]
    pub fn vec2ii(x: i64, y: i64) -> Vec2 {
        Vec2::new(x as f64, y as f64)
    }

    #[rhai_fn(name = "vec2")]
    pub fn vec2if(x: i64, y: f64) -> Vec2 {
        Vec2::new(x as f64, y)
    }

    #[rhai_fn(name = "vec2")]
    pub fn vec2fi(x: f64, y: i64) -> Vec2 {
        Vec2::new(x, y as f64)
    }

    #[rhai_fn(name = "+")]
    pub fn add(obj: &mut Vec2, other: Vec2) -> Vec2 {
        *obj + other
    }

    #[rhai_fn(name = "-")]
    pub fn sub(obj: &mut Vec2, other: Vec2) -> Vec2 {
        *obj - other
    }

    #[rhai_fn(name = "-")]
    pub fn negate(obj: &mut Vec2) -> Vec2 {
        -*obj
    }

    #[rhai_fn(name = "*")]
    pub fn scalef(obj: &mut Vec2, other: f64) -> Vec2 {
        *obj * other
    }

    #[rhai_fn(name = "*")]
    pub fn scalei(obj: &mut Vec2, other: i64) -> Vec2 {
        *obj * other as f64
    }

    #[rhai_fn(name = "*")]
    pub fn scale2f(obj: &mut f64, other: Vec2) -> Vec2 {
        *obj * other
    }

    #[rhai_fn(name = "*")]
    pub fn scale2i(obj: &mut i64, other: Vec2) -> Vec2 {
        *obj as f64 * other
    }

    #[rhai_fn(name = "/")]
    pub fn divf(obj: &mut Vec2, other: f64) -> Vec2 {
        *obj / other
    }

    #[rhai_fn(name = "/")]
    pub fn divi(obj: &mut Vec2, other: i64) -> Vec2 {
        *obj / other as f64
    }

    #[rhai_fn(get = "x", pure)]
    pub fn get_x(obj: &mut Vec2) -> f64 {
        obj.x
    }

    #[rhai_fn(set = "x")]
    pub fn set_xf(obj: &mut Vec2, value: f64) {
        obj.x = value;
    }

    #[rhai_fn(set = "x")]
    pub fn set_xi(obj: &mut Vec2, value: i64) {
        obj.x = value as f64;
    }

    #[rhai_fn(get = "y", pure)]
    pub fn get_y(obj: &mut Vec2) -> f64 {
        obj.y
    }

    #[rhai_fn(set = "y")]
    pub fn set_yf(obj: &mut Vec2, value: f64) {
        obj.y = value;
    }

    #[rhai_fn(set = "y")]
    pub fn set_yi(obj: &mut Vec2, value: i64) {
        obj.y = value as f64;
    }

    #[rhai_fn(name = "magnitude")]
    pub fn magnitude(obj: &mut Vec2) -> f64 {
        obj.magnitude()
    }

    #[rhai_fn(name = "dot")]
    pub fn dot(obj: &mut Vec2, other: Vec2) -> f64 {
        obj.dot(&other)
    }

    #[rhai_fn(name = "distance")]
    pub fn distance(obj: &mut Vec2, other: Vec2) -> f64 {
        obj.metric_distance(&other)
    }

    #[rhai_fn(name = "angle")]
    pub fn angle(obj: &mut Vec2) -> f64 {
        let mut a = obj.y.atan2(obj.x);
        if a < 0.0 {
            a += std::f64::consts::TAU;
        }
        a
    }

    #[rhai_fn(name = "normalize")]
    pub fn normalize(obj: &mut Vec2) -> Vec2 {
        obj.normalize()
    }

    #[rhai_fn(name = "rotate")]
    pub fn rotatef(obj: &mut Vec2, angle: f64) -> Vec2 {
        Rotation2::new(angle).transform_vector(obj)
    }

    #[rhai_fn(name = "rotate")]
    pub fn rotatei(obj: &mut Vec2, angle: i64) -> Vec2 {
        rotatef(obj, angle as f64)
    }

    #[rhai_fn(name = "to_string")]
    pub fn to_string(obj: &mut Vec2) -> String {
        format!("({:.2}, {:.2})", obj.x, obj.y)
    }

    #[rhai_fn(name = "to_debug")]
    pub fn to_debug(obj: &mut Vec2) -> String {
        format!("({}, {})", obj.x, obj.y)
    }
}
