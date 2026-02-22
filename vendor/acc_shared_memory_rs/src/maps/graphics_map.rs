use crate::datatypes::{Vector3f, Wheels};
use crate::enums::{
    AccFlagType, AccPenaltyType, AccRainIntensity, AccSessionType, AccStatus, AccTrackGripStatus,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// GraphicsMap contains medium-frequency simulation state information from Assetto Corsa Competizione (ACC).
///
/// # Fields
///
/// ## Metadata
/// - `packet_id`: i32 - Unique identifier for the graphics packet (increments every update).
/// - `status`: AccStatus - Current game status (e.g., in session, replay, paused).
/// - `session_type`: AccSessionType - Type of session (Practice, Qualifying, Race, etc.).
/// - `session_index`: i32 - Index of the current session.
///
/// ## Lap Timing & Positioning
/// - `current_time_str`: String - Current lap time as a string.
/// - `last_time_str`: String - Last lap time as a string.
/// - `best_time_str`: String - Best lap time as a string.
/// - `last_sector_time_str`: String - Last sector time as a string.
/// - `completed_lap`: i32 - Number of completed laps.
/// - `position`: i32 - Current car position in the session.
/// - `current_time`: i32 - Current lap time in milliseconds.
/// - `last_time`: i32 - Last lap time in milliseconds.
/// - `best_time`: i32 - Best lap time in milliseconds.
/// - `last_sector_time`: i32 - Last sector time in milliseconds.
/// - `number_of_laps`: i32 - Total number of laps in the session.
/// - `delta_lap_time_str`: String - Delta lap time as a string.
/// - `estimated_lap_time_str`: String - Estimated lap time as a string.
/// - `delta_lap_time`: i32 - Delta lap time in milliseconds.
/// - `estimated_lap_time`: i32 - Estimated lap time in milliseconds.
/// - `is_delta_positive`: bool - Whether the delta lap time is positive.
/// - `is_valid_lap`: bool - Whether the current lap is valid.
/// - `fuel_estimated_laps`: f32 - Estimated laps left with current fuel.
/// - `distance_traveled`: f32 - Distance traveled in the session (meters).
/// - `normalized_car_position`: f32 - Normalized car position on the track (0.0-1.0).
/// - `session_time_left`: f32 - Time left in the session (seconds).
/// - `current_sector_index`: i32 - Current sector index.
///
/// ## Car & Pit Status
/// - `is_in_pit`: bool - Whether the car is in the pit area.
/// - `is_in_pit_lane`: bool - Whether the car is in the pit lane.
/// - `ideal_line_on`: bool - Whether the ideal line is enabled.
/// - `mandatory_pit_done`: bool - Whether the mandatory pit stop is done.
/// - `missing_mandatory_pits`: i32 - Number of missing mandatory pit stops.
/// - `penalty_time`: f32 - Penalty time in seconds.
/// - `penalty`: AccPenaltyType - Type of penalty applied.
/// - `flag`: AccFlagType - Current flag status (yellow, green, etc.).
///
/// ## Player/Car Identifiers
/// - `car_coordinates`: Vec<Vector3f> - World coordinates for all cars.
/// - `car_id`: Vec<i32> - Car IDs for all cars in the session.
/// - `player_car_id`: i32 - Car ID of the player.
/// - `active_cars`: i32 - Number of active cars in the session.
///
/// ## Environment & Conditions
/// - `wind_speed`: f32 - Wind speed (m/s).
/// - `wind_direction`: f32 - Wind direction (degrees).
/// - `rain_intensity`: AccRainIntensity - Current rain intensity.
/// - `rain_intensity_in_10min`: AccRainIntensity - Rain intensity forecast in 10 minutes.
/// - `rain_intensity_in_30min`: AccRainIntensity - Rain intensity forecast in 30 minutes.
/// - `track_grip_status`: AccTrackGripStatus - Current track grip status.
/// - `track_status`: String - Track status as a string.
/// - `clock`: f32 - In-game clock time (seconds).
///
/// ## Driver & Controls
/// - `tc_level`: i32 - Traction control level.
/// - `tc_cut_level`: i32 - Traction control cut level.
/// - `engine_map`: i32 - Engine map setting.
/// - `abs_level`: i32 - ABS level.
/// - `wiper_stage`: i32 - Wiper stage.
/// - `driver_stint_total_time_left`: i32 - Total time left in driver stint (seconds).
/// - `driver_stint_time_left`: i32 - Time left in current driver stint (seconds).
/// - `rain_tyres`: bool - Whether rain tyres are equipped.
///
/// ## Lighting & Signals
/// - `rain_light`: bool - Whether rain light is on.
/// - `flashing_light`: bool - Whether flashing light is on.
/// - `light_stage`: i32 - Light stage.
/// - `direction_light_left`: bool - Whether left indicator is on.
/// - `direction_light_right`: bool - Whether right indicator is on.
///
/// ## Setup/Interface State
/// - `tyre_compound`: String - Tyre compound name.
/// - `is_setup_menu_visible`: bool - Whether the setup menu is visible.
/// - `main_display_index`: i32 - Main display index.
/// - `secondary_display_index`: i32 - Secondary display index.
///
/// ## Telemetry Extras
/// - `fuel_per_lap`: f32 - Average fuel used per lap.
/// - `used_fuel`: f32 - Total fuel used.
/// - `exhaust_temp`: f32 - Exhaust temperature (°C).
/// - `gap_ahead`: i32 - Gap to car ahead (milliseconds).
/// - `gap_behind`: i32 - Gap to car behind (milliseconds).
///
/// ## Race Control Flags
/// - `global_yellow`: bool - Global yellow flag active.
/// - `global_yellow_s1`: bool - Sector 1 yellow flag active.
/// - `global_yellow_s2`: bool - Sector 2 yellow flag active.
/// - `global_yellow_s3`: bool - Sector 3 yellow flag active.
/// - `global_white`: bool - Global white flag active.
/// - `global_green`: bool - Global green flag active.
/// - `global_chequered`: bool - Global chequered flag active.
/// - `global_red`: bool - Global red flag active.
///
/// ## MFD (Multifunction Display) Inputs
/// - `mfd_tyre_set`: i32 - MFD tyre set selection.
/// - `mfd_fuel_to_add`: f32 - MFD fuel to add (liters).
/// - `mfd_tyre_pressure`: Wheels - MFD tyre pressure settings.
///
/// ## Tyre Strategy
/// - `current_tyre_set`: i32 - Current tyre set index.
/// - `strategy_tyre_set`: i32 - Strategy tyre set index.
///
/// This struct is used in the `ACCMap` object to provide access to all simulation state and session data from ACC. Consumers of this library can use these fields to perform analytics, UI updates, or make decisions based on the current session state.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphicsMap {
    // Metadata
    pub packet_id: i32,
    pub status: AccStatus,
    pub session_type: AccSessionType,
    pub session_index: i32,

    // Lap Timing & Positioning
    pub current_time_str: String,
    pub last_time_str: String,
    pub best_time_str: String,
    pub last_sector_time_str: String,
    pub completed_lap: i32,
    pub position: i32,
    pub current_time: i32,
    pub last_time: i32,
    pub best_time: i32,
    pub last_sector_time: i32,
    pub number_of_laps: i32,
    pub delta_lap_time_str: String,
    pub estimated_lap_time_str: String,
    pub delta_lap_time: i32,
    pub estimated_lap_time: i32,
    pub is_delta_positive: bool,
    pub is_valid_lap: bool,
    pub fuel_estimated_laps: f32,
    pub distance_traveled: f32,
    pub normalized_car_position: f32,
    pub session_time_left: f32,
    pub current_sector_index: i32,

    // Car & Pit Status
    pub is_in_pit: bool,
    pub is_in_pit_lane: bool,
    pub ideal_line_on: bool,
    pub mandatory_pit_done: bool,
    pub missing_mandatory_pits: i32,
    pub penalty_time: f32,
    pub penalty: AccPenaltyType,
    pub flag: AccFlagType,

    // Player/Car Identifiers
    pub car_coordinates: Vec<Vector3f>,
    pub car_id: Vec<i32>,
    pub player_car_id: i32,
    pub active_cars: i32,

    // Environment & Conditions
    pub wind_speed: f32,
    pub wind_direction: f32,
    pub rain_intensity: AccRainIntensity,
    pub rain_intensity_in_10min: AccRainIntensity,
    pub rain_intensity_in_30min: AccRainIntensity,
    pub track_grip_status: AccTrackGripStatus,
    pub track_status: String,
    pub clock: f32,

    // Driver & Controls
    pub tc_level: i32,
    pub tc_cut_level: i32,
    pub engine_map: i32,
    pub abs_level: i32,
    pub wiper_stage: i32,
    pub driver_stint_total_time_left: i32,
    pub driver_stint_time_left: i32,
    pub rain_tyres: bool,

    // Lighting & Signals
    pub rain_light: bool,
    pub flashing_light: bool,
    pub light_stage: i32,
    pub direction_light_left: bool,
    pub direction_light_right: bool,

    // Setup/Interface State
    pub tyre_compound: String,
    pub is_setup_menu_visible: bool,
    pub main_display_index: i32,
    pub secondary_display_index: i32,

    // Telemetry Extras
    pub fuel_per_lap: f32,
    pub used_fuel: f32,
    pub exhaust_temp: f32,
    pub gap_ahead: i32,
    pub gap_behind: i32,

    // Race Control Flags
    pub global_yellow: bool,
    pub global_yellow_s1: bool,
    pub global_yellow_s2: bool,
    pub global_yellow_s3: bool,
    pub global_white: bool,
    pub global_green: bool,
    pub global_chequered: bool,
    pub global_red: bool,

    // MFD (Multifunction Display) Inputs
    pub mfd_tyre_set: i32,
    pub mfd_fuel_to_add: f32,
    pub mfd_tyre_pressure: Wheels,

    // Tyre Strategy
    pub current_tyre_set: i32,
    pub strategy_tyre_set: i32,
}

impl GraphicsMap {
    /// Check if the session is currently active
    pub fn is_session_active(&self) -> bool {
        self.status.is_active()
    }

    /// Check if there are any yellow flags active
    pub fn has_yellow_flags(&self) -> bool {
        self.global_yellow
            || self.global_yellow_s1
            || self.global_yellow_s2
            || self.global_yellow_s3
            || self.flag == AccFlagType::YellowFlag
    }

    /// Check if conditions are wet
    pub fn is_wet_conditions(&self) -> bool {
        self.rain_intensity.is_wet() || self.track_grip_status.is_wet()
    }

    /// Get the current lap time in seconds
    pub fn current_lap_time_seconds(&self) -> f32 {
        self.current_time as f32 / 1000.0
    }

    /// Get the last lap time in seconds
    pub fn last_lap_time_seconds(&self) -> f32 {
        self.last_time as f32 / 1000.0
    }

    /// Get the best lap time in seconds
    pub fn best_lap_time_seconds(&self) -> f32 {
        self.best_time as f32 / 1000.0
    }

    /// Check if driver is currently serving a penalty
    pub fn has_active_penalty(&self) -> bool {
        self.penalty != AccPenaltyType::None && self.penalty != AccPenaltyType::Unknown
    }
}
