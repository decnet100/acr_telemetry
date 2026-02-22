//! Format metadata JSON for rkyv recordings.

use std::path::Path;
use std::time::SystemTime;

use serde::Serialize;

#[derive(Serialize)]
struct FormatMetadata<'a> {
    format_version: u16,
    binary_file: &'a str,
    created_at: String,
    sample_rate_hz: u32,
    source: &'static str,
    file_format: FileFormat,
    schema: Schema,
    #[serde(skip_serializing_if = "Option::is_none")]
    statics: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct FileFormat {
    header: HeaderFormat,
    chunks: ChunkFormat,
    serialization: &'static str,
}

#[derive(Serialize)]
struct HeaderFormat {
    size_bytes: u32,
    layout: Vec<HeaderField>,
    byte_order: &'static str,
}

#[derive(Serialize)]
struct HeaderField {
    offset: u32,
    size: u32,
    name: &'static str,
    r#type: &'static str,
    description: &'static str,
}

#[derive(Serialize)]
struct ChunkFormat {
    structure: &'static str,
    length_prefix: LengthPrefix,
    payload: &'static str,
}

#[derive(Serialize)]
struct LengthPrefix {
    size_bytes: u32,
    r#type: &'static str,
    byte_order: &'static str,
}

#[derive(Serialize)]
struct Schema {
    root_type: &'static str,
    root_description: &'static str,
    types: Vec<TypeDef>,
}

#[derive(Serialize)]
struct TypeDef {
    name: &'static str,
    description: &'static str,
    fields: Vec<FieldDef>,
}

#[derive(Serialize)]
struct FieldDef {
    name: &'static str,
    r#type: &'static str,
    unit: Option<&'static str>,
}

/// Write format metadata JSON alongside the rkyv file.
pub fn write_format_metadata(
    rkyv_path: &Path,
    statics: Option<&crate::record::StaticsRecord>,
) -> std::io::Result<()> {
    let binary_name = rkyv_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("recording.rkyv");

    let created_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let created_at = format_iso8601(created_at);

    let meta = FormatMetadata {
        format_version: 1,
        binary_file: binary_name,
        created_at,
        sample_rate_hz: 333,
        source: "ACC/AC Rally shared memory (acc_shared_memory_rs)",
        file_format: FileFormat {
            header: HeaderFormat {
                size_bytes: 16,
                byte_order: "little-endian",
                layout: vec![
                    HeaderField {
                        offset: 0,
                        size: 4,
                        name: "magic",
                        r#type: "bytes",
                        description: "File signature, must be b\"ACCR\"",
                    },
                    HeaderField {
                        offset: 4,
                        size: 2,
                        name: "version",
                        r#type: "u16",
                        description: "Format version",
                    },
                    HeaderField {
                        offset: 6,
                        size: 4,
                        name: "sample_rate",
                        r#type: "u32",
                        description: "Target sample rate in Hz (typically 333)",
                    },
                    HeaderField {
                        offset: 10,
                        size: 6,
                        name: "reserved",
                        r#type: "bytes",
                        description: "Reserved for future use",
                    },
                ],
            },
            chunks: ChunkFormat {
                structure: "Repeated: [length_prefix][payload] from offset 16 until EOF",
                length_prefix: LengthPrefix {
                    size_bytes: 4,
                    r#type: "u32",
                    byte_order: "little-endian",
                },
                payload: "rkyv-serialized Vec<PhysicsRecord>",
            },
            serialization: "rkyv 0.7 (https://rkyv.org). Use rkyv::from_bytes::<Vec<PhysicsRecord>>() with the schema below.",
        },
        schema: Schema {
            root_type: "Vec<PhysicsRecord>",
            root_description: "Array of physics snapshots, one per sample",
            types: schema_types(),
        },
        statics: statics.and_then(|s| serde_json::to_value(s).ok()),
    };

    let json_path = rkyv_path.with_extension("json");
    let json = serde_json::to_string_pretty(&meta).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
    })?;
    std::fs::write(json_path, json)
}

fn format_iso8601(secs: u64) -> String {
    let days = secs / 86400;
    let (y, m, d) = days_to_ymd(days);
    let h = (secs / 3600) % 24;
    let min = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, h, min, s)
}

fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    const EPOCH: i64 = 719163;
    let j = days as i64 + EPOCH;
    let a = (4 * j + 3) / 146097;
    let b = j - (146097 * a) / 4;
    let c = (4 * b + 3) / 1461;
    let d = b - (1461 * c) / 4;
    let e = (5 * d + 2) / 153;
    let day = (d - (153 * e + 2) / 5) as u32 + 1;
    let month = ((e + 2) % 12) as u32 + 1;
    let year = (100 * a + c) as u32;
    (year, month, day)
}

