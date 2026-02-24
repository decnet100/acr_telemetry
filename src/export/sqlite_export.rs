//! SQLite export for Grafana and offline analysis.
//!
//! Schema designed for multiple recordings and future StaticsMap support.

use std::path::Path;

use rusqlite::{params, Connection};

use crate::record::{GraphicsRecord, PhysicsRecord};

const SCHEMA: &str = r#"
-- Recording metadata (one row per session)
CREATE TABLE IF NOT EXISTS recordings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_file TEXT NOT NULL,
    created_at TEXT NOT NULL,
    duration_secs REAL NOT NULL,
    sample_count INTEGER NOT NULL
);

-- Physics samples (333 Hz)
CREATE TABLE IF NOT EXISTS physics (
    recording_id INTEGER NOT NULL,
    time_offset REAL NOT NULL,
    packet_id INTEGER,
    gas REAL, brake REAL, clutch REAL, steer_angle REAL, gear INTEGER, rpm INTEGER,
    autoshifter_on INTEGER, ignition_on INTEGER, starter_engine_on INTEGER, is_engine_running INTEGER,
    speed_kmh REAL,
    velocity_x REAL, velocity_y REAL, velocity_z REAL,
    local_velocity_x REAL, local_velocity_y REAL, local_velocity_z REAL,
    local_angular_vel_x REAL, local_angular_vel_y REAL, local_angular_vel_z REAL,
    g_force_x REAL, g_force_y REAL, g_force_z REAL,
    heading REAL, pitch REAL, roll REAL, final_ff REAL,
    wheel_slip_fl REAL, wheel_slip_fr REAL, wheel_slip_rl REAL, wheel_slip_rr REAL,
    wheel_load_fl REAL, wheel_load_fr REAL, wheel_load_rl REAL, wheel_load_rr REAL,
    wheel_pressure_fl REAL, wheel_pressure_fr REAL, wheel_pressure_rl REAL, wheel_pressure_rr REAL,
    wheel_angular_speed_fl REAL, wheel_angular_speed_fr REAL, wheel_angular_speed_rl REAL, wheel_angular_speed_rr REAL,
    tyre_wear_fl REAL, tyre_wear_fr REAL, tyre_wear_rl REAL, tyre_wear_rr REAL,
    tyre_dirty_level_fl REAL, tyre_dirty_level_fr REAL, tyre_dirty_level_rl REAL, tyre_dirty_level_rr REAL,
    tyre_core_temp_fl REAL, tyre_core_temp_fr REAL, tyre_core_temp_rl REAL, tyre_core_temp_rr REAL,
    camber_rad_fl REAL, camber_rad_fr REAL, camber_rad_rl REAL, camber_rad_rr REAL,
    suspension_travel_fl REAL, suspension_travel_fr REAL, suspension_travel_rl REAL, suspension_travel_rr REAL,
    brake_temp_fl REAL, brake_temp_fr REAL, brake_temp_rl REAL, brake_temp_rr REAL,
    brake_pressure_fl REAL, brake_pressure_fr REAL, brake_pressure_rl REAL, brake_pressure_rr REAL,
    suspension_damage_fl REAL, suspension_damage_fr REAL, suspension_damage_rl REAL, suspension_damage_rr REAL,
    slip_ratio_fl REAL, slip_ratio_fr REAL, slip_ratio_rl REAL, slip_ratio_rr REAL,
    slip_angle_fl REAL, slip_angle_fr REAL, slip_angle_rl REAL, slip_angle_rr REAL,
    pad_life_fl REAL, pad_life_fr REAL, pad_life_rl REAL, pad_life_rr REAL,
    disc_life_fl REAL, disc_life_fr REAL, disc_life_rl REAL, disc_life_rr REAL,
    front_brake_compound INTEGER, rear_brake_compound INTEGER,
    tyre_temp_i_fl REAL, tyre_temp_i_fr REAL, tyre_temp_i_rl REAL, tyre_temp_i_rr REAL,
    tyre_temp_m_fl REAL, tyre_temp_m_fr REAL, tyre_temp_m_rl REAL, tyre_temp_m_rr REAL,
    tyre_temp_o_fl REAL, tyre_temp_o_fr REAL, tyre_temp_o_rl REAL, tyre_temp_o_rr REAL,
    tyre_contact_point_fl_x REAL, tyre_contact_point_fl_y REAL, tyre_contact_point_fl_z REAL,
    tyre_contact_point_fr_x REAL, tyre_contact_point_fr_y REAL, tyre_contact_point_fr_z REAL,
    tyre_contact_point_rl_x REAL, tyre_contact_point_rl_y REAL, tyre_contact_point_rl_z REAL,
    tyre_contact_point_rr_x REAL, tyre_contact_point_rr_y REAL, tyre_contact_point_rr_z REAL,
    tyre_contact_normal_fl_x REAL, tyre_contact_normal_fl_y REAL, tyre_contact_normal_fl_z REAL,
    tyre_contact_normal_fr_x REAL, tyre_contact_normal_fr_y REAL, tyre_contact_normal_fr_z REAL,
    tyre_contact_normal_rl_x REAL, tyre_contact_normal_rl_y REAL, tyre_contact_normal_rl_z REAL,
    tyre_contact_normal_rr_x REAL, tyre_contact_normal_rr_y REAL, tyre_contact_normal_rr_z REAL,
    tyre_contact_heading_fl_x REAL, tyre_contact_heading_fl_y REAL, tyre_contact_heading_fl_z REAL,
    tyre_contact_heading_fr_x REAL, tyre_contact_heading_fr_y REAL, tyre_contact_heading_fr_z REAL,
    tyre_contact_heading_rl_x REAL, tyre_contact_heading_rl_y REAL, tyre_contact_heading_rl_z REAL,
    tyre_contact_heading_rr_x REAL, tyre_contact_heading_rr_y REAL, tyre_contact_heading_rr_z REAL,
    fuel REAL, tc REAL, abs REAL, pit_limiter_on INTEGER, turbo_boost REAL,
    air_temp REAL, road_temp REAL, water_temp REAL,
    car_damage_front REAL, car_damage_rear REAL, car_damage_left REAL, car_damage_right REAL, car_damage_center REAL,
    is_ai_controlled INTEGER, brake_bias REAL,
    tc_in_action INTEGER, abs_in_action INTEGER,
    drs INTEGER, cg_height REAL, number_of_tyres_out INTEGER,
    kers_charge REAL, kers_input REAL, ride_height_front REAL, ride_height_rear REAL,
    ballast REAL, air_density REAL, performance_meter REAL,
    engine_brake INTEGER, ers_recovery_level INTEGER, ers_power_level INTEGER,
    ers_heat_charging INTEGER, ers_is_charging INTEGER, kers_current_kj REAL,
    drs_available INTEGER, drs_enabled INTEGER, p2p_activation INTEGER, p2p_status INTEGER,
    current_max_rpm INTEGER,
    mz_fl REAL, mz_fr REAL, mz_rl REAL, mz_rr REAL,
    fz_fl REAL, fz_fr REAL, fz_rl REAL, fz_rr REAL,
    my_fl REAL, my_fr REAL, my_rl REAL, my_rr REAL,
    kerb_vibration REAL, slip_vibration REAL, g_vibration REAL, abs_vibration REAL,
    FOREIGN KEY (recording_id) REFERENCES recordings(id)
);

