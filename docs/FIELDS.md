# Available Fields (acr_telemetry_bridge)

Complete list of fields the bridge outputs as flat JSON. Usable in `dashboard_slots` (in `acr_telemetry_bridge.toml`) and in `telemetry_color.toml` (example: **`config-examples/telemetry_color.toml`**).

The `/config` API also returns this list as `available_fields`.

---

# Available Fields (acr_telemetry_bridge)

Complete list of fields the bridge outputs as flat JSON. Usable in `dashboard_slots` (in `acr_telemetry_bridge.toml`) and in `telemetry_color.toml` (example: **`config-examples/telemetry_color.toml`**).

The `/config` API also returns this list as `available_fields`.

**Variability (telemetry.db):** The "Variable" and "Range" columns are derived from analysis of the physics table. Assumption: **0 = no data** (zeros excluded from min/max). "Constant" = field does not vary in the dataset.

---

## Global Fields

Fields which are known to be filled with useful data in AC Rally 0.2 are marked with *. Dubious fields are labelled *?, reason for doubt given after name and description. 
Please note that all coordinates use "Y-UP" formatting. For car-referenced data x: lateral (left-right) axis, y: height (up-down) axis, z: longitudinal (front-back) axis.
All Temperatures are given as Kelvin by the game, any reference to °C is a later conversion.

| Field | Description | Variable | Range in own recorded data |
|-------|-------------|----------|-------|
| `packet_id` | *Packet ID (increments per update) | yes | 1 … infinity |
| `gas` | *Throttle pedal (0–1) | yes | 0 … 1, 1= throttle pressed |
| `brake` | *Brake pedal (0–1) | yes | 0 … 1, 1= brake pressed |
| `clutch` | *Clutch pedal (0–1) | yes | 0 … 1, 1= clutch NOT pressed |
| `steer_angle` | *Steering angle (normalized −1…1) | yes | −1 … 1, 1 = full right |
| `gear` | *Gear (0=neutral, 1–7) | yes | 1–7 (1=neutral) |
| `rpm` | *Engine RPM | yes | 0 … 8500 |
| `autoshifter_on` | Autoshifter enabled | no | constant (no data) |
| `ignition_on` | *Ignition on | no | constant 1 (when on) |
| `starter_engine_on` | Starter engaged | no | constant 1 (when engaged) |
| `is_engine_running` | Engine running | no | constant 1 (when running) |
| `speed_kmh` | *Speed (km/h) | yes | ~0 … 200 |
| `velocity_x`, `velocity_y`, `velocity_z` | *Velocity vector (world) | yes | ~−54 … 51 |
| `local_velocity_x/y/z` | Velocity vector (local) | yes | ~−25 … 54 |
| `g_force_x`, `g_force_y`, `g_force_z` | G-forces (g=9.81m/s²) | yes | usually -1.x g ... 1.x g; g_force_x: lateral, negative = right turn; g_force_y: up/down; z: longitudinal, negative=braking  |
| `heading`, `pitch`, `roll` | *Orientation (rad) | yes | heading/roll: −π … π, pitch: ~−−π/2 … π/2|
| `final_ff` | Force feedback value(?) | yes | ~−3.2 … 2.4 |
| `fuel` | *?Fuel (L) | yes | 30 … 46 (when tank has fuel) - no fuel consumption modelled |
| `water_temp` | *Water/coolant temperature (K→°C) | yes | ~96 … 351 K (up to ~78°C); starting out impossibly low |
| `road_temp` | *Track surface temperature (K→°C) | no | constant 304 K (31°C) |
| `air_temp` | *Air temperature | yes | ~269 … 297 K (~−4 … 24°C) - matching in-game weather forecast |
| `tc` | *Traction control (0=off, 1=on) | no | 1 when active |
| `abs` | *ABS (level) | no | 1 when active |
| `brake_bias` | *Brake bias (front) | yes | 0.5 … ~0.67 |
| `turbo_boost` | Turbo boost (bar) | no | constant (no data) |
| `pit_limiter_on` | Pit limiter active | no | constant (no data) |
| `tc_in_action` | TC engaging | no | constant (no data) |
| `abs_in_action` | ABS engaging | no | constant (no data) |
| `is_ai_controlled` | AI controlling | no | constant (no data) |
| `car_damage_front`, `car_damage_rear`, `car_damage_left`, `car_damage_right`, `car_damage_center` | Car damage | no | constant (no data) |
| `drs` | DRS status | no | constant (no data) |
| `cg_height` | Center of gravity height | no | constant (no data) |
| `number_of_tyres_out` | Tyres outside track | no | constant (no data) |
| `kers_charge`, `kers_input`, `kers_current_kj` | KERS | no | constant (no data) |
| `ride_height_front`, `ride_height_rear` | Ride height | no | constant (no data) |
| `ballast` | Ballast | no | constant (no data) |
| `air_density` | Air density | no | constant (no data) |
| `performance_meter` | Performance meter | no | constant (no data) |
| `engine_brake` | Engine brake | no | constant (no data) |
| `ers_recovery_level`, `ers_power_level` | ERS | no | constant (no data) |
| `current_max_rpm` | Current max RPM | yes | 7250 … 8500 - constant for each car model |
| `drs_available`, `drs_enabled` | DRS available/active | no | constant (no data) |
| `p2p_activation`, `p2p_status` | Push-to-Pass | no | constant (no data) |
| `front_brake_compound`, `rear_brake_compound` | Brake compound index | no | constant (no data) |
| `kerb_vibration`, `slip_vibration`, `g_vibration`, `abs_vibration` | *Vibrations | slip/g/abs: yes; kerb: no | slip/abs: −1 … 1, g: ~0 … 48 |

