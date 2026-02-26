# Available Fields (acr_telemetry_bridge)

Complete list of fields the bridge outputs as flat JSON. Usable in `dashboard_slots` (in `acr_telemetry_bridge.toml`) and in `telemetry_color.toml` (example: **`config-examples/telemetry_color.toml`**).

The `/config` API also returns this list as `available_fields`.

**Variabilität (Stand: telemetry.db):** Spalte "Veränderlich" und "Bereich" basieren auf empirischer Auswertung der physics-Tabelle. Konstant = Feld hat in der Datenbasis keinen Wertewechsel.

---

## Global Fields

Fields which are known to be filled with useful data in AC Rally 0.2 are marked with *. Dubious fields are labelled *?, reason for doubt given after name and description.

| Field | Description | Veränderlich | Bereich |
|-------|-------------|--------------|---------|
| `packet_id` | *Packet ID (increments per update) | ja | 1 … ∞ |
| `gas` | *Throttle pedal (0–1) | ja | 0–1 |
| `brake` | *Brake pedal (0–1) | ja | 0–1 |
| `clutch` | *Clutch pedal (0–1) | ja | 0–1 |
| `steer_angle` | *Steering angle (normalisiert -1…1) | ja | −1 … 1 |
| `gear` | *Gear (0=neutral, 1–7) | ja | 0–7 |
| `rpm` | *Engine RPM | ja | ca. −2600 … 8500 |
| `autoshifter_on` | Autoshifter enabled | nein | konstant 0 |
| `ignition_on` | *Ignition on | ja | 0–1 |
| `starter_engine_on` | Starter engaged | ja | 0–1 |
| `is_engine_running` | Engine running | ja | 0–1 |
| `speed_kmh` | *Speed (km/h) | ja | 0 … ca. 195 |
| `velocity_x`, `velocity_y`, `velocity_z` | *Velocity vector (world) | ja | ca. −54 … 51 (m/s oder Einheit sim) |
| `local_velocity_x/y/z` | Velocity vector (local) | ja | ca. −25 … 54 |
| `g_force_x`, `g_force_y`, `g_force_z` | G-forces (Einheit sim-spezifisch) | ja | stark variabel |
| `heading`, `pitch`, `roll` | *Orientation (rad) | ja | heading/roll: −π … π, pitch: ca. −1.23 … 1.0 |
| `final_ff` | Force feedback | ja | ca. −3.2 … 2.4 |
| `fuel` | *Fuel (L) | ja | 0 … 46 (verbrauchbar) |
| `water_temp` | *Water/coolant temperature (K→°C) | ja | 0 … ca. 78°C |
| `road_temp` | *Track surface temperature (K→°C) | ja | 0 … 304 K (0 … ca. 31°C) |
| `air_temp` | *Air temperature (K→°C) | ja | 0 … ca. 24°C |
| `tc` | *Traction control (0=aus, 1=an) | ja | 0–1 |
| `abs` | *ABS (level) | ja | 0–1 |
| `brake_bias` | *Brake bias (front) | ja | 0 … ca. 0.67 |
| `turbo_boost` | Turbo boost (bar) | nein | konstant 0 |
| `pit_limiter_on` | Pit limiter active | nein | konstant 0 |
| `tc_in_action` | TC engaging | nein | konstant 0 |
| `abs_in_action` | ABS engaging | nein | konstant 0 |
| `is_ai_controlled` | AI controlling | nein | konstant 0 |
| `car_damage_front`, `car_damage_rear`, `car_damage_left`, `car_damage_right`, `car_damage_center` | Car damage | nein | konstant 0 |
| `drs` | DRS status | nein | konstant 0 |
| `cg_height` | Center of gravity height | nein | konstant 0 |
| `number_of_tyres_out` | Tyres outside track | nein | konstant 0 |
| `kers_charge`, `kers_input`, `kers_current_kj` | KERS | nein | konstant 0 |
| `ride_height_front`, `ride_height_rear` | Ride height | nein | konstant 0 |
| `ballast` | Ballast | nein | konstant 0 |
| `air_density` | Air density | nein | konstant 0 |
| `performance_meter` | Performance meter | nein | konstant 0 |
| `engine_brake` | Engine brake | nein | konstant 0 |
| `ers_recovery_level`, `ers_power_level` | ERS | nein | konstant 0 |
| `current_max_rpm` | Current max RPM | ja | 0 … 8500 |
| `drs_available`, `drs_enabled` | DRS available/active | nein | konstant 0 |
| `p2p_activation`, `p2p_status` | Push-to-Pass | nein | konstant 0 |
| `front_brake_compound`, `rear_brake_compound` | Brake compound index | nein | konstant 0 |
| `kerb_vibration`, `slip_vibration`, `g_vibration`, `abs_vibration` | *Vibrations | slip/g/abs: ja; kerb: nein | slip/abs: −1 … 1, g: 0 … ca. 48 |

---

## Per-Wheel (`_fl`, `_fr`, `_rl`, `_rr`)

| Base | Description | Veränderlich | Bereich |
|------|-------------|--------------|---------|
| `wheel_slip` | *Wheel slip | ja | 0 … ca. 127 |
| `wheel_load` | *Wheel load | ja | 0 … ca. 44 700 N |
| `wheel_pressure` | *Tyre pressure (psi) | ja | 0 … ca. 40 psi |
| `wheel_angular_speed` | *Angular speed (rad/s) | ja | ca. −162 … 240 rad/s |
| `tyre_core_temp` | *Tyre core temperature (K→°C) | ja | 0 … ca. 131°C |
| `brake_temp` | *Brake temperature (K→°C) | ja | 0 … ca. 492°C (vorne), ca. 437°C (hinten) |
| `tyre_wear` | Tyre wear | nein | konstant 0 |
| `tyre_dirty_level` | Tyre dirt level | nein | konstant 0 |
| `camber_rad` | Camber (rad) | nein | konstant 0 |
| `suspension_travel` | *Suspension travel (DB: m) | ja | ca. −0.04 … 0.24 m (≈ −40 … 240 mm) |
| `brake_pressure` | Brake pressure (bar) | ja | 0 … ca. 0.58 (vorne), ca. 0.30 (hinten) |
| `slip_ratio` | Slip ratio | ja | ca. −10 … 24 |
| `slip_angle` | Slip angle (rad) | ja | ca. −1.5 … 1.5 rad |
| `pad_life` | Pad life (%) | ja | 0 … ca. 0.00002% (praktisch konstant) |
| `disc_life` | Disc life (%) | ja | 0 … ca. 0.00003% (praktisch konstant) |
| `tyre_temp_i`, `tyre_temp_m`, `tyre_temp_o` | Tyre temp inner/middle/outer | nein | konstant 0 |
| `mz`, `fz`, `my` | *Tyre moments/forces | ja | mz: ca. −1500 … 1250; fz: ca. −16 700 … 16 400; my: ca. −18 600 … 18 500 |
| `suspension_damage` | Suspension damage | nein | konstant 0 |

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
