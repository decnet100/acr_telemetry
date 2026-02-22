//! F9 global hotkey to stop recording and exit.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use rdev::{listen, Event, EventType, Key};

static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

const DEBOUNCE_MS: u64 = 250;

pub fn recording_active() -> bool {
    true
}

pub fn should_stop() -> bool {
    STOP_REQUESTED.load(Ordering::Relaxed)
}

pub fn spawn_hotkey_thread() {
    thread::spawn(|| {
        let mut last_toggle = Instant::now();

        let callback = move |event: Event| {
            if let EventType::KeyPress(Key::F9) = event.event_type {
                if last_toggle.elapsed() >= Duration::from_millis(DEBOUNCE_MS) {
                    STOP_REQUESTED.store(true, Ordering::Relaxed);
                    eprintln!("F9: stopping recording...");
                    last_toggle = Instant::now();
                }
            }
        };

        if let Err(e) = listen(callback) {
            eprintln!("Hotkey listener error: {:?}", e);
        }
    });
}
