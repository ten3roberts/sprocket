use crate::math::*;

/// A component representing a the position, rotation, and scale of an entity
pub struct Transform {
    pub position: Vec3,
    // rotation: Quaternion,
    // scale: Vec3,
}

impl Transform {
    pub fn new(position: Vec3) -> Self {
        Transform { position }
    }

    /// Creates a new worldmatrix from the contained position, rotation, and scale
    pub fn create_worldmatrix(&self) -> Mat4 {
        Mat4::translate(self.position)
    }
}
