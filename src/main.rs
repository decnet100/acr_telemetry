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
    let output_path = output_path(&cfg)?;
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
    acr_recorder::notes::reset_notes_at_start(&notes_dir)?;
    let start_time = chrono::Utc::now();
    eprintln!("Recording to: {}", output_path.display());
    eprintln!("Ctrl+C to stop, or run acr_stop.bat / create {} to stop from game.", stop_path.display());

    let mut acc = ACCSharedMemory::new()?;
    
    // Capture statics once at start
    let statics = acc.read_shared_memory()?
        .map(|data| StaticsRecord::from_statics(&data.statics));
    
    let mut recorder = Recorder::new(&output_path, statics.as_ref(), record_graphics)?;

    let poll_interval = acr_recorder::recorder::poll_interval();
    let idle_sleep = Duration::from_millis(16); // when no data (e.g. menu), sleep longer to reduce CPU/input lag
    const IDLE_THRESHOLD: u32 = 20; // after this many consecutive Nones, use idle_sleep instead of poll_interval
    let mut consecutive_none = 0u32;
    let mut last_print = std::time::Instant::now();
    let mut last_elapsed_write = std::time::Instant::now();
    let mut last_graphics_capture = std::time::Instant::now();
    let graphics_interval = Duration::from_millis(16); // ~60 Hz

    while RUNNING.load(Ordering::Relaxed) && !stop_requested(&stop_path) {
        if let Some(data) = acc.read_shared_memory()? {
            consecutive_none = 0;
            let record = PhysicsRecord::from_physics(&data.physics);
            recorder.record(record)?;
            
            // Record graphics at ~60 Hz (time-based) - only if enabled
            if record_graphics && last_graphics_capture.elapsed() >= graphics_interval {
                let graphics_record = GraphicsRecord::from_graphics(&data.graphics);
                recorder.record_graphics(graphics_record)?;
                last_graphics_capture = std::time::Instant::now();
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
    if let Err(e) = acr_recorder::notes::save_notes_to_json(
        &output_path,
        &notes_dir,
        &start_time.to_rfc3339(),
        &end_time.to_rfc3339(),
    ) {
        eprintln!("Note: could not save notes JSON: {}", e);
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
