//! ACC/AC Rally physics recorder at ~333 Hz.
//!
//! Records complete physics data to rkyv format.
//! Ctrl+C or stop file (acr_stop) to stop. Run acr_stop.bat or create the stop file to stop from game.
//!
//! Usage: acr_recorder [--graphics | --no-graphics]
//!   --graphics: Record GraphicsMap data (~60 Hz); default when record_graphics = true in config.
//!   --no-graphics: Disable graphics recording (overrides config).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use acc_shared_memory_rs::{ACCError, ACCSharedMemory};

use acr_recorder::{config, record::{GraphicsRecord, PhysicsRecord, StaticsRecord}, recorder::Recorder};

static RUNNING: AtomicBool = AtomicBool::new(true);

#[derive(serde::Serialize)]
struct RingState {
    version: u32,
    prefix: String,
    slot_count: usize,
    current_slot: usize,
    previous_slot: usize,
    current_file: String,
    previous_file: String,
    updated_at_utc: String,
}

struct DistanceResetDetector {
    min_prev_m: f32,
    max_curr_m: f32,
    cooldown: Duration,
    last_distance: Option<f32>,
    last_rotation_at: Option<std::time::Instant>,
}

fn statics_has_content(s: &acc_shared_memory_rs::maps::StaticsMap) -> bool {
    !s.track.trim().is_empty()
        || !s.car_model.trim().is_empty()
        || !s.player_name.trim().is_empty()
        || !s.player_surname.trim().is_empty()
        || !s.player_nick.trim().is_empty()
        || s.max_rpm > 0
        || s.max_fuel > 0.0
}

impl DistanceResetDetector {
    fn new(min_prev_m: f32, max_curr_m: f32, cooldown_secs: u64) -> Self {
        Self {
            min_prev_m,
            max_curr_m,
            cooldown: Duration::from_secs(cooldown_secs.max(1)),
            last_distance: None,
            last_rotation_at: None,
        }
    }

