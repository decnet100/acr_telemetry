//! Shared "wait for ACC/AC Rally shared memory" helper, used by both the
//! rkyv recording path (`main.rs`) and the MoTeC-live path (`motec_live.rs`)
//! so neither crashes when the game hasn't created its shared-memory
//! segments yet.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use acc_shared_memory_rs::{ACCError, ACCSharedMemory};

fn stop_requested(stop_path: &Path) -> bool {
    if stop_path.exists() {
        let _ = std::fs::remove_file(stop_path);
        true
    } else {
        false
    }
}

/// Block until ACC shared memory is available, `running` is cleared
/// (Ctrl+C), or the stop file appears. Returns `Ok(None)` if stopped
/// before connecting.
pub fn open_or_wait(
    running: &AtomicBool,
    stop_path: &Path,
) -> Result<Option<ACCSharedMemory>, Box<dyn std::error::Error>> {
    let poll = Duration::from_millis(500);
    let mut last_msg = std::time::Instant::now() - Duration::from_secs(10);

    loop {
        if !running.load(Ordering::Relaxed) || stop_requested(stop_path) {
            eprintln!("Stopped before ACC shared memory became available.");
            return Ok(None);
        }

        match ACCSharedMemory::new() {
            Ok(acc) => {
                eprintln!("Connected to ACC shared memory.");
                return Ok(Some(acc));
            }
            Err(ACCError::SharedMemoryNotAvailable) => {
                if last_msg.elapsed() >= Duration::from_secs(5) {
                    eprintln!("Waiting for ACC shared memory (start ACC / enter a session)…");
                    last_msg = std::time::Instant::now();
                }
                std::thread::sleep(poll);
            }
            Err(e) => return Err(e.into()),
        }
    }
}

/// Whether a frame holds live driving data. ACC reports this in the graphics status (Live or Replay). Assetto Corsa
/// Rally never fills that status (it stays Off for the whole session), so there the physics decides: the page goes
/// flat at zero once the stage is left (menu, results), and a live car always has fuel or revs.
pub fn frame_is_live(data: &acc_shared_memory_rs::ACCMap) -> bool {
    data.graphics.status.is_active() || data.physics.fuel > 0.0 || data.physics.rpm > 0
}
