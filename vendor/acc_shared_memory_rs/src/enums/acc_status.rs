use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents the current operational state of ACC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(i32)]
pub enum AccStatus {
    /// ACC is not running or shared memory is inactive
    Off = 0,
    /// The game is playing back a replay
    Replay = 1,
    /// Live gameplay is active
    Live = 2,
    /// The simulation is paused
    Pause = 3,
}

impl AccStatus {
    /// Check if ACC is actively running (Live or Replay)
    pub fn is_active(&self) -> bool {
        matches!(self, AccStatus::Live | AccStatus::Replay)
    }

    /// Check if this is live gameplay
    pub fn is_live(&self) -> bool {
        matches!(self, AccStatus::Live)
    }
}

impl fmt::Display for AccStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AccStatus::Off => "Off",
            AccStatus::Replay => "Replay",
            AccStatus::Live => "Live",
            AccStatus::Pause => "Pause",
        };
        write!(f, "{}", name)
    }
}

impl TryFrom<i32> for AccStatus {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AccStatus::Off),
            1 => Ok(AccStatus::Replay),
            2 => Ok(AccStatus::Live),
            3 => Ok(AccStatus::Pause),
            _ => Err(()),
        }
    }
}