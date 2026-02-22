use crate::datatypes::{CarDamage, ContactPoint, Vector3f, Wheels};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// PhysicsMap contains high-frequency telemetry and dynamic physics data from Assetto Corsa Competizione (ACC).
///
/// # Fields
///
/// - `packet_id`: i32 - Unique identifier for the telemetry packet (increments every update).
///
/// ## Driver Inputs
/// - `gas`: f32 - Throttle pedal position (0.0 = not pressed, 1.0 = fully pressed).
/// - `brake`: f32 - Brake pedal position (0.0 = not pressed, 1.0 = fully pressed).
/// - `clutch`: f32 - Clutch pedal position (0.0 = not pressed, 1.0 = fully pressed).
/// - `steer_angle`: f32 - Steering wheel angle in degrees.
/// - `gear`: i32 - Current gear (0 = neutral, 1 = first gear, etc.).
/// - `rpm`: i32 - Engine revolutions per minute.
/// - `autoshifter_on`: bool - Whether the automatic shifter is enabled.
/// - `ignition_on`: bool - Whether the ignition is on.
/// - `starter_engine_on`: bool - Whether the starter engine is engaged.
/// - `is_engine_running`: bool - Whether the engine is currently running.
///
/// ## Car Dynamics & Motion
/// - `speed_kmh`: f32 - Car speed in kilometers per hour.
/// - `velocity`: Vector3f - World-space velocity vector (x, y, z).
/// - `local_velocity`: Vector3f - Local-space velocity vector (x, y, z).
/// - `local_angular_vel`: Vector3f - Local-space angular velocity vector (x, y, z).
/// - `g_force`: Vector3f - G-force vector (x, y, z).
/// - `heading`: f32 - Car heading angle in radians.
/// - `pitch`: f32 - Car pitch angle in radians.
/// - `roll`: f32 - Car roll angle in radians.
/// - `final_ff`: f32 - Final force feedback value.
///
/// ## Wheels & Tyres
/// - `wheel_slip`: Wheels - Wheel slip values for each wheel.
/// - `wheel_pressure`: Wheels - Tyre pressure for each wheel (psi).
/// - `wheel_angular_speed`: Wheels - Angular speed for each wheel (rad/s).
/// - `tyre_core_temp`: Wheels - Tyre core temperature for each wheel (°C).
/// - `suspension_travel`: Wheels - Suspension travel for each wheel (mm).
/// - `brake_temp`: Wheels - Brake temperature for each wheel (°C).
/// - `brake_pressure`: Wheels - Brake pressure for each wheel (bar).
/// - `suspension_damage`: Wheels - Suspension damage for each wheel (0.0 = no damage, 1.0 = fully damaged).
/// - `slip_ratio`: Wheels - Tyre slip ratio for each wheel.
/// - `slip_angle`: Wheels - Tyre slip angle for each wheel (degrees).
/// - `pad_life`: Wheels - Brake pad life for each wheel (percentage).
/// - `disc_life`: Wheels - Brake disc life for each wheel (percentage).
/// - `front_brake_compound`: i32 - Front brake compound index.
/// - `rear_brake_compound`: i32 - Rear brake compound index.
///
/// ## Tyre Contact Patches (3D)
/// - `tyre_contact_point`: ContactPoint - Contact point for each tyre (3D position).
/// - `tyre_contact_normal`: ContactPoint - Contact normal for each tyre (3D vector).
/// - `tyre_contact_heading`: ContactPoint - Contact heading for each tyre (3D vector).
///
/// ## Car Status
/// - `fuel`: f32 - Current fuel level (liters).
/// - `tc`: f32 - Traction control setting.
/// - `abs`: f32 - ABS setting.
/// - `pit_limiter_on`: bool - Whether the pit limiter is active.
/// - `turbo_boost`: f32 - Turbo boost pressure (bar).
/// - `air_temp`: f32 - Ambient air temperature (°C).
/// - `road_temp`: f32 - Track surface temperature (°C).
/// - `water_temp`: f32 - Water temperature (°C).
/// - `car_damage`: CarDamage - Car damage information.
/// - `is_ai_controlled`: bool - Whether the car is controlled by AI.
/// - `brake_bias`: f32 - Brake bias percentage (front).
///
/// ## Vibration Feedback
/// - `kerb_vibration`: f32 - Kerb vibration intensity.
/// - `slip_vibration`: f32 - Tyre slip vibration intensity.
/// - `g_vibration`: f32 - G-force vibration intensity.
/// - `abs_vibration`: f32 - ABS vibration intensity.
///
/// This struct is used in the `ACCMap` object to provide access to all real-time physics and telemetry data from ACC. Consumers of this library can use these fields to perform computations, analytics, or make decisions based on the car's current state.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PhysicsMap {
    // Metadata
    pub packet_id: i32,

    // Driver Inputs
    pub gas: f32,
    pub brake: f32,
    pub clutch: f32,
    pub steer_angle: f32,
    pub gear: i32,
    pub rpm: i32,
    pub autoshifter_on: bool,
    pub ignition_on: bool,
    pub starter_engine_on: bool,
    pub is_engine_running: bool,

    // Car Dynamics & Motion
    pub speed_kmh: f32,
    pub velocity: Vector3f,
    pub local_velocity: Vector3f,
    pub local_angular_vel: Vector3f,
    pub g_force: Vector3f,
    pub heading: f32,
    pub pitch: f32,
    pub roll: f32,
    pub final_ff: f32,

    // Wheels & Tyres
    pub wheel_slip: Wheels,
    pub wheel_load: Wheels,
    pub wheel_pressure: Wheels,
    pub wheel_angular_speed: Wheels,
    pub tyre_wear: Wheels,
    pub tyre_dirty_level: Wheels,
    pub tyre_core_temp: Wheels,
    pub camber_rad: Wheels,
    pub suspension_travel: Wheels,
    pub brake_temp: Wheels,
    pub brake_pressure: Wheels,
    pub suspension_damage: Wheels,
    pub slip_ratio: Wheels,
    pub slip_angle: Wheels,
    pub pad_life: Wheels,
    pub disc_life: Wheels,
    pub front_brake_compound: i32,
    pub rear_brake_compound: i32,
    pub tyre_temp_i: Wheels,
    pub tyre_temp_m: Wheels,
    pub tyre_temp_o: Wheels,

    // Tyre Contact Patches (3D)
    pub tyre_contact_point: ContactPoint,
    pub tyre_contact_normal: ContactPoint,
    pub tyre_contact_heading: ContactPoint,

    // Car Status
    pub fuel: f32,
    pub tc: f32,
    pub abs: f32,
    pub pit_limiter_on: bool,
    pub turbo_boost: f32,
    pub air_temp: f32,
    pub road_temp: f32,
    pub water_temp: f32,
    pub car_damage: CarDamage,
    pub is_ai_controlled: bool,
    pub brake_bias: f32,
    pub tc_in_action: bool,
    pub abs_in_action: bool,
    
    // Additional physics data
    pub drs: i32,
    pub cg_height: f32,
    pub number_of_tyres_out: i32,
    pub kers_charge: f32,
    pub kers_input: f32,
    pub ride_height_front: f32,
    pub ride_height_rear: f32,
    pub ballast: f32,
    pub air_density: f32,
    pub performance_meter: f32,
    pub engine_brake: i32,
    pub ers_recovery_level: i32,
    pub ers_power_level: i32,
    pub ers_heat_charging: i32,
    pub ers_is_charging: i32,
    pub kers_current_kj: f32,
    pub drs_available: i32,
    pub drs_enabled: i32,
    pub p2p_activation: i32,
    pub p2p_status: i32,
    pub current_max_rpm: i32,
    pub mz: Wheels,
    pub fz: Wheels,
    pub my: Wheels,

    // Vibration Feedback
    pub kerb_vibration: f32,
    pub slip_vibration: f32,
    pub g_vibration: f32,
    pub abs_vibration: f32,
}

