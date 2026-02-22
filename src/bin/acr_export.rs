//! Export rkyv telemetry to CSV, LD, or SQLite.
//!
//! Usage:
//!   acr_export <input.rkyv> [--csv | --sqlite [db_path]]
//!   acr_export <directory>  [--csv | --sqlite [db_path]]  # batch
//!   acr_export --rawDir [--csv | --sqlite [db_path]]      # batch from config raw_output_dir
//!
//! If --csv/--sqlite omitted, uses config default_method.
//! Batch mode skips files that already have output (CSV exists or recording in DB).

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use acr_recorder::config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let cfg = config::load_config();

    let (use_raw_dir, path_arg, do_sqlite, do_csv, sqlite_db) = parse_args(&args, &cfg)?;
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

    if input.is_dir() {
        batch_export(&input, do_sqlite, do_csv, &sqlite_db)?;
    } else if input.extension().map_or(false, |e| e == "rkyv") {
        export_single(&input, do_sqlite, do_csv, &sqlite_db)?;
    } else {
        return Err(format!("Expected .rkyv file or directory: {}", input.display()).into());
    }

    Ok(())
}

fn parse_args(
    args: &[String],
    cfg: &config::Config,
) -> Result<(bool, Option<String>, bool, bool, String), Box<dyn std::error::Error>> {
    let mut use_raw_dir = false;
    let mut path_arg = None;
    let mut do_sqlite = false;
    let mut do_csv = false;
    let mut sqlite_db = String::new();
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
        } else if !a.starts_with('-') && path_arg.is_none() {
            path_arg = Some(a.clone());
        }
        i += 1;
    }

    let (do_sqlite, do_csv) = match (do_sqlite, do_csv) {
        (true, true) => return Err("Use either --csv or --sqlite, not both".into()),
        (true, false) => (true, false),
        (false, true) => (false, true),
        (false, false) => match cfg.export.default_method.to_lowercase().as_str() {
            "sqlite" => {
                if sqlite_db.is_empty() {
                    sqlite_db = config::resolve_path(&cfg.export.sqlite_db_path)
                        .to_string_lossy()
                        .into_owned();
                }
                (true, false)
            }
            _ => (false, true),
        },
    };

    if do_sqlite && sqlite_db.is_empty() {
        sqlite_db = config::resolve_path(&cfg.export.sqlite_db_path)
            .to_string_lossy()
            .into_owned();
    }

    Ok((use_raw_dir, path_arg, do_sqlite, do_csv, sqlite_db))
}

fn print_usage() {
    eprintln!("Usage: acr_export [--rawDir] [<input.rkyv|directory>] [--csv | --sqlite [db_path]]");
    eprintln!("       --rawDir: use configured raw_output_dir, batch export (skips already exported)");
    eprintln!("       Batch: pass directory (or --rawDir) to export all .rkyv");
    eprintln!("       Single: pass .rkyv file");
    eprintln!("       --csv: export to CSV/LD (default if not configured)");
    eprintln!("       --sqlite [path]: export to SQLite (default path from config)");
    eprintln!("       Config: ./acr_recorder.toml or ~/.config/acr_recorder/config.toml");
}

fn batch_export(
    dir: &Path,
    do_sqlite: bool,
    do_csv: bool,
    sqlite_db: &str,
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

        export_single(input, do_sqlite, do_csv, sqlite_db)?;
        exported += 1;
    }

    eprintln!("Batch done: {} exported, {} skipped", exported, skipped);
    Ok(())
}

fn export_single(
    input: &Path,
    do_sqlite: bool,
    do_csv: bool,
    sqlite_db: &str,
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
        let rid = acr_recorder::export::sqlite_export::export_to_sqlite(
            sqlite_db,
            source_file,
            &records,
            sample_rate,
            statics.as_ref(),
        )?;
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
        }
    }

    // LD always with CSV for single-file
    let ld_path = out_dir.join(format!("{}.ld", stem));
    acr_recorder::export::motec_ld::write_ld(&ld_path, &records, sample_rate)?;
    eprintln!("Wrote {}", ld_path.display());

    Ok(())
}
