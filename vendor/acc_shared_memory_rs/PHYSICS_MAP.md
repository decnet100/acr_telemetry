# PhysicsMap Documentation

The `PhysicsMap` struct contains high-frequency telemetry and dynamic physics data from Assetto Corsa Competizione (ACC). This documentation details all available fields, their types, and their meanings, so users of this library can understand and utilize the data for their own computations or decision logic.

---

## Fields

### Metadata
- **`packet_id: i32`**  
  Unique identifier for the telemetry packet (increments every update).

### Driver Inputs
- **`gas: f32`**  
  Throttle pedal position (0.0 = not pressed, 1.0 = fully pressed).
- **`brake: f32`**  
  Brake pedal position (0.0 = not pressed, 1.0 = fully pressed).
- **`clutch: f32`**  
  Clutch pedal position (0.0 = not pressed, 1.0 = fully pressed).
- **`steer_angle: f32`**  
  Steering wheel angle in degrees.
- **`gear: i32`**  
  Current gear (0 = neutral, 1 = first gear, etc.).
- **`rpm: i32`**  
  Engine revolutions per minute.
- **`autoshifter_on: bool`**  
  Whether the automatic shifter is enabled.
- **`ignition_on: bool`**  
  Whether the ignition is on.
- **`starter_engine_on: bool`**  
  Whether the starter engine is engaged.
- **`is_engine_running: bool`**  
  Whether the engine is currently running.

### Car Dynamics & Motion
- **`speed_kmh: f32`**  
  Car speed in kilometers per hour.
- **`velocity: Vector3f`**  
  World-space velocity vector (x, y, z).
- **`local_velocity: Vector3f`**  
  Local-space velocity vector (x, y, z).
- **`local_angular_vel: Vector3f`**  
  Local-space angular velocity vector (x, y, z).
- **`g_force: Vector3f`**  
  G-force vector (x, y, z).
- **`heading: f32`**  
  Car heading angle in radians.
- **`pitch: f32`**  
  Car pitch angle in radians.
- **`roll: f32`**  
  Car roll angle in radians.
- **`final_ff: f32`**  
  Final force feedback value.

### Wheels & Tyres
- **`wheel_slip: Wheels`**  
  Wheel slip values for each wheel.
- **`wheel_pressure: Wheels`**  
  Tyre pressure for each wheel (psi).
- **`wheel_angular_speed: Wheels`**  
  Angular speed for each wheel (rad/s).
- **`tyre_core_temp: Wheels`**  
  Tyre core temperature for each wheel (°C).
- **`suspension_travel: Wheels`**  
  Suspension travel for each wheel (mm).
- **`brake_temp: Wheels`**  
  Brake temperature for each wheel (°C).
- **`brake_pressure: Wheels`**  
  Brake pressure for each wheel (bar).
- **`suspension_damage: Wheels`**  
  Suspension damage for each wheel (0.0 = no damage, 1.0 = fully damaged).
- **`slip_ratio: Wheels`**  
  Tyre slip ratio for each wheel.
- **`slip_angle: Wheels`**  
  Tyre slip angle for each wheel (degrees).
- **`pad_life: Wheels`**  
  Brake pad life for each wheel (percentage).
- **`disc_life: Wheels`**  
  Brake disc life for each wheel (percentage).
- **`front_brake_compound: i32`**  
  Front brake compound index.
- **`rear_brake_compound: i32`**  
  Rear brake compound index.

### Tyre Contact Patches (3D)
- **`tyre_contact_point: ContactPoint`**  
  Contact point for each tyre (3D position).
- **`tyre_contact_normal: ContactPoint`**  
  Contact normal for each tyre (3D vector).
- **`tyre_contact_heading: ContactPoint`**  
  Contact heading for each tyre (3D vector).

### Car Status
- **`fuel: f32`**  
  Current fuel level (liters).
- **`tc: f32`**  
  Traction control setting.
- **`abs: f32`**  
  ABS setting.
- **`pit_limiter_on: bool`**  
  Whether the pit limiter is active.
- **`turbo_boost: f32`**  
  Turbo boost pressure (bar).
- **`air_temp: f32`**  
  Ambient air temperature (°C).
- **`road_temp: f32`**  
  Track surface temperature (°C).
- **`water_temp: f32`**  
  Water temperature (°C).
- **`car_damage: CarDamage`**  
  Car damage information.
- **`is_ai_controlled: bool`**  
  Whether the car is controlled by AI.
- **`brake_bias: f32`**  
  Brake bias percentage (front).

### Vibration Feedback
- **`kerb_vibration: f32`**  
  Kerb vibration intensity.
- **`slip_vibration: f32`**  
  Tyre slip vibration intensity.
- **`g_vibration: f32`**  
  G-force vibration intensity.
- **`abs_vibration: f32`**  
  ABS vibration intensity.

---

## Usage

This struct is used in the `ACCMap` object to provide access to all real-time physics and telemetry data from ACC. Consumers of this library can use these fields to perform computations, analytics, or make decisions based on the car's current state.

---

*Generated on: 2025-05-30*
