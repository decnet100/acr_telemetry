//! Export rkyv telemetry to CSV, LD, or SQLite.
//!
//! Usage:
//!   acr_export <input.rkyv> [--csv | --sqlite [db_path]]
//!   acr_export <directory>  [--csv | --sqlite [db_path]]  # batch
//!   acr_export --rawDir [--csv | --sqlite [db_path]]      # batch from config raw_output_dir
//!
//! If --csv/--sqlite omitted, uses config default_method.
//! Batch mode skips files that already have output (CSV exists or recording in DB).

use std::convert::TryInto;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use acr_recorder::config;
use acr_recorder::export::subtiming::{compute_subtiming_markers, write_subtiming_shapefile, ShpSample, SubtimingParams};
use acr_recorder::export::sqlite_export::RecordingNotesContent;
use acr_recorder::notes::{Annotation, RecordingNotesJson};
use acr_recorder::record::PhysicsRecord;
use shapefile::dbase::{FieldValue, Record, TableWriterBuilder};
use shapefile::{Point, Writer};

/// Build annotations for time synchronization from physics: first air_temp > 0, first speed_kmh > 0,
/// and each time air_temp crosses from <= 0 to > 0 (e.g. after returning to menu and re-entering).
/// Runs only during export (offline); no impact on recording performance.
fn sync_annotations_from_physics(records: &[PhysicsRecord], dt_sec: f64) -> Vec<Annotation> {
    let mut out = Vec::new();
    let mut added_first_speed = false;
    for (i, r) in records.iter().enumerate() {
        let t = i as f64 * dt_sec;
        // A) Each time air_temp crosses from <= 0 to > 0
        if r.air_temp > 0.0 && (i == 0 || records[i - 1].air_temp <= 0.0) {
            out.push(Annotation {
                time_offset_sec: t,
                time_end_sec: None,
                text: format!("air_temp > 0 ({:.1} °C)", r.air_temp),
                tag: "sync_air_temp_gt_0".into(),
            });
        }
        // B) First time speed_kmh > 3 only (threshold avoids noise at standstill)
        if !added_first_speed && r.speed_kmh > 3.0 && (i == 0 || records[i - 1].speed_kmh <= 3.0) {
            added_first_speed = true;
            out.push(Annotation {
                time_offset_sec: t,
                time_end_sec: None,
                text: format!("speed_kmh > 3 ({:.1} km/h)", r.speed_kmh),
                tag: "sync_speed_gt_0".into(),
            });
        }
    }
    out.sort_by(|a, b| a.time_offset_sec.partial_cmp(&b.time_offset_sec).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// Read <stem>.notes.json. Returns (content, annotations, start_utc, end_utc).
/// If notes/annotations are empty, caller may load from acr_notes using start/end.
fn read_notes_json(
    rkyv_path: &Path,
) -> Option<(RecordingNotesContent, Vec<acr_recorder::notes::Annotation>, String, String)> {
    let stem = rkyv_path.file_stem()?.to_str()?;
    let parent = rkyv_path.parent().unwrap_or(Path::new("."));
    let path = parent.join(format!("{}.notes.json", stem));
    let json_str = std::fs::read_to_string(&path).ok()?;
    let j: RecordingNotesJson = serde_json::from_str(&json_str).ok()?;
    let start_utc = j.recording_start_utc.clone();
    let end_utc = j.recording_end_utc.clone();
    let trim = |s: &str| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    };
    let notes = trim(&j.notes);
    let content = RecordingNotesContent {
        notes,
        laptime: j.fields.get("laptime").and_then(|s| trim(s)),
        result: j.fields.get("result").and_then(|s| trim(s)),
        driver_impression: j.fields.get("driver_impression").and_then(|s| trim(s)),
        tested_parameters: j.fields.get("tested_parameters").and_then(|s| trim(s)),
        conditions: j.fields.get("conditions").and_then(|s| trim(s)),
        setup_notes: j.fields.get("setup_notes").and_then(|s| trim(s)),
        session_goal: j.fields.get("session_goal").and_then(|s| trim(s)),
        incident: j.fields.get("incident").and_then(|s| trim(s)),
    };
    Some((content, j.annotations, start_utc, end_utc))
}

/// Result of the notes/label/tags interactive flow.
struct NotesFlowResult {
    notes_content: RecordingNotesContent,
    annotations: Vec<Annotation>,
    label: Option<String>,
    tags: Vec<String>,
}

/// Suggest recording label from first 5 voicenote texts.
fn suggest_label(annotations: &[Annotation]) -> String {
    annotations
        .iter()
        .filter(|a| a.tag == "voicenote")
        .take(5)
        .map(|a| a.text.as_str())
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Full interactive flow: notes (include/edit/delete), label (from first 5 notes), recording name, tags.
fn prompt_notes_and_label_and_tags(
    sync_ann: &[Annotation],
    user_ann: &[Annotation],
    notes_text: &str,
    source_file: &str,
    batch_mode: bool,
) -> Result<NotesFlowResult, Box<dyn std::error::Error>> {
    let voicenotes: Vec<_> = user_ann.iter().filter(|a| a.tag == "voicenote").collect();
    let suggested_label = if voicenotes.is_empty() {
        String::new()
    } else {
        suggest_label(user_ann)
    };

    let mut all: Vec<Annotation> = sync_ann.to_vec();
    all.extend(user_ann.iter().cloned());
    all.sort_by(|a, b| {
        a.time_offset_sec
            .partial_cmp(&b.time_offset_sec)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut resolved_notes = notes_text.to_string();
    let mut resolved_annotations = all.clone();

    let (label, tags) = if batch_mode && (voicenotes.is_empty() && notes_text.trim().is_empty()) {
        (
            if suggested_label.is_empty() {
                None
            } else {
                Some(suggested_label)
            },
            Vec::new(),
        )
    } else {
        if !voicenotes.is_empty() || !notes_text.trim().is_empty() {
            eprintln!("\n--- Notes for {} ---", source_file);
            for a in voicenotes.iter() {
                eprintln!("  {:.1}s: {}", a.time_offset_sec, a.text);
            }
            if !notes_text.trim().is_empty() {
                eprintln!("  (notes): {}", notes_text.trim());
            }
            eprintln!("------------------------\n");

            eprint!("Include these notes? (y/n) [y]: ");
            std::io::stdout().flush()?;
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf)?;
            let include_notes = !buf.trim().eq_ignore_ascii_case("n");
            if !include_notes {
                resolved_annotations = sync_ann.to_vec();
                resolved_notes = String::new();
            } else {
                eprint!("Edit or delete notes in editor? (y/n) [n]: ");
                std::io::stdout().flush()?;
                buf.clear();
                std::io::stdin().read_line(&mut buf)?;
                if buf.trim().eq_ignore_ascii_case("y") {
                    resolved_annotations = edit_annotations_in_editor(&resolved_annotations)?;
                }
            }
        }

        // Recompute label suggestion from final annotations (after include/edit)
        let suggested_label = suggest_label(&resolved_annotations);

        eprint!("Recording label [{}]: ", suggested_label);
        std::io::stdout().flush()?;
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        let label_input = buf.trim();
        let label = if label_input.is_empty() {
            if suggested_label.is_empty() {
                None
            } else {
                Some(suggested_label)
            }
        } else {
            Some(label_input.to_string())
        };

        eprint!("Tags (comma-separated, e.g. wet,qualifying): ");
        std::io::stdout().flush()?;
        buf.clear();
        std::io::stdin().read_line(&mut buf)?;
        let tags: Vec<String> = buf
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        (label, tags)
    };

    let notes_content = RecordingNotesContent {
        notes: if resolved_notes.trim().is_empty() {
            None
        } else {
            Some(resolved_notes.trim().to_string())
        },
        ..Default::default()
    };

    Ok(NotesFlowResult {
        notes_content,
        annotations: resolved_annotations,
        label,
        tags,
    })
}

fn edit_annotations_in_editor(annotations: &[Annotation]) -> Result<Vec<Annotation>, Box<dyn std::error::Error>> {
    let dir = env::temp_dir();
    let path = dir.join("acr_export_annotations.txt");
    let content: String = annotations
        .iter()
        .map(|a| format!("{:.2}\t{}\t{}", a.time_offset_sec, a.tag, a.text))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, content)?;

    let editor = env::var("EDITOR")
        .unwrap_or_else(|_| env::var("VISUAL").unwrap_or_else(|_| "notepad.exe".to_string()));
    let status = Command::new(editor.split_whitespace().next().unwrap_or("notepad"))
        .args(editor.split_whitespace().skip(1))
        .arg(&path)
        .status()?;
    if !status.success() {
        return Err("Editor exited with error".into());
    }

    let f = BufReader::new(File::open(&path)?);
    let mut out = Vec::new();
    for line in f.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() >= 3 {
            if let Ok(t) = parts[0].trim().parse::<f64>() {
                out.push(Annotation {
                    time_offset_sec: t,
                    time_end_sec: None,
                    text: parts[2].to_string(),
                    tag: parts[1].to_string(),
                });
            }
        }
    }
    let _ = std::fs::remove_file(&path);
    Ok(out)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let cfg = config::load_config();

    let (use_raw_dir, path_arg, do_sqlite, do_csv, do_shp, sqlite_db, downsample, subtiming) =
        parse_args(&args, &cfg)?;
    let input: PathBuf = if use_raw_dir {
        config::resolve_path(&cfg.recorder.raw_output_dir)
    } else if let Some(p) = path_arg {
        PathBuf::from(p)
    } else {
        print_usage();
        return Err("Need path or --rawDir".into());
    };

    if !input.exists() {
        return Err(format!("Not found: {}", input.display()).into());
    }

    let notes_dir = config::resolve_notes_dir(&cfg.recorder);
    if input.is_dir() {
        batch_export(
            &input,
            do_sqlite,
            do_csv,
            do_shp,
            &sqlite_db,
            &notes_dir,
            true,
            downsample,
            subtiming.as_ref(),
        )?;
    } else if input.extension().map_or(false, |e| e == "rkyv") {
        export_single(
            &input,
            do_sqlite,
            do_csv,
            do_shp,
            &sqlite_db,
            &notes_dir,
            false,
            downsample,
            subtiming.as_ref(),
        )?;
    } else {
        return Err(format!("Expected .rkyv file or directory: {}", input.display()).into());
    }

    Ok(())
}

fn parse_args(
    args: &[String],
    cfg: &config::Config,
) -> Result<
    (
        bool,
        Option<String>,
        bool,
        bool,
        bool,
        String,
        usize,
        Option<SubtimingParams>,
    ),
    Box<dyn std::error::Error>,
> {
    let mut use_raw_dir = false;
    let mut path_arg = None;
    let mut do_sqlite = false;
    let mut do_csv = false;
    let mut do_shp = false;
    let mut sqlite_db = String::new();
    let mut downsample: usize = 10;
    let mut subtiming: Option<SubtimingParams> = None;
    let mut i = 1;

    while i < args.len() {
        let a = &args[i];
        if a == "--rawDir" || a == "--raw-dir" {
            use_raw_dir = true;
        } else if a == "--sqlite" {
            do_sqlite = true;
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                sqlite_db = args[i + 1].clone();
                i += 1;
            } else {
                sqlite_db = config::resolve_path(&cfg.export.sqlite_db_path)
                    .to_string_lossy()
                    .into_owned();
            }
        } else if a == "--csv" {
            do_csv = true;
        } else if a == "--shp" || a == "--shape" || a == "--shapefile" {
            do_shp = true;
        } else if a == "--subtiming" {
            subtiming.get_or_insert_with(SubtimingParams::default);
        } else if a == "--subtiming-speed" {
            let v = args
                .get(i + 1)
                .ok_or("--subtiming-speed needs km/h value")?
                .parse::<f64>()?;
            subtiming.get_or_insert_with(SubtimingParams::default).speed_min_kmh = v;
            i += 1;
        } else if a == "--subtiming-steer" {
            let v = args
                .get(i + 1)
                .ok_or("--subtiming-steer needs max |steer| value")?
                .parse::<f64>()?;
            subtiming.get_or_insert_with(SubtimingParams::default).steer_max_abs = v;
            i += 1;
        } else if a == "--subtiming-min-m" {
            let v = args
                .get(i + 1)
                .ok_or("--subtiming-min-m needs meters")?
                .parse::<f64>()?;
            subtiming.get_or_insert_with(SubtimingParams::default).min_run_m = v;
            i += 1;
        } else if a == "--subtiming-min-sec" {
            let v = args
                .get(i + 1)
                .ok_or("--subtiming-min-sec needs seconds")?
                .parse::<f64>()?;
            subtiming.get_or_insert_with(SubtimingParams::default).min_run_sec = v;
            i += 1;
        } else if a == "--subtiming-merge-m" {
            let v = args
                .get(i + 1)
                .ok_or("--subtiming-merge-m needs meters")?
                .parse::<f64>()?;
            subtiming.get_or_insert_with(SubtimingParams::default).merge_close_m = v;
            i += 1;
        } else if a == "--subtiming-temporal-merge" {
            subtiming.get_or_insert_with(SubtimingParams::default).use_chain_order_merge = false;
        } else if a == "--downsample" {
            if i + 1 >= args.len() {
                return Err("--downsample requires a positive integer value".into());
            }
            let value = args[i + 1].parse::<usize>()?;
            if value == 0 {
                return Err("--downsample must be >= 1".into());
            }
            downsample = value;
            i += 1;
        } else if !a.starts_with('-') && path_arg.is_none() {
            path_arg = Some(a.clone());
        }
        i += 1;
    }

    if (do_sqlite as u8 + do_csv as u8 + do_shp as u8) > 1 {
        return Err("Use exactly one export format: --csv, --sqlite, or --shp".into());
    }

    let (do_sqlite, do_csv, do_shp) = match (do_sqlite, do_csv, do_shp) {
        (true, false, false) => (true, false, false),
        (false, true, false) => (false, true, false),
        (false, false, true) => (false, false, true),
        (false, false, false) => match cfg.export.default_method.to_lowercase().as_str() {
            "sqlite" => {
                if sqlite_db.is_empty() {
                    sqlite_db = config::resolve_path(&cfg.export.sqlite_db_path)
                        .to_string_lossy()
                        .into_owned();
                }
                (true, false, false)
            }
            "shp" | "shape" | "shapefile" => (false, false, true),
            _ => (false, true, false),
        },
        _ => unreachable!("format combination already validated"),
    };

    if do_sqlite && sqlite_db.is_empty() {
        sqlite_db = config::resolve_path(&cfg.export.sqlite_db_path)
            .to_string_lossy()
            .into_owned();
    }

    if subtiming.is_some() && !do_shp {
        return Err("--subtiming (or any --subtiming-*) requires --shp export mode".into());
    }

    Ok((
        use_raw_dir,
        path_arg,
        do_sqlite,
        do_csv,
        do_shp,
        sqlite_db,
        downsample,
        subtiming,
    ))
}

