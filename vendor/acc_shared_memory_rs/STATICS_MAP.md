# StaticsMap Documentation

The `StaticsMap` struct contains low-frequency static configuration data from Assetto Corsa Competizione (ACC). This documentation details all available fields, their types, and their meanings, so users of this library can understand and utilize the data for session context, car, and player configuration.

---

## Fields

### Versioning
- **`sm_version: String`**  
  Shared memory version string.
- **`ac_version: String`**  
  ACC game version string.

### Session & Track Info
- **`number_of_sessions: i32`**  
  Number of sessions in the event.
- **`num_cars: i32`**  
  Number of cars in the session.
- **`track: String`**  
  Track name.
- **`sector_count: i32`**  
  Number of sectors on the track.

### Player Profile
- **`player_name: String`**  
  Player's first name.
- **`player_surname: String`**  
  Player's surname.
- **`player_nick: String`**  
  Player's nickname.

### Vehicle Info
- **`car_model: String`**  
  Car model name.
- **`max_rpm: i32`**  
  Maximum engine RPM.
- **`max_fuel: f32`**  
  Maximum fuel capacity (liters).

### Session Rules / Aids
- **`penalty_enabled: bool`**  
  Whether penalties are enabled.
- **`aid_fuel_rate: f32`**  
  Fuel rate aid multiplier.
- **`aid_tyre_rate: f32`**  
  Tyre wear rate aid multiplier.
- **`aid_mechanical_damage: f32`**  
  Mechanical damage aid multiplier.
- **`aid_stability: f32`**  
  Stability control aid level.
- **`aid_auto_clutch: bool`**  
  Whether auto clutch is enabled.

### Pit Strategy
- **`pit_window_start: i32`**  
  Pit window start time (seconds).
- **`pit_window_end: i32`**  
  Pit window end time (seconds).

### Online Context
- **`is_online: bool`**  
  Whether the session is online/multiplayer.

### Tyre Options
- **`dry_tyres_name: String`**  
  Name of the dry tyre compound.
- **`wet_tyres_name: String`**  
  Name of the wet tyre compound.

---

## Usage

This struct is used in the `ACCMap` object to provide access to all static configuration and session data from ACC. Consumers of this library can use these fields to understand the session context, car, and player configuration.

---

*Generated on: 2025-05-30*
