//! MoTeC .ld file writer.
//!
//! Format ported from Python ldparser (gotzl/ldparser) - reverse-engineered ACC MoTeC export.

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

use crate::record::{GraphicsRecord, PhysicsRecord};

const REC_FREQ: u16 = 333;

/// Write physics records to MoTeC .ld format (compatible with i2).
pub fn write_ld(
    path: impl AsRef<Path>,
    records: &[PhysicsRecord],
    sample_rate_hz: u32,
) -> std::io::Result<()> {
    write_ld_with_graphics(path, records, sample_rate_hz, None)
}

/// Write physics records (+ optional graphics sidecar channels) to MoTeC .ld format.
pub fn write_ld_with_graphics(
    path: impl AsRef<Path>,
    records: &[PhysicsRecord],
    _sample_rate_hz: u32,
    graphics: Option<(&[GraphicsRecord], u32)>,
) -> std::io::Result<()> {
    let mut f = File::create(path)?;

    // Build channels (ACC-style names)
    let time: Vec<f32> = (0..records.len())
        .map(|i| i as f32 / REC_FREQ as f32)
        .collect();
    let speed: Vec<f32> = records.iter().map(|r| r.speed_kmh).collect();
    let rpm: Vec<f32> = records.iter().map(|r| r.rpm as f32).collect();
    let throttle: Vec<f32> = records.iter().map(|r| r.gas * 100.0).collect();
    let brake: Vec<f32> = records.iter().map(|r| r.brake * 100.0).collect();
    let steer: Vec<f32> = records.iter().map(|r| r.steer_angle).collect();
    let gear: Vec<f32> = records.iter().map(|r| r.gear as f32).collect();
    let g_lat: Vec<f32> = records.iter().map(|r| r.g_force.x).collect();
    let g_lon: Vec<f32> = records.iter().map(|r| r.g_force.y).collect();
    let g_total: Vec<f32> = records
        .iter()
        .map(|r| (r.g_force.x * r.g_force.x + r.g_force.y * r.g_force.y).sqrt())
        .collect();
    let lf_deflection: Vec<f32> = records.iter().map(|r| r.suspension_travel.front_left).collect();
    let rf_deflection: Vec<f32> = records.iter().map(|r| r.suspension_travel.front_right).collect();
    let lb_deflection: Vec<f32> = records.iter().map(|r| r.suspension_travel.rear_left).collect();
    let rb_deflection: Vec<f32> = records.iter().map(|r| r.suspension_travel.rear_right).collect();
    // Workspace-compatible aliases (RBR Motec v105).
    let speed_alias: Vec<f32> = speed.clone();
    let throttle_alias: Vec<f32> = records.iter().map(|r| r.gas).collect();
    let brake_alias: Vec<f32> = records.iter().map(|r| r.brake).collect();
    let engine_rotation_alias: Vec<f32> = records
        .iter()
        .map(|r| r.rpm as f32 * std::f32::consts::TAU / 60.0)
        .collect();
    let gear_ok_alias: Vec<f32> = records.iter().map(|r| (r.gear - 1) as f32).collect();
    let lf_tyre_temp_c: Vec<f32> = records
        .iter()
        .map(|r| r.tyre_core_temp.front_left - 273.15)
        .collect();
    let rf_tyre_temp_c: Vec<f32> = records
        .iter()
        .map(|r| r.tyre_core_temp.front_right - 273.15)
        .collect();
    let lb_tyre_temp_c: Vec<f32> = records
        .iter()
        .map(|r| r.tyre_core_temp.rear_left - 273.15)
        .collect();
    let rb_tyre_temp_c: Vec<f32> = records
        .iter()
        .map(|r| r.tyre_core_temp.rear_right - 273.15)
        .collect();
    const PSI_TO_BAR: f32 = 0.068_947_57;
    let lf_pressure_bar: Vec<f32> = records
        .iter()
        .map(|r| r.wheel_pressure.front_left * PSI_TO_BAR)
        .collect();
    let rf_pressure_bar: Vec<f32> = records
        .iter()
        .map(|r| r.wheel_pressure.front_right * PSI_TO_BAR)
        .collect();
    let lb_pressure_bar: Vec<f32> = records
        .iter()
        .map(|r| r.wheel_pressure.rear_left * PSI_TO_BAR)
        .collect();
    let rb_pressure_bar: Vec<f32> = records
        .iter()
        .map(|r| r.wheel_pressure.rear_right * PSI_TO_BAR)
        .collect();
    let lf_brake_temp_c: Vec<f32> = records.iter().map(|r| r.brake_temp.front_left - 273.15).collect();
    let rf_brake_temp_c: Vec<f32> = records.iter().map(|r| r.brake_temp.front_right - 273.15).collect();
    let lb_brake_temp_c: Vec<f32> = records.iter().map(|r| r.brake_temp.rear_left - 273.15).collect();
    let rb_brake_temp_c: Vec<f32> = records.iter().map(|r| r.brake_temp.rear_right - 273.15).collect();
    let lf_tyre_wear_pct: Vec<f32> = records.iter().map(|r| r.tyre_wear.front_left * 100.0).collect();
    let rf_tyre_wear_pct: Vec<f32> = records.iter().map(|r| r.tyre_wear.front_right * 100.0).collect();
    let lb_tyre_wear_pct: Vec<f32> = records.iter().map(|r| r.tyre_wear.rear_left * 100.0).collect();
    let rb_tyre_wear_pct: Vec<f32> = records.iter().map(|r| r.tyre_wear.rear_right * 100.0).collect();

    let mut channels: Vec<(&str, &str, Vec<f32>)> = vec![
        ("Time", "s", time),
        ("Speed", "km/h", speed),
        ("RPM", "rpm", rpm),
        ("Throttle", "%", throttle),
        ("Brake", "%", brake),
        // RBR Motec workspace expects this exact channel id.
        ("steering", "", steer),
        ("Gear", "", gear),
        ("speed", "km/h", speed_alias),
        ("throttle", "", throttle_alias),
        ("brake", "", brake_alias),
        ("engineRotation", "rad/s", engine_rotation_alias),
        ("gear_ok", "", gear_ok_alias),
        ("vecLinearAccelerationCar.x", "g", g_lat),
        ("vecLinearAccelerationCar.y", "g", g_lon),
        ("G ForceTotal", "g", g_total),
        ("LF.deflection", "m", lf_deflection),
        ("RF.deflection", "m", rf_deflection),
        ("LB.deflection", "m", lb_deflection),
        ("RB.deflection", "m", rb_deflection),
        ("LF.tyreTemperature", "C", lf_tyre_temp_c),
        ("RF.tyreTemperature", "C", rf_tyre_temp_c),
        ("LB.tyreTemperature", "C", lb_tyre_temp_c),
        ("RB.tyreTemperature", "C", rb_tyre_temp_c),
        ("LF.pressure", "bar", lf_pressure_bar),
        ("RF.pressure", "bar", rf_pressure_bar),
        ("LB.pressure", "bar", lb_pressure_bar),
        ("RB.pressure", "bar", rb_pressure_bar),
        ("LF.brakeDiskTempC", "C", lf_brake_temp_c),
        ("RF.brakeDiskTempC", "C", rf_brake_temp_c),
        ("LB.brakeDiskTempC", "C", lb_brake_temp_c),
        ("RB.brakeDiskTempC", "C", rb_brake_temp_c),
        ("LF.tyreWear%", "%", lf_tyre_wear_pct),
        ("RF.tyreWear%", "%", rf_tyre_wear_pct),
        ("LB.tyreWear%", "%", lb_tyre_wear_pct),
        ("RB.tyreWear%", "%", rb_tyre_wear_pct),
    ];

    if let Some((graphics_records, _graphics_hz)) = graphics {
        if !graphics_records.is_empty() {
            let gx = resample_graphics_to_len(graphics_records, records.len(), |g| g.car_coordinates_x);
            let gy = resample_graphics_to_len(graphics_records, records.len(), |g| g.car_coordinates_y);
            channels.push(("position.x", "m", gx));
            channels.push(("position.y", "m", gy));
        }
    }

    // Layout (from ldparser)
    // ldHead: 1762 bytes
    let head_size = 1762u32;
    // ldEvent: 1154 bytes
    // Keep a minimal event block to maximize i2 compatibility.
    let event_ptr: u32 = head_size;
    let event_size = 1154u32;
    let chan_head_size = 124u32; // ldChan struct size from ldparser

    let meta_ptr = head_size + event_size;
    let data_ptr = meta_ptr + channels.len() as u32 * chan_head_size;

    // Calculate data offsets per channel
    let _n = records.len() as u32;
    let mut data_offsets = Vec::with_capacity(channels.len());
    let mut offset = data_ptr;
    for (_, _, data) in &channels {
        data_offsets.push(offset);
        offset += data.len() as u32 * 4; // f32 = 4 bytes
    }

    // Write ldHead
    write_ld_head(&mut f, meta_ptr, data_ptr, event_ptr, channels.len() as u32)?;

    // Write ldEvent block.
    f.seek(SeekFrom::Start(event_ptr as u64))?;
    write_ld_event(&mut f)?;

    // Seek to meta region and write channel headers
    f.seek(SeekFrom::Start(meta_ptr as u64))?;
    for (i, ((name, unit, data), &data_off)) in channels.iter().zip(data_offsets.iter()).enumerate() {
        let prev = if i == 0 { 0u32 } else { meta_ptr + (i - 1) as u32 * chan_head_size };
        let next = if i + 1 < channels.len() {
            meta_ptr + (i + 1) as u32 * chan_head_size
        } else {
            0
        };
        write_ld_chan(&mut f, prev, next, data_off, data.len() as u32, name, unit, i)?;
    }

    // Write channel data
    for (_, _, data) in &channels {
        for &v in data {
            f.write_all(&v.to_le_bytes())?;
        }
    }

    Ok(())
}