fn print_usage() {
    eprintln!("Usage: acr_export [--rawDir] [<input.rkyv|directory>] [--csv | --sqlite [db_path] | --shp] [--downsample N]");
    eprintln!("       --rawDir: use configured raw_output_dir, batch export (skips already exported)");
    eprintln!("       Batch: pass directory (or --rawDir) to export all .rkyv");
    eprintln!("       Single: pass .rkyv file");
    eprintln!("       --csv: export to CSV/LD (default if not configured)");
    eprintln!("       --sqlite [path]: export to SQLite (default path from config)");
    eprintln!("       --shp: export point Shapefile from <input>.graphics.rkyv");
    eprintln!("       --downsample N: keep every N-th graphics sample for --shp (default: 10)");
    eprintln!("       --subtiming: with --shp, also write <stem>.subtiming.shp (midpoints; merge order = lap + dist_m)");
    eprintln!("         Straight = speed > 50 km/h and |steer| < 0.1 (defaults; override below)");
    eprintln!("         Run kept if length >= 80 m OR duration >= 2 s; merge markers closer than 40 m (keep first)");
    eprintln!("       --subtiming-speed / --subtiming-steer / --subtiming-min-m / --subtiming-min-sec / --subtiming-merge-m");
    eprintln!("       --subtiming-temporal-merge: merge duplicates in time order only (no lap+dist_m sort)");
    eprintln!("       Config: ./acr_recorder.toml or ~/.config/acr_recorder/config.toml");
}

