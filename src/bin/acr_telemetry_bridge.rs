//! ACC/AC Rally telemetry bridge – reads shared memory and sends data via UDP and/or HTTP.
//!
//! For monitoring temperatures (and other values) on phone or second device.
//! Rate: 1–10 Hz (no need for 333 Hz).
//!
//! ACC provides temperatures in Kelvin. Use config or --unit to choose °C or °F.
//!
//! Config: acr_telemetry_bridge.toml (CWD or ~/.config/acr_recorder/)
//! CLI options override config.

use std::net::{SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use acc_shared_memory_rs::datatypes::Wheels;
use acc_shared_memory_rs::maps::PhysicsMap;
use acc_shared_memory_rs::ACCSharedMemory;
use acr_recorder::color_config;
use acr_recorder::config;
use serde::Serialize;
use serde_json::{Map, Number, Value};
use tiny_http::{Response, Server};

static RUNNING: AtomicBool = AtomicBool::new(true);

/// Check if acr_recorder is currently running by checking if acr_elapsed_secs exists and was modified recently.
/// This function is intentionally cheap - only checks file modification time, no reads.
fn is_recorder_active(notes_dir: &PathBuf) -> bool {
    let elapsed_file = notes_dir.join("acr_elapsed_secs");
    
    // Fast path: if file doesn't exist, recorder is not running
    // Using metadata() is relatively fast as it only queries file system metadata, not file contents
    match std::fs::metadata(&elapsed_file) {
        Ok(metadata) => {
            // Check if file was modified within the last 3 seconds (recorder updates it every ~1s)
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    return elapsed.as_secs() < 3;
                }
            }
            false
        }
        Err(_) => false,
    }
}