    fn should_rotate(&mut self, current_distance: f32) -> bool {
        let previous = self.last_distance.replace(current_distance);
        let Some(previous_distance) = previous else {
            return false;
        };
        if previous_distance < self.min_prev_m || current_distance > self.max_curr_m {
            return false;
        }
        if let Some(last_rotation_at) = self.last_rotation_at {
            if last_rotation_at.elapsed() < self.cooldown {
                return false;
            }
        }
        self.last_rotation_at = Some(std::time::Instant::now());
        true
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ctrlc_handler();

    let args: Vec<String> = std::env::args().collect();
    let cli_graphics = args.iter().any(|a| a == "--graphics");
    let cli_no_graphics = args.iter().any(|a| a == "--no-graphics");

    let cfg = config::load_config();
    let record_graphics = if cli_no_graphics {
        false
    } else if cli_graphics {
        true
    } else {
        cfg.recorder.record_graphics
    };

    if record_graphics {
        eprintln!("GraphicsMap recording enabled (~60 Hz, for Grafana/distance_traveled)");
    }
    let ring_mode = cfg.recorder.ring_mode;
    let slot_count = cfg.recorder.ring_slots.max(2);
    let ring_prefix = cfg.recorder.ring_prefix.trim();
    let ring_prefix = if ring_prefix.is_empty() { "acc_ring" } else { ring_prefix };
    let output_path = if ring_mode {
        ring_slot_path(&cfg, ring_prefix, 0)?
    } else {
        output_path(&cfg)?
    };
    let ring_state_path = ring_state_path(&cfg, ring_prefix);
    let notes_dir = config::resolve_notes_dir(&cfg.recorder);
    let mut stop_path = config::resolve_stop_file_path(&cfg.recorder);
    if stop_path.is_relative() {
        if let Ok(cwd) = std::env::current_dir() {
            stop_path = cwd.join(stop_path);
        }
    }
    if stop_path.exists() {
        let _ = std::fs::remove_file(&stop_path);
    }
    let _ = std::fs::create_dir_all(&notes_dir);
    // Notes are no longer handled by recorder; acr_export reads acr_notes when exporting
    let start_time = chrono::Utc::now();
    eprintln!("Recording to: {}", output_path.display());
    if ring_mode {
        eprintln!("Ring mode enabled: {} slots, prefix '{}'", slot_count, ring_prefix);
    }
    eprintln!("Ctrl+C to stop, or run acr_stop.bat / create {} to stop from game.", stop_path.display());

    let mut acc = ACCSharedMemory::new()?;
    
    // Capture statics once at start
    let mut statics = acc.read_shared_memory()?
        .map(|data| StaticsRecord::from_statics(&data.statics));
    
    let mut recorder = Recorder::new(&output_path, statics.as_ref(), record_graphics)?;
    let mut current_output_path = output_path.clone();
    let mut current_slot: usize = 0;
    let mut previous_slot: usize = slot_count.saturating_sub(1);
    if ring_mode {
        write_ring_state(
            &ring_state_path,
            ring_prefix,
            slot_count,
            current_slot,
            previous_slot,
            &slot_file_name(ring_prefix, current_slot),
            &slot_file_name(ring_prefix, previous_slot),
        );
    }
    let mut reset_detector = DistanceResetDetector::new(
        cfg.recorder.distance_reset_min_prev_m,
        cfg.recorder.distance_reset_max_curr_m,
        cfg.recorder.distance_reset_cooldown_secs,
    );

    let poll_interval = acr_recorder::recorder::poll_interval();
    let idle_sleep = Duration::from_millis(16); // when no data (e.g. menu), sleep longer to reduce CPU/input lag
    const IDLE_THRESHOLD: u32 = 20; // after this many consecutive Nones, use idle_sleep instead of poll_interval
    let mut consecutive_none = 0u32;
    let mut last_print = std::time::Instant::now();
    let mut last_elapsed_write = std::time::Instant::now();
    let mut last_graphics_capture = std::time::Instant::now();
    let mut last_statics_debug = std::time::Instant::now();
    let graphics_interval = Duration::from_millis(16); // ~60 Hz

    while RUNNING.load(Ordering::Relaxed) && !stop_requested(&stop_path) {
        if let Some(data) = acc.read_shared_memory()? {
            let statics_missing = statics.is_none();
            let track_missing = statics
                .as_ref()
                .map_or(true, |s| s.track.trim().is_empty());
            let incoming_has_content = statics_has_content(&data.statics);
            let incoming_has_track = !data.statics.track.trim().is_empty();
            if (statics_missing && incoming_has_content) || (track_missing && incoming_has_track) {
                statics = Some(StaticsRecord::from_statics(&data.statics));
                if let Err(e) = acr_recorder::format_meta::write_format_metadata(
                    &current_output_path,
                    statics.as_ref(),
                ) {
                    eprintln!("Could not refresh metadata statics: {}", e);
                } else {
                    let track = data.statics.track.trim();
                    if track.is_empty() {
                        eprintln!("Captured statics without track yet (car_model={})", data.statics.car_model);
                    } else {
                        eprintln!("Resolved track from statics: {}", track);
                    }
                }
            } else if track_missing && last_statics_debug.elapsed() >= Duration::from_secs(10) {
                eprintln!(
                    "Waiting for non-empty statics.track (raw='{}', car_model='{}')",
                    data.statics.track,
                    data.statics.car_model
                );
                last_statics_debug = std::time::Instant::now();
            }
            consecutive_none = 0;
            let record = PhysicsRecord::from_physics(&data.physics);
            recorder.record(record)?;
            
            // Record graphics at ~60 Hz (time-based) - only if enabled
            if record_graphics && last_graphics_capture.elapsed() >= graphics_interval {
                let graphics_record = GraphicsRecord::from_graphics(&data.graphics);
                recorder.record_graphics(graphics_record)?;
                last_graphics_capture = std::time::Instant::now();
            }

            if ring_mode && record_graphics && cfg.recorder.rotate_on_distance_reset {
                let distance = data.graphics.distance_traveled;
                if reset_detector.should_rotate(distance) {
                    previous_slot = current_slot;
                    current_slot = (current_slot + 1) % slot_count;
                    let next_path = ring_slot_path(&cfg, ring_prefix, current_slot)?;
                    recorder = Recorder::new(&next_path, statics.as_ref(), record_graphics)?;
                    current_output_path = next_path;
                    write_ring_state(
                        &ring_state_path,
                        ring_prefix,
                        slot_count,
                        current_slot,
                        previous_slot,
                        &slot_file_name(ring_prefix, current_slot),
                        &slot_file_name(ring_prefix, previous_slot),
                    );
                    eprintln!(
                        "Ring rotate: slot {} -> {} (distance reset: {:.1}m)",
                        previous_slot,
                        current_slot,
                        distance
                    );
                }
            }

            // Write elapsed secs for batch scripts (e.g. acr_note_good.bat) about once per second
            if last_elapsed_write.elapsed() >= Duration::from_secs(1) {
                let _ = acr_recorder::notes::write_elapsed_secs(&notes_dir, recorder.elapsed().as_secs());
                last_elapsed_write = std::time::Instant::now();
            }
            // Progress every 5 seconds
            if last_print.elapsed() >= Duration::from_secs(5) {
                let elapsed = recorder.elapsed().as_secs_f64();
                let rate = recorder.sample_count() as f64 / elapsed.max(0.001);
                eprintln!(
                    "{:.0}s | {} samples | {:.0} Hz",
                    elapsed,
                    recorder.sample_count(),
                    rate
                );
                last_print = std::time::Instant::now();
            }
        } else {
            consecutive_none = consecutive_none.saturating_add(1);
            // Idle (e.g. menu): sleep longer to reduce CPU usage and keyboard lag
            let sleep = if consecutive_none >= IDLE_THRESHOLD {
                idle_sleep
            } else {
                poll_interval
            };
            std::thread::sleep(sleep);
        }
    }

    recorder.flush()?;
    let end_time = chrono::Utc::now();
    if let Err(e) = acr_recorder::notes::save_recording_times(
        &output_path,
        &start_time.to_rfc3339(),
        &end_time.to_rfc3339(),
    ) {
        eprintln!("Note: could not save recording times: {}", e);
    }
    eprintln!(
        "Done. Recorded {} samples in {:.1}s",
        recorder.sample_count(),
        recorder.elapsed().as_secs_f64()
    );

    Ok(())
}

fn output_path(cfg: &config::Config) -> Result<PathBuf, ACCError> {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| ACCError::InvalidData(e.to_string()))?
        .as_secs();
    let name = format!("acc_physics_{}.rkyv", secs);
    let dir = config::resolve_path(&cfg.recorder.raw_output_dir);
    std::fs::create_dir_all(&dir).map_err(|e| ACCError::InvalidData(e.to_string()))?;
    Ok(dir.join(name))
}