fn schema_types() -> Vec<TypeDef> {
    vec![
        TypeDef {
            name: "PhysicsRecord",
            description: "Complete physics snapshot at one timestep (~333 Hz)",
            fields: vec![
                FieldDef { name: "packet_id", r#type: "i32", unit: None },
                FieldDef { name: "gas", r#type: "f32", unit: Some("0–1") },
                FieldDef { name: "brake", r#type: "f32", unit: Some("0–1") },
                FieldDef { name: "clutch", r#type: "f32", unit: Some("0–1") },
                FieldDef { name: "steer_angle", r#type: "f32", unit: Some("deg") },
                FieldDef { name: "gear", r#type: "i32", unit: None },
                FieldDef { name: "rpm", r#type: "i32", unit: None },
                FieldDef { name: "autoshifter_on", r#type: "bool", unit: None },
                FieldDef { name: "ignition_on", r#type: "bool", unit: None },
                FieldDef { name: "starter_engine_on", r#type: "bool", unit: None },
                FieldDef { name: "is_engine_running", r#type: "bool", unit: None },
                FieldDef { name: "speed_kmh", r#type: "f32", unit: Some("km/h") },
                FieldDef { name: "velocity", r#type: "Vector3fRecord", unit: None },
                FieldDef { name: "local_velocity", r#type: "Vector3fRecord", unit: None },
                FieldDef { name: "local_angular_vel", r#type: "Vector3fRecord", unit: None },
                FieldDef { name: "g_force", r#type: "Vector3fRecord", unit: None },
                FieldDef { name: "heading", r#type: "f32", unit: Some("rad") },
                FieldDef { name: "pitch", r#type: "f32", unit: Some("rad") },
                FieldDef { name: "roll", r#type: "f32", unit: Some("rad") },
                FieldDef { name: "final_ff", r#type: "f32", unit: None },
                FieldDef { name: "wheel_slip", r#type: "WheelsRecord", unit: None },
                FieldDef { name: "wheel_pressure", r#type: "WheelsRecord", unit: Some("psi") },
                FieldDef { name: "wheel_angular_speed", r#type: "WheelsRecord", unit: Some("rad/s") },
                FieldDef { name: "tyre_core_temp", r#type: "WheelsRecord", unit: Some("°C") },
                FieldDef { name: "suspension_travel", r#type: "WheelsRecord", unit: Some("mm") },
                FieldDef { name: "brake_temp", r#type: "WheelsRecord", unit: Some("°C") },
                FieldDef { name: "brake_pressure", r#type: "WheelsRecord", unit: Some("bar") },
                FieldDef { name: "suspension_damage", r#type: "WheelsRecord", unit: None },
                FieldDef { name: "slip_ratio", r#type: "WheelsRecord", unit: None },
                FieldDef { name: "slip_angle", r#type: "WheelsRecord", unit: Some("deg") },
                FieldDef { name: "pad_life", r#type: "WheelsRecord", unit: Some("%") },
                FieldDef { name: "disc_life", r#type: "WheelsRecord", unit: Some("%") },
                FieldDef { name: "front_brake_compound", r#type: "i32", unit: None },
                FieldDef { name: "rear_brake_compound", r#type: "i32", unit: None },
                FieldDef { name: "tyre_contact_point", r#type: "ContactPointRecord", unit: None },
                FieldDef { name: "tyre_contact_normal", r#type: "ContactPointRecord", unit: None },
                FieldDef { name: "tyre_contact_heading", r#type: "ContactPointRecord", unit: None },
                FieldDef { name: "fuel", r#type: "f32", unit: Some("L") },
                FieldDef { name: "tc", r#type: "f32", unit: None },
                FieldDef { name: "abs", r#type: "f32", unit: None },
                FieldDef { name: "pit_limiter_on", r#type: "bool", unit: None },
                FieldDef { name: "turbo_boost", r#type: "f32", unit: Some("bar") },
                FieldDef { name: "air_temp", r#type: "f32", unit: Some("°C") },
                FieldDef { name: "road_temp", r#type: "f32", unit: Some("°C") },
                FieldDef { name: "water_temp", r#type: "f32", unit: Some("°C") },
                FieldDef { name: "car_damage", r#type: "CarDamageRecord", unit: None },
                FieldDef { name: "is_ai_controlled", r#type: "bool", unit: None },
                FieldDef { name: "brake_bias", r#type: "f32", unit: None },
                FieldDef { name: "kerb_vibration", r#type: "f32", unit: None },
                FieldDef { name: "slip_vibration", r#type: "f32", unit: None },
                FieldDef { name: "g_vibration", r#type: "f32", unit: None },
                FieldDef { name: "abs_vibration", r#type: "f32", unit: None },
            ],
        },
        TypeDef {
            name: "Vector3fRecord",
            description: "3D vector (x, y, z)",
            fields: vec![
                FieldDef { name: "x", r#type: "f32", unit: None },
                FieldDef { name: "y", r#type: "f32", unit: None },
                FieldDef { name: "z", r#type: "f32", unit: None },
            ],
        },
        TypeDef {
            name: "WheelsRecord",
            description: "Per-wheel values (front_left, front_right, rear_left, rear_right)",
            fields: vec![
                FieldDef { name: "front_left", r#type: "f32", unit: None },
                FieldDef { name: "front_right", r#type: "f32", unit: None },
                FieldDef { name: "rear_left", r#type: "f32", unit: None },
                FieldDef { name: "rear_right", r#type: "f32", unit: None },
            ],
        },
        TypeDef {
            name: "ContactPointRecord",
            description: "3D contact points for all four tyres",
            fields: vec![
                FieldDef { name: "front_left", r#type: "Vector3fRecord", unit: None },
                FieldDef { name: "front_right", r#type: "Vector3fRecord", unit: None },
                FieldDef { name: "rear_left", r#type: "Vector3fRecord", unit: None },
                FieldDef { name: "rear_right", r#type: "Vector3fRecord", unit: None },
            ],
        },
        TypeDef {
            name: "CarDamageRecord",
            description: "Car damage (front, rear, left, right, center)",
            fields: vec![
                FieldDef { name: "front", r#type: "f32", unit: None },
                FieldDef { name: "rear", r#type: "f32", unit: None },
                FieldDef { name: "left", r#type: "f32", unit: None },
                FieldDef { name: "right", r#type: "f32", unit: None },
                FieldDef { name: "center", r#type: "f32", unit: None },
            ],
        },
    ]
}