CREATE INDEX IF NOT EXISTS idx_physics_recording ON physics(recording_id);
CREATE INDEX IF NOT EXISTS idx_physics_time ON physics(recording_id, time_offset);

-- Statics: session/car config (one row per recording)
CREATE TABLE IF NOT EXISTS statics (
    recording_id INTEGER PRIMARY KEY,
    sm_version TEXT,
    ac_version TEXT,
    number_of_sessions INTEGER,
    num_cars INTEGER,
    track TEXT,
    sector_count INTEGER,
    player_name TEXT,
    player_surname TEXT,
    player_nick TEXT,
    car_model TEXT,
    max_rpm INTEGER,
    max_fuel REAL,
    penalty_enabled INTEGER,
    aid_fuel_rate REAL,
    aid_tyre_rate REAL,
    aid_mechanical_damage REAL,
    aid_stability REAL,
    aid_auto_clutch INTEGER,
    pit_window_start INTEGER,
    pit_window_end INTEGER,
    is_online INTEGER,
    dry_tyres_name TEXT,
    wet_tyres_name TEXT,
    FOREIGN KEY (recording_id) REFERENCES recordings(id)
);

-- Graphics samples (~60 Hz)
CREATE TABLE IF NOT EXISTS graphics (
    recording_id INTEGER NOT NULL,
    time_offset REAL NOT NULL,
    packet_id INTEGER,
    status INTEGER, session_type INTEGER, session_index INTEGER,
    current_time_str TEXT, last_time_str TEXT, best_time_str TEXT, last_sector_time_str TEXT,
    completed_lap INTEGER, position INTEGER,
    current_time INTEGER, last_time INTEGER, best_time INTEGER, last_sector_time INTEGER,
    number_of_laps INTEGER,
    delta_lap_time_str TEXT, estimated_lap_time_str TEXT,
    delta_lap_time INTEGER, estimated_lap_time INTEGER,
    is_delta_positive INTEGER, is_valid_lap INTEGER,
    fuel_estimated_laps REAL, distance_traveled REAL, normalized_car_position REAL,
    session_time_left REAL, current_sector_index INTEGER,
    is_in_pit INTEGER, is_in_pit_lane INTEGER, ideal_line_on INTEGER,
    mandatory_pit_done INTEGER, missing_mandatory_pits INTEGER,
    penalty_time REAL, penalty INTEGER, flag INTEGER,
    player_car_id INTEGER, active_cars INTEGER,
    car_coordinates_x REAL, car_coordinates_y REAL, car_coordinates_z REAL,
    wind_speed REAL, wind_direction REAL,
    rain_intensity INTEGER, rain_intensity_in_10min INTEGER, rain_intensity_in_30min INTEGER,
    track_grip_status INTEGER, track_status TEXT, clock REAL,
    tc_level INTEGER, tc_cut_level INTEGER, engine_map INTEGER, abs_level INTEGER,
    wiper_stage INTEGER, driver_stint_total_time_left INTEGER, driver_stint_time_left INTEGER,
    rain_tyres INTEGER,
    rain_light INTEGER, flashing_light INTEGER, light_stage INTEGER,
    direction_light_left INTEGER, direction_light_right INTEGER,
    tyre_compound TEXT, is_setup_menu_visible INTEGER,
    main_display_index INTEGER, secondary_display_index INTEGER,
    fuel_per_lap REAL, used_fuel REAL, exhaust_temp REAL,
    gap_ahead INTEGER, gap_behind INTEGER,
    global_yellow INTEGER, global_yellow_s1 INTEGER, global_yellow_s2 INTEGER, global_yellow_s3 INTEGER,
    global_white INTEGER, global_green INTEGER, global_chequered INTEGER, global_red INTEGER,
    mfd_tyre_set INTEGER, mfd_fuel_to_add REAL,
    mfd_tyre_pressure_fl REAL, mfd_tyre_pressure_fr REAL, mfd_tyre_pressure_rl REAL, mfd_tyre_pressure_rr REAL,
    current_tyre_set INTEGER, strategy_tyre_set INTEGER,
    FOREIGN KEY (recording_id) REFERENCES recordings(id)
);

