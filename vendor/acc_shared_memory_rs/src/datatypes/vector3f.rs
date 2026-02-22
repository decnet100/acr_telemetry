use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A 3D vector structure for representing coordinates or directional vectors in 3D space.
/// Used extensively in ACC shared memory for positions, velocities, and forces.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3f {
    /// Create a new Vector3f
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Create a zero vector
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Calculate the magnitude of the vector
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Calculate the dot product with another vector
    pub fn dot(&self, other: &Vector3f) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl fmt::Display for Vector3f {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "x: {}, y: {}, z: {}", self.x, self.y, self.z)
    }
}

impl From<[f32; 3]> for Vector3f {
    fn from(arr: [f32; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }
}

impl From<Vector3f> for [f32; 3] {
    fn from(v: Vector3f) -> Self {
        [v.x, v.y, v.z]
    }
}