fn ring_state_path(cfg: &config::Config, ring_prefix: &str) -> PathBuf {
    let dir = config::resolve_path(&cfg.recorder.raw_output_dir);
    dir.join(format!("{}.state.json", ring_prefix))
}

fn slot_file_name(ring_prefix: &str, slot: usize) -> String {
    format!("{}_slot_{:02}.rkyv", ring_prefix, slot)
}

fn ring_slot_path(cfg: &config::Config, ring_prefix: &str, slot: usize) -> Result<PathBuf, ACCError> {
    let dir = config::resolve_path(&cfg.recorder.raw_output_dir);
    std::fs::create_dir_all(&dir).map_err(|e| ACCError::InvalidData(e.to_string()))?;
    Ok(dir.join(slot_file_name(ring_prefix, slot)))
}

fn write_ring_state(
    path: &Path,
    ring_prefix: &str,
    slot_count: usize,
    current_slot: usize,
    previous_slot: usize,
    current_file: &str,
    previous_file: &str,
) {
    let state = RingState {
        version: 1,
        prefix: ring_prefix.to_string(),
        slot_count,
        current_slot,
        previous_slot,
        current_file: current_file.to_string(),
        previous_file: previous_file.to_string(),
        updated_at_utc: chrono::Utc::now().to_rfc3339(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&state) {
        let _ = std::fs::write(path, json);
    }
}

fn stop_requested(stop_path: &Path) -> bool {
    if stop_path.exists() {
        let _ = std::fs::remove_file(stop_path);
        true
    } else {
        false
    }
}

fn ctrlc_handler() {
    ctrlc::set_handler(|| {
        RUNNING.store(false, Ordering::Relaxed);
    })
    .expect("could not set Ctrl+C handler");
}