CREATE INDEX IF NOT EXISTS idx_graphics_recording ON graphics(recording_id);
CREATE INDEX IF NOT EXISTS idx_graphics_time ON graphics(recording_id, time_offset);

-- User-editable notes and experiment metadata (one row per recording; all TEXT, mostly empty)
CREATE TABLE IF NOT EXISTS recording_notes (
    recording_id INTEGER PRIMARY KEY,
    notes TEXT,
    laptime TEXT,
    result TEXT,
    driver_impression TEXT,
    tested_parameters TEXT,
    conditions TEXT,
    setup_notes TEXT,
    session_goal TEXT,
    incident TEXT,
    FOREIGN KEY (recording_id) REFERENCES recordings(id)
);

-- Annotations for Grafana (point or range). time_offset_sec aligns with physics time_offset; Grafana uses (1000000000 + time_offset_sec)*1000 for time column.
CREATE TABLE IF NOT EXISTS annotations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recording_id INTEGER NOT NULL,
    time_offset_sec REAL NOT NULL,
    time_end_sec REAL,
    text TEXT NOT NULL,
    tag TEXT,
    FOREIGN KEY (recording_id) REFERENCES recordings(id)
);
CREATE INDEX IF NOT EXISTS idx_annotations_recording ON annotations(recording_id);
"#;

/// Check if a recording with the given source_file is already in the database.
pub fn recording_exists(db_path: impl AsRef<Path>, source_file: &str) -> rusqlite::Result<bool> {
    if !db_path.as_ref().exists() {
        return Ok(false);
    }
    let conn = Connection::open(db_path)?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM recordings WHERE source_file = ?1",
        params![source_file],
        |r| r.get(0),
    )?;
    Ok(count > 0)
}

/// Optional content for recording_notes columns (from .notes and .notes_<field> sidecars).
#[derive(Default)]
pub struct RecordingNotesContent {
    pub notes: Option<String>,
    pub laptime: Option<String>,
    pub result: Option<String>,
    pub driver_impression: Option<String>,
    pub tested_parameters: Option<String>,
    pub conditions: Option<String>,
    pub setup_notes: Option<String>,
    pub session_goal: Option<String>,
    pub incident: Option<String>,
}

