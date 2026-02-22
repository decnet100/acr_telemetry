use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::Vector3f;

/// Represents the 3D contact points for all four tyres of a car.
/// These contact points describe where each tyre touches the track surface.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ContactPoint {
    pub front_left: Vector3f,
    pub front_right: Vector3f,
    pub rear_left: Vector3f,
    pub rear_right: Vector3f,
}

impl ContactPoint {
    /// Create a new ContactPoint structure
    pub fn new(
        front_left: Vector3f,
        front_right: Vector3f,
        rear_left: Vector3f,
        rear_right: Vector3f,
    ) -> Self {
        Self {
            front_left,
            front_right,
            rear_left,
            rear_right,
        }
    }

    /// Create a ContactPoint with all points at zero
    pub fn zero() -> Self {
        Self::new(
            Vector3f::zero(),
            Vector3f::zero(),
            Vector3f::zero(),
            Vector3f::zero(),
        )
    }

    /// Create from a nested array of contact points
    pub fn from_nested_array(points: [[f32; 3]; 4]) -> Self {
        Self::new(
            Vector3f::from(points[0]),
            Vector3f::from(points[1]),
            Vector3f::from(points[2]),
            Vector3f::from(points[3]),
        )
    }

    /// Convert to a nested array
    pub fn to_nested_array(&self) -> [[f32; 3]; 4] {
        [
            self.front_left.into(),
            self.front_right.into(),
            self.rear_left.into(),
            self.rear_right.into(),
        ]
    }
}

impl fmt::Display for ContactPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FL: {},\nFR: {},\nRL: {},\nRR: {}",
            self.front_left, self.front_right, self.rear_left, self.rear_right
        )
    }
}

impl From<[[f32; 3]; 4]> for ContactPoint {
    fn from(arr: [[f32; 3]; 4]) -> Self {
        Self::from_nested_array(arr)
    }
}