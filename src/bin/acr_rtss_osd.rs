//! Push plain text to RTSS on-screen display via `RTSSSharedMemoryV2`.
//!
//! Typical usage:
//!   acr_rtss_osd --owner acr_track_match --follow "%APPDATA%/acr_telemetry/acr_detected_track.txt" --poll-ms 200
//!
//! Notes:
//! - Owner and text are passed through narrow (ANSI) C strings, like most RTSS samples.
//! - RTSS >= 2.7 uses the extended `szOSDEx` buffer (<=4095 chars + NUL).

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::PathBuf;
    use std::time::SystemTime;

    let args: Vec<String> = std::env::args().collect();
    let mut owner = "acr_rtss_osd".to_string();
    let mut text: Option<String> = None;
    let mut file: Option<PathBuf> = None;
    let mut poll_ms = 200u64;
    let mut slot = 0u32;
    let mut release = false;
    let mut clear_all = false;
    let mut dump = false;
    let mut follow = false;

    let mut i = 1;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "--owner" => {
                owner = args.get(i + 1).ok_or("--owner needs value")?.clone();
                i += 1;
            }
            "--text" => {
                text = Some(args.get(i + 1).ok_or("--text needs value")?.clone());
                i += 1;
            }
            "--file" => {
                file = Some(PathBuf::from(args.get(i + 1).ok_or("--file needs path")?));
                i += 1;
            }
            "--poll-ms" => {
                poll_ms = args
                    .get(i + 1)
                    .ok_or("--poll-ms needs ms")?
                    .parse::<u64>()?;
                i += 1;
            }
            "--slot" => {
                slot = args
                    .get(i + 1)
                    .ok_or("--slot needs integer")?
                    .parse::<u32>()?;
                i += 1;
            }
            "--release" => release = true,
            "--clear-all" => clear_all = true,
            "--dump" => dump = true,
            "--follow" => follow = true,
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    if release {
        acr_recorder::rtss_osd::release(&owner)?;
        return Ok(());
    }
    if clear_all {
        acr_recorder::rtss_osd::clear_all()?;
        return Ok(());
    }

    if dump {
        let d = acr_recorder::rtss_osd::debug_dump(32)?;
        eprintln!("{}", d);
        return Ok(());
    }

    if follow {
        let path = file.ok_or("Need --file with --follow")?;
        let mut last_mtime = SystemTime::UNIX_EPOCH;
        let mut last_content = String::new();
        loop {
            if let Ok(meta) = std::fs::metadata(&path) {
                if let Ok(mt) = meta.modified() {
                    if mt != last_mtime {
                        last_mtime = mt;
                        if let Ok(s) = std::fs::read_to_string(&path) {
                            let s = s.trim_end_matches(['\r', '\n']).to_string();
                            if s != last_content {
                                last_content = s.clone();
                                acr_recorder::rtss_osd::update(&owner, &s, slot)?;
                            }
                        }
                    }
                }
            }
            acr_recorder::rtss_osd::sleep_ms(poll_ms.min(2000) as u32);
        }
    }

    let msg = if let Some(t) = text {
        t
    } else if let Some(path) = file {
        std::fs::read_to_string(path)?.trim().to_string()
    } else {
        print_usage();
        return Err("Need --text or --file".into());
    };

    acr_recorder::rtss_osd::update(&owner, &msg, slot)?;
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("acr_rtss_osd is only supported on Windows.");
    std::process::exit(2);
}

#[cfg(windows)]
fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  acr_rtss_osd --owner NAME --text \"Hello\" [--slot N]");
    eprintln!("  acr_rtss_osd --owner NAME --file path.txt [--slot N]");
    eprintln!("  acr_rtss_osd --owner NAME --file path.txt --follow [--poll-ms 200] [--slot N]");
    eprintln!("  acr_rtss_osd --owner NAME --release");
    eprintln!("  acr_rtss_osd --clear-all");
    eprintln!("  acr_rtss_osd --dump");
}
