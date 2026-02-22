use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents the current grip condition of the track surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(i32)]
pub enum AccTrackGripStatus {
    /// New or freshly washed track with very low grip
    Green = 0,
    /// Rubber is beginning to build up, grip improving
    Fast = 1,
    /// Ideal track condition, maximum grip
    Optimum = 2,
    /// Track is slick due to light rain, oil, or humidity
    Greasy = 3,
    /// Track has residual moisture
    Damp = 4,
    /// Actively wet surface
    Wet = 5,
    /// Track is waterlogged or heavily flooded
    Flooded = 6,
}

impl AccTrackGripStatus {
    /// Check if track conditions are wet
    pub fn is_wet(&self) -> bool {
        matches!(
            self,
            AccTrackGripStatus::Damp | AccTrackGripStatus::Wet | AccTrackGripStatus::Flooded
        )
    }

    /// Check if conditions are slippery
    pub fn is_slippery(&self) -> bool {
        matches!(
            self,
            AccTrackGripStatus::Green
                | AccTrackGripStatus::Greasy
                | AccTrackGripStatus::Damp
                | AccTrackGripStatus::Wet
                | AccTrackGripStatus::Flooded
        )
    }

    /// Get relative grip level (0.0 = no grip, 1.0 = maximum grip)
    pub fn grip_level(&self) -> f32 {
        match self {
            AccTrackGripStatus::Green => 0.6,
            AccTrackGripStatus::Fast => 0.8,
            AccTrackGripStatus::Optimum => 1.0,
            AccTrackGripStatus::Greasy => 0.7,
            AccTrackGripStatus::Damp => 0.5,
            AccTrackGripStatus::Wet => 0.3,
            AccTrackGripStatus::Flooded => 0.1,
        }
    }
}

impl fmt::Display for AccTrackGripStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AccTrackGripStatus::Green => "Green",
            AccTrackGripStatus::Fast => "Fast",
            AccTrackGripStatus::Optimum => "Optimum",
            AccTrackGripStatus::Greasy => "Greasy",
            AccTrackGripStatus::Damp => "Damp",
            AccTrackGripStatus::Wet => "Wet",
            AccTrackGripStatus::Flooded => "Flooded",
        };
        write!(f, "{}", name)
    }
}

impl TryFrom<i32> for AccTrackGripStatus {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AccTrackGripStatus::Green),
            1 => Ok(AccTrackGripStatus::Fast),
            2 => Ok(AccTrackGripStatus::Optimum),
            3 => Ok(AccTrackGripStatus::Greasy),
            4 => Ok(AccTrackGripStatus::Damp),
            5 => Ok(AccTrackGripStatus::Wet),
            6 => Ok(AccTrackGripStatus::Flooded),
            _ => Err(()),
        }
    }
}