---

## Per-Wheel (`_fl`, `_fr`, `_rl`, `_rr`)

| Base | Description | Variable | Range |
|------|-------------|----------|-------|
| `wheel_slip` | *Wheel slip (total slip both lateral/longitudinal, as deg°) | yes | ~0 … 90 |
| `wheel_load` | *Wheel load | yes | ~0 … 44 700 N |
| `wheel_pressure` | *Tyre pressure (psi) | yes | ~10 … 40 psi |
| `wheel_angular_speed` | *Angular speed (rad/s) | yes | ~−162 … 240 rad/s |
| `tyre_core_temp` | *Tyre core temperature (K→°C) | no (since 0.3) |  fixed to ideal temps since 0.3 |
| `brake_temp` | *Brake temperature (K→°C) | yes | ~269 … 766 K front, ~269 … 436 K rear |
| `tyre_wear` | Tyre wear | no | constant (no data) |
| `tyre_dirty_level` | Tyre dirt level | no | constant (no data) |
| `camber_rad` | Camber (rad) | no | constant (no data) |
| `suspension_travel` | *Suspension travel | yes | ~−0.04 … 0.24 m (≈ −40 … 240 mm) |
| `brake_pressure` | Brake pressure (bar) | no | usually off, but I once got data there??? ~0.001 … 0.58 front, ~0.0006 … 0.30 rear |
| `slip_ratio` | Slip ratio, longitudinal slip (=wheel speed/vehicle speed?)  | yes | ~−10 … 24 |
| `slip_angle` | Slip angle, lateral slip | yes | ~−1.5 … 1.5 rad |
| `pad_life` | Pad life (%) | yes | ~0.000016 … 0.000018 (negligible variation) |
| `disc_life` | Disc life (%) | yes | ~0.00001 … 0.000032 (negligible variation) |
| `tyre_temp_i`, `tyre_temp_m`, `tyre_temp_o` | Tyre temp inner/middle/outer | no | constant (no data) |
| `mz`, `fz`, `my` | *Tyre moments/forces | yes | mz: ~−1500 … 1250; fz: ~−16 700 … 16 400; my: ~−18 600 … 18 500 |
| `suspension_damage` | Suspension damage | no | constant (no data) |

Examples: `tyre_core_temp_fl`, `brake_temp_rr`, `slip_angle_fr`, etc.

---

## Aliases (for brevity in bridge display)

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



