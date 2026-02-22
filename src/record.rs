//! rkyv-serializable physics snapshot for high-rate recording.

use acc_shared_memory_rs::maps::{GraphicsMap, PhysicsMap, StaticsMap};
use rkyv::{Archive, Deserialize, Serialize};

/// 3D vector (rkyv-compatible).
#[derive(Archive, Clone, Serialize, Deserialize, Debug)]
#[archive_attr(derive(Debug))]
pub struct Vector3fRecord {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Per-wheel values (rkyv-compatible).
#[derive(Archive, Clone, Serialize, Deserialize, Debug)]
#[archive_attr(derive(Debug))]
pub struct WheelsRecord {
    pub front_left: f32,
    pub front_right: f32,
    pub rear_left: f32,
    pub rear_right: f32,
}

/// Tyre contact points (rkyv-compatible).
#[derive(Archive, Clone, Serialize, Deserialize, Debug)]
#[archive_attr(derive(Debug))]
pub struct ContactPointRecord {
    pub front_left: Vector3fRecord,
    pub front_right: Vector3fRecord,
    pub rear_left: Vector3fRecord,
    pub rear_right: Vector3fRecord,
}

/// Car damage (rkyv-compatible).
#[derive(Archive, Clone, Serialize, Deserialize, Debug)]
#[archive_attr(derive(Debug))]
pub struct CarDamageRecord {
    pub front: f32,
    pub rear: f32,
    pub left: f32,
    pub right: f32,
    pub center: f32,
}

/// Complete physics snapshot for recording at ~333 Hz.
#[derive(Archive, Clone, Serialize, Deserialize, Debug)]
#[archive_attr(derive(Debug))]
pub struct PhysicsRecord {
    pub packet_id: i32,

    // Driver inputs
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

    // Car dynamics & motion
    pub speed_kmh: f32,
    pub velocity: Vector3fRecord,
    pub local_velocity: Vector3fRecord,
    pub local_angular_vel: Vector3fRecord,
    pub g_force: Vector3fRecord,
    pub heading: f32,
    pub pitch: f32,
    pub roll: f32,
    pub final_ff: f32,

    // Wheels & tyres
    pub wheel_slip: WheelsRecord,
    pub wheel_load: WheelsRecord,
    pub wheel_pressure: WheelsRecord,
    pub wheel_angular_speed: WheelsRecord,
    pub tyre_wear: WheelsRecord,
    pub tyre_dirty_level: WheelsRecord,
    pub tyre_core_temp: WheelsRecord,
    pub camber_rad: WheelsRecord,
    pub suspension_travel: WheelsRecord,
    pub brake_temp: WheelsRecord,
    pub brake_pressure: WheelsRecord,
    pub suspension_damage: WheelsRecord,
    pub slip_ratio: WheelsRecord,
    pub slip_angle: WheelsRecord,
    pub pad_life: WheelsRecord,
    pub disc_life: WheelsRecord,
    pub front_brake_compound: i32,
    pub rear_brake_compound: i32,
    pub tyre_temp_i: WheelsRecord,
    pub tyre_temp_m: WheelsRecord,
    pub tyre_temp_o: WheelsRecord,

    // Tyre contact patches
    pub tyre_contact_point: ContactPointRecord,
    pub tyre_contact_normal: ContactPointRecord,
    pub tyre_contact_heading: ContactPointRecord,

    // Car status
    pub fuel: f32,
    pub tc: f32,
    pub abs: f32,
    pub pit_limiter_on: bool,
    pub turbo_boost: f32,
    pub air_temp: f32,
    pub road_temp: f32,
    pub water_temp: f32,
    pub car_damage: CarDamageRecord,
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
    pub mz: WheelsRecord,
    pub fz: WheelsRecord,
    pub my: WheelsRecord,

