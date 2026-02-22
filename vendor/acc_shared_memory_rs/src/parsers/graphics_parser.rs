use crate::core::SharedMemoryReader;
use crate::datatypes::{Vector3f, Wheels};
use crate::enums::{
    AccFlagType, AccPenaltyType, AccRainIntensity, AccSessionType, AccStatus, AccTrackGripStatus,
};
use crate::maps::GraphicsMap;
use crate::{ACCError, Result};

/// Parse the graphics shared memory segment into a GraphicsMap structure.
pub fn parse_graphics_map(reader: &SharedMemoryReader) -> Result<GraphicsMap> {
    let mut _offset = 0;

    // Helper macro to read values and advance offset
    macro_rules! read_value {
        ($type:ty) => {{
            let value = reader.read_at::<$type>(_offset)?;
            _offset += std::mem::size_of::<$type>();
            value
        }};
    }

    // Helper macro to read arrays and advance offset
    macro_rules! read_array {
        ($type:ty, $count:expr) => {{
            let values = reader.read_array_at::<$type>(_offset, $count)?;
            _offset += std::mem::size_of::<$type>() * $count;
            values
        }};
    }

    // Helper function to read UTF-16 strings
    let read_string = |offset: &mut usize, char_count: usize, padding: usize| -> Result<String> {
        let s = reader.read_utf16_string_at(*offset, char_count)?;
        *offset += (char_count * 2) + padding;
        Ok(s)
    };

    // Helper function to safely convert to enum with fallback
    let try_enum_or_default = |value: i32| -> AccPenaltyType {
        AccPenaltyType::try_from(value).unwrap_or(AccPenaltyType::Unknown)
    };

    // Start parsing according to the SPageFileGraphic structure
    let packet_id = read_value!(i32);

    let status = AccStatus::try_from(read_value!(i32))
        .map_err(|_| ACCError::InvalidData("Invalid ACC status".to_string()))?;

    let session_type = AccSessionType::try_from(read_value!(i32))
        .map_err(|_| ACCError::InvalidData("Invalid session type".to_string()))?;

    // Timing strings (15 chars each)
    let current_time_str = read_string(&mut _offset, 15, 0)?;
    let last_time_str = read_string(&mut _offset, 15, 0)?;
    let best_time_str = read_string(&mut _offset, 15, 0)?;
    let last_sector_time_str = read_string(&mut _offset, 15, 0)?;

    let completed_lap = read_value!(i32);
    let position = read_value!(i32);
    let current_time = read_value!(i32);
    let last_time = read_value!(i32);
    let best_time = read_value!(i32);
    let session_time_left = read_value!(f32);
    let distance_traveled = read_value!(f32);
    let is_in_pit = read_value!(i32) != 0;
    let current_sector_index = read_value!(i32);
    let last_sector_time = read_value!(i32);
    let number_of_laps = read_value!(i32);

    let tyre_compound = read_string(&mut _offset, 33, 2)?;
    let _replay_time_multiplier = read_value!(f32); // Not used in ACC
    let normalized_car_position = read_value!(f32);
    let active_cars = read_value!(i32);

    // Car coordinates (60 cars x 3 floats each)
    let mut car_coordinates = Vec::with_capacity(60);
    for _ in 0..60 {
        let coords = read_array!(f32, 3);
        car_coordinates.push(Vector3f::new(coords[0], coords[1], coords[2]));
    }

    let car_id = read_array!(i32, 60);
    let player_car_id = read_value!(i32);
    let penalty_time = read_value!(f32);

    let flag = AccFlagType::try_from(read_value!(i32))
        .map_err(|_| ACCError::InvalidData("Invalid flag type".to_string()))?;

    let penalty = try_enum_or_default(read_value!(i32));

    let ideal_line_on = read_value!(i32) != 0;
    let is_in_pit_lane = read_value!(i32) != 0;
    let _surface_grip = read_value!(f32); // Not used
    let mandatory_pit_done = read_value!(i32) != 0;
    let wind_speed = read_value!(f32);
    let wind_direction = read_value!(f32);
    let is_setup_menu_visible = read_value!(i32) != 0;
    let main_display_index = read_value!(i32);
    let secondary_display_index = read_value!(i32);
    let tc_level = read_value!(i32);
    let tc_cut_level = read_value!(i32);
    let engine_map = read_value!(i32);
    let abs_level = read_value!(i32);
    let fuel_per_lap = read_value!(f32);
    let rain_light = read_value!(i32) != 0;
    let flashing_light = read_value!(i32) != 0;
    let light_stage = read_value!(i32);
    let exhaust_temp = read_value!(f32);
    let wiper_stage = read_value!(i32);
    let driver_stint_total_time_left = read_value!(i32);
    let driver_stint_time_left = read_value!(i32);
    let rain_tyres = read_value!(i32) != 0;
    let session_index = read_value!(i32);
    let used_fuel = read_value!(f32);

    let delta_lap_time_str = read_string(&mut _offset, 15, 2)?;
    let delta_lap_time = read_value!(i32);
    let estimated_lap_time_str = read_string(&mut _offset, 15, 2)?;
    let estimated_lap_time = read_value!(i32);
    let is_delta_positive = read_value!(i32) != 0;
    let _i_split = read_value!(i32); // Not used
    let is_valid_lap = read_value!(i32) != 0;
    let fuel_estimated_laps = read_value!(f32);

    let track_status = read_string(&mut _offset, 33, 2)?;
    let missing_mandatory_pits = read_value!(i32);
    let clock = read_value!(f32);
    let direction_light_left = read_value!(i32) != 0;
    let direction_light_right = read_value!(i32) != 0;

    // Global flags
    let global_yellow = read_value!(i32) != 0;
    let global_yellow_s1 = read_value!(i32) != 0;
    let global_yellow_s2 = read_value!(i32) != 0;
    let global_yellow_s3 = read_value!(i32) != 0;
    let global_white = read_value!(i32) != 0;
    let global_green = read_value!(i32) != 0;
    let global_chequered = read_value!(i32) != 0;
    let global_red = read_value!(i32) != 0;

    // MFD data
    let mfd_tyre_set = read_value!(i32);
    let mfd_fuel_to_add = read_value!(f32);
    let mfd_tyre_pressure_fl = read_value!(f32);
    let mfd_tyre_pressure_fr = read_value!(f32);
    let mfd_tyre_pressure_rl = read_value!(f32);
    let mfd_tyre_pressure_rr = read_value!(f32);
    let mfd_tyre_pressure = Wheels::new(
        mfd_tyre_pressure_fl,
        mfd_tyre_pressure_fr,
        mfd_tyre_pressure_rl,
        mfd_tyre_pressure_rr,
    );

    let track_grip_status = AccTrackGripStatus::try_from(read_value!(i32))
        .map_err(|_| ACCError::InvalidData("Invalid track grip status".to_string()))?;

    let rain_intensity = AccRainIntensity::try_from(read_value!(i32))
        .map_err(|_| ACCError::InvalidData("Invalid rain intensity".to_string()))?;

    let rain_intensity_in_10min = AccRainIntensity::try_from(read_value!(i32))
        .map_err(|_| ACCError::InvalidData("Invalid rain intensity (10min)".to_string()))?;

    let rain_intensity_in_30min = AccRainIntensity::try_from(read_value!(i32))
        .map_err(|_| ACCError::InvalidData("Invalid rain intensity (30min)".to_string()))?;

    let current_tyre_set = read_value!(i32);
    let strategy_tyre_set = read_value!(i32);
    let gap_ahead = read_value!(i32);
    let gap_behind = read_value!(i32);

    Ok(GraphicsMap {
        packet_id,
        status,
        session_type,
        session_index,
        current_time_str,
        last_time_str,
        best_time_str,
        last_sector_time_str,
        completed_lap,
        position,
        current_time,
        last_time,
        best_time,
        last_sector_time,
        number_of_laps,
        delta_lap_time_str,
        estimated_lap_time_str,
        delta_lap_time,
        estimated_lap_time,
        is_delta_positive,
        is_valid_lap,
        fuel_estimated_laps,
        distance_traveled,
        normalized_car_position,
        session_time_left,
        current_sector_index,
        is_in_pit,
        is_in_pit_lane,
        ideal_line_on,
        mandatory_pit_done,
        missing_mandatory_pits,
        penalty_time,
        penalty,
        flag,
        car_coordinates,
        car_id,
        player_car_id,
        active_cars,
        wind_speed,
        wind_direction,
        rain_intensity,
        rain_intensity_in_10min,
        rain_intensity_in_30min,
        track_grip_status,
        track_status,
        clock,
        tc_level,
        tc_cut_level,
        engine_map,
        abs_level,
        wiper_stage,
        driver_stint_total_time_left,
        driver_stint_time_left,
        rain_tyres,
        rain_light,
        flashing_light,
        light_stage,
        direction_light_left,
        direction_light_right,
        tyre_compound,
        is_setup_menu_visible,
        main_display_index,
        secondary_display_index,
        fuel_per_lap,
        used_fuel,
        exhaust_temp,
        gap_ahead,
        gap_behind,
        global_yellow,
        global_yellow_s1,
        global_yellow_s2,
        global_yellow_s3,
        global_white,
        global_green,
        global_chequered,
        global_red,
        mfd_tyre_set,
        mfd_fuel_to_add,
        mfd_tyre_pressure,
        current_tyre_set,
        strategy_tyre_set,
    })
}