fn batch_export(
    dir: &Path,
    do_sqlite: bool,
    do_csv: bool,
    do_shp: bool,
    sqlite_db: &str,
    notes_dir: &PathBuf,
    batch_mode: bool,
    downsample: usize,
    subtiming: Option<&SubtimingParams>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rkyv_files: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            p.extension().map_or(false, |e| e == "rkyv")
                && !name.starts_with('.')
                && !name.contains(".graphics.rkyv")
        })
        .collect();
    rkyv_files.sort();

    let mut exported = 0;
    let mut skipped = 0;

    for input in &rkyv_files {
        let source_file = input.file_name().and_then(|n| n.to_str()).unwrap_or("unknown.rkyv");

        // Skip if already exported
        if do_sqlite {
            if acr_recorder::export::sqlite_export::recording_exists(sqlite_db, source_file)? {
                eprintln!("Skip (in DB): {}", input.display());
                skipped += 1;
                continue;
            }
        }
        if do_csv {
            let csv_path = input.with_extension("csv");
            if csv_path.exists() {
                eprintln!("Skip (CSV exists): {}", input.display());
                skipped += 1;
                continue;
            }
        }
        if do_shp {
            let shp_path = input.with_extension("points.shp");
            if shp_path.exists() {
                eprintln!("Skip (SHP exists): {}", input.display());
                skipped += 1;
                continue;
            }
        }

        match export_single(
            input,
            do_sqlite,
            do_csv,
            do_shp,
            sqlite_db,
            notes_dir,
            batch_mode,
            downsample,
            subtiming,
        ) {
            Ok(()) => exported += 1,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("No records") || msg.contains("empty") {
                    eprintln!("Empty file, skipping: {}", input.display());
                } else {
                    eprintln!("Corrupt or unreadable file, skipping: {} — {}", input.display(), msg);
                }
            }
        }
    }

    eprintln!("Batch done: {} exported, {} skipped", exported, skipped);
    Ok(())
}