fn wheel_values<F>(physics: &PhysicsMap, get: F, temp_unit: &str, is_temp: bool) -> [(&'static str, f32); 4]
where
    F: Fn(&PhysicsMap) -> &Wheels,
{
    let w = get(physics);
    let conv = |v: f32| if is_temp { k_to_unit(v, temp_unit) } else { v };
    [
        ("_fl", conv(w.front_left)),
        ("_fr", conv(w.front_right)),
        ("_rl", conv(w.rear_left)),
        ("_rr", conv(w.rear_right)),
    ]
}

/// Convert Kelvin to target unit. ACC shared memory uses Kelvin.
fn k_to_unit(k: f32, unit: &str) -> f32 {
    match unit.to_lowercase().as_str() {
        "f" | "fahrenheit" => (k - 273.15) * 9.0 / 5.0 + 32.0,
        "k" | "kelvin" => k,
        _ => k - 273.15, // default: Celsius
    }
}

fn normalize_temp_unit(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "f" | "fahrenheit" => "f",
        "k" | "kelvin" => "k",
        _ => "c",
    }
    .to_string()
}

fn f(v: f32) -> Value {
    Value::Number(Number::from_f64(v as f64).unwrap_or(Number::from(0)))
}
fn i(v: i32) -> Value {
    Value::Number(Number::from(v))
}
fn b(v: bool) -> Value {
    Value::Bool(v)
}

/// Build flat JSON payload with all physics fields. Temperatures converted from Kelvin.
fn build_payload(
    physics: &acc_shared_memory_rs::maps::PhysicsMap,
    temp_unit: &str,
    recorder_active: bool,
) -> Value {
    let k = |v: f32| k_to_unit(v, temp_unit);
    let unit = normalize_temp_unit(temp_unit);

    let mut m = Map::new();
    m.insert("packet_id".into(), i(physics.packet_id));
    m.insert("temperature_unit".into(), Value::String(unit));
    m.insert("recorder_active".into(), b(recorder_active));

    // Driver inputs
    m.insert("gas".into(), f(physics.gas));
    m.insert("brake".into(), f(physics.brake));
    m.insert("clutch".into(), f(physics.clutch));
    m.insert("steer_angle".into(), f(physics.steer_angle));
    m.insert("gear".into(), i(physics.gear));
    m.insert("rpm".into(), i(physics.rpm));
    m.insert("autoshifter_on".into(), b(physics.autoshifter_on));
    m.insert("ignition_on".into(), b(physics.ignition_on));
    m.insert("starter_engine_on".into(), b(physics.starter_engine_on));
    m.insert("is_engine_running".into(), b(physics.is_engine_running));

    // Motion
    m.insert("speed_kmh".into(), f(physics.speed_kmh));
    m.insert("velocity_x".into(), f(physics.velocity.x));
    m.insert("velocity_y".into(), f(physics.velocity.y));
    m.insert("velocity_z".into(), f(physics.velocity.z));
    m.insert("local_velocity_x".into(), f(physics.local_velocity.x));
    m.insert("local_velocity_y".into(), f(physics.local_velocity.y));
    m.insert("local_velocity_z".into(), f(physics.local_velocity.z));
    m.insert("g_force_x".into(), f(physics.g_force.x));
    m.insert("g_force_y".into(), f(physics.g_force.y));
    m.insert("g_force_z".into(), f(physics.g_force.z));
    m.insert("heading".into(), f(physics.heading));
    m.insert("pitch".into(), f(physics.pitch));
    m.insert("roll".into(), f(physics.roll));
    m.insert("final_ff".into(), f(physics.final_ff));

    // Wheels (flat: *_fl, *_fr, *_rl, *_rr)
    for (base, is_temp) in [
        ("wheel_slip", false),
        ("wheel_load", false),
        ("wheel_pressure", false),
        ("wheel_angular_speed", false),
        ("tyre_core_temp", true),
        ("brake_temp", true),
        ("tyre_wear", false),
        ("tyre_dirty_level", false),
        ("camber_rad", false),
        ("suspension_travel", false),
        ("brake_pressure", false),
        ("slip_ratio", false),
        ("slip_angle", false),
        ("pad_life", false),
        ("disc_life", false),
        ("tyre_temp_i", true),
        ("tyre_temp_m", true),
        ("tyre_temp_o", true),
        ("mz", false),
        ("fz", false),
        ("my", false),
        ("suspension_damage", false),
    ] {
        let get: fn(&PhysicsMap) -> &Wheels = match base {
            "wheel_slip" => |p| &p.wheel_slip,
            "wheel_load" => |p| &p.wheel_load,
            "wheel_pressure" => |p| &p.wheel_pressure,
            "wheel_angular_speed" => |p| &p.wheel_angular_speed,
            "tyre_core_temp" => |p| &p.tyre_core_temp,
            "brake_temp" => |p| &p.brake_temp,
            "tyre_wear" => |p| &p.tyre_wear,
            "tyre_dirty_level" => |p| &p.tyre_dirty_level,
            "camber_rad" => |p| &p.camber_rad,
            "suspension_travel" => |p| &p.suspension_travel,
            "brake_pressure" => |p| &p.brake_pressure,
            "slip_ratio" => |p| &p.slip_ratio,
            "slip_angle" => |p| &p.slip_angle,
            "pad_life" => |p| &p.pad_life,
            "disc_life" => |p| &p.disc_life,
            "tyre_temp_i" => |p| &p.tyre_temp_i,
            "tyre_temp_m" => |p| &p.tyre_temp_m,
            "tyre_temp_o" => |p| &p.tyre_temp_o,
            "mz" => |p| &p.mz,
            "fz" => |p| &p.fz,
            "my" => |p| &p.my,
            "suspension_damage" => |p| &p.suspension_damage,
            _ => unreachable!(),
        };
        for (sfx, v) in wheel_values(physics, get, temp_unit, is_temp) {
            m.insert(format!("{}{}", base, sfx), f(v));
        }
    }

    // Aliases for backward compatibility (tyre_fl = tyre_core_temp_fl etc.)
    m.insert("tyre_fl".into(), m.get("tyre_core_temp_fl").cloned().unwrap_or(f(0.0)));
    m.insert("tyre_fr".into(), m.get("tyre_core_temp_fr").cloned().unwrap_or(f(0.0)));
    m.insert("tyre_rl".into(), m.get("tyre_core_temp_rl").cloned().unwrap_or(f(0.0)));
    m.insert("tyre_rr".into(), m.get("tyre_core_temp_rr").cloned().unwrap_or(f(0.0)));
    m.insert("brake_fl".into(), m.get("brake_temp_fl").cloned().unwrap_or(f(0.0)));
    m.insert("brake_fr".into(), m.get("brake_temp_fr").cloned().unwrap_or(f(0.0)));
    m.insert("brake_rl".into(), m.get("brake_temp_rl").cloned().unwrap_or(f(0.0)));
    m.insert("brake_rr".into(), m.get("brake_temp_rr").cloned().unwrap_or(f(0.0)));

    // Car status
    m.insert("fuel".into(), f(physics.fuel));
    m.insert("water_temp".into(), f(k(physics.water_temp)));
    m.insert("road_temp".into(), f(k(physics.road_temp)));
    m.insert("air_temp".into(), f(k(physics.air_temp)));
    m.insert("tc".into(), f(physics.tc));
    m.insert("abs".into(), f(physics.abs));
    m.insert("brake_bias".into(), f(physics.brake_bias));
    m.insert("turbo_boost".into(), f(physics.turbo_boost));
    m.insert("pit_limiter_on".into(), b(physics.pit_limiter_on));
    m.insert("tc_in_action".into(), b(physics.tc_in_action));
    m.insert("abs_in_action".into(), b(physics.abs_in_action));
    m.insert("is_ai_controlled".into(), b(physics.is_ai_controlled));
    m.insert("car_damage_front".into(), f(physics.car_damage.front));
    m.insert("car_damage_rear".into(), f(physics.car_damage.rear));
    m.insert("car_damage_left".into(), f(physics.car_damage.left));
    m.insert("car_damage_right".into(), f(physics.car_damage.right));
    m.insert("car_damage_center".into(), f(physics.car_damage.center));

    // Additional
    m.insert("drs".into(), i(physics.drs));
    m.insert("cg_height".into(), f(physics.cg_height));
    m.insert("number_of_tyres_out".into(), i(physics.number_of_tyres_out));
    m.insert("kers_charge".into(), f(physics.kers_charge));
    m.insert("kers_input".into(), f(physics.kers_input));
    m.insert("kers_current_kj".into(), f(physics.kers_current_kj));
    m.insert("ride_height_front".into(), f(physics.ride_height_front));
    m.insert("ride_height_rear".into(), f(physics.ride_height_rear));
    m.insert("ballast".into(), f(physics.ballast));
    m.insert("air_density".into(), f(physics.air_density));
    m.insert("performance_meter".into(), f(physics.performance_meter));
    m.insert("engine_brake".into(), i(physics.engine_brake));
    m.insert("ers_recovery_level".into(), i(physics.ers_recovery_level));
    m.insert("ers_power_level".into(), i(physics.ers_power_level));
    m.insert("current_max_rpm".into(), i(physics.current_max_rpm));
    m.insert("drs_available".into(), i(physics.drs_available));
    m.insert("drs_enabled".into(), i(physics.drs_enabled));
    m.insert("p2p_activation".into(), i(physics.p2p_activation));
    m.insert("p2p_status".into(), i(physics.p2p_status));
    m.insert("front_brake_compound".into(), i(physics.front_brake_compound));
    m.insert("rear_brake_compound".into(), i(physics.rear_brake_compound));

    // Vibration
    m.insert("kerb_vibration".into(), f(physics.kerb_vibration));
    m.insert("slip_vibration".into(), f(physics.slip_vibration));
    m.insert("g_vibration".into(), f(physics.g_vibration));
    m.insert("abs_vibration".into(), f(physics.abs_vibration));

    Value::Object(m)
}

/// All field IDs exposed in the flat JSON payload. Used for /config available_fields.
fn available_field_ids() -> Vec<String> {
    let mut ids: Vec<String> = vec![
        "packet_id".into(), "temperature_unit".into(),
        "gas".into(), "brake".into(), "clutch".into(), "steer_angle".into(), "gear".into(), "rpm".into(),
        "autoshifter_on".into(), "ignition_on".into(), "starter_engine_on".into(), "is_engine_running".into(),
        "speed_kmh".into(), "velocity_x".into(), "velocity_y".into(), "velocity_z".into(),
        "local_velocity_x".into(), "local_velocity_y".into(), "local_velocity_z".into(),
        "g_force_x".into(), "g_force_y".into(), "g_force_z".into(), "heading".into(), "pitch".into(), "roll".into(), "final_ff".into(),
        "fuel".into(), "water_temp".into(), "road_temp".into(), "air_temp".into(), "tc".into(), "abs".into(), "brake_bias".into(),
        "turbo_boost".into(), "pit_limiter_on".into(), "tc_in_action".into(), "abs_in_action".into(), "is_ai_controlled".into(),
        "car_damage_front".into(), "car_damage_rear".into(), "car_damage_left".into(), "car_damage_right".into(), "car_damage_center".into(),
        "drs".into(), "cg_height".into(), "number_of_tyres_out".into(), "kers_charge".into(), "kers_input".into(), "kers_current_kj".into(),
        "ride_height_front".into(), "ride_height_rear".into(), "ballast".into(), "air_density".into(), "performance_meter".into(),
        "engine_brake".into(), "ers_recovery_level".into(), "ers_power_level".into(), "current_max_rpm".into(),
        "drs_available".into(), "drs_enabled".into(), "p2p_activation".into(), "p2p_status".into(),
        "front_brake_compound".into(), "rear_brake_compound".into(),
        "kerb_vibration".into(), "slip_vibration".into(), "g_vibration".into(), "abs_vibration".into(),
    ];
    for base in &[
        "wheel_slip", "wheel_load", "wheel_pressure", "wheel_angular_speed",
        "tyre_core_temp", "brake_temp", "tyre_wear", "tyre_dirty_level", "camber_rad",
        "suspension_travel", "brake_pressure", "slip_ratio", "slip_angle",
        "pad_life", "disc_life", "tyre_temp_i", "tyre_temp_m", "tyre_temp_o",
        "mz", "fz", "my", "suspension_damage",
    ] {
        for sfx in &["_fl", "_fr", "_rl", "_rr"] {
            ids.push(format!("{}{}", base, sfx));
        }
    }
    ids.extend(["tyre_fl", "tyre_fr", "tyre_rl", "tyre_rr", "brake_fl", "brake_fr", "brake_rl", "brake_rr"].iter().map(|s| (*s).to_string()));
    ids.sort();
    ids
}

#[derive(Serialize)]
struct DashboardConfigResponse {
    slots: Vec<DashboardSlotJson>,
    temperature_unit: String,
    colors: ColorPaletteJson,
    fields: std::collections::HashMap<String, FieldThresholdsJson>,
    available_fields: Vec<String>,
}

#[derive(Serialize)]
struct ColorPaletteJson {
    very_low: String,
    low: String,
    normal: String,
    high: String,
    very_high: String,
    ignore: String,
}

#[derive(Serialize)]
struct FieldThresholdsJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    very_low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    normal: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    high: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    very_high: Option<f64>,
}