/// Export physics records to SQLite. Appends to existing db or creates it.
/// Returns the recording_id for the new session.
/// `notes_content`: optional content for recording_notes (from .notes.json).
/// `annotations`: optional annotations for Grafana (from .notes.json); inserted into `annotations` table.
pub fn export_to_sqlite(
    db_path: impl AsRef<Path>,
    source_file: &str,
    records: &[PhysicsRecord],
    sample_rate_hz: u32,
    statics: Option<&crate::record::StaticsRecord>,
    notes_content: Option<&RecordingNotesContent>,
    annotations: Option<&[crate::notes::Annotation]>,
) -> rusqlite::Result<i64> {
    let mut conn = Connection::open(db_path)?;
    conn.execute_batch(SCHEMA)?;

    let dt = 1.0 / sample_rate_hz as f64;
    let duration_secs = records.len() as f64 * dt;
    let created_at = format_timestamp();

    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO recordings (source_file, created_at, duration_secs, sample_count) VALUES (?1, ?2, ?3, ?4)",
        params![source_file, created_at, duration_secs, records.len()],
    )?;
    let recording_id = tx.last_insert_rowid();

    let (notes, laptime, result, driver_impression, tested_parameters, conditions, setup_notes, session_goal, incident) =
        notes_content
            .map(|n| {
                (
                    n.notes.as_deref(),
                    n.laptime.as_deref(),
                    n.result.as_deref(),
                    n.driver_impression.as_deref(),
                    n.tested_parameters.as_deref(),
                    n.conditions.as_deref(),
                    n.setup_notes.as_deref(),
                    n.session_goal.as_deref(),
                    n.incident.as_deref(),
                )
            })
            .unwrap_or((None, None, None, None, None, None, None, None, None));

    tx.execute(
        r#"INSERT INTO recording_notes (recording_id, notes, laptime, result, driver_impression, tested_parameters, conditions, setup_notes, session_goal, incident)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
        params![
            recording_id, notes, laptime, result, driver_impression, tested_parameters, conditions, setup_notes,
            session_goal, incident
        ],
    )?;

    if let Some(ann) = annotations {
        let mut stmt_ann = tx.prepare(
            "INSERT INTO annotations (recording_id, time_offset_sec, time_end_sec, text, tag) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for a in ann.iter() {
            stmt_ann.execute(params![
                recording_id,
                a.time_offset_sec,
                a.time_end_sec,
                a.text,
                a.tag,
            ])?;
        }
    }

    let mut stmt = tx.prepare(
        r#"
        INSERT INTO physics (
            recording_id, time_offset, packet_id, gas, brake, clutch, steer_angle, gear, rpm,
            autoshifter_on, ignition_on, starter_engine_on, is_engine_running,
            speed_kmh, velocity_x, velocity_y, velocity_z,
            local_velocity_x, local_velocity_y, local_velocity_z,
            local_angular_vel_x, local_angular_vel_y, local_angular_vel_z,
            g_force_x, g_force_y, g_force_z, heading, pitch, roll, final_ff,
            wheel_slip_fl, wheel_slip_fr, wheel_slip_rl, wheel_slip_rr,
            wheel_load_fl, wheel_load_fr, wheel_load_rl, wheel_load_rr,
            wheel_pressure_fl, wheel_pressure_fr, wheel_pressure_rl, wheel_pressure_rr,
            wheel_angular_speed_fl, wheel_angular_speed_fr, wheel_angular_speed_rl, wheel_angular_speed_rr,
            tyre_wear_fl, tyre_wear_fr, tyre_wear_rl, tyre_wear_rr,
            tyre_dirty_level_fl, tyre_dirty_level_fr, tyre_dirty_level_rl, tyre_dirty_level_rr,
            tyre_core_temp_fl, tyre_core_temp_fr, tyre_core_temp_rl, tyre_core_temp_rr,
            camber_rad_fl, camber_rad_fr, camber_rad_rl, camber_rad_rr,
            suspension_travel_fl, suspension_travel_fr, suspension_travel_rl, suspension_travel_rr,
            brake_temp_fl, brake_temp_fr, brake_temp_rl, brake_temp_rr,
            brake_pressure_fl, brake_pressure_fr, brake_pressure_rl, brake_pressure_rr,
            suspension_damage_fl, suspension_damage_fr, suspension_damage_rl, suspension_damage_rr,
            slip_ratio_fl, slip_ratio_fr, slip_ratio_rl, slip_ratio_rr,
            slip_angle_fl, slip_angle_fr, slip_angle_rl, slip_angle_rr,
            pad_life_fl, pad_life_fr, pad_life_rl, pad_life_rr,
            disc_life_fl, disc_life_fr, disc_life_rl, disc_life_rr,
            front_brake_compound, rear_brake_compound,
            tyre_temp_i_fl, tyre_temp_i_fr, tyre_temp_i_rl, tyre_temp_i_rr,
            tyre_temp_m_fl, tyre_temp_m_fr, tyre_temp_m_rl, tyre_temp_m_rr,
            tyre_temp_o_fl, tyre_temp_o_fr, tyre_temp_o_rl, tyre_temp_o_rr,
            tyre_contact_point_fl_x, tyre_contact_point_fl_y, tyre_contact_point_fl_z,
            tyre_contact_point_fr_x, tyre_contact_point_fr_y, tyre_contact_point_fr_z,
            tyre_contact_point_rl_x, tyre_contact_point_rl_y, tyre_contact_point_rl_z,
            tyre_contact_point_rr_x, tyre_contact_point_rr_y, tyre_contact_point_rr_z,
            tyre_contact_normal_fl_x, tyre_contact_normal_fl_y, tyre_contact_normal_fl_z,
            tyre_contact_normal_fr_x, tyre_contact_normal_fr_y, tyre_contact_normal_fr_z,
            tyre_contact_normal_rl_x, tyre_contact_normal_rl_y, tyre_contact_normal_rl_z,
            tyre_contact_normal_rr_x, tyre_contact_normal_rr_y, tyre_contact_normal_rr_z,
            tyre_contact_heading_fl_x, tyre_contact_heading_fl_y, tyre_contact_heading_fl_z,
            tyre_contact_heading_fr_x, tyre_contact_heading_fr_y, tyre_contact_heading_fr_z,
            tyre_contact_heading_rl_x, tyre_contact_heading_rl_y, tyre_contact_heading_rl_z,
            tyre_contact_heading_rr_x, tyre_contact_heading_rr_y, tyre_contact_heading_rr_z,
            fuel, tc, abs, pit_limiter_on, turbo_boost, air_temp, road_temp, water_temp,
            car_damage_front, car_damage_rear, car_damage_left, car_damage_right, car_damage_center,
            is_ai_controlled, brake_bias,
            tc_in_action, abs_in_action,
            drs, cg_height, number_of_tyres_out,
            kers_charge, kers_input, ride_height_front, ride_height_rear,
            ballast, air_density, performance_meter,
            engine_brake, ers_recovery_level, ers_power_level,
            ers_heat_charging, ers_is_charging, kers_current_kj,
            drs_available, drs_enabled, p2p_activation, p2p_status,
            current_max_rpm,
            mz_fl, mz_fr, mz_rl, mz_rr,
            fz_fl, fz_fr, fz_rl, fz_rr,
            my_fl, my_fr, my_rl, my_rr,
            kerb_vibration, slip_vibration, g_vibration, abs_vibration
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
            ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34,
            ?35, ?36, ?37, ?38, ?39, ?40, ?41, ?42, ?43, ?44, ?45, ?46, ?47, ?48, ?49, ?50, ?51,
            ?52, ?53, ?54, ?55, ?56, ?57, ?58, ?59, ?60, ?61, ?62, ?63, ?64, ?65, ?66, ?67,
            ?68, ?69, ?70, ?71, ?72, ?73, ?74, ?75, ?76, ?77, ?78, ?79, ?80, ?81, ?82, ?83,
            ?84, ?85, ?86, ?87, ?88, ?89, ?90, ?91, ?92, ?93, ?94, ?95, ?96, ?97, ?98, ?99,
            ?100, ?101, ?102, ?103, ?104, ?105, ?106, ?107, ?108, ?109, ?110, ?111, ?112, ?113,
            ?114, ?115, ?116, ?117, ?118, ?119, ?120, ?121, ?122, ?123, ?124, ?125, ?126,
            ?127, ?128, ?129, ?130, ?131, ?132, ?133, ?134, ?135, ?136, ?137, ?138, ?139, ?140,
            ?141, ?142, ?143, ?144, ?145, ?146, ?147, ?148, ?149, ?150, ?151, ?152, ?153, ?154,
            ?155, ?156, ?157, ?158, ?159, ?160, ?161, ?162, ?163, ?164, ?165, ?166, ?167, ?168,
            ?169, ?170, ?171, ?172, ?173, ?174, ?175, ?176, ?177, ?178, ?179, ?180, ?181, ?182,
            ?183, ?184, ?185, ?186, ?187, ?188, ?189, ?190, ?191, ?192, ?193, ?194, ?195, ?196,
            ?197, ?198
        )
        "#,
    )?;

    let b = |v: bool| if v { 1i32 } else { 0i32 };

    for (i, r) in records.iter().enumerate() {
        let time_offset = i as f64 * dt;
        stmt.execute(params![
            recording_id,
            time_offset,
            r.packet_id,
            r.gas,
            r.brake,
            r.clutch,
            r.steer_angle,
            r.gear,
            r.rpm,
            b(r.autoshifter_on),
            b(r.ignition_on),
            b(r.starter_engine_on),
            b(r.is_engine_running),
            r.speed_kmh,
            r.velocity.x,
            r.velocity.y,
            r.velocity.z,
            r.local_velocity.x,
            r.local_velocity.y,
            r.local_velocity.z,
            r.local_angular_vel.x,
            r.local_angular_vel.y,
            r.local_angular_vel.z,
            r.g_force.x,
            r.g_force.y,
            r.g_force.z,
            r.heading,
            r.pitch,
            r.roll,
            r.final_ff,
            r.wheel_slip.front_left,
            r.wheel_slip.front_right,
            r.wheel_slip.rear_left,
            r.wheel_slip.rear_right,
            r.wheel_load.front_left,
            r.wheel_load.front_right,
            r.wheel_load.rear_left,
            r.wheel_load.rear_right,
            r.wheel_pressure.front_left,
            r.wheel_pressure.front_right,
            r.wheel_pressure.rear_left,
            r.wheel_pressure.rear_right,
            r.wheel_angular_speed.front_left,
            r.wheel_angular_speed.front_right,
            r.wheel_angular_speed.rear_left,
            r.wheel_angular_speed.rear_right,
            r.tyre_wear.front_left,
            r.tyre_wear.front_right,
            r.tyre_wear.rear_left,
            r.tyre_wear.rear_right,
            r.tyre_dirty_level.front_left,
            r.tyre_dirty_level.front_right,
            r.tyre_dirty_level.rear_left,
            r.tyre_dirty_level.rear_right,
            r.tyre_core_temp.front_left,
            r.tyre_core_temp.front_right,
            r.tyre_core_temp.rear_left,
            r.tyre_core_temp.rear_right,
            r.camber_rad.front_left,
            r.camber_rad.front_right,
            r.camber_rad.rear_left,
            r.camber_rad.rear_right,
            r.suspension_travel.front_left,
            r.suspension_travel.front_right,
            r.suspension_travel.rear_left,
            r.suspension_travel.rear_right,
            r.brake_temp.front_left,
            r.brake_temp.front_right,
            r.brake_temp.rear_left,
            r.brake_temp.rear_right,
            r.brake_pressure.front_left,
            r.brake_pressure.front_right,
            r.brake_pressure.rear_left,
            r.brake_pressure.rear_right,
            r.suspension_damage.front_left,
            r.suspension_damage.front_right,
            r.suspension_damage.rear_left,
            r.suspension_damage.rear_right,
            r.slip_ratio.front_left,
            r.slip_ratio.front_right,
            r.slip_ratio.rear_left,
            r.slip_ratio.rear_right,
            r.slip_angle.front_left,
            r.slip_angle.front_right,
            r.slip_angle.rear_left,
            r.slip_angle.rear_right,
            r.pad_life.front_left,
            r.pad_life.front_right,
            r.pad_life.rear_left,
            r.pad_life.rear_right,
            r.disc_life.front_left,
            r.disc_life.front_right,
            r.disc_life.rear_left,
            r.disc_life.rear_right,
            r.front_brake_compound,
            r.rear_brake_compound,
            r.tyre_temp_i.front_left,
            r.tyre_temp_i.front_right,
            r.tyre_temp_i.rear_left,
            r.tyre_temp_i.rear_right,
            r.tyre_temp_m.front_left,
            r.tyre_temp_m.front_right,
            r.tyre_temp_m.rear_left,
            r.tyre_temp_m.rear_right,
            r.tyre_temp_o.front_left,
            r.tyre_temp_o.front_right,
            r.tyre_temp_o.rear_left,
            r.tyre_temp_o.rear_right,
            r.tyre_contact_point.front_left.x,
            r.tyre_contact_point.front_left.y,
            r.tyre_contact_point.front_left.z,
            r.tyre_contact_point.front_right.x,
            r.tyre_contact_point.front_right.y,
            r.tyre_contact_point.front_right.z,
            r.tyre_contact_point.rear_left.x,
            r.tyre_contact_point.rear_left.y,
            r.tyre_contact_point.rear_left.z,
            r.tyre_contact_point.rear_right.x,
            r.tyre_contact_point.rear_right.y,
            r.tyre_contact_point.rear_right.z,
            r.tyre_contact_normal.front_left.x,
            r.tyre_contact_normal.front_left.y,
            r.tyre_contact_normal.front_left.z,
            r.tyre_contact_normal.front_right.x,
            r.tyre_contact_normal.front_right.y,
            r.tyre_contact_normal.front_right.z,
            r.tyre_contact_normal.rear_left.x,
            r.tyre_contact_normal.rear_left.y,
            r.tyre_contact_normal.rear_left.z,
            r.tyre_contact_normal.rear_right.x,
            r.tyre_contact_normal.rear_right.y,
            r.tyre_contact_normal.rear_right.z,
            r.tyre_contact_heading.front_left.x,
            r.tyre_contact_heading.front_left.y,
            r.tyre_contact_heading.front_left.z,
            r.tyre_contact_heading.front_right.x,
            r.tyre_contact_heading.front_right.y,
            r.tyre_contact_heading.front_right.z,
            r.tyre_contact_heading.rear_left.x,
            r.tyre_contact_heading.rear_left.y,
            r.tyre_contact_heading.rear_left.z,
            r.tyre_contact_heading.rear_right.x,
            r.tyre_contact_heading.rear_right.y,
            r.tyre_contact_heading.rear_right.z,
            r.fuel,
            r.tc,
            r.abs,
            b(r.pit_limiter_on),
            r.turbo_boost,
            r.air_temp,
            r.road_temp,
            r.water_temp,
            r.car_damage.front,
            r.car_damage.rear,
            r.car_damage.left,
            r.car_damage.right,
            r.car_damage.center,
            b(r.is_ai_controlled),
            r.brake_bias,
            b(r.tc_in_action),
            b(r.abs_in_action),
            r.drs,
            r.cg_height,
            r.number_of_tyres_out,
            r.kers_charge,
            r.kers_input,
            r.ride_height_front,
            r.ride_height_rear,
            r.ballast,
            r.air_density,
            r.performance_meter,
            r.engine_brake,
            r.ers_recovery_level,
            r.ers_power_level,
            r.ers_heat_charging,
            r.ers_is_charging,
            r.kers_current_kj,
            r.drs_available,
            r.drs_enabled,
            r.p2p_activation,
            r.p2p_status,
            r.current_max_rpm,
            r.mz.front_left,
            r.mz.front_right,
            r.mz.rear_left,
            r.mz.rear_right,
            r.fz.front_left,
            r.fz.front_right,
            r.fz.rear_left,
            r.fz.rear_right,
            r.my.front_left,
            r.my.front_right,
            r.my.rear_left,
            r.my.rear_right,
            r.kerb_vibration,
            r.slip_vibration,
            r.g_vibration,
            r.abs_vibration,
        ])?;
    }
    drop(stmt);

    // Insert statics if available
    if let Some(s) = statics {
        let b = |v: bool| if v { 1i32 } else { 0i32 };
        tx.execute(
            r#"INSERT INTO statics (
                recording_id, sm_version, ac_version, number_of_sessions, num_cars, track, sector_count,
                player_name, player_surname, player_nick, car_model, max_rpm, max_fuel,
                penalty_enabled, aid_fuel_rate, aid_tyre_rate, aid_mechanical_damage, aid_stability, aid_auto_clutch,
                pit_window_start, pit_window_end, is_online, dry_tyres_name, wet_tyres_name
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24)"#,
            params![
                recording_id,
                s.sm_version,
                s.ac_version,
                s.number_of_sessions,
                s.num_cars,
                s.track,
                s.sector_count,
                s.player_name,
                s.player_surname,
                s.player_nick,
                s.car_model,
                s.max_rpm,
                s.max_fuel,
                b(s.penalty_enabled),
                s.aid_fuel_rate,
                s.aid_tyre_rate,
                s.aid_mechanical_damage,
                s.aid_stability,
                b(s.aid_auto_clutch),
                s.pit_window_start,
                s.pit_window_end,
                b(s.is_online),
                s.dry_tyres_name,
                s.wet_tyres_name,
            ],
        )?;
    } else {
        tx.execute("INSERT INTO statics (recording_id) VALUES (?1)", [recording_id])?;
    }

    tx.commit()?;
    Ok(recording_id)
}

