//! Export physics segments to analysis.db based on Grafana annotations.
//!
//! Reads annotations from grafana.db (tag rid_<recording_id>), slices physics/graphics
//! to those time ranges, copies recordings/statics, and writes to analysis.db.
//! Creates backup of analysis.db before overwriting.
//!
//! Usage:
//!   acr_analysis_export <recording_id> [--grafana-db PATH] [--telemetry-db PATH] [--analysis-db PATH]
//!   acr_analysis_export --serve [--port PORT] [--grafana-db PATH] [--telemetry-db PATH] [--analysis-db PATH]
//!
//! --serve: HTTP on /export?recording_id=X. analysis.db default: same dir as telemetry.db.

use std::fs;
use std::path::Path;

use acr_recorder::config;
use rusqlite::Connection;
use tiny_http::{Response, Server};

const ANALYSIS_DB_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS recordings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_file TEXT NOT NULL,
    created_at TEXT NOT NULL,
    duration_secs REAL NOT NULL,
    sample_count INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS statics (
    recording_id INTEGER PRIMARY KEY,
    sm_version TEXT, ac_version TEXT, number_of_sessions INTEGER, num_cars INTEGER,
    track TEXT, sector_count INTEGER, player_name TEXT, player_surname TEXT, player_nick TEXT,
    car_model TEXT, max_rpm INTEGER, max_fuel REAL, penalty_enabled INTEGER,
    aid_fuel_rate REAL, aid_tyre_rate REAL, aid_mechanical_damage REAL, aid_stability REAL,
    aid_auto_clutch INTEGER, pit_window_start INTEGER, pit_window_end INTEGER, is_online INTEGER,
    dry_tyres_name TEXT, wet_tyres_name TEXT,
    FOREIGN KEY (recording_id) REFERENCES recordings(id)
);

