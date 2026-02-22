use crate::core::SharedMemoryReader;
use crate::maps::StaticsMap;
use crate::Result;

/// Parse the statics shared memory segment into a StaticsMap structure.
pub fn parse_statics_map(reader: &SharedMemoryReader) -> Result<StaticsMap> {
    let mut _offset = 0;

    // Helper macro to read values and advance offset
    macro_rules! read_value {
        ($type:ty) => {{
            let value = reader.read_at::<$type>(_offset)?;
            _offset += std::mem::size_of::<$type>();
            value
        }};
    }

    // Helper macro to read arrays and advance offset (used for skipped data)
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

    // Start parsing according to the SPageFileStatic structure
    let sm_version = read_string(&mut _offset, 15, 0)?;
    let ac_version = read_string(&mut _offset, 15, 0)?;
    let number_of_sessions = read_value!(i32);
    let num_cars = read_value!(i32);
    let car_model = read_string(&mut _offset, 33, 0)?;
    let track = read_string(&mut _offset, 33, 0)?;
    let player_name = read_string(&mut _offset, 33, 0)?;
    let player_surname = read_string(&mut _offset, 33, 0)?;
    let player_nick = read_string(&mut _offset, 33, 2)?;
    let sector_count = read_value!(i32);

    // Skip some unused values but need to advance offset
    let _max_torque = read_value!(f32);
    let _max_power = read_value!(f32);
    let max_rpm = read_value!(i32);
    let max_fuel = read_value!(f32);

    // Skip suspension max travel (4 floats)
    let _suspension_max_travel = read_array!(f32, 4);

    // Skip tyre radius (4 floats)
    let _tyre_radius = read_array!(f32, 4);

    let _max_turbo_boost = read_value!(f32);
    let _deprecated_1 = read_value!(f32);
    let _deprecated_2 = read_value!(f32);

    let penalty_enabled = read_value!(i32) != 0;
    let aid_fuel_rate = read_value!(f32);
    let aid_tyre_rate = read_value!(f32);
    let aid_mechanical_damage = read_value!(f32);
    let _allow_tyre_blankets = read_value!(f32); // Not used
    let aid_stability = read_value!(f32);
    let aid_auto_clutch = read_value!(i32) != 0;

    // Skip more unused fields
    let _aid_auto_blip = read_value!(i32);
    let _has_drs = read_value!(i32);
    let _has_ers = read_value!(i32);
    let _has_kers = read_value!(i32);
    let _kers_max_j = read_value!(f32);
    let _engine_brake_settings_count = read_value!(i32);
    let _ers_power_controller_count = read_value!(i32);
    let _track_spline_length = read_value!(f32);

    let _track_configuration = read_string(&mut _offset, 33, 2)?;

    let _ers_max_j = read_value!(f32);
    let _is_timed_race = read_value!(i32);
    let _has_extra_lap = read_value!(i32);

    let _car_skin = read_string(&mut _offset, 33, 2)?;

    let _reversed_grid_positions = read_value!(i32);
    let pit_window_start = read_value!(i32);
    let pit_window_end = read_value!(i32);
    let is_online = read_value!(i32) != 0;

    let dry_tyres_name = read_string(&mut _offset, 33, 0)?;
    let wet_tyres_name = read_string(&mut _offset, 33, 0)?;

    Ok(StaticsMap {
        sm_version,
        ac_version,
        number_of_sessions,
        num_cars,
        track,
        sector_count,
        player_name,
        player_surname,
        player_nick,
        car_model,
        max_rpm,
        max_fuel,
        penalty_enabled,
        aid_fuel_rate,
        aid_tyre_rate,
        aid_mechanical_damage,
        aid_stability,
        aid_auto_clutch,
        pit_window_start,
        pit_window_end,
        is_online,
        dry_tyres_name,
        wet_tyres_name,
    })
}
