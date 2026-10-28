//! Live shared-memory capture that writes a MoTeC LD file on stop (no rkyv intermediate).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::acc_wait::open_or_wait;
use crate::config;
use crate::export::motec_ld;
use crate::record::{PhysicsRecord, StaticsRecord};
use crate::recorder::{poll_interval, TARGET_HZ};

/// Options for [`run`].
pub struct Options {
    /// Output directory for the `.ld` file. Default: `raw_output_dir` from config.
    pub out_dir: Option<PathBuf>,
}

/// Parse `--out <dir>` or `--out=<dir>` from CLI args.
pub fn parse_out_dir(args: &[String]) -> Option<PathBuf> {
    for i in 0..args.len() {
        if args[i] == "--out" {
            return args.get(i + 1).map(PathBuf::from);
        }
        if let Some(val) = args[i].strip_prefix("--out=") {
            return Some(PathBuf::from(val));
        }
    }
    None
}

/// Record physics from ACC shared memory until `running` is cleared or the stop file appears.
pub fn run(options: Options, running: &AtomicBool) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load_motec_config();

    let out_dir = match options.out_dir {
        Some(d) => {
            std::fs::create_dir_all(&d)?;
            d
        }
        None => {
            let d = config::resolve_path(&cfg.recorder.raw_output_dir);
            std::fs::create_dir_all(&d)?;
            d
        }
    };

    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let ld_path = out_dir.join(format!("acr_motec_{}.ld", secs));

    let mut stop_path = config::resolve_stop_file_path(&cfg.recorder);
    if stop_path.is_relative() {
        if let Ok(cwd) = std::env::current_dir() {
            stop_path = cwd.join(stop_path);
        }
    }
    if stop_path.exists() {
        let _ = std::fs::remove_file(&stop_path);
    }

    let motec_cfg = &cfg.export.motec;
    eprintln!(
        "MoTeC live: output → {} (profile={})",
        ld_path.display(),
        motec_cfg.profile
    );
    eprintln!("Ctrl+C or create '{}' to stop.", stop_path.display());

    let Some(mut acc) = open_or_wait(running, &stop_path)? else {
        return Ok(());
    };

    let mut statics: Option<StaticsRecord> = acc
        .read_shared_memory()?
        .map(|data| StaticsRecord::from_statics(&data.statics));

    let mut physics_records: Vec<PhysicsRecord> = Vec::new();
    let capture_start = std::time::Instant::now();

    let poll = poll_interval();
    let idle_sleep = Duration::from_millis(16);
    const IDLE_THRESHOLD: u32 = 20;
    let mut consecutive_none = 0u32;
    let mut last_print = std::time::Instant::now();
    let mut last_statics_debug = std::time::Instant::now();

    while running.load(Ordering::Relaxed) && !stop_requested(&stop_path) {
        if let Some(data) = acc.read_shared_memory()? {
            update_statics(
                &mut statics,
                &data.statics,
                &mut last_statics_debug,
            );

            // See the same check in `main.rs`'s record loop: the shared
            // memory block stays readable for as long as the game is open,
            // but its physics fields go flat at zero once the session
            // leaves Live/Replay (menu, results, paused). Skip those
            // samples so the exported .ld doesn't end in a dead flatline.
            if !crate::acc_wait::frame_is_live(&data) {
                consecutive_none = consecutive_none.saturating_add(1);
                let sleep = if consecutive_none >= IDLE_THRESHOLD {
                    idle_sleep
                } else {
                    poll
                };
                std::thread::sleep(sleep);
                continue;
            }

            consecutive_none = 0;
            physics_records.push(PhysicsRecord::from_physics(
                &data.physics,
                capture_start.elapsed().as_secs_f64(),
            ));

            if last_print.elapsed() >= Duration::from_secs(5) {
                let elapsed = physics_records.len() as f64 / TARGET_HZ as f64;
                eprintln!(
                    "{:.0}s | {} samples | {:.0} Hz",
                    elapsed,
                    physics_records.len(),
                    physics_records.len() as f64 / elapsed.max(0.001),
                );
                last_print = std::time::Instant::now();
            }
        } else {
            consecutive_none = consecutive_none.saturating_add(1);
            let sleep = if consecutive_none >= IDLE_THRESHOLD {
                idle_sleep
            } else {
                poll
            };
            std::thread::sleep(sleep);
        }
    }

    eprintln!("Stopped. {} samples — writing LD…", physics_records.len());

    if physics_records.is_empty() {
        eprintln!("No samples recorded — nothing to write.");
        return Ok(());
    }

    motec_ld::write_ld(&ld_path, &physics_records, TARGET_HZ)?;

    eprintln!("Wrote {}", ld_path.display());
    Ok(())
}

fn update_statics(
    statics: &mut Option<StaticsRecord>,
    incoming: &acc_shared_memory_rs::maps::StaticsMap,
    last_debug: &mut std::time::Instant,
) {
    let statics_missing = statics.is_none();
    let track_missing = statics.as_ref().map_or(true, |s| s.track.trim().is_empty());
    let incoming_has_content = statics_has_content(incoming);
    let incoming_has_track = !incoming.track.trim().is_empty();

    if (statics_missing && incoming_has_content) || (track_missing && incoming_has_track) {
        *statics = Some(StaticsRecord::from_statics(incoming));
        let track = incoming.track.trim();
        if track.is_empty() {
            eprintln!(
                "Captured statics (no track yet, car={})",
                incoming.car_model
            );
        } else {
            eprintln!("Track: {}", track);
        }
    } else if track_missing && last_debug.elapsed() >= Duration::from_secs(10) {
        eprintln!(
            "Waiting for track (raw='{}', car='{}')",
            incoming.track, incoming.car_model
        );
        *last_debug = std::time::Instant::now();
    }
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

fn stop_requested(stop_path: &Path) -> bool {
    if stop_path.exists() {
        let _ = std::fs::remove_file(stop_path);
        true
    } else {
        false
    }
}
