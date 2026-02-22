use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents all possible penalty types that can be issued in ACC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(i32)]
pub enum AccPenaltyType {
    /// Fallback for unknown penalty values
    Unknown = -1,
    /// No penalty issued
    None = 0,

    // Cutting track penalties
    DriveThoughCutting = 1,
    StopAndGo10Cutting = 2,
    StopAndGo20Cutting = 3,
    StopAndGo30Cutting = 4,
    DisqualifiedCutting = 5,
    RemoveBestLaptimeCutting = 6,

    // Pit speeding penalties
    DriveThoughPitSpeeding = 7,
    StopAndGo10PitSpeeding = 8,
    StopAndGo20PitSpeeding = 9,
    StopAndGo30PitSpeeding = 10,
    DisqualifiedPitSpeeding = 11,
    RemoveBestLaptimePitSpeeding = 12,

    // Pit compliance
    DisqualifiedIgnoredMandatoryPit = 13,

    // Miscellaneous
    PostRaceTime = 14,
    DisqualifiedTrolling = 15,
    DisqualifiedPitEntry = 16,
    DisqualifiedPitExit = 17,
    DisqualifiedWrongWayOld = 18, // Possibly deprecated

    // Stint-related
    DriveThoughIgnoredDriverStint = 19,
    DisqualifiedIgnoredDriverStint = 20,
    DisqualifiedExceededDriverStintLimit = 21,

    // Current wrong-way flag
    DisqualifiedWrongWay = 22,
}

impl AccPenaltyType {
    /// Check if this penalty results in disqualification
    pub fn is_disqualification(&self) -> bool {
        matches!(
            self,
            AccPenaltyType::DisqualifiedCutting
                | AccPenaltyType::DisqualifiedPitSpeeding
                | AccPenaltyType::DisqualifiedIgnoredMandatoryPit
                | AccPenaltyType::DisqualifiedTrolling
                | AccPenaltyType::DisqualifiedPitEntry
                | AccPenaltyType::DisqualifiedPitExit
                | AccPenaltyType::DisqualifiedWrongWayOld
                | AccPenaltyType::DisqualifiedIgnoredDriverStint
                | AccPenaltyType::DisqualifiedExceededDriverStintLimit
                | AccPenaltyType::DisqualifiedWrongWay
        )
    }

    /// Check if this penalty is related to cutting the track
    pub fn is_cutting_penalty(&self) -> bool {
        matches!(
            self,
            AccPenaltyType::DriveThoughCutting
                | AccPenaltyType::StopAndGo10Cutting
                | AccPenaltyType::StopAndGo20Cutting
                | AccPenaltyType::StopAndGo30Cutting
                | AccPenaltyType::DisqualifiedCutting
                | AccPenaltyType::RemoveBestLaptimeCutting
        )
    }

    /// Check if this penalty is related to pit lane speeding
    pub fn is_pit_speeding_penalty(&self) -> bool {
        matches!(
            self,
            AccPenaltyType::DriveThoughPitSpeeding
                | AccPenaltyType::StopAndGo10PitSpeeding
                | AccPenaltyType::StopAndGo20PitSpeeding
                | AccPenaltyType::StopAndGo30PitSpeeding
                | AccPenaltyType::DisqualifiedPitSpeeding
                | AccPenaltyType::RemoveBestLaptimePitSpeeding
        )
    }
}

