/// A two-dimensional vector.
pub type Vec2 = maths_rs::vec::Vec2<f64>;

/// Returns a [Vec2] with the given coordinates.
pub fn vec2(x: f64, y: f64) -> Vec2 {
    Vec2::new(x, y)
}

/// Extra methods for Vec2.
pub trait Vec2Extras {
    /// Returns the length (or distance from origin).
    fn length(self) -> f64;

    /// Returns a normalized vector with the same direction but length of 1.
    fn normalize(self) -> Vec2;

    /// Returns the distance to `other`.
    fn distance(self, other: Vec2) -> f64;

    /// Returns the dot product with `other`.
    fn dot(self, other: Vec2) -> f64;

    /// Returns the angle of the vector (in radians).
    fn angle(self) -> f64;

    /// Returns this vector rotated by the given angle (in radians).
    fn rotate(self, angle: f64) -> Vec2;
}

impl Vec2Extras for Vec2 {
    fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalize(self) -> Vec2 {
        self / self.length()
    }

    fn distance(self, other: Vec2) -> f64 {
        (self - other).length()
    }

    fn dot(self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    fn angle(self) -> f64 {
        let mut a = self.y.atan2(self.x);
        if a < 0.0 {
            a += std::f64::consts::TAU;
        }
        a
    }

    fn rotate(self, angle: f64) -> Vec2 {
        let cos = angle.cos();
        let sin = angle.sin();
        Vec2 {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }
}