    // Vibration
    pub kerb_vibration: f32,
    pub slip_vibration: f32,
    pub g_vibration: f32,
    pub abs_vibration: f32,
}

impl PhysicsRecord {
    pub fn from_physics(p: &PhysicsMap) -> Self {
        Self {
            packet_id: p.packet_id,

            gas: p.gas,
            brake: p.brake,
            clutch: p.clutch,
            steer_angle: p.steer_angle,
            gear: p.gear,
            rpm: p.rpm,
            autoshifter_on: p.autoshifter_on,
            ignition_on: p.ignition_on,
            starter_engine_on: p.starter_engine_on,
            is_engine_running: p.is_engine_running,

            speed_kmh: p.speed_kmh,
            velocity: Vector3fRecord { x: p.velocity.x, y: p.velocity.y, z: p.velocity.z },
            local_velocity: Vector3fRecord { x: p.local_velocity.x, y: p.local_velocity.y, z: p.local_velocity.z },
            local_angular_vel: Vector3fRecord { x: p.local_angular_vel.x, y: p.local_angular_vel.y, z: p.local_angular_vel.z },
            g_force: Vector3fRecord { x: p.g_force.x, y: p.g_force.y, z: p.g_force.z },
            heading: p.heading,
            pitch: p.pitch,
            roll: p.roll,
            final_ff: p.final_ff,

            wheel_slip: WheelsRecord {
                front_left: p.wheel_slip.front_left,
                front_right: p.wheel_slip.front_right,
                rear_left: p.wheel_slip.rear_left,
                rear_right: p.wheel_slip.rear_right,
            },
            wheel_load: WheelsRecord {
                front_left: p.wheel_load.front_left,
                front_right: p.wheel_load.front_right,
                rear_left: p.wheel_load.rear_left,
                rear_right: p.wheel_load.rear_right,
            },
            wheel_pressure: WheelsRecord {
                front_left: p.wheel_pressure.front_left,
                front_right: p.wheel_pressure.front_right,
                rear_left: p.wheel_pressure.rear_left,
                rear_right: p.wheel_pressure.rear_right,
            },
            wheel_angular_speed: WheelsRecord {
                front_left: p.wheel_angular_speed.front_left,
                front_right: p.wheel_angular_speed.front_right,
                rear_left: p.wheel_angular_speed.rear_left,
                rear_right: p.wheel_angular_speed.rear_right,
            },
            tyre_wear: WheelsRecord {
                front_left: p.tyre_wear.front_left,
                front_right: p.tyre_wear.front_right,
                rear_left: p.tyre_wear.rear_left,
                rear_right: p.tyre_wear.rear_right,
            },
            tyre_dirty_level: WheelsRecord {
                front_left: p.tyre_dirty_level.front_left,
                front_right: p.tyre_dirty_level.front_right,
                rear_left: p.tyre_dirty_level.rear_left,
                rear_right: p.tyre_dirty_level.rear_right,
            },
            tyre_core_temp: WheelsRecord {
                front_left: p.tyre_core_temp.front_left,
                front_right: p.tyre_core_temp.front_right,
                rear_left: p.tyre_core_temp.rear_left,
                rear_right: p.tyre_core_temp.rear_right,
            },
            camber_rad: WheelsRecord {
                front_left: p.camber_rad.front_left,
                front_right: p.camber_rad.front_right,
                rear_left: p.camber_rad.rear_left,
                rear_right: p.camber_rad.rear_right,
            },
            suspension_travel: WheelsRecord {
                front_left: p.suspension_travel.front_left,
                front_right: p.suspension_travel.front_right,
                rear_left: p.suspension_travel.rear_left,
                rear_right: p.suspension_travel.rear_right,
            },
            brake_temp: WheelsRecord {
                front_left: p.brake_temp.front_left,
                front_right: p.brake_temp.front_right,
                rear_left: p.brake_temp.rear_left,
                rear_right: p.brake_temp.rear_right,
            },
            brake_pressure: WheelsRecord {
                front_left: p.brake_pressure.front_left,
                front_right: p.brake_pressure.front_right,
                rear_left: p.brake_pressure.rear_left,
                rear_right: p.brake_pressure.rear_right,
            },
            suspension_damage: WheelsRecord {
                front_left: p.suspension_damage.front_left,
                front_right: p.suspension_damage.front_right,
                rear_left: p.suspension_damage.rear_left,
                rear_right: p.suspension_damage.rear_right,
            },
            slip_ratio: WheelsRecord {
                front_left: p.slip_ratio.front_left,
                front_right: p.slip_ratio.front_right,
                rear_left: p.slip_ratio.rear_left,
                rear_right: p.slip_ratio.rear_right,
            },
            slip_angle: WheelsRecord {
                front_left: p.slip_angle.front_left,
                front_right: p.slip_angle.front_right,
                rear_left: p.slip_angle.rear_left,
                rear_right: p.slip_angle.rear_right,
            },
            pad_life: WheelsRecord {
                front_left: p.pad_life.front_left,
                front_right: p.pad_life.front_right,
                rear_left: p.pad_life.rear_left,
                rear_right: p.pad_life.rear_right,
            },
            disc_life: WheelsRecord {
                front_left: p.disc_life.front_left,
                front_right: p.disc_life.front_right,
                rear_left: p.disc_life.rear_left,
                rear_right: p.disc_life.rear_right,
            },
            front_brake_compound: p.front_brake_compound,
            rear_brake_compound: p.rear_brake_compound,
            tyre_temp_i: WheelsRecord {
                front_left: p.tyre_temp_i.front_left,
                front_right: p.tyre_temp_i.front_right,
                rear_left: p.tyre_temp_i.rear_left,
                rear_right: p.tyre_temp_i.rear_right,
            },
            tyre_temp_m: WheelsRecord {
                front_left: p.tyre_temp_m.front_left,
                front_right: p.tyre_temp_m.front_right,
                rear_left: p.tyre_temp_m.rear_left,
                rear_right: p.tyre_temp_m.rear_right,
            },
            tyre_temp_o: WheelsRecord {
                front_left: p.tyre_temp_o.front_left,
                front_right: p.tyre_temp_o.front_right,
                rear_left: p.tyre_temp_o.rear_left,
                rear_right: p.tyre_temp_o.rear_right,
            },

            tyre_contact_point: ContactPointRecord {
                front_left: Vector3fRecord { x: p.tyre_contact_point.front_left.x, y: p.tyre_contact_point.front_left.y, z: p.tyre_contact_point.front_left.z },
                front_right: Vector3fRecord { x: p.tyre_contact_point.front_right.x, y: p.tyre_contact_point.front_right.y, z: p.tyre_contact_point.front_right.z },
                rear_left: Vector3fRecord { x: p.tyre_contact_point.rear_left.x, y: p.tyre_contact_point.rear_left.y, z: p.tyre_contact_point.rear_left.z },
                rear_right: Vector3fRecord { x: p.tyre_contact_point.rear_right.x, y: p.tyre_contact_point.rear_right.y, z: p.tyre_contact_point.rear_right.z },
            },
            tyre_contact_normal: ContactPointRecord {
                front_left: Vector3fRecord { x: p.tyre_contact_normal.front_left.x, y: p.tyre_contact_normal.front_left.y, z: p.tyre_contact_normal.front_left.z },
                front_right: Vector3fRecord { x: p.tyre_contact_normal.front_right.x, y: p.tyre_contact_normal.front_right.y, z: p.tyre_contact_normal.front_right.z },
                rear_left: Vector3fRecord { x: p.tyre_contact_normal.rear_left.x, y: p.tyre_contact_normal.rear_left.y, z: p.tyre_contact_normal.rear_left.z },
                rear_right: Vector3fRecord { x: p.tyre_contact_normal.rear_right.x, y: p.tyre_contact_normal.rear_right.y, z: p.tyre_contact_normal.rear_right.z },
            },
            tyre_contact_heading: ContactPointRecord {
                front_left: Vector3fRecord { x: p.tyre_contact_heading.front_left.x, y: p.tyre_contact_heading.front_left.y, z: p.tyre_contact_heading.front_left.z },
                front_right: Vector3fRecord { x: p.tyre_contact_heading.front_right.x, y: p.tyre_contact_heading.front_right.y, z: p.tyre_contact_heading.front_right.z },
                rear_left: Vector3fRecord { x: p.tyre_contact_heading.rear_left.x, y: p.tyre_contact_heading.rear_left.y, z: p.tyre_contact_heading.rear_left.z },
                rear_right: Vector3fRecord { x: p.tyre_contact_heading.rear_right.x, y: p.tyre_contact_heading.rear_right.y, z: p.tyre_contact_heading.rear_right.z },
            },

            fuel: p.fuel,
            tc: p.tc,
            abs: p.abs,
            pit_limiter_on: p.pit_limiter_on,
            turbo_boost: p.turbo_boost,
            air_temp: p.air_temp,
            road_temp: p.road_temp,
            water_temp: p.water_temp,
            car_damage: CarDamageRecord {
                front: p.car_damage.front,
                rear: p.car_damage.rear,
                left: p.car_damage.left,
                right: p.car_damage.right,
                center: p.car_damage.center,
            },
            is_ai_controlled: p.is_ai_controlled,
            brake_bias: p.brake_bias,
            tc_in_action: p.tc_in_action,
            abs_in_action: p.abs_in_action,

            drs: p.drs,
            cg_height: p.cg_height,
            number_of_tyres_out: p.number_of_tyres_out,
            kers_charge: p.kers_charge,
            kers_input: p.kers_input,
            ride_height_front: p.ride_height_front,
            ride_height_rear: p.ride_height_rear,
            ballast: p.ballast,
            air_density: p.air_density,
            performance_meter: p.performance_meter,
            engine_brake: p.engine_brake,
            ers_recovery_level: p.ers_recovery_level,
            ers_power_level: p.ers_power_level,
            ers_heat_charging: p.ers_heat_charging,
            ers_is_charging: p.ers_is_charging,
            kers_current_kj: p.kers_current_kj,
            drs_available: p.drs_available,
            drs_enabled: p.drs_enabled,
            p2p_activation: p.p2p_activation,
            p2p_status: p.p2p_status,
            current_max_rpm: p.current_max_rpm,
            mz: WheelsRecord {
                front_left: p.mz.front_left,
                front_right: p.mz.front_right,
                rear_left: p.mz.rear_left,
                rear_right: p.mz.rear_right,
            },
            fz: WheelsRecord {
                front_left: p.fz.front_left,
                front_right: p.fz.front_right,
                rear_left: p.fz.rear_left,
                rear_right: p.fz.rear_right,
            },
            my: WheelsRecord {
                front_left: p.my.front_left,
                front_right: p.my.front_right,
                rear_left: p.my.rear_left,
                rear_right: p.my.rear_right,
            },

            kerb_vibration: p.kerb_vibration,
            slip_vibration: p.slip_vibration,
            g_vibration: p.g_vibration,
            abs_vibration: p.abs_vibration,
        }
    }
}

/// Static session/car configuration (captured once per recording).
#[derive(Archive, Clone, Serialize, Deserialize, Debug)]
#[archive_attr(derive(Debug))]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct StaticsRecord {
    pub sm_version: String,
    pub ac_version: String,
    pub number_of_sessions: i32,
    pub num_cars: i32,
    pub track: String,
    pub sector_count: i32,
    pub player_name: String,
    pub player_surname: String,
    pub player_nick: String,
    pub car_model: String,
    pub max_rpm: i32,
    pub max_fuel: f32,
    pub penalty_enabled: bool,
    pub aid_fuel_rate: f32,
    pub aid_tyre_rate: f32,
    pub aid_mechanical_damage: f32,
    pub aid_stability: f32,
    pub aid_auto_clutch: bool,
    pub pit_window_start: i32,
    pub pit_window_end: i32,
    pub is_online: bool,
    pub dry_tyres_name: String,
    pub wet_tyres_name: String,
}

