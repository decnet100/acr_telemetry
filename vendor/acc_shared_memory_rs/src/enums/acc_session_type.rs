use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents the different types of driving sessions in ACC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(i32)]
pub enum AccSessionType {
    Unknown = -1,
    Practice = 0,
    Qualifying = 1,
    Race = 2,
    Hotlap = 3,
    TimeAttack = 4,
    Drift = 5,
    Drag = 6,
    Hotstint = 7,
    HotlapSuperpole = 8,
}

impl fmt::Display for AccSessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AccSessionType::Unknown => "Unknown",
            AccSessionType::Practice => "Practice",
            AccSessionType::Qualifying => "Qualifying",
            AccSessionType::Race => "Race",
            AccSessionType::Hotlap => "Hotlap",
            AccSessionType::TimeAttack => "Time Attack",
            AccSessionType::Drift => "Drift",
            AccSessionType::Drag => "Drag",
            AccSessionType::Hotstint => "Hotstint",
            AccSessionType::HotlapSuperpole => "Superpole",
        };
        write!(f, "{}", name)
    }
}

impl TryFrom<i32> for AccSessionType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(AccSessionType::Unknown),
            0 => Ok(AccSessionType::Practice),
            1 => Ok(AccSessionType::Qualifying),
            2 => Ok(AccSessionType::Race),
            3 => Ok(AccSessionType::Hotlap),
            4 => Ok(AccSessionType::TimeAttack),
            5 => Ok(AccSessionType::Drift),
            6 => Ok(AccSessionType::Drag),
            7 => Ok(AccSessionType::Hotstint),
            8 => Ok(AccSessionType::HotlapSuperpole),
            _ => Err(()),
        }
    }
}