impl fmt::Display for AccPenaltyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AccPenaltyType::Unknown => "Unknown",
            AccPenaltyType::None => "No Penalty",
            AccPenaltyType::DriveThoughCutting => "Drive Through (Cutting)",
            AccPenaltyType::StopAndGo10Cutting => "Stop & Go 10s (Cutting)",
            AccPenaltyType::StopAndGo20Cutting => "Stop & Go 20s (Cutting)",
            AccPenaltyType::StopAndGo30Cutting => "Stop & Go 30s (Cutting)",
            AccPenaltyType::DisqualifiedCutting => "Disqualified (Cutting)",
            AccPenaltyType::RemoveBestLaptimeCutting => "Best Lap Removed (Cutting)",
            AccPenaltyType::DriveThoughPitSpeeding => "Drive Through (Pit Speeding)",
            AccPenaltyType::StopAndGo10PitSpeeding => "Stop & Go 10s (Pit Speeding)",
            AccPenaltyType::StopAndGo20PitSpeeding => "Stop & Go 20s (Pit Speeding)",
            AccPenaltyType::StopAndGo30PitSpeeding => "Stop & Go 30s (Pit Speeding)",
            AccPenaltyType::DisqualifiedPitSpeeding => "Disqualified (Pit Speeding)",
            AccPenaltyType::RemoveBestLaptimePitSpeeding => "Best Lap Removed (Pit Speeding)",
            AccPenaltyType::DisqualifiedIgnoredMandatoryPit => {
                "Disqualified (Ignored Mandatory Pit)"
            }
            AccPenaltyType::PostRaceTime => "Post Race Time Penalty",
            AccPenaltyType::DisqualifiedTrolling => "Disqualified (Trolling)",
            AccPenaltyType::DisqualifiedPitEntry => "Disqualified (Pit Entry)",
            AccPenaltyType::DisqualifiedPitExit => "Disqualified (Pit Exit)",
            AccPenaltyType::DisqualifiedWrongWayOld => "Disqualified (Wrong Way - Old)",
            AccPenaltyType::DriveThoughIgnoredDriverStint => {
                "Drive Through (Ignored Driver Stint)"
            }
            AccPenaltyType::DisqualifiedIgnoredDriverStint => {
                "Disqualified (Ignored Driver Stint)"
            }
            AccPenaltyType::DisqualifiedExceededDriverStintLimit => {
                "Disqualified (Exceeded Driver Stint Limit)"
            }
            AccPenaltyType::DisqualifiedWrongWay => "Disqualified (Wrong Way)",
        };
        write!(f, "{}", name)
    }
}

impl TryFrom<i32> for AccPenaltyType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(AccPenaltyType::Unknown),
            0 => Ok(AccPenaltyType::None),
            1 => Ok(AccPenaltyType::DriveThoughCutting),
            2 => Ok(AccPenaltyType::StopAndGo10Cutting),
            3 => Ok(AccPenaltyType::StopAndGo20Cutting),
            4 => Ok(AccPenaltyType::StopAndGo30Cutting),
            5 => Ok(AccPenaltyType::DisqualifiedCutting),
            6 => Ok(AccPenaltyType::RemoveBestLaptimeCutting),
            7 => Ok(AccPenaltyType::DriveThoughPitSpeeding),
            8 => Ok(AccPenaltyType::StopAndGo10PitSpeeding),
            9 => Ok(AccPenaltyType::StopAndGo20PitSpeeding),
            10 => Ok(AccPenaltyType::StopAndGo30PitSpeeding),
            11 => Ok(AccPenaltyType::DisqualifiedPitSpeeding),
            12 => Ok(AccPenaltyType::RemoveBestLaptimePitSpeeding),
            13 => Ok(AccPenaltyType::DisqualifiedIgnoredMandatoryPit),
            14 => Ok(AccPenaltyType::PostRaceTime),
            15 => Ok(AccPenaltyType::DisqualifiedTrolling),
            16 => Ok(AccPenaltyType::DisqualifiedPitEntry),
            17 => Ok(AccPenaltyType::DisqualifiedPitExit),
            18 => Ok(AccPenaltyType::DisqualifiedWrongWayOld),
            19 => Ok(AccPenaltyType::DriveThoughIgnoredDriverStint),
            20 => Ok(AccPenaltyType::DisqualifiedIgnoredDriverStint),
            21 => Ok(AccPenaltyType::DisqualifiedExceededDriverStintLimit),
            22 => Ok(AccPenaltyType::DisqualifiedWrongWay),
            _ => Ok(AccPenaltyType::Unknown), // Fallback for unknown values
        }
    }
}