impl StaticsRecord {
    pub fn from_statics(s: &StaticsMap) -> Self {
        Self {
            sm_version: s.sm_version.clone(),
            ac_version: s.ac_version.clone(),
            number_of_sessions: s.number_of_sessions,
            num_cars: s.num_cars,
            track: s.track.clone(),
            sector_count: s.sector_count,
            player_name: s.player_name.clone(),
            player_surname: s.player_surname.clone(),
            player_nick: s.player_nick.clone(),
            car_model: s.car_model.clone(),
            max_rpm: s.max_rpm,
            max_fuel: s.max_fuel,
            penalty_enabled: s.penalty_enabled,
            aid_fuel_rate: s.aid_fuel_rate,
            aid_tyre_rate: s.aid_tyre_rate,
            aid_mechanical_damage: s.aid_mechanical_damage,
            aid_stability: s.aid_stability,
            aid_auto_clutch: s.aid_auto_clutch,
            pit_window_start: s.pit_window_start,
            pit_window_end: s.pit_window_end,
            is_online: s.is_online,
            dry_tyres_name: s.dry_tyres_name.clone(),
            wet_tyres_name: s.wet_tyres_name.clone(),
        }
    }
}

/// Graphics/session data snapshot (~60 Hz).
#[derive(Archive, Clone, Serialize, Deserialize, Debug)]
#[archive_attr(derive(Debug))]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GraphicsRecord {
    pub packet_id: i32,
    pub status: i32,
    pub session_type: i32,
    pub session_index: i32,
    
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
    
    pub is_in_pit: bool,
    pub is_in_pit_lane: bool,
    pub ideal_line_on: bool,
    pub mandatory_pit_done: bool,
    pub missing_mandatory_pits: i32,
    pub penalty_time: f32,
    pub penalty: i32,
    pub flag: i32,
    
    pub player_car_id: i32,
    pub active_cars: i32,
    pub car_coordinates_x: f32,
    pub car_coordinates_y: f32,
    pub car_coordinates_z: f32,
    
    pub wind_speed: f32,
    pub wind_direction: f32,
    pub rain_intensity: i32,
    pub rain_intensity_in_10min: i32,
    pub rain_intensity_in_30min: i32,
    pub track_grip_status: i32,
    pub track_status: String,
    pub clock: f32,
    
    pub tc_level: i32,
    pub tc_cut_level: i32,
    pub engine_map: i32,
    pub abs_level: i32,
    pub wiper_stage: i32,
    pub driver_stint_total_time_left: i32,
    pub driver_stint_time_left: i32,
    pub rain_tyres: bool,
    
    pub rain_light: bool,
    pub flashing_light: bool,
    pub light_stage: i32,
    pub direction_light_left: bool,
    pub direction_light_right: bool,
    
    pub tyre_compound: String,
    pub is_setup_menu_visible: bool,
    pub main_display_index: i32,
    pub secondary_display_index: i32,
    
    pub fuel_per_lap: f32,
    pub used_fuel: f32,
    pub exhaust_temp: f32,
    pub gap_ahead: i32,
    pub gap_behind: i32,
    
    pub global_yellow: bool,
    pub global_yellow_s1: bool,
    pub global_yellow_s2: bool,
    pub global_yellow_s3: bool,
    pub global_white: bool,
    pub global_green: bool,
    pub global_chequered: bool,
    pub global_red: bool,
    
    pub mfd_tyre_set: i32,
    pub mfd_fuel_to_add: f32,
    pub mfd_tyre_pressure_fl: f32,
    pub mfd_tyre_pressure_fr: f32,
    pub mfd_tyre_pressure_rl: f32,
    pub mfd_tyre_pressure_rr: f32,
    
    pub current_tyre_set: i32,
    pub strategy_tyre_set: i32,
}

