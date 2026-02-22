#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// StaticsMap contains low-frequency static configuration data from Assetto Corsa Competizione (ACC).
/// This information is initialized once per session and doesn't change.
///
/// # Fields
///
/// ## Versioning
/// - `sm_version`: String - Shared memory version string.
/// - `ac_version`: String - ACC game version string.
///
/// ## Session & Track Info
/// - `number_of_sessions`: i32 - Number of sessions in the event.
/// - `num_cars`: i32 - Number of cars in the session.
/// - `track`: String - Track name.
/// - `sector_count`: i32 - Number of sectors on the track.
///
/// ## Player Profile
/// - `player_name`: String - Player's first name.
/// - `player_surname`: String - Player's surname.
/// - `player_nick`: String - Player's nickname.
///
/// ## Vehicle Info
/// - `car_model`: String - Car model name.
/// - `max_rpm`: i32 - Maximum engine RPM.
/// - `max_fuel`: f32 - Maximum fuel capacity (liters).
///
/// ## Session Rules / Aids
/// - `penalty_enabled`: bool - Whether penalties are enabled.
/// - `aid_fuel_rate`: f32 - Fuel rate aid multiplier.
/// - `aid_tyre_rate`: f32 - Tyre wear rate aid multiplier.
/// - `aid_mechanical_damage`: f32 - Mechanical damage aid multiplier.
/// - `aid_stability`: f32 - Stability control aid level.
/// - `aid_auto_clutch`: bool - Whether auto clutch is enabled.
///
/// ## Pit Strategy
/// - `pit_window_start`: i32 - Pit window start time (seconds).
/// - `pit_window_end`: i32 - Pit window end time (seconds).
///
/// ## Online Context
/// - `is_online`: bool - Whether the session is online/multiplayer.
///
/// ## Tyre Options
/// - `dry_tyres_name`: String - Name of the dry tyre compound.
/// - `wet_tyres_name`: String - Name of the wet tyre compound.
///
/// This struct is used in the `ACCMap` object to provide access to all static configuration and session data from ACC. Consumers of this library can use these fields to understand the session context, car, and player configuration.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StaticsMap {
    // Versioning
    pub sm_version: String,
    pub ac_version: String,

    // Session & Track Info
    pub number_of_sessions: i32,
    pub num_cars: i32,
    pub track: String,
    pub sector_count: i32,

    // Player Profile
    pub player_name: String,
    pub player_surname: String,
    pub player_nick: String,

    // Vehicle Info
    pub car_model: String,
    pub max_rpm: i32,
    pub max_fuel: f32,

    // Session Rules / Aids
    pub penalty_enabled: bool,
    pub aid_fuel_rate: f32,
    pub aid_tyre_rate: f32,
    pub aid_mechanical_damage: f32,
    pub aid_stability: f32,
    pub aid_auto_clutch: bool,

    // Pit Strategy
    pub pit_window_start: i32,
    pub pit_window_end: i32,

    // Online Context
    pub is_online: bool,

    // Tyre Options
    pub dry_tyres_name: String,
    pub wet_tyres_name: String,
}

impl StaticsMap {
    /// Get the full player name
    pub fn full_player_name(&self) -> String {
        format!("{} {}", self.player_name, self.player_surname)
    }

    /// Check if this is a multiplayer session
    pub fn is_multiplayer(&self) -> bool {
        self.is_online && self.num_cars > 1
    }

    /// Check if there's a mandatory pit window
    pub fn has_pit_window(&self) -> bool {
        self.pit_window_start > 0 && self.pit_window_end > self.pit_window_start
    }

    /// Get pit window duration in seconds
    pub fn pit_window_duration(&self) -> i32 {
        if self.has_pit_window() {
            self.pit_window_end - self.pit_window_start
        } else {
            0
        }
    }

    /// Check if assists are enabled
    pub fn has_assists(&self) -> bool {
        self.aid_auto_clutch || self.aid_stability > 0.0
    }
}