fn export_single(
    input: &Path,
    do_sqlite: bool,
    do_csv: bool,
    do_shp: bool,
    sqlite_db: &str,
    notes_dir: &PathBuf,
    batch_mode: bool,
    downsample: usize,
    subtiming: Option<&SubtimingParams>,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_path = input.with_extension("json");
    let statics = std::fs::read_to_string(&json_path)
        .ok()
        .and_then(|json_str| serde_json::from_str::<serde_json::Value>(&json_str).ok())
        .and_then(|json| {
            json.get("statics")
                .and_then(|s| serde_json::from_value::<acr_recorder::record::StaticsRecord>(s.clone()).ok())
        });

    let (sample_rate, records) = acr_recorder::export::rkyv_reader::read_rkyv(input)?;
    eprintln!(
        "Read {} samples at {} Hz from {}",
        records.len(),
        sample_rate,
        input.display()
    );

    if records.is_empty() {
        return Err("No records in file".into());
    }

    let source_file = input
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown.rkyv");

    if do_sqlite {
        eprintln!("Notes directory: {}", notes_dir.display());

        let dt_sec = 1.0 / sample_rate as f64;
        let sync_ann = sync_annotations_from_physics(&records, dt_sec);

        let (notes_content, user_annotations, notes_text, start_utc, end_utc) =
            match read_notes_json(input) {
                Some((c, a, s, e)) => {
                    eprintln!("Recording: {} – {}", s, e);
                    // Only skip acr_notes load when we have actual user content (notes, fields, or non-sync annotations).
                    // Sync annotations (air_temp, speed_kmh) are regenerated from physics; ignore them here.
                    let has_user_notes = c.notes.as_ref().map_or(false, |n| !n.trim().is_empty())
                        || c.laptime.is_some()
                        || c.result.is_some()
                        || c.driver_impression.is_some()
                        || c.tested_parameters.is_some()
                        || c.conditions.is_some()
                        || c.setup_notes.is_some()
                        || c.session_goal.is_some()
                        || c.incident.is_some();
                    let has_user_annotations = a.iter().any(|ann| !ann.tag.starts_with("sync_"));
                    let has_any = has_user_notes || has_user_annotations;
                    let notes_txt = c.notes.clone().unwrap_or_default();
                    if has_any {
                        (
                            Some(c),
                            a,
                            notes_txt,
                            s,
                            e,
                        )
                    } else {
                        // Empty .notes.json – try loading from acr_notes
                        eprintln!(
                            "Loading notes from {}/acr_notes (filtered by recording time)",
                            notes_dir.display()
                        );
                        match acr_recorder::notes::load_notes_from_acr_notes(
                            notes_dir,
                            &s,
                            &e,
                        ) {
                            Ok((notes_body, ann)) => (
                                Some(RecordingNotesContent {
                                    notes: if notes_body.is_empty() {
                                        None
                                    } else {
                                        Some(notes_body.clone())
                                    },
                                    ..Default::default()
                                }),
                                ann,
                                notes_body,
                                s,
                                e,
                            ),
                            Err(_) => (None, Vec::new(), String::new(), s, e),
                        }
                    }
                }
                None => {
                    eprintln!("No .notes.json found; no recording times available for notes lookup.");
                    (None, Vec::new(), String::new(), String::new(), String::new())
                }
            };

        let flow = if !user_annotations.is_empty() || !notes_text.trim().is_empty() || !batch_mode {
            prompt_notes_and_label_and_tags(
                &sync_ann,
                &user_annotations,
                &notes_text,
                source_file,
                batch_mode,
            )?
        } else {
            NotesFlowResult {
                notes_content: notes_content.unwrap_or_default(),
                annotations: sync_ann.clone(),
                label: None,
                tags: Vec::new(),
            }
        };

        let rid = acr_recorder::export::sqlite_export::export_to_sqlite(
            sqlite_db,
            source_file,
            &records,
            sample_rate,
            statics.as_ref(),
            Some(&flow.notes_content),
            if flow.annotations.is_empty() {
                None
            } else {
                Some(&flow.annotations)
            },
            flow.label.as_deref(),
        )?;

        if !flow.tags.is_empty() {
            acr_recorder::export::sqlite_export::insert_tags_for_recording(
                sqlite_db,
                rid,
                &flow.tags,
            )?;
        }

        // Write final .notes.json (only when we have recording times)
        if !start_utc.is_empty() && !end_utc.is_empty() {
            let notes_for_json = flow.notes_content.notes.as_deref().unwrap_or("").to_string();
            let payload = acr_recorder::notes::RecordingNotesJson {
                recording_start_utc: start_utc,
                recording_end_utc: end_utc,
                notes: notes_for_json,
                fields: std::collections::HashMap::new(),
                annotations: flow.annotations,
            };
            let _ = acr_recorder::notes::write_notes_json(input, &payload);
        }

        eprintln!("Appended to {} (recording_id={})", sqlite_db, rid);

        let graphics_path = input.with_extension("graphics.rkyv");
        if graphics_path.exists() {
            let (graphics_sample_rate, graphics_records) =
                acr_recorder::export::rkyv_reader::read_graphics_rkyv(&graphics_path)?;
            acr_recorder::export::sqlite_export::export_graphics_to_sqlite(
                sqlite_db,
                rid,
                &graphics_records,
                graphics_sample_rate,
            )?;
            eprintln!("Exported {} graphics samples", graphics_records.len());
        }
        return Ok(());
    }

    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("export");
    let out_dir = input.parent().unwrap_or(Path::new("."));

    if do_csv {
        let csv_path = out_dir.join(format!("{}.csv", stem));
        let f = File::create(&csv_path)?;
        let mut w = BufWriter::new(f);
        acr_recorder::export::motec_csv::write_csv(&mut w, &records, sample_rate)?;
        w.flush()?;
        eprintln!("Wrote {}", csv_path.display());

        let mut ld_graphics: Option<(Vec<acr_recorder::record::GraphicsRecord>, u32)> = None;
        let graphics_path = input.with_extension("graphics.rkyv");
        if graphics_path.exists() {
            let (graphics_sample_rate, graphics_records) =
                acr_recorder::export::rkyv_reader::read_graphics_rkyv(&graphics_path)?;
            let graphics_csv_path = out_dir.join(format!("{}.graphics.csv", stem));
            let f = File::create(&graphics_csv_path)?;
            let mut w = BufWriter::new(f);
            acr_recorder::export::motec_csv::write_graphics_csv(
                &mut w,
                &graphics_records,
                graphics_sample_rate,
            )?;
            w.flush()?;
            eprintln!("Wrote {}", graphics_csv_path.display());
            ld_graphics = Some((graphics_records, graphics_sample_rate));
        }
        // LD always with CSV for single-file
        let ld_path = out_dir.join(format!("{}.ld", stem));
        if let Some((graphics_records, graphics_sample_rate)) = ld_graphics.as_ref() {
            acr_recorder::export::motec_ld::write_ld_with_graphics(
                &ld_path,
                &records,
                sample_rate,
                Some((graphics_records.as_slice(), *graphics_sample_rate)),
            )?;
        } else {
            acr_recorder::export::motec_ld::write_ld(&ld_path, &records, sample_rate)?;
        }
        eprintln!("Wrote {}", ld_path.display());
    }

    if do_shp {
        export_shapefile_points(input, downsample, subtiming)?;
    }

    Ok(())
}

