use super::vec3::Vec3;
use std::ops;
/// Representation of 3D vectors and points
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    /// Creates a vector given x,y,z
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vec4 { x, y, z, w }
    }

    pub fn from_vec3(xyz: Vec3, w: f32) -> Self {
        Vec4 {
            x: xyz.x,
            y: xyz.y,
            z: xyz.z,
            w,
        }
    }

    /// Creates a vector with all components being zero
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }
    }

    /// Creates a vector with all components being one
    pub fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
            w: 1.0,
        }
    }

    /// Returns the dot product of two vectors
    /// The dot product is the cosine of the angle between a, b
    /// Example outputs assuming a, b are normalized
    /// 1: vectors are pointing in the same direction
    /// 0: vectors are perpendicular
    /// -1: vectors are opposite
    pub fn dot(a: &Self, b: &Self) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
    }

    /// Project a vector onto another
    pub fn project(a: Self, b: Self) -> Self {
        let b = b.norm();
        b * Self::dot(&a, &b)
    }

    /// Projects a vector onto a plane
    pub fn project_plane(a: Self, normal: Self) -> Self {
        let normal = normal.norm();
        a - normal * Self::dot(&a, &normal)
    }

    /// Linearly interpolates between two vectors with t
    /// When t > 1, lerp(a, b) = b
    /// When t = 0, lerp(a, b) = t
    /// Clamps t between 0, 1
    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        let t = if t < 0.0 {
            0.0
        } else if t > 1.0 {
            1.0
        } else {
            t
        };
        a * (1.0 - t) + b * t
    }

    /// Linearly interpolates between two vectors with t
    /// When t = 1, lerp(a, b) = b
    /// When t = 0, lerp(a, b) = t
    /// Does not clamp t between 0, 1
    pub fn lerp_unclamped(a: Self, b: Self, t: f32) -> Self {
        a * (1.0 - t) + b * t
    }

    // Instance method

    /// Returns the magnitude/length of the vector
    /// Note: for comparing two vectors, sqrmag is faster
    pub fn mag(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    /// Returns the squared length of the vector
    /// Is faster than mag due to not using sqrt
    pub fn sqrmag(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// Returns the normal version the vector
    pub fn norm(&self) -> Vec4 {
        *self / self.mag()
    }

    pub fn xyz(&self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

// Traits
impl Clone for Vec4 {
    fn clone(&self) -> Self {
        Vec4 { ..(*self) }
    }
}

impl Copy for Vec4 {}

impl std::fmt::Display for Vec4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}

// Math operators

/// Adds two vectors component wise
impl ops::Add for Vec4 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

/// Compound addition to vector component wise
impl ops::AddAssign for Vec4 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
        self.w += other.w;
    }
}

/// Subtracts two vectors component wise
impl ops::Sub for Vec4 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

/// Compound subtraction to vector component wise
impl ops::SubAssign for Vec4 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
        self.w -= other.w;
    }
}

/// Multiplies two vectors component wise
impl ops::Mul for Vec4 {
    type Output = Vec4;
    fn mul(self, other: Self) -> Self {
        Vec4 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
            w: self.w * other.w,
        }
    }
}

// Compound multiplies two vectors component wise
impl ops::MulAssign for Vec4 {
    fn mul_assign(&mut self, other: Self) {
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
        self.w *= other.w;
    }
}

/// Negates the vector
impl ops::Neg for Vec4 {
    type Output = Vec4;
    fn neg(self) -> Vec4 {
        Vec4 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

/// Multiplies the length of the vector by rhs
impl ops::Mul<f32> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: f32) -> Vec4 {
        Vec4 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

/// Compound multiplies the length of the vector
impl ops::MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

/// Divides the length of the vector by rhs
impl ops::Div<f32> for Vec4 {
    type Output = Vec4;
    fn div(self, rhs: f32) -> Self {
        Vec4 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}

/// Compound divides the length of the vector by rhs
impl ops::DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
        self.w /= rhs;
    }
}

impl From<(f32, f32, f32, f32)> for Vec4 {
    fn from(t: (f32, f32, f32, f32)) -> Self {
        Vec4 {
            x: t.0,
            y: t.1,
            z: t.2,
            w: t.3,
        }
    }
}

impl From<(i32, i32, i32, i32)> for Vec4 {
    fn from(t: (i32, i32, i32, i32)) -> Self {
        Vec4 {
            x: t.0 as f32,
            y: t.1 as f32,
            z: t.2 as f32,
            w: t.3 as f32,
        }
    }
}
