use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents race control flags used by ACC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(i32)]
pub enum AccFlagType {
    /// No flag currently being displayed
    NoFlag = 0,
    /// Blue flag - slower car must allow faster cars to pass
    BlueFlag = 1,
    /// Yellow flag - caution on track, slow down and no overtaking
    YellowFlag = 2,
    /// Black flag - driver disqualified or must return to pits
    BlackFlag = 3,
    /// White flag - slow-moving vehicle ahead
    WhiteFlag = 4,
    /// Checkered flag - end of session or race
    CheckeredFlag = 5,
    /// Penalty flag - warning for driver penalties
    PenaltyFlag = 6,
    /// Green flag - clear track, full racing conditions
    GreenFlag = 7,
    /// Orange flag - mechanical issue warning
    OrangeFlag = 8,
}

impl AccFlagType {
    /// Check if this flag requires caution or reduced pace
    pub fn requires_caution(&self) -> bool {
        matches!(
            self,
            AccFlagType::YellowFlag | AccFlagType::WhiteFlag | AccFlagType::OrangeFlag
        )
    }

    /// Check if this flag is a racing flag (not administrative)
    pub fn is_racing_flag(&self) -> bool {
        !matches!(
            self,
            AccFlagType::NoFlag | AccFlagType::PenaltyFlag | AccFlagType::BlackFlag
        )
    }
}

impl fmt::Display for AccFlagType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AccFlagType::NoFlag => "No Flag",
            AccFlagType::BlueFlag => "Blue Flag",
            AccFlagType::YellowFlag => "Yellow Flag",
            AccFlagType::BlackFlag => "Black Flag",
            AccFlagType::WhiteFlag => "White Flag",
            AccFlagType::CheckeredFlag => "Checkered Flag",
            AccFlagType::PenaltyFlag => "Penalty Flag",
            AccFlagType::GreenFlag => "Green Flag",
            AccFlagType::OrangeFlag => "Orange Flag",
        };
        write!(f, "{}", name)
    }
}

impl TryFrom<i32> for AccFlagType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AccFlagType::NoFlag),
            1 => Ok(AccFlagType::BlueFlag),
            2 => Ok(AccFlagType::YellowFlag),
            3 => Ok(AccFlagType::BlackFlag),
            4 => Ok(AccFlagType::WhiteFlag),
            5 => Ok(AccFlagType::CheckeredFlag),
            6 => Ok(AccFlagType::PenaltyFlag),
            7 => Ok(AccFlagType::GreenFlag),
            8 => Ok(AccFlagType::OrangeFlag),
            _ => Err(()),
        }
    }
}