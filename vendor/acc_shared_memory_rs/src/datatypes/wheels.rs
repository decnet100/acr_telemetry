use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Generic structure representing per-wheel values for a four-wheel vehicle.
/// Used for tyre pressures, temperatures, slip, brake temperatures, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Wheels {
    pub front_left: f32,
    pub front_right: f32,
    pub rear_left: f32,
    pub rear_right: f32,
}

impl Wheels {
    /// Create a new Wheels structure
    pub fn new(front_left: f32, front_right: f32, rear_left: f32, rear_right: f32) -> Self {
        Self {
            front_left,
            front_right,
            rear_left,
            rear_right,
        }
    }

    /// Create a Wheels structure with all values set to zero
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Get the average value across all wheels
    pub fn average(&self) -> f32 {
        (self.front_left + self.front_right + self.rear_left + self.rear_right) / 4.0
    }

    /// Get the front average
    pub fn front_average(&self) -> f32 {
        (self.front_left + self.front_right) / 2.0
    }

    /// Get the rear average
    pub fn rear_average(&self) -> f32 {
        (self.rear_left + self.rear_right) / 2.0
    }

    /// Get the left average
    pub fn left_average(&self) -> f32 {
        (self.front_left + self.rear_left) / 2.0
    }

    /// Get the right average
    pub fn right_average(&self) -> f32 {
        (self.front_right + self.rear_right) / 2.0
    }
}

impl fmt::Display for Wheels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FL: {}\nFR: {}\nRL: {}\nRR: {}",
            self.front_left, self.front_right, self.rear_left, self.rear_right
        )
    }
}

impl From<[f32; 4]> for Wheels {
    fn from(arr: [f32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

impl From<Wheels> for [f32; 4] {
    fn from(w: Wheels) -> Self {
        [w.front_left, w.front_right, w.rear_left, w.rear_right]
    }
}