fn format_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Export graphics records to SQLite. Appends to existing recording.
pub fn export_graphics_to_sqlite(
    db_path: impl AsRef<Path>,
    recording_id: i64,
    graphics_records: &[GraphicsRecord],
    sample_rate_hz: u32,
) -> rusqlite::Result<()> {
    let mut conn = Connection::open(db_path)?;
    let dt = 1.0 / sample_rate_hz as f64;
    
    let tx = conn.transaction()?;
    
    let mut stmt = tx.prepare(
        r#"
        INSERT INTO graphics (
            recording_id, time_offset, packet_id, status, session_type, session_index,
            current_time_str, last_time_str, best_time_str, last_sector_time_str,
            completed_lap, position,
            current_time, last_time, best_time, last_sector_time, number_of_laps,
            delta_lap_time_str, estimated_lap_time_str,
            delta_lap_time, estimated_lap_time,
            is_delta_positive, is_valid_lap,
            fuel_estimated_laps, distance_traveled, normalized_car_position,
            session_time_left, current_sector_index,
            is_in_pit, is_in_pit_lane, ideal_line_on,
            mandatory_pit_done, missing_mandatory_pits,
            penalty_time, penalty, flag,
            player_car_id, active_cars,
            car_coordinates_x, car_coordinates_y, car_coordinates_z,
            wind_speed, wind_direction,
            rain_intensity, rain_intensity_in_10min, rain_intensity_in_30min,
            track_grip_status, track_status, clock,
            tc_level, tc_cut_level, engine_map, abs_level,
            wiper_stage, driver_stint_total_time_left, driver_stint_time_left,
            rain_tyres,
            rain_light, flashing_light, light_stage,
            direction_light_left, direction_light_right,
            tyre_compound, is_setup_menu_visible,
            main_display_index, secondary_display_index,
            fuel_per_lap, used_fuel, exhaust_temp,
            gap_ahead, gap_behind,
            global_yellow, global_yellow_s1, global_yellow_s2, global_yellow_s3,
            global_white, global_green, global_chequered, global_red,
            mfd_tyre_set, mfd_fuel_to_add,
            mfd_tyre_pressure_fl, mfd_tyre_pressure_fr, mfd_tyre_pressure_rl, mfd_tyre_pressure_rr,
            current_tyre_set, strategy_tyre_set
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
            ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
            ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30,
            ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39, ?40,
            ?41, ?42, ?43, ?44, ?45, ?46, ?47, ?48, ?49, ?50,
            ?51, ?52, ?53, ?54, ?55, ?56, ?57, ?58, ?59, ?60,
            ?61, ?62, ?63, ?64, ?65, ?66, ?67, ?68, ?69, ?70,
            ?71, ?72, ?73, ?74, ?75, ?76, ?77, ?78, ?79, ?80,
            ?81, ?82, ?83, ?84, ?85, ?86, ?87
        )
        "#,
    )?;

    let b = |x: bool| if x { 1 } else { 0 };

    for (i, r) in graphics_records.iter().enumerate() {
        let time_offset = i as f64 * dt;
        stmt.execute(params![
            recording_id,
            time_offset,
            r.packet_id,
            r.status,
            r.session_type,
            r.session_index,
            r.current_time_str,
            r.last_time_str,
            r.best_time_str,
            r.last_sector_time_str,
            r.completed_lap,
            r.position,
            r.current_time,
            r.last_time,
            r.best_time,
            r.last_sector_time,
            r.number_of_laps,
            r.delta_lap_time_str,
            r.estimated_lap_time_str,
            r.delta_lap_time,
            r.estimated_lap_time,
            b(r.is_delta_positive),
            b(r.is_valid_lap),
            r.fuel_estimated_laps,
            r.distance_traveled,
            r.normalized_car_position,
            r.session_time_left,
            r.current_sector_index,
            b(r.is_in_pit),
            b(r.is_in_pit_lane),
            b(r.ideal_line_on),
            b(r.mandatory_pit_done),
            r.missing_mandatory_pits,
            r.penalty_time,
            r.penalty,
            r.flag,
            r.player_car_id,
            r.active_cars,
            r.car_coordinates_x,
            r.car_coordinates_y,
            r.car_coordinates_z,
            r.wind_speed,
            r.wind_direction,
            r.rain_intensity,
            r.rain_intensity_in_10min,
            r.rain_intensity_in_30min,
            r.track_grip_status,
            r.track_status.clone(),
            r.clock,
            r.tc_level,
            r.tc_cut_level,
            r.engine_map,
            r.abs_level,
            r.wiper_stage,
            r.driver_stint_total_time_left,
            r.driver_stint_time_left,
            b(r.rain_tyres),
            b(r.rain_light),
            b(r.flashing_light),
            r.light_stage,
            b(r.direction_light_left),
            b(r.direction_light_right),
            r.tyre_compound.clone(),
            b(r.is_setup_menu_visible),
            r.main_display_index,
            r.secondary_display_index,
            r.fuel_per_lap,
            r.used_fuel,
            r.exhaust_temp,
            r.gap_ahead,
            r.gap_behind,
            b(r.global_yellow),
            b(r.global_yellow_s1),
            b(r.global_yellow_s2),
            b(r.global_yellow_s3),
            b(r.global_white),
            b(r.global_green),
            b(r.global_chequered),
            b(r.global_red),
            r.mfd_tyre_set,
            r.mfd_fuel_to_add,
            r.mfd_tyre_pressure_fl,
            r.mfd_tyre_pressure_fr,
            r.mfd_tyre_pressure_rl,
            r.mfd_tyre_pressure_rr,
            r.current_tyre_set,
            r.strategy_tyre_set,
        ])?;
    }
    
    drop(stmt);
    tx.commit()?;
    Ok(())
}
