//! MoTeC .ld file writer.
//!
//! Format ported from Python ldparser (gotzl/ldparser) - reverse-engineered ACC MoTeC export.

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

use crate::record::PhysicsRecord;

const REC_FREQ: u16 = 333;

/// Write physics records to MoTeC .ld format (compatible with i2).
pub fn write_ld(
    path: impl AsRef<Path>,
    records: &[PhysicsRecord],
    _sample_rate_hz: u32,
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
    let lat_g: Vec<f32> = records.iter().map(|r| r.g_force.y).collect();
    let lon_g: Vec<f32> = records.iter().map(|r| r.g_force.x).collect();
    let fuel: Vec<f32> = records.iter().map(|r| r.fuel).collect();
    let fl_temp: Vec<f32> = records.iter().map(|r| r.tyre_core_temp.front_left).collect();
    let fr_temp: Vec<f32> = records.iter().map(|r| r.tyre_core_temp.front_right).collect();
    let rl_temp: Vec<f32> = records.iter().map(|r| r.tyre_core_temp.rear_left).collect();
    let rr_temp: Vec<f32> = records.iter().map(|r| r.tyre_core_temp.rear_right).collect();
    let engine_brake: Vec<f32> = records.iter().map(|r| r.engine_brake as f32).collect();
    let tc_active: Vec<f32> = records.iter().map(|r| if r.tc_in_action { 1.0 } else { 0.0 }).collect();
    let abs_active: Vec<f32> = records.iter().map(|r| if r.abs_in_action { 1.0 } else { 0.0 }).collect();
    let fl_load: Vec<f32> = records.iter().map(|r| r.wheel_load.front_left).collect();
    let fr_load: Vec<f32> = records.iter().map(|r| r.wheel_load.front_right).collect();
    let rl_load: Vec<f32> = records.iter().map(|r| r.wheel_load.rear_left).collect();
    let rr_load: Vec<f32> = records.iter().map(|r| r.wheel_load.rear_right).collect();
    let fl_camber: Vec<f32> = records.iter().map(|r| r.camber_rad.front_left).collect();
    let fr_camber: Vec<f32> = records.iter().map(|r| r.camber_rad.front_right).collect();
    let rl_camber: Vec<f32> = records.iter().map(|r| r.camber_rad.rear_left).collect();
    let rr_camber: Vec<f32> = records.iter().map(|r| r.camber_rad.rear_right).collect();

    let channels: &[(&str, &str, &[f32])] = &[
        ("Time", "s", &time),
        ("Speed", "km/h", &speed),
        ("RPM", "rpm", &rpm),
        ("Throttle", "%", &throttle),
        ("Brake", "%", &brake),
        ("Steer", "deg", &steer),
        ("Gear", "", &gear),
        ("Lat G", "g", &lat_g),
        ("Lon G", "g", &lon_g),
        ("Fuel", "", &fuel), // "L" not in MoTeC unit DB – use unitless
        ("FL Tyre Temp", "C", &fl_temp),
        ("FR Tyre Temp", "C", &fr_temp),
        ("RL Tyre Temp", "C", &rl_temp),
        ("RR Tyre Temp", "C", &rr_temp),
        ("Engine Brake", "", &engine_brake),
        ("TC Active", "", &tc_active),
        ("ABS Active", "", &abs_active),
        ("FL Load", "N", &fl_load),
        ("FR Load", "N", &fr_load),
        ("RL Load", "N", &rl_load),
        ("RR Load", "N", &rr_load),
        ("FL Camber", "rad", &fl_camber),
        ("FR Camber", "rad", &fr_camber),
        ("RL Camber", "rad", &rl_camber),
        ("RR Camber", "rad", &rr_camber),
    ];

    // Layout (from ldparser)
    // ldHead: 1762 bytes
    // ldEvent: 1154 bytes (if event_ptr > 0)
    // We use event_ptr = 0 to skip event block
    let event_ptr: u32 = 0;
    let head_size = 1762u32;
    let event_size = 1154u32;
    let chan_head_size = 124u32; // ldChan struct size from ldparser

    let meta_ptr = head_size + if event_ptr > 0 { event_size } else { 0 };
    let data_ptr = meta_ptr + channels.len() as u32 * chan_head_size;

    // Calculate data offsets per channel
    let _n = records.len() as u32;
    let mut data_offsets = Vec::with_capacity(channels.len());
    let mut offset = data_ptr;
    for (_, _, data) in channels {
        data_offsets.push(offset);
        offset += data.len() as u32 * 4; // f32 = 4 bytes
    }

    // Write ldHead
    write_ld_head(&mut f, meta_ptr, data_ptr, event_ptr, channels.len() as u32)?;

    // Write ldEvent block (zeros if event_ptr=0,但我们已跳过)
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
    for (_, _, data) in channels {
        for &v in *data {
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