fn export_shapefile_points(
    input: &Path,
    downsample: usize,
    subtiming: Option<&SubtimingParams>,
) -> Result<(), Box<dyn std::error::Error>> {
    let graphics_path = input.with_extension("graphics.rkyv");
    if !graphics_path.exists() {
        return Err(format!(
            "Missing graphics file for SHP export: {}",
            graphics_path.display()
        )
        .into());
    }

    let (sample_rate, graphics_records) =
        acr_recorder::export::rkyv_reader::read_graphics_rkyv(&graphics_path)?;
    if graphics_records.is_empty() {
        return Err("No graphics records in file".into());
    }
    let (physics_sample_rate, physics_records) = acr_recorder::export::rkyv_reader::read_rkyv(input)?;
    if physics_records.is_empty() {
        return Err("No physics records in file".into());
    }

    let shp_path = input.with_extension("points.shp");
    let table_builder = TableWriterBuilder::new()
        .add_numeric_field("idx".try_into()?, 12, 0)
        .add_numeric_field("t_sec".try_into()?, 12, 3)
        .add_numeric_field("lap".try_into()?, 8, 0)
        .add_numeric_field("dist_m".try_into()?, 12, 3)
        .add_numeric_field("speed_kmh".try_into()?, 12, 3)
        .add_numeric_field("steer_ang".try_into()?, 12, 5);
    let mut writer = Writer::from_path(&shp_path, table_builder)?;

    let mut written = 0usize;
    let mut sub_samples: Vec<ShpSample> = Vec::new();
    for (i, gr) in graphics_records.iter().enumerate().step_by(downsample) {
        let pt = Point::new(gr.car_coordinates_x as f64, gr.car_coordinates_z as f64);
        let physics_idx =
            ((i as f64 * physics_sample_rate as f64 / sample_rate as f64).round() as usize)
                .min(physics_records.len() - 1);
        let physics = &physics_records[physics_idx];
        let t_sec = i as f64 / sample_rate as f64;
        let speed = physics.speed_kmh as f64;
        let steer = physics.steer_angle as f64;
        if subtiming.is_some() {
            sub_samples.push(ShpSample {
                idx: i as u32,
                t_sec,
                lap: gr.completed_lap,
                dist_m: gr.distance_traveled as f64,
                x: gr.car_coordinates_x as f64,
                z: gr.car_coordinates_z as f64,
                speed_kmh: speed,
                steer_angle: steer,
            });
        }
        let mut rec = Record::default();
        rec.insert("idx".to_string(), FieldValue::Numeric(Some(i as f64)));
        rec.insert(
            "t_sec".to_string(),
            FieldValue::Numeric(Some(t_sec)),
        );
        rec.insert(
            "lap".to_string(),
            FieldValue::Numeric(Some(gr.completed_lap as f64)),
        );
        rec.insert(
            "dist_m".to_string(),
            FieldValue::Numeric(Some(gr.distance_traveled as f64)),
        );
        rec.insert(
            "speed_kmh".to_string(),
            FieldValue::Numeric(Some(speed)),
        );
        rec.insert(
            "steer_ang".to_string(),
            FieldValue::Numeric(Some(steer)),
        );
        writer.write_shape_and_record(&pt, &rec)?;
        written += 1;
    }

    eprintln!(
        "Wrote {} points to {} (downsample={} from {} graphics samples)",
        written,
        shp_path.display(),
        downsample,
        graphics_records.len()
    );

    if let Some(st) = subtiming {
        let markers = compute_subtiming_markers(&sub_samples, st);
        let sub_path = input.with_extension("subtiming.shp");
        write_subtiming_shapefile(&sub_path, &markers)?;
        eprintln!(
            "Wrote {} subtiming marker(s) to {}",
            markers.len(),
            sub_path.display()
        );
    }
    Ok(())
}
