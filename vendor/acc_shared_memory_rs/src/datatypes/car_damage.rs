use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents the damage state of different parts of the car.
/// Maps to the carDamage[5] array in ACC shared memory.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CarDamage {
    pub front: f32,
    pub rear: f32,
    pub left: f32,
    pub right: f32,
    pub center: f32,
}

impl CarDamage {
    /// Create a new CarDamage structure
    pub fn new(front: f32, rear: f32, left: f32, right: f32, center: f32) -> Self {
        Self {
            front,
            rear,
            left,
            right,
            center,
        }
    }

    /// Create a CarDamage with no damage
    pub fn none() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0.0)
    }

    /// Get the total damage (sum of all parts)
    pub fn total_damage(&self) -> f32 {
        self.front + self.rear + self.left + self.right + self.center
    }

    /// Check if the car has any damage
    pub fn has_damage(&self) -> bool {
        self.total_damage() > 0.0
    }

    /// Get the maximum damage value
    pub fn max_damage(&self) -> f32 {
        self.front
            .max(self.rear)
            .max(self.left)
            .max(self.right)
            .max(self.center)
    }
}

impl fmt::Display for CarDamage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Front: {}, Rear: {}, Left: {}, Right: {}, Center: {}",
            self.front, self.rear, self.left, self.right, self.center
        )
    }
}

impl From<[f32; 5]> for CarDamage {
    fn from(arr: [f32; 5]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3], arr[4])
    }
}

impl From<CarDamage> for [f32; 5] {
    fn from(damage: CarDamage) -> Self {
        [
            damage.front,
            damage.rear,
            damage.left,
            damage.right,
            damage.center,
        ]
    }
}