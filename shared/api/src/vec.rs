use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A two-dimensional vector.
#[derive(Clone, Copy, Debug)]
pub struct Vec2 {
    /// The X coordinate.
    pub x: f64,
    /// The Y coordinate.
    pub y: f64,
}

impl Vec2 {
    /// Constructs a [Vec2].
    pub fn new(x: f64, y: f64) -> Vec2 {
        Vec2 { x, y }
    }

    /// Returns the length (or distance from origin).
    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Returns a normalized vector with the same direction but length of 1.
    pub fn normalize(self) -> Vec2 {
        self / self.length()
    }

    /// Returns the distance to `other`.
    pub fn distance(self, other: Vec2) -> f64 {
        (self - other).length()
    }

    /// Returns the dot product with `other`.
    pub fn dot(self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Returns the angle of the vector (in radians).
    pub fn angle(self) -> f64 {
        let mut a = self.y.atan2(self.x);
        if a < 0.0 {
            a += std::f64::consts::TAU;
        }
        a
    }

    /// Returns this vector rotated by the given angle (in radians).
    pub fn rotate(self, angle: f64) -> Vec2 {
        let cos = angle.cos();
        let sin = angle.sin();
        Vec2 {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Mul<Vec2> for f64 {
    type Output = Vec2;

    fn mul(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self * other.x,
            y: self * other.y,
        }
    }
}

impl MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, other: f64) {
        *self = *self * other;
    }
}

impl Div<f64> for Vec2 {
    type Output = Self;

    fn div(self, other: f64) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl DivAssign<f64> for Vec2 {
    fn div_assign(&mut self, other: f64) {
        *self = *self / other;
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

/// Returns a [Vec2] with the given coordinates.
pub fn vec2(x: f64, y: f64) -> Vec2 {
    Vec2::new(x, y)
}
