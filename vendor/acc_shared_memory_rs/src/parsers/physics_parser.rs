use crate::core::SharedMemoryReader;
use crate::datatypes::{CarDamage, ContactPoint, Vector3f, Wheels};
use crate::maps::PhysicsMap;
use crate::Result;

/// Parse the physics shared memory segment into a PhysicsMap structure.
pub fn parse_physics_map(reader: &SharedMemoryReader) -> Result<PhysicsMap> {
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

    // Helper function to read Vector3f
    let read_vector3f = |offset: &mut usize| -> Result<Vector3f> {
        let values = reader.read_array_at::<f32>(*offset, 3)?;
        *offset += std::mem::size_of::<f32>() * 3;
        Ok(Vector3f::new(values[0], values[1], values[2]))
    };

    // Helper function to read Wheels
    let read_wheels = |offset: &mut usize| -> Result<Wheels> {
        let values = reader.read_array_at::<f32>(*offset, 4)?;
        *offset += std::mem::size_of::<f32>() * 4;
        Ok(Wheels::new(values[0], values[1], values[2], values[3]))
    };

    // Helper function to read ContactPoint (4x3 array)
    let read_contact_point = |offset: &mut usize| -> Result<ContactPoint> {
        let mut points = [[0.0f32; 3]; 4];
        for i in 0..4 {
            let values = reader.read_array_at::<f32>(*offset, 3)?;
            *offset += std::mem::size_of::<f32>() * 3;
            points[i] = [values[0], values[1], values[2]];
        }
        Ok(ContactPoint::from_nested_array(points))
    };

    // Start parsing according to the SPageFilePhysics structure
    let packet_id = read_value!(i32);
    let gas = read_value!(f32);
    let brake = read_value!(f32);
    let fuel = read_value!(f32);
    let gear = read_value!(i32);
    let rpm = read_value!(i32);
    let steer_angle = read_value!(f32);

    let speed_kmh = read_value!(f32);
    let velocity = read_vector3f(&mut _offset)?;
    let g_force = read_vector3f(&mut _offset)?;

    // Wheels data
    let wheel_slip = read_wheels(&mut _offset)?;
    let wheel_load = read_wheels(&mut _offset)?;
    let wheel_pressure = read_wheels(&mut _offset)?;
    let wheel_angular_speed = read_wheels(&mut _offset)?;
    let tyre_wear = read_wheels(&mut _offset)?;
    let tyre_dirty_level = read_wheels(&mut _offset)?;
    let tyre_core_temp = read_wheels(&mut _offset)?;
    let camber_rad = read_wheels(&mut _offset)?;
    let suspension_travel = read_wheels(&mut _offset)?;

    let drs = read_value!(i32);

    let tc = read_value!(f32);
    let heading = read_value!(f32);
    let pitch = read_value!(f32);
    let roll = read_value!(f32);
    let cg_height = read_value!(f32);

    // Car damage (5 floats)
    let damage_values = read_array!(f32, 5);
    let car_damage = CarDamage::from([
        damage_values[0],
        damage_values[1],
        damage_values[2],
        damage_values[3],
        damage_values[4],
    ]);

    let number_of_tyres_out = read_value!(i32);
    let pit_limiter_on = read_value!(i32) != 0;
    let abs = read_value!(f32);

    let kers_charge = read_value!(f32);
    let kers_input = read_value!(f32);

    let autoshifter_on = read_value!(i32) != 0;
    let ride_height = read_array!(f32, 2);
    let turbo_boost = read_value!(f32);
    let ballast = read_value!(f32);
    let air_density = read_value!(f32);
    let air_temp = read_value!(f32);
    let road_temp = read_value!(f32);
    let local_angular_vel = read_vector3f(&mut _offset)?;
    let final_ff = read_value!(f32);
    let performance_meter = read_value!(f32);

    let engine_brake = read_value!(i32);
    let ers_recovery_level = read_value!(i32);
    let ers_power_level = read_value!(i32);
    let ers_heat_charging = read_value!(i32);
    let ers_is_charging = read_value!(i32);
    let kers_current_kj = read_value!(f32);
    let drs_available = read_value!(i32);
    let drs_enabled = read_value!(i32);

    let brake_temp = read_wheels(&mut _offset)?;
    let clutch = read_value!(f32);

    let tyre_temp_i = read_wheels(&mut _offset)?;
    let tyre_temp_m = read_wheels(&mut _offset)?;
    let tyre_temp_o = read_wheels(&mut _offset)?;

    let is_ai_controlled = read_value!(i32) != 0;

    // Tyre contact points (3 sets of 4x3 arrays)
    let tyre_contact_point = read_contact_point(&mut _offset)?;
    let tyre_contact_normal = read_contact_point(&mut _offset)?;
    let tyre_contact_heading = read_contact_point(&mut _offset)?;

    let brake_bias = read_value!(f32);
    let local_velocity = read_vector3f(&mut _offset)?;

    let p2p_activation = read_value!(i32);
    let p2p_status = read_value!(i32);

    let current_max_rpm = read_value!(i32);

    let mz = read_wheels(&mut _offset)?;
    let fz = read_wheels(&mut _offset)?;
    let my = read_wheels(&mut _offset)?;

    let slip_ratio = read_wheels(&mut _offset)?;
    let slip_angle = read_wheels(&mut _offset)?;

    let tc_in_action = read_value!(i32) != 0;
    let abs_in_action = read_value!(i32) != 0;
    let suspension_damage = read_wheels(&mut _offset)?;
    let _tyre_temp = read_wheels(&mut _offset)?; // Duplicate, skip
    let water_temp = read_value!(f32);

    let brake_pressure = read_wheels(&mut _offset)?;
    let front_brake_compound = read_value!(i32);
    let rear_brake_compound = read_value!(i32);
    let pad_life = read_wheels(&mut _offset)?;
    let disc_life = read_wheels(&mut _offset)?;

    let ignition_on = read_value!(i32) != 0;
    let starter_engine_on = read_value!(i32) != 0;
    let is_engine_running = read_value!(i32) != 0;

    let kerb_vibration = read_value!(f32);
    let slip_vibration = read_value!(f32);
    let g_vibration = read_value!(f32);
    let abs_vibration = read_value!(f32);

    Ok(PhysicsMap {
        packet_id,
        gas,
        brake,
        clutch,
        steer_angle,
        gear,
        rpm,
        autoshifter_on,
        ignition_on,
        starter_engine_on,
        is_engine_running,
        speed_kmh,
        velocity,
        local_velocity,
        local_angular_vel,
        g_force,
        heading,
        pitch,
        roll,
        final_ff,
        wheel_slip,
        wheel_load,
        wheel_pressure,
        wheel_angular_speed,
        tyre_wear,
        tyre_dirty_level,
        tyre_core_temp,
        camber_rad,
        suspension_travel,
        brake_temp,
        brake_pressure,
        suspension_damage,
        slip_ratio,
        slip_angle,
        pad_life,
        disc_life,
        front_brake_compound,
        rear_brake_compound,
        tyre_temp_i,
        tyre_temp_m,
        tyre_temp_o,
        tyre_contact_point,
        tyre_contact_normal,
        tyre_contact_heading,
        fuel,
        tc,
        abs,
        pit_limiter_on,
        turbo_boost,
        air_temp,
        road_temp,
        water_temp,
        car_damage,
        is_ai_controlled,
        brake_bias,
        tc_in_action,
        abs_in_action,
        drs,
        cg_height,
        number_of_tyres_out,
        kers_charge,
        kers_input,
        ride_height_front: ride_height[0],
        ride_height_rear: ride_height[1],
        ballast,
        air_density,
        performance_meter,
        engine_brake,
        ers_recovery_level,
        ers_power_level,
        ers_heat_charging,
        ers_is_charging,
        kers_current_kj,
        drs_available,
        drs_enabled,
        p2p_activation,
        p2p_status,
        current_max_rpm,
        mz,
        fz,
        my,
        kerb_vibration,
        slip_vibration,
        g_vibration,
        abs_vibration,
    })
}