fn pad(w: &mut impl Write, n: usize) -> std::io::Result<()> {
    w.write_all(&vec![0u8; n])
}

fn write_str_fixed(w: &mut impl Write, s: &str, len: usize) -> std::io::Result<()> {
    let bytes = s.as_bytes();
    let n = bytes.len().min(len);
    w.write_all(&bytes[..n])?;
    pad(w, len - n)
}

fn write_ld_head(
    f: &mut File,
    meta_ptr: u32,
    data_ptr: u32,
    event_ptr: u32,
    n_chans: u32,
) -> std::io::Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let days = secs / 86400;
    let (y, m, d) = days_to_ymd(days);
    let h = (secs / 3600) % 24;
    let min = (secs / 60) % 60;
    let s = secs % 60;
    let date = format!("{:02}/{:02}/{:04}", d, m, y);
    let time = format!("{:02}:{:02}:{:02}", h, min, s);

    f.write_all(&0x40u32.to_le_bytes())?; // ldmarker
    pad(f, 4)?;
    f.write_all(&meta_ptr.to_le_bytes())?;
    f.write_all(&data_ptr.to_le_bytes())?;
    pad(f, 20)?;
    f.write_all(&event_ptr.to_le_bytes())?;
    pad(f, 24)?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&0x4240u16.to_le_bytes())?;
    f.write_all(&0xfu16.to_le_bytes())?;
    f.write_all(&0x1f44u32.to_le_bytes())?;
    write_str_fixed(f, "ADL", 8)?;
    f.write_all(&420u16.to_le_bytes())?;
    f.write_all(&0xadb0u16.to_le_bytes())?;
    f.write_all(&n_chans.to_le_bytes())?;
    pad(f, 4)?;
    write_str_fixed(f, &date, 16)?;
    pad(f, 16)?;
    write_str_fixed(f, &time, 16)?;
    pad(f, 16)?;
    write_str_fixed(f, "ACR", 64)?;
    write_str_fixed(f, "AC Rally", 64)?;
    pad(f, 64)?;
    write_str_fixed(f, "Telemetry", 64)?;
    pad(f, 64)?;
    pad(f, 1024)?;
    f.write_all(&0xc81a4u32.to_le_bytes())?;
    pad(f, 66)?;
    write_str_fixed(f, "acr_recorder export", 64)?;
    pad(f, 126)?;

    Ok(())
}

