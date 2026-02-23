# Available Fields (acr_telemetry_bridge)

Complete list of fields the bridge outputs as flat JSON. Usable in `dashboard_slots` and `telemetry_color.toml`.

The `/config` API also returns this list as `available_fields`.

---

## Global Fields

Fields which are known to be filled with useful data in AC Rally 0.2 are marked with *. Dubious fields are labelled *?, reason for doubt given after name and description.

| Field | Description |
|-------|-------------|
| `packet_id` | *Packet ID (increments per update) |
| `gas` | *Throttle pedal (0–1) |
| `brake` | *Brake pedal (0–1) |
| `clutch` | *Clutch pedal (0–1) |
| `steer_angle` | *Steering angle (°) |
| `gear` | *Gear (0=neutral, 1–7) |
| `rpm` | *Engine RPM |
| `autoshifter_on` | Autoshifter enabled |
| `ignition_on` | *Ignition on |
| `starter_engine_on` | Starter engaged |
| `is_engine_running` | Engine running |
| `speed_kmh` | *Speed (km/h) |
| `velocity_x`, `velocity_y`, `velocity_z` | *Velocity vector (world) |
| `local_velocity_x/y/z` | Velocity vector (local) |
| `g_force_x`, `g_force_y`, `g_force_z` | G-forces |
| `heading`, `pitch`, `roll` | *Orientation (rad) |
| `final_ff` | Force feedback |
| `fuel` | *?Fuel (L) -always at starting amount |
| `water_temp` | *Water/coolant temperature |
| `road_temp` | * Track surface temperature - always at 304K/30.9°C)|
| `air_temp` | *Air temperature |
| `tc` | *Traction control (on/off) |
| `abs` | *ABS (level - 1, 0,-1) |
| `brake_bias` | *?Brake bias (front) - always at 0.5 right now |
| `turbo_boost` | Turbo boost (bar) |
| `pit_limiter_on` | Pit limiter active |
| `tc_in_action` | TC engaging |
| `abs_in_action` | ABS engaging |
| `is_ai_controlled` | AI controlling |
| `car_damage_front`, `car_damage_rear`, `car_damage_left`, `car_damage_right`, `car_damage_center` | Car damage |
| `drs` | DRS status |
| `cg_height` | Center of gravity height |
| `number_of_tyres_out` | Tyres outside track |
| `kers_charge`, `kers_input`, `kers_current_kj` | KERS |
| `ride_height_front`, `ride_height_rear` | Ride height |
| `ballast` | Ballast |
| `air_density` | Air density |
| `performance_meter` | Performance meter |
| `engine_brake` | Engine brake |
| `ers_recovery_level`, `ers_power_level` | ERS |
| `current_max_rpm` | Current max RPM |
| `drs_available`, `drs_enabled` | DRS available/active |
| `p2p_activation`, `p2p_status` | Push-to-Pass |
| `front_brake_compound`, `rear_brake_compound` | Brake compound index |
| `kerb_vibration`, `slip_vibration`, `g_vibration`, `abs_vibration` | *Vibrations |

---

## Per-Wheel (`_fl`, `_fr`, `_rl`, `_rr`)

| Base | Description |
|------|-------------|
| `wheel_slip` | *Wheel slip |
| `wheel_load` | *Wheel load |
| `wheel_pressure` | *Tyre pressure (psi) |
| `wheel_angular_speed` | *Angular speed (rad/s) |
| `tyre_core_temp` | *Tyre core temperature |
| `brake_temp` | *Brake temperature |
| `tyre_wear` | Tyre wear |
| `tyre_dirty_level` | Tyre dirt level |
| `camber_rad` | Camber (rad) |
| `suspension_travel` | *Suspension travel (mm) |
| `brake_pressure` | Brake pressure (bar) |
| `slip_ratio` | Slip ratio |
| `slip_angle` | Slip angle (°) |
| `pad_life` | Pad life (%) |
| `disc_life` | Disc life (%) |
| `tyre_temp_i`, `tyre_temp_m`, `tyre_temp_o` | Tyre temp inner/middle/outer |
| `mz`, `fz`, `my` | *Tyre moments/forces |
| `suspension_damage` | Suspension damage |

Examples: `tyre_core_temp_fl`, `brake_temp_rr`, `slip_angle_fr`, etc.

---

## Aliases (for backward compatibility)

| Alias | Equivalent to |
|-------|---------------|
| `tyre_fl` | `tyre_core_temp_fl` |
| `tyre_fr` | `tyre_core_temp_fr` |
| `tyre_rl` | `tyre_core_temp_rl` |
| `tyre_rr` | `tyre_core_temp_rr` |
| `brake_fl` | `brake_temp_fl` |
| `brake_fr` | `brake_temp_fr` |
| `brake_rl` | `brake_temp_rl` |
| `brake_rr` | `brake_temp_rr` |