CREATE TABLE IF NOT EXISTS graphics (
    recording_id INTEGER NOT NULL, time_offset REAL NOT NULL, packet_id INTEGER,
    status INTEGER, session_type INTEGER, session_index INTEGER,
    current_time_str TEXT, last_time_str TEXT, best_time_str TEXT, last_sector_time_str TEXT,
    completed_lap INTEGER, position INTEGER,
    current_time INTEGER, last_time INTEGER, best_time INTEGER, last_sector_time INTEGER,
    number_of_laps INTEGER, delta_lap_time_str TEXT, estimated_lap_time_str TEXT,
    delta_lap_time INTEGER, estimated_lap_time INTEGER, is_delta_positive INTEGER, is_valid_lap INTEGER,
    fuel_estimated_laps REAL, distance_traveled REAL, normalized_car_position REAL,
    session_time_left REAL, current_sector_index INTEGER,
    is_in_pit INTEGER, is_in_pit_lane INTEGER, ideal_line_on INTEGER,
    mandatory_pit_done INTEGER, missing_mandatory_pits INTEGER, penalty_time REAL, penalty INTEGER, flag INTEGER,
    player_car_id INTEGER, active_cars INTEGER,
    car_coordinates_x REAL, car_coordinates_y REAL, car_coordinates_z REAL,
    wind_speed REAL, wind_direction REAL,
    rain_intensity INTEGER, rain_intensity_in_10min INTEGER, rain_intensity_in_30min INTEGER,
    track_grip_status INTEGER, track_status TEXT, clock REAL,
    tc_level INTEGER, tc_cut_level INTEGER, engine_map INTEGER, abs_level INTEGER,
    wiper_stage INTEGER, driver_stint_total_time_left INTEGER, driver_stint_time_left INTEGER,
    rain_tyres INTEGER, rain_light INTEGER, flashing_light INTEGER, light_stage INTEGER,
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

CREATE TABLE IF NOT EXISTS physics (
    recording_id INTEGER NOT NULL, time_offset REAL NOT NULL, packet_id INTEGER,
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
    annotation_id INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_physics_recording ON physics(recording_id);
CREATE INDEX IF NOT EXISTS idx_physics_annotation ON physics(annotation_id);
CREATE INDEX IF NOT EXISTS idx_physics_time ON physics(recording_id, time_offset);

CREATE TABLE IF NOT EXISTS annotation (
    id INTEGER PRIMARY KEY, org_id INTEGER NOT NULL, alert_id INTEGER, user_id INTEGER,
    dashboard_id INTEGER, panel_id INTEGER, category_id INTEGER, type TEXT NOT NULL, title TEXT NOT NULL,
    text TEXT NOT NULL, metric TEXT, prev_state TEXT NOT NULL, new_state TEXT NOT NULL, data TEXT NOT NULL,
    epoch INTEGER NOT NULL, region_id INTEGER, tags TEXT, created INTEGER, updated INTEGER,
    epoch_end INTEGER NOT NULL, dashboard_uid TEXT
);
CREATE TABLE IF NOT EXISTS tag (id INTEGER PRIMARY KEY, key TEXT NOT NULL, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS annotation_tag (
    id INTEGER PRIMARY KEY, annotation_id INTEGER NOT NULL, tag_id INTEGER NOT NULL,
    FOREIGN KEY (annotation_id) REFERENCES annotation(id), FOREIGN KEY (tag_id) REFERENCES tag(id)
);
CREATE INDEX IF NOT EXISTS idx_graphics_recording ON graphics(recording_id);
CREATE INDEX IF NOT EXISTS idx_graphics_time ON graphics(recording_id, time_offset);
"#;

fn epoch_ms_to_offset(epoch_ms: i64) -> f64 {
    (epoch_ms as f64 / 1000.0) - 1_000_000_000.0
}

/// Read annotations from grafana.db with tag rid_<recording_id>.
/// Grafana schema uses epoch (and epoch_end for regions).
fn read_grafana_annotations(grafana_db: &Path, recording_id: i64) -> rusqlite::Result<Vec<(i64, f64, f64)>> {
    let conn = Connection::open(grafana_db)?;
    let tag = format!("rid_{}", recording_id);

    // Grafana: a.epoch, a.epoch_end; tag.key (not term) for tag string
    let mut stmt = conn.prepare(
        "SELECT a.id, a.epoch, COALESCE(a.epoch_end, a.epoch)
         FROM annotation a
         JOIN annotation_tag at ON a.id = at.annotation_id
         JOIN tag t ON at.tag_id = t.id
         WHERE t.key = ?1
         ORDER BY a.epoch",
    )?;

    let mut out = Vec::new();
    let mut iter = stmt.query(rusqlite::params![tag])?;
    while let Some(row) = iter.next()? {
        let id: i64 = row.get(0)?;
        let start_ms: i64 = row.get(1)?;
        let end_ms: i64 = row.get(2)?;
        out.push((id, epoch_ms_to_offset(start_ms), epoch_ms_to_offset(end_ms)));
    }
    Ok(out)
}

fn run_export(
    recording_id: i64,
    grafana_db: &Path,
    telemetry_db: &Path,
    analysis_db: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    if !grafana_db.exists() {
        return Err(format!("grafana.db not found: {}", grafana_db.display()).into());
    }
    if !telemetry_db.exists() {
        return Err(format!("telemetry.db not found: {}", telemetry_db.display()).into());
    }

    let ranges = read_grafana_annotations(grafana_db, recording_id)?;
    if ranges.is_empty() {
        // Clear analysis.db for this recording so stale data doesn't remain
        if analysis_db.exists() {
            let conn = Connection::open(analysis_db)?;
            conn.execute_batch(ANALYSIS_DB_SCHEMA)?;
            conn.execute("DELETE FROM physics WHERE recording_id = ?1", rusqlite::params![recording_id])?;
            conn.execute("DELETE FROM graphics WHERE recording_id = ?1", rusqlite::params![recording_id])?;
            conn.execute("DELETE FROM statics WHERE recording_id = ?1", rusqlite::params![recording_id])?;
            conn.execute("DELETE FROM recordings WHERE id = ?1", rusqlite::params![recording_id])?;
            conn.execute("DELETE FROM annotation_tag", [])?;
            conn.execute("DELETE FROM annotation", [])?;
            conn.execute("DELETE FROM tag", [])?;
        }
        return Ok(format!("OK: No annotations with tag rid_{} – analysis.db cleared for recording {}", recording_id, recording_id));
    }

    // Backup analysis.db before overwrite
    let backup_path = analysis_db.with_extension("db.bak");
    if analysis_db.exists() {
        fs::copy(analysis_db, &backup_path)?;
    }

    let mut conn = Connection::open(analysis_db)?;
    conn.execute_batch(ANALYSIS_DB_SCHEMA)?;

    let telemetry_path = telemetry_db.canonicalize().unwrap_or_else(|_| telemetry_db.to_path_buf());
    let telemetry_str = telemetry_path.to_str().ok_or("telemetry path invalid")?;

    let grafana_path = grafana_db.canonicalize().unwrap_or_else(|_| grafana_db.to_path_buf());
    let grafana_str = grafana_path.to_str().ok_or("grafana path invalid")?;

    conn.execute("ATTACH DATABASE ?1 AS src", rusqlite::params![telemetry_str])?;
    conn.execute("ATTACH DATABASE ?1 AS grafana", rusqlite::params![grafana_str])?;

    let ann_ids: Vec<i64> = ranges.iter().map(|(id, _, _)| *id).collect();
    let placeholders = ann_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    let tx = conn.transaction()?;

    // Clear existing data for this recording (in case of re-run)
    tx.execute("DELETE FROM physics WHERE recording_id = ?1", rusqlite::params![recording_id])?;
    tx.execute("DELETE FROM graphics WHERE recording_id = ?1", rusqlite::params![recording_id])?;
    tx.execute("DELETE FROM statics WHERE recording_id = ?1", rusqlite::params![recording_id])?;
    tx.execute("DELETE FROM recordings WHERE id = ?1", rusqlite::params![recording_id])?;

    // Clear annotation/annotation_tag/tag (we replace with current export)
    tx.execute("DELETE FROM annotation_tag", [])?;
    tx.execute("DELETE FROM annotation", [])?;
    tx.execute("DELETE FROM tag", [])?;

    // Copy tag, annotation, annotation_tag from grafana.db (only for our annotations)
    let tag_ids: Vec<i64> = {
        let mut tag_ids = Vec::new();
        for id in &ann_ids {
            let mut stmt = tx.prepare("SELECT tag_id FROM grafana.annotation_tag WHERE annotation_id = ?1")?;
            let mut rows = stmt.query(rusqlite::params![id])?;
            while let Some(row) = rows.next()? {
                tag_ids.push(row.get::<_, i64>(0)?);
            }
        }
        tag_ids.sort();
        tag_ids.dedup();
        tag_ids
    };
    for tid in &tag_ids {
        tx.execute("INSERT OR IGNORE INTO main.tag SELECT * FROM grafana.tag WHERE id = ?1", rusqlite::params![tid])?;
    }
    tx.execute(
        &format!("INSERT INTO main.annotation SELECT * FROM grafana.annotation WHERE id IN ({})", placeholders),
        rusqlite::params_from_iter(ann_ids.iter().copied()),
    )?;
    tx.execute(
        &format!("INSERT INTO main.annotation_tag SELECT * FROM grafana.annotation_tag WHERE annotation_id IN ({})", placeholders),
        rusqlite::params_from_iter(ann_ids.iter().copied()),
    )?;

    // recordings: id is PK (not recording_id)
    tx.execute(
        "INSERT INTO main.recordings (id, source_file, created_at, duration_secs, sample_count)
         SELECT id, source_file, created_at, duration_secs, sample_count FROM src.recordings WHERE id = ?1",
        rusqlite::params![recording_id],
    )?;

    // statics
    tx.execute(
        "INSERT INTO main.statics SELECT * FROM src.statics WHERE recording_id = ?1",
        rusqlite::params![recording_id],
    )?;

    // graphics: slice by annotation time ranges
    for (_, start, end) in &ranges {
        tx.execute(
            "INSERT INTO main.graphics SELECT * FROM src.graphics
             WHERE recording_id = ?1 AND time_offset >= ?2 AND time_offset <= ?3",
            rusqlite::params![recording_id, start, end],
        )?;
    }

    // physics: sliced by ranges + annotation_id (same structure as telemetry physics + annotation_id)
    for (ann_id, start, end) in &ranges {
        tx.execute(
            "INSERT INTO main.physics SELECT p.*, ?4 FROM src.physics p
             WHERE p.recording_id = ?1 AND p.time_offset >= ?2 AND p.time_offset <= ?3",
            rusqlite::params![recording_id, start, end, ann_id],
        )?;
    }

    tx.commit()?;
    conn.execute("DETACH DATABASE src", [])?;
    conn.execute("DETACH DATABASE grafana", [])?;

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM physics WHERE recording_id = ?1", rusqlite::params![recording_id], |r| r.get(0))?;
    let msg = if backup_path.exists() {
        format!(
            "OK: {} rows in analysis for recording {} → {} (backup: {})",
            count, recording_id, analysis_db.display(), backup_path.display()
        )
    } else {
        format!("OK: {} rows in analysis for recording {} → {}", count, recording_id, analysis_db.display())
    };
    Ok(msg)
}

fn parse_paths(args: &[String]) -> (Option<std::path::PathBuf>, Option<std::path::PathBuf>, Option<std::path::PathBuf>) {
    let cfg = config::load_config();
    let mut telemetry_db = Some(config::resolve_path(&cfg.export.sqlite_db_path));
    let mut grafana_db = std::env::var("GRAFANA_DB").ok().map(std::path::PathBuf::from);
    let mut analysis_db: Option<std::path::PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--grafana-db" && i + 1 < args.len() {
            grafana_db = Some(args[i + 1].clone().into());
            i += 2;
        } else if args[i] == "--telemetry-db" && i + 1 < args.len() {
            telemetry_db = Some(config::resolve_path(&args[i + 1]));
            i += 2;
        } else if args[i] == "--analysis-db" && i + 1 < args.len() {
            analysis_db = Some(config::resolve_path(&args[i + 1]));
            i += 2;
        } else {
            i += 1;
        }
    }

    if analysis_db.is_none() && telemetry_db.is_some() {
        let tb = telemetry_db.as_ref().unwrap();
        analysis_db = Some(tb.parent().unwrap_or(Path::new(".")).join("analysis.db"));
    }
    (grafana_db, telemetry_db, analysis_db)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage:");
        eprintln!("  acr_analysis_export <recording_id> [--grafana-db PATH] [--telemetry-db PATH] [--analysis-db PATH]");
        eprintln!("  acr_analysis_export --serve [--port PORT] [--grafana-db PATH] [--telemetry-db PATH] [--analysis-db PATH]");
        eprintln!("");
        eprintln!("  Writes to analysis.db (default: same dir as telemetry.db).");
        std::process::exit(1);
    }

    if args[0] == "--serve" {
        let rest: Vec<String> = args.into_iter().skip(1).collect();
        let mut port = 9876u16;
        let mut i = 0;
        while i < rest.len() {
            if rest[i] == "--port" && i + 1 < rest.len() {
                port = rest[i + 1].parse().unwrap_or(9876);
                i += 2;
            } else {
                i += 1;
            }
        }
        let (grafana_db, telemetry_db, analysis_db) = parse_paths(&rest);
        let grafana_db = grafana_db.ok_or("--grafana-db PATH or GRAFANA_DB env required in serve mode")?;
        let telemetry_db = telemetry_db.ok_or("--telemetry-db PATH or config required in serve mode")?;
        let analysis_db = analysis_db.ok_or("--analysis-db PATH or telemetry-db required")?;

        let server = Server::http(("0.0.0.0", port)).map_err(|e| format!("bind :{}: {}", port, e))?;
        eprintln!("acr_analysis_export on http://localhost:{}/export?recording_id=X", port);

        for mut req in server.incoming_requests() {
            let _ = std::io::Read::read_to_end(&mut req.as_reader(), &mut vec![]);
            let path = req.url().split('?').next().unwrap_or("");
            let query: Option<&str> = req.url().split('?').nth(1);
            let recording_id = query
                .and_then(|q| q.split('&').find(|p| p.starts_with("recording_id=")))
                .and_then(|p| p.strip_prefix("recording_id="))
                .and_then(|v| v.parse::<i64>().ok());

            let (status, body) = if path == "/export" {
                match recording_id {
                    Some(rid) => match run_export(rid, &grafana_db, &telemetry_db, &analysis_db) {
                        Ok(msg) => (200, format!("<html><body>{}</body></html>", msg)),
                        Err(e) => (500, format!("<html><body>Error: {}</body></html>", e)),
                    },
                    None => (400, "<html><body>Missing recording_id</body></html>".into()),
                }
            } else {
                (404, "<html><body>Not found. Use /export?recording_id=X</body></html>".into())
            };

            let _ = req.respond(Response::from_string(body).with_status_code(status));
        }
        return Ok(());
    }

    let recording_id: i64 = args[0].parse().map_err(|_| "recording_id must be integer")?;
    let (grafana_db, telemetry_db, analysis_db) = parse_paths(&args[1..]);
    let grafana_db = grafana_db.ok_or("--grafana-db PATH or GRAFANA_DB env required")?;
    let telemetry_db = telemetry_db.ok_or("--telemetry-db PATH or config required")?;
    let analysis_db = analysis_db.ok_or("--analysis-db PATH or telemetry-db required")?;

    match run_export(recording_id, &grafana_db, &telemetry_db, &analysis_db) {
        Ok(msg) => {
            eprintln!("{}", msg);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