impl PhysicsMap {
    /// Compare two PhysicsMap instances for equality based on suspension travel.
    /// This is used to detect when fresh telemetry data is available.
    pub fn is_equal(&self, other: &PhysicsMap) -> bool {
        self.suspension_travel == other.suspension_travel
    }

    /// Check if the car is currently moving
    pub fn is_moving(&self) -> bool {
        self.speed_kmh > 1.0
    }

    /// Check if the car is on track (not in pit)
    pub fn is_on_track(&self) -> bool {
        !self.pit_limiter_on
    }

    /// Get the maximum tyre temperature
    pub fn max_tyre_temp(&self) -> f32 {
        [
            self.tyre_core_temp.front_left,
            self.tyre_core_temp.front_right,
            self.tyre_core_temp.rear_left,
            self.tyre_core_temp.rear_right,
        ]
        .iter()
        .fold(f32::NEG_INFINITY, |a, &b| a.max(b))
    }

    /// Get the maximum brake temperature
    pub fn max_brake_temp(&self) -> f32 {
        [
            self.brake_temp.front_left,
            self.brake_temp.front_right,
            self.brake_temp.rear_left,
            self.brake_temp.rear_right,
        ]
        .iter()
        .fold(f32::NEG_INFINITY, |a, &b| a.max(b))
    }
}
