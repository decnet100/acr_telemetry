use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents various levels of rain intensity in ACC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(i32)]
pub enum AccRainIntensity {
    /// Completely dry conditions
    NoRain = 0,
    /// Light, intermittent precipitation
    Drizzle = 1,
    /// Consistent light rain
    LightRain = 2,
    /// Moderate rain - track actively wet
    MediumRain = 3,
    /// Heavy downpour - aquaplaning risk
    HeavyRain = 4,
    /// Intense storm - extreme conditions
    Thunderstorm = 5,
}

impl AccRainIntensity {
    /// Check if conditions are wet (any rain)
    pub fn is_wet(&self) -> bool {
        !matches!(self, AccRainIntensity::NoRain)
    }

    /// Check if conditions require wet tyres
    pub fn requires_wet_tyres(&self) -> bool {
        matches!(
            self,
            AccRainIntensity::MediumRain
                | AccRainIntensity::HeavyRain
                | AccRainIntensity::Thunderstorm
        )
    }

    /// Get a subjective grip level (0.0 = no grip, 1.0 = full grip)
    pub fn grip_level(&self) -> f32 {
        match self {
            AccRainIntensity::NoRain => 1.0,
            AccRainIntensity::Drizzle => 0.9,
            AccRainIntensity::LightRain => 0.7,
            AccRainIntensity::MediumRain => 0.5,
            AccRainIntensity::HeavyRain => 0.3,
            AccRainIntensity::Thunderstorm => 0.1,
        }
    }
}

impl fmt::Display for AccRainIntensity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AccRainIntensity::NoRain => "No Rain",
            AccRainIntensity::Drizzle => "Drizzle",
            AccRainIntensity::LightRain => "Light Rain",
            AccRainIntensity::MediumRain => "Medium Rain",
            AccRainIntensity::HeavyRain => "Heavy Rain",
            AccRainIntensity::Thunderstorm => "Thunderstorm",
        };
        write!(f, "{}", name)
    }
}

impl TryFrom<i32> for AccRainIntensity {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AccRainIntensity::NoRain),
            1 => Ok(AccRainIntensity::Drizzle),
            2 => Ok(AccRainIntensity::LightRain),
            3 => Ok(AccRainIntensity::MediumRain),
            4 => Ok(AccRainIntensity::HeavyRain),
            5 => Ok(AccRainIntensity::Thunderstorm),
            _ => Err(()),
        }
    }
}