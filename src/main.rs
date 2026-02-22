//! ACC/AC Rally physics recorder at ~333 Hz.
//!
//! Records complete physics data to rkyv format.
//! F9 = start/pause, Ctrl+C = stop and flush.
//!
//! Usage: acr_recorder [--graphics]
//!   --graphics: Also record GraphicsMap data (~60 Hz)

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use acc_shared_memory_rs::{ACCError, ACCSharedMemory};

use acr_recorder::{config, record::{GraphicsRecord, PhysicsRecord, StaticsRecord}, recorder::Recorder};

static RUNNING: AtomicBool = AtomicBool::new(true);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    acr_recorder::hotkey::spawn_hotkey_thread();
    ctrlc_handler();

    let args: Vec<String> = std::env::args().collect();
    let record_graphics = args.iter().any(|a| a == "--graphics");
    
    if record_graphics {
        eprintln!("GraphicsMap recording enabled (experimental - may be all zeros in AC Rally)");
    }

    let output_path = output_path()?;
    eprintln!("Recording to: {}", output_path.display());
    eprintln!("F9 = stop recording, Ctrl+C = force quit.");

    let mut acc = ACCSharedMemory::new()?;
    
    // Capture statics once at start
    let statics = acc.read_shared_memory()?
        .map(|data| StaticsRecord::from_statics(&data.statics));
    
    let mut recorder = Recorder::new(&output_path, statics.as_ref(), record_graphics)?;

    let poll_interval = acr_recorder::recorder::poll_interval();
    let mut last_print = std::time::Instant::now();
    let mut last_graphics_capture = std::time::Instant::now();
    let graphics_interval = Duration::from_millis(16); // ~60 Hz

    while RUNNING.load(Ordering::Relaxed) && !acr_recorder::hotkey::should_stop() {
        if let Some(data) = acc.read_shared_memory()? {
            let record = PhysicsRecord::from_physics(&data.physics);
            recorder.record(record)?;
            
            // Record graphics at ~60 Hz (time-based) - only if enabled
            if record_graphics && last_graphics_capture.elapsed() >= graphics_interval {
                let graphics_record = GraphicsRecord::from_graphics(&data.graphics);
                recorder.record_graphics(graphics_record)?;
                last_graphics_capture = std::time::Instant::now();
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
            // No new data – short sleep to poll quickly and not miss 333 Hz
            std::thread::sleep(poll_interval);
        }
    }

    recorder.flush()?;
    eprintln!(
        "Done. Recorded {} samples in {:.1}s",
        recorder.sample_count(),
        recorder.elapsed().as_secs_f64()
    );

    Ok(())
}

fn output_path() -> Result<PathBuf, ACCError> {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| ACCError::InvalidData(e.to_string()))?
        .as_secs();
    let name = format!("acc_physics_{}.rkyv", secs);
    let cfg = config::load_config();
    let dir = config::resolve_path(&cfg.recorder.raw_output_dir);
    std::fs::create_dir_all(&dir).map_err(|e| ACCError::InvalidData(e.to_string()))?;
    Ok(dir.join(name))
}

fn ctrlc_handler() {
    ctrlc::set_handler(|| {
        RUNNING.store(false, Ordering::Relaxed);
    })
    .expect("could not set Ctrl+C handler");
}
