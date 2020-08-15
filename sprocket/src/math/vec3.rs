use super::vec2::Vec2;
use std::ops;
/// Representation of 3D vectors and points
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    /// Creates a vector given x,y,z
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    pub fn from_vec3(xy: Vec2, z: f32) -> Self {
        Vec3 {
            x: xy.x,
            y: xy.y,
            z,
        }
    }

    /// Creates a vector with all components being zero
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Creates a vector with all components being one
    pub fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }

    /// Creates a vector with z = 1
    pub fn forward() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    }

    /// Creates a vector with x = 1
    pub fn right() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Creates a vector with y = 1
    pub fn up() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    /// Returns the dot product of two vectors
    /// The dot product is the cosine of the angle between a, b
    /// Example outputs assuming a, b are normalized
    /// 1: vectors are pointing in the same direction
    /// 0: vectors are perpendicular
    /// -1: vectors are opposite
    pub fn dot(a: &Self, b: &Self) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    /// Returns the cross product of two vectors, I.e; a vector perdencicular to both input vectors
    pub fn cross(a: &Self, b: &Self) -> Self {
        Self {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
        }
    }

    /// Reflects a vector about a normal
    pub fn reflect(ray: Self, normal: Self) -> Self {
        let n = normal.norm();
        ray - n * Self::dot(&ray, &n) * 2.0
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
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Returns the squared length of the vector
    /// Is faster than mag due to not using sqrt
    pub fn sqrmag(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns the normal version the vector
    pub fn norm(&self) -> Vec3 {
        *self / self.mag()
    }

    /// Returns the smallest component
    pub fn smallest(&self) -> f32 {
        if self.x < self.y {
            if self.x < self.z {
                return self.x;
            }
            return self.z;
        } else if self.y < self.x {
            if self.y < self.z {
                return self.y;
            }
            return self.z;
        } else if self.z < self.x {
            if self.z < self.y {
                return self.z;
            }
            return self.y;
        }
        // All are equal
        self.x
    }

    /// Returns the largest component
    pub fn largest(&self) -> f32 {
        if self.x > self.y {
            if self.x > self.z {
                return self.x;
            }
            return self.z;
        } else if self.y > self.x {
            if self.y > self.z {
                return self.y;
            }
            return self.z;
        } else if self.z > self.x {
            if self.z > self.y {
                return self.z;
            }
            return self.y;
        }
        // All are equal
        self.x
    }

    pub fn xy(&self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}

// Traits
impl Clone for Vec3 {
    fn clone(&self) -> Self {
        Vec3 { ..(*self) }
    }
}

impl Copy for Vec3 {}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl std::fmt::Debug for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

// Math operators

/// Adds two vectors component wise
impl ops::Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

/// Compound addition to vector component wise
impl ops::AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

/// Subtracts two vectors component wise
impl ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

/// Compound subtraction to vector component wise
impl ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

/// Multiplies two vectors component wise
impl ops::Mul for Vec3 {
    type Output = Vec3;
    fn mul(self, other: Self) -> Self {
        Vec3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

// Compound multiplies two vectors component wise
impl ops::MulAssign for Vec3 {
    fn mul_assign(&mut self, other: Self) {
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
    }
}

/// Negates the vector
impl ops::Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

/// Multiplies the length of the vector by rhs
impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

/// Compound multiplies the length of the vector
impl ops::MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

/// Divides the length of the vector by rhs
impl ops::Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f32) -> Self {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

/// Compound divides the length of the vector by rhs
impl ops::DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    fn from(t: (f32, f32, f32)) -> Self {
        Vec3 {
            x: t.0,
            y: t.1,
            z: t.2,
        }
    }
}

impl From<(i32, i32, i32)> for Vec3 {
    fn from(t: (i32, i32, i32)) -> Self {
        Vec3 {
            x: t.0 as f32,
            y: t.1 as f32,
            z: t.2 as f32,
        }
    }
}
