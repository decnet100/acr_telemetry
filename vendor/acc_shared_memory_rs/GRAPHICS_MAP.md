# GraphicsMap Documentation

The `GraphicsMap` struct contains medium-frequency simulation state information from Assetto Corsa Competizione (ACC). This documentation details all available fields, their types, and their meanings, so users of this library can understand and utilize the data for analytics, UI, or decision logic.

---

## Fields

### Metadata
- **`packet_id: i32`**  
  Unique identifier for the graphics packet (increments every update).
- **`status: AccStatus`**  
  Current game status (e.g., in session, replay, paused).
- **`session_type: AccSessionType`**  
  Type of session (Practice, Qualifying, Race, etc.).
- **`session_index: i32`**  
  Index of the current session.

### Lap Timing & Positioning
- **`current_time_str: String`**  
  Current lap time as a string.
- **`last_time_str: String`**  
  Last lap time as a string.
- **`best_time_str: String`**  
  Best lap time as a string.
- **`last_sector_time_str: String`**  
  Last sector time as a string.
- **`completed_lap: i32`**  
  Number of completed laps.
- **`position: i32`**  
  Current car position in the session.
- **`current_time: i32`**  
  Current lap time in milliseconds.
- **`last_time: i32`**  
  Last lap time in milliseconds.
- **`best_time: i32`**  
  Best lap time in milliseconds.
- **`last_sector_time: i32`**  
  Last sector time in milliseconds.
- **`number_of_laps: i32`**  
  Total number of laps in the session.
- **`delta_lap_time_str: String`**  
  Delta lap time as a string.
- **`estimated_lap_time_str: String`**  
  Estimated lap time as a string.
- **`delta_lap_time: i32`**  
  Delta lap time in milliseconds.
- **`estimated_lap_time: i32`**  
  Estimated lap time in milliseconds.
- **`is_delta_positive: bool`**  
  Whether the delta lap time is positive.
- **`is_valid_lap: bool`**  
  Whether the current lap is valid.
- **`fuel_estimated_laps: f32`**  
  Estimated laps left with current fuel.
- **`distance_traveled: f32`**  
  Distance traveled in the session (meters).
- **`normalized_car_position: f32`**  
  Normalized car position on the track (0.0-1.0).
- **`session_time_left: f32`**  
  Time left in the session (seconds).
- **`current_sector_index: i32`**  
  Current sector index.

### Car & Pit Status
- **`is_in_pit: bool`**  
  Whether the car is in the pit area.
- **`is_in_pit_lane: bool`**  
  Whether the car is in the pit lane.
- **`ideal_line_on: bool`**  
  Whether the ideal line is enabled.
- **`mandatory_pit_done: bool`**  
  Whether the mandatory pit stop is done.
- **`missing_mandatory_pits: i32`**  
  Number of missing mandatory pit stops.
- **`penalty_time: f32`**  
  Penalty time in seconds.
- **`penalty: AccPenaltyType`**  
  Type of penalty applied.
- **`flag: AccFlagType`**  
  Current flag status (yellow, green, etc.).

### Player/Car Identifiers
- **`car_coordinates: Vec<Vector3f>`**  
  World coordinates for all cars.
- **`car_id: Vec<i32>`**  
  Car IDs for all cars in the session.
- **`player_car_id: i32`**  
  Car ID of the player.
- **`active_cars: i32`**  
  Number of active cars in the session.

### Environment & Conditions
- **`wind_speed: f32`**  
  Wind speed (m/s).
- **`wind_direction: f32`**  
  Wind direction (degrees).
- **`rain_intensity: AccRainIntensity`**  
  Current rain intensity.
- **`rain_intensity_in_10min: AccRainIntensity`**  
  Rain intensity forecast in 10 minutes.
- **`rain_intensity_in_30min: AccRainIntensity`**  
  Rain intensity forecast in 30 minutes.
- **`track_grip_status: AccTrackGripStatus`**  
  Current track grip status.
- **`track_status: String`**  
  Track status as a string.
- **`clock: f32`**  
  In-game clock time (seconds).

### Driver & Controls
- **`tc_level: i32`**  
  Traction control level.
- **`tc_cut_level: i32`**  
  Traction control cut level.
- **`engine_map: i32`**  
  Engine map setting.
- **`abs_level: i32`**  
  ABS level.
- **`wiper_stage: i32`**  
  Wiper stage.
- **`driver_stint_total_time_left: i32`**  
  Total time left in driver stint (seconds).
- **`driver_stint_time_left: i32`**  
  Time left in current driver stint (seconds).
- **`rain_tyres: bool`**  
  Whether rain tyres are equipped.

### Lighting & Signals
- **`rain_light: bool`**  
  Whether rain light is on.
- **`flashing_light: bool`**  
  Whether flashing light is on.
- **`light_stage: i32`**  
  Light stage.
- **`direction_light_left: bool`**  
  Whether left indicator is on.
- **`direction_light_right: bool`**  
  Whether right indicator is on.

### Setup/Interface State
- **`tyre_compound: String`**  
  Tyre compound name.
- **`is_setup_menu_visible: bool`**  
  Whether the setup menu is visible.
- **`main_display_index: i32`**  
  Main display index.
- **`secondary_display_index: i32`**  
  Secondary display index.

### Telemetry Extras
- **`fuel_per_lap: f32`**  
  Average fuel used per lap.
- **`used_fuel: f32`**  
  Total fuel used.
- **`exhaust_temp: f32`**  
  Exhaust temperature (°C).
- **`gap_ahead: i32`**  
  Gap to car ahead (milliseconds).
- **`gap_behind: i32`**  
  Gap to car behind (milliseconds).

### Race Control Flags
- **`global_yellow: bool`**  
  Global yellow flag active.
- **`global_yellow_s1: bool`**  
  Sector 1 yellow flag active.
- **`global_yellow_s2: bool`**  
  Sector 2 yellow flag active.
- **`global_yellow_s3: bool`**  
  Sector 3 yellow flag active.
- **`global_white: bool`**  
  Global white flag active.
- **`global_green: bool`**  
  Global green flag active.
- **`global_chequered: bool`**  
  Global chequered flag active.
- **`global_red: bool`**  
  Global red flag active.

### MFD (Multifunction Display) Inputs
- **`mfd_tyre_set: i32`**  
  MFD tyre set selection.
- **`mfd_fuel_to_add: f32`**  
  MFD fuel to add (liters).
- **`mfd_tyre_pressure: Wheels`**  
  MFD tyre pressure settings.

### Tyre Strategy
- **`current_tyre_set: i32`**  
  Current tyre set index.
- **`strategy_tyre_set: i32`**  
  Strategy tyre set index.

---

## Usage

This struct is used in the `ACCMap` object to provide access to all simulation state and session data from ACC. Consumers of this library can use these fields to perform analytics, UI updates, or make decisions based on the current session state.

---

*Generated on: 2025-05-30*
