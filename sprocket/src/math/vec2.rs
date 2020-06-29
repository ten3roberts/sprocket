use std::ops;
/// Representation of 3D vectors and points
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// Creates a vector given x,y,z
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    /// Creates a vector with all components being zero
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Creates a vector with all components being one
    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    /// Creates a vector with x = 1
    pub fn right() -> Self {
        Self { x: 1.0, y: 0.0 }
    }

    /// Creates a vector with y = 1
    pub fn up() -> Self {
        Self { x: 0.0, y: 1.0 }
    }

    /// Returns the dot product of two vectors
    /// The dot product is the cosine of the angle between a, b
    /// Example outputs assuming a, b are normalized
    /// 1: vectors are pointing in the same direction
    /// 0: vectors are perpendicular
    /// -1: vectors are opposite
    pub fn dot(a: &Self, b: &Self) -> f32 {
        a.x * b.x + a.y * b.y
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
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Returns the squared length of the vector
    /// Is faster than mag due to not using sqrt
    pub fn sqrmag(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Returns the normal version the vector
    pub fn norm(&self) -> Vec2 {
        *self / self.mag()
    }
}

// Traits
impl Clone for Vec2 {
    fn clone(&self) -> Self {
        Vec2 { ..(*self) }
    }
}

impl Copy for Vec2 {}

impl std::fmt::Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// Math operators

/// Adds two vectors component wise
impl ops::Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

/// Compound addition to vector component wise
impl ops::AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

/// Subtracts two vectors component wise
impl ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

/// Compound subtraction to vector component wise
impl ops::SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

/// Multiplies two vectors component wise
impl ops::Mul for Vec2 {
    type Output = Vec2;
    fn mul(self, other: Self) -> Self {
        Vec2 {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

// Compound multiplies two vectors component wise
impl ops::MulAssign for Vec2 {
    fn mul_assign(&mut self, other: Self) {
        self.x *= other.x;
        self.y *= other.y;
    }
}

/// Negates the vector
impl ops::Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

/// Multiplies the length of the vector by rhs
impl ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

/// Compound multiplies the length of the vector
impl ops::MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

/// Divides the length of the vector by rhs
impl ops::Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Self {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

/// Compound divides the length of the vector by rhs
impl ops::DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