impl GraphicsRecord {
    pub fn from_graphics(g: &GraphicsMap) -> Self {
        let default_coords = acc_shared_memory_rs::datatypes::Vector3f::new(0.0, 0.0, 0.0);
        let player_coords = g.car_coordinates
            .iter()
            .zip(&g.car_id)
            .find(|&(_, &id)| id == g.player_car_id)
            .map(|(coords, _)| coords)
            .unwrap_or(&default_coords);
        
        Self {
            packet_id: g.packet_id,
            status: g.status as i32,
            session_type: g.session_type as i32,
            session_index: g.session_index,
            current_time_str: g.current_time_str.clone(),
            last_time_str: g.last_time_str.clone(),
            best_time_str: g.best_time_str.clone(),
            last_sector_time_str: g.last_sector_time_str.clone(),
            completed_lap: g.completed_lap,
            position: g.position,
            current_time: g.current_time,
            last_time: g.last_time,
            best_time: g.best_time,
            last_sector_time: g.last_sector_time,
            number_of_laps: g.number_of_laps,
            delta_lap_time_str: g.delta_lap_time_str.clone(),
            estimated_lap_time_str: g.estimated_lap_time_str.clone(),
            delta_lap_time: g.delta_lap_time,
            estimated_lap_time: g.estimated_lap_time,
            is_delta_positive: g.is_delta_positive,
            is_valid_lap: g.is_valid_lap,
            fuel_estimated_laps: g.fuel_estimated_laps,
            distance_traveled: g.distance_traveled,
            normalized_car_position: g.normalized_car_position,
            session_time_left: g.session_time_left,
            current_sector_index: g.current_sector_index,
            is_in_pit: g.is_in_pit,
            is_in_pit_lane: g.is_in_pit_lane,
            ideal_line_on: g.ideal_line_on,
            mandatory_pit_done: g.mandatory_pit_done,
            missing_mandatory_pits: g.missing_mandatory_pits,
            penalty_time: g.penalty_time,
            penalty: g.penalty as i32,
            flag: g.flag as i32,
            player_car_id: g.player_car_id,
            active_cars: g.active_cars,
            car_coordinates_x: player_coords.x,
            car_coordinates_y: player_coords.y,
            car_coordinates_z: player_coords.z,
            wind_speed: g.wind_speed,
            wind_direction: g.wind_direction,
            rain_intensity: g.rain_intensity as i32,
            rain_intensity_in_10min: g.rain_intensity_in_10min as i32,
            rain_intensity_in_30min: g.rain_intensity_in_30min as i32,
            track_grip_status: g.track_grip_status as i32,
            track_status: g.track_status.clone(),
            clock: g.clock,
            tc_level: g.tc_level,
            tc_cut_level: g.tc_cut_level,
            engine_map: g.engine_map,
            abs_level: g.abs_level,
            wiper_stage: g.wiper_stage,
            driver_stint_total_time_left: g.driver_stint_total_time_left,
            driver_stint_time_left: g.driver_stint_time_left,
            rain_tyres: g.rain_tyres,
            rain_light: g.rain_light,
            flashing_light: g.flashing_light,
            light_stage: g.light_stage,
            direction_light_left: g.direction_light_left,
            direction_light_right: g.direction_light_right,
            tyre_compound: g.tyre_compound.clone(),
            is_setup_menu_visible: g.is_setup_menu_visible,
            main_display_index: g.main_display_index,
            secondary_display_index: g.secondary_display_index,
            fuel_per_lap: g.fuel_per_lap,
            used_fuel: g.used_fuel,
            exhaust_temp: g.exhaust_temp,
            gap_ahead: g.gap_ahead,
            gap_behind: g.gap_behind,
            global_yellow: g.global_yellow,
            global_yellow_s1: g.global_yellow_s1,
            global_yellow_s2: g.global_yellow_s2,
            global_yellow_s3: g.global_yellow_s3,
            global_white: g.global_white,
            global_green: g.global_green,
            global_chequered: g.global_chequered,
            global_red: g.global_red,
            mfd_tyre_set: g.mfd_tyre_set,
            mfd_fuel_to_add: g.mfd_fuel_to_add,
            mfd_tyre_pressure_fl: g.mfd_tyre_pressure.front_left,
            mfd_tyre_pressure_fr: g.mfd_tyre_pressure.front_right,
            mfd_tyre_pressure_rl: g.mfd_tyre_pressure.rear_left,
            mfd_tyre_pressure_rr: g.mfd_tyre_pressure.rear_right,
            current_tyre_set: g.current_tyre_set,
            strategy_tyre_set: g.strategy_tyre_set,
        }
    }
}