fn write_ld_event(f: &mut File) -> std::io::Result<()> {
    // ldEvent format in ldparser: <64s64s1024sH
    write_str_fixed(f, "ACR Session", 64)?;
    write_str_fixed(f, "0", 64)?;
    write_str_fixed(f, "acr_recorder synthetic/minimal export", 1024)?;
    f.write_all(&0u16.to_le_bytes())?; // venue_ptr = 0 (no venue block)
    Ok(())
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

fn write_ld_chan(
    f: &mut File,
    prev: u32,
    next: u32,
    data_ptr: u32,
    n_data: u32,
    name: &str,
    unit: &str,
    idx: usize,
) -> std::io::Result<()> {
    let counter = 0x2ee1u16 + idx as u16;
    let dtype_a: u16 = 0x07; // float
    let dtype: u16 = 4;     // float32
    let shift: i16 = 0;
    let mul: i16 = 1;
    let scale: i16 = 1;
    let dec: i16 = 0;

    f.write_all(&prev.to_le_bytes())?;
    f.write_all(&next.to_le_bytes())?;
    f.write_all(&data_ptr.to_le_bytes())?;
    f.write_all(&n_data.to_le_bytes())?;
    f.write_all(&counter.to_le_bytes())?;
    f.write_all(&dtype_a.to_le_bytes())?;
    f.write_all(&dtype.to_le_bytes())?;
    f.write_all(&REC_FREQ.to_le_bytes())?;
    f.write_all(&shift.to_le_bytes())?;
    f.write_all(&mul.to_le_bytes())?;
    f.write_all(&scale.to_le_bytes())?;
    f.write_all(&dec.to_le_bytes())?;
    write_str_fixed(f, name, 32)?;
    write_str_fixed(f, &name.chars().take(8).collect::<String>(), 8)?;
    write_str_fixed(f, unit, 12)?;
    pad(f, 40)?;

    Ok(())
}

fn resample_graphics_to_len(
    graphics: &[GraphicsRecord],
    target_len: usize,
    getter: impl Fn(&GraphicsRecord) -> f32,
) -> Vec<f32> {
    if target_len == 0 || graphics.is_empty() {
        return Vec::new();
    }
    if target_len == 1 {
        return vec![getter(&graphics[0])];
    }
    if graphics.len() == 1 {
        return vec![getter(&graphics[0]); target_len];
    }
    (0..target_len)
        .map(|i| {
            let src_idx = i * (graphics.len() - 1) / (target_len - 1);
            getter(&graphics[src_idx])
        })
        .collect()
}