#[derive(Serialize)]
struct DashboardSlotJson {
    field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

fn run_http_server(
    addr: &str,
    state: Arc<RwLock<Option<Value>>>,
    dashboard_config: DashboardConfigResponse,
) {
    let server = match Server::http(addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("HTTP server failed to bind to {}: {}", addr, e);
            return;
        }
    };
    eprintln!("HTTP dashboard: http://{}", addr);

    let dashboard_html = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/acr_receiver/index.html"
    ));

    for request in server.incoming_requests() {
        if !RUNNING.load(Ordering::Relaxed) {
            break;
        }
        match request.url() {
            "/" | "/index.html" => {
                let _ = request.respond(Response::from_string(dashboard_html)
                    .with_header(tiny_http::Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap()));
            }
            "/data" => {
                let payload = state.read().unwrap().clone();
                let body = match payload {
                    Some(p) => serde_json::to_string(&p).unwrap_or_else(|_| "{}".into()),
                    None => "null".into(),
                };
                let _ = request.respond(Response::from_string(body)
                    .with_header(tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap())
                    .with_header(tiny_http::Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap()));
            }
            "/config" => {
                let body = serde_json::to_string(&dashboard_config).unwrap_or_else(|_| "{}".into());
                let _ = request.respond(Response::from_string(body)
                    .with_header(tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap())
                    .with_header(tiny_http::Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap()));
            }
            _ => {
                let _ = request.respond(Response::from_string("404").with_status_code(404));
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ctrlc::set_handler(|| RUNNING.store(false, Ordering::Relaxed))?;

    let bridge = config::load_bridge_config();

    let args: Vec<String> = std::env::args().collect();
    let mut rate_hz = bridge.rate_hz;
    let mut udp_target = bridge.udp_target.clone();
    let mut http_addr = if bridge.http_addr.is_empty() {
        None
    } else {
        Some(bridge.http_addr.clone())
    };
    let mut temp_unit = bridge.temperature_unit.clone();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        let (name, value) = if let Some(eq) = arg.find('=') {
            let (a, b) = arg.split_at(eq);
            (a, Some(b[1..].trim_matches('"').to_string()))
        } else {
            (arg.as_str(), None)
        };

        match name {
            "--rate" => {
                if let Some(v) = value {
                    rate_hz = v.parse().unwrap_or(rate_hz);
                } else {
                    i += 1;
                    if i < args.len() {
                        rate_hz = args[i].parse().unwrap_or(rate_hz);
                    }
                }
            }
            "--udp" => {
                if let Some(v) = value {
                    udp_target = Some(v);
                } else {
                    i += 1;
                    if i < args.len() {
                        udp_target = Some(args[i].clone());
                    }
                }
            }
            "--http" | "--http_addr" | "--http-addr" => {
                if let Some(v) = value {
                    http_addr = if v.is_empty() { None } else { Some(v) };
                } else {
                    i += 1;
                    http_addr = Some(
                        if i < args.len() && !args[i].starts_with('-') {
                            let a = args[i].clone();
                            i += 1;
                            a
                        } else {
                            "0.0.0.0:8080".into()
                        },
                    );
                }
            }
            "--no-game-exit" | "--no_game_exit" => {} // Deprecated: no longer exits on disconnect
            "--unit" | "-u" => {
                if let Some(v) = value {
                    temp_unit = v;
                } else {
                    i += 1;
                    if i < args.len() {
                        temp_unit = args[i].clone();
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    let interval = Duration::from_millis(1000 / rate_hz.max(1));
    let udp_socket = udp_target.as_ref().map(|t| {
        let sock = UdpSocket::bind("0.0.0.0:0").expect("UDP bind");
        eprintln!("UDP target: {} ({} Hz)", t, rate_hz);
        (t.clone(), sock)
    });

    let cfg = config::load_config();
    let notes_dir = config::resolve_notes_dir(&cfg.recorder);
    let state: Arc<RwLock<Option<Value>>> = Arc::new(RwLock::new(None));

    if let Some(ref addr) = http_addr {
        let state_clone = state.clone();
        let addr_clone = addr.clone();
        let color_cfg = bridge
            .telemetry_colors
            .clone()
            .unwrap_or_else(color_config::load_color_config);
        let dashboard_config = DashboardConfigResponse {
            slots: bridge
                .dashboard_slots
                .iter()
                .map(|s| DashboardSlotJson {
                    field: s.field_id().to_string(),
                    label: s.label().map(String::from),
                })
                .collect(),
            temperature_unit: normalize_temp_unit(&temp_unit),
            available_fields: available_field_ids(),
            colors: ColorPaletteJson {
                very_low: color_cfg.colors.very_low.clone(),
                low: color_cfg.colors.low.clone(),
                normal: color_cfg.colors.normal.clone(),
                high: color_cfg.colors.high.clone(),
                very_high: color_cfg.colors.very_high.clone(),
                ignore: color_cfg.colors.ignore.clone(),
            },
            fields: color_cfg
                .fields
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        FieldThresholdsJson {
                            very_low: v.very_low,
                            low: v.low,
                            normal: v.normal,
                            high: v.high,
                            very_high: v.very_high,
                        },
                    )
                })
                .collect(),
        };
        std::thread::spawn(move || run_http_server(&addr_clone, state_clone, dashboard_config));
    }

    // Lower thread priority to minimize impact on game performance
    #[cfg(windows)]
    {
        unsafe {
            // THREAD_PRIORITY_BELOW_NORMAL = -1
            // This ensures the bridge doesn't interfere with game rendering
            winapi::um::processthreadsapi::SetThreadPriority(
                winapi::um::processthreadsapi::GetCurrentThread(),
                -1,
            );
        }
    }

    let mut acc = ACCSharedMemory::new()?;

    let unit_label = match temp_unit.to_lowercase().as_str() {
        "f" | "fahrenheit" => "°F",
        "k" | "kelvin" => "K",
        _ => "°C",
    };
    eprintln!(
        "Bridge running at {} Hz, temperatures in {}. Ctrl+C to stop.",
        rate_hz, unit_label
    );

    // Cache recorder status to avoid filesystem checks on every iteration
    let mut recorder_active_cached = false;
    let mut last_recorder_check = std::time::Instant::now();
    let recorder_check_interval = Duration::from_secs(2); // Check every 2 seconds instead of every iteration

    while RUNNING.load(Ordering::Relaxed) {
        match acc.read_shared_memory() {
            Ok(Some(data)) => {
                // Only check recorder status every 2 seconds to minimize filesystem overhead
                if last_recorder_check.elapsed() >= recorder_check_interval {
                    recorder_active_cached = is_recorder_active(&notes_dir);
                    last_recorder_check = std::time::Instant::now();
                }
                
                let payload = build_payload(&data.physics, &temp_unit, recorder_active_cached);

                if let Some((ref target, ref sock)) = udp_socket {
                    let buf = serde_json::to_vec(&payload).unwrap_or_default();
                    if let Some((host, port)) = target.split_once(':') {
                        if let Ok(addr) = format!("{}:{}", host, port).parse::<SocketAddr>() {
                            let _ = sock.send_to(&buf, addr);
                        }
                    }
                }

                *state.write().unwrap() = Some(payload);
            }
            Ok(None) | Err(_) => {}
        }

        std::thread::sleep(interval);
    }

    Ok(())
}
