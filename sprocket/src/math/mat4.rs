use super::Vec3;
use std::ops;

pub struct Mat4([f32; 16]);

impl Mat4 {
    pub fn zero() -> Self {
        Mat4([0.0; 16])
    }

    pub fn one() -> Self {
        Mat4([1.0; 16])
    }

    pub fn identity() -> Self {
        Mat4([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn transpose(&self) -> Self {
        Mat4([
            self.0[0], self.0[4], self.0[8], self.0[12], self.0[1], self.0[5], self.0[9],
            self.0[13], self.0[2], self.0[6], self.0[10], self.0[14], self.0[3], self.0[7],
            self.0[11], self.0[15],
        ])
    }

    pub fn perspective(aspect: f32, fov: f32, near: f32, far: f32) -> Self {
        let tf = 1.0 / (fov * 0.5).tan();

        Mat4([
            tf / aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            tf,
            0.0,
            0.0,
            0.0,
            0.0,
            far / (near - far),
            -1.0,
            0.0,
            0.0,
            (near * far) / (near - far),
            0.0,
        ])
    }

    pub fn ortho(width: f32, height: f32, near: f32, far: f32) -> Self {
        Mat4([
            2.0 / width,
            0.0,
            0.0,
            0.0,
            0.0,
            2.0 / height,
            0.0,
            0.0,
            0.0,
            0.0,
            -1.0 / (far - near),
            0.0,
            0.0,
            0.0,
            near / (near - far),
            1.0,
        ])
    }

    pub fn translate(v: Vec3) -> Self {
        Mat4([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, v.x, v.y, v.z, 1.0,
        ])
    }

    pub fn scale(scale: Vec3) -> Self {
        Mat4([
            scale.x, 0.0, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, 0.0, scale.z, 0.0, 0.0, 0.0, 0.0,
            1.0,
        ])
    }

    pub fn rotate_x(angle: f32) -> Self {
        let cosa = angle.cos();
        let sina = angle.sin();

        Mat4([
            1.0, 0.0, 0.0, 0.0, 0.0, cosa, -sina, 0.0, 0.0, sina, cosa, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn rotate_z(angle: f32) -> Self {
        let cosa = angle.cos();
        let sina = angle.sin();

        Mat4([
            cosa, -sina, 0.0, 0.0, sina, cosa, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn rotate_y(angle: f32) -> Self {
        let cosa = angle.cos();
        let sina = angle.sin();

        Mat4([
            cosa, 0.0, sina, 0.0, 0.0, 1.0, 0.0, 0.0, -sina, 0.0, cosa, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }
}

impl ops::Mul for Mat4 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let mut result = Mat4::zero();
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result.0[i * 4 + k] += self.0[i * 4 + j] * rhs.0[j * 4 + k];
                }
            }
        }
        result
    }
}

impl std::fmt::Display for Mat4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n{:?}\n{:?}\n{:?}\n{:?}",
            &self.0[0..4],
            &self.0[4..8],
            &self.0[8..12],
            &self.0[12..]
        )
    }
}
