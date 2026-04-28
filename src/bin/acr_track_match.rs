//! Live/Offline track matching against reference tracks.
//!
//! Usage examples:
//!   acr_track_match --refs A.rkyv,B.rkyv,C.points.shp --input current.rkyv
//!   acr_track_match --refs A.rkyv,B.rkyv,C.rkyv --live

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use acc_shared_memory_rs::ACCSharedMemory;
use acr_recorder::config;
use acr_recorder::export::rkyv_reader;
use acr_recorder::export::subtiming::{SectorPassEvent, SectorPassTracker, SectorTravelDirection};
use serde::Deserialize;
use shapefile::dbase::FieldValue;

static RUNNING: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Copy, Debug)]
struct Point2 {
    x: f64,
    z: f64,
}

#[derive(Debug)]
struct ReferenceTrack {
    name: String,
    points: Vec<Point2>,
    headings: Vec<f64>,
}

#[derive(Debug)]
struct MatchScore {
    name: String,
    coarse_pass: bool,
    coarse_inlier_ratio: f64,
    mean_dist_m: f64,
    mean_heading_diff_rad: f64,
    final_score: f64,
}

#[derive(Clone, Debug)]
struct SectorBoundary {
    sector_id: i32,
    a: Point2,
    b: Point2,
}

#[derive(Clone, Debug)]
struct SectorSet {
    boundaries: Vec<SectorBoundary>,
    ring_ids: Vec<i32>,
}

#[derive(Debug)]
struct LiveTimingState {
    tracker: SectorPassTracker,
    ring_ids: Vec<i32>,
    last_anchor_t_sec: Option<f64>,
    last_anchor_instant: Option<Instant>,
    last_anchor_drive_m: Option<f64>,
    last_sector_idx: Option<usize>,
    cooldown_until: HashMap<usize, Instant>,
}

impl LiveTimingState {
    fn new(ring_ids: Vec<i32>) -> Self {
        Self {
            tracker: SectorPassTracker::new(ring_ids.len().max(1)),
            ring_ids,
            last_anchor_t_sec: None,
            last_anchor_instant: None,
            last_anchor_drive_m: None,
            last_sector_idx: None,
            cooldown_until: HashMap::new(),
        }
    }

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ctrlc_handler();
    let cfg = parse_args(std::env::args().collect())?;
    let ref_files = resolve_reference_files(&cfg.refs)?;
    let labels = load_labels(&cfg)?;
    let refs = load_references(
        &ref_files,
        cfg.downsample,
        cfg.min_ref_spacing_m,
        &labels,
    )?;
    if refs.is_empty() {
        return Err("No valid references loaded".into());
    }

    if cfg.live {
        run_live(&refs, &cfg)?;
        #[cfg(windows)]
        {
            if cfg.rtss {
                let _ = acr_recorder::rtss_osd::release(&cfg.rtss_owner);
            }
        }
    } else {
        let input = cfg
            .input
            .as_ref()
            .ok_or("Need --input <file.rkyv> unless --live is set")?;
        run_offline(&refs, input, &cfg)?;
    }
    Ok(())
}

#[derive(Debug)]
struct CliConfig {
    refs: Vec<PathBuf>,
    input: Option<PathBuf>,
    live: bool,
    downsample: usize,
    coarse_buffer_m: f64,
    coarse_required_ratio: f64,
    history_points: usize,
    live_rate_hz: u64,
    min_ref_spacing_m: f64,
    labels_path: Option<PathBuf>,
    overlay_file: PathBuf,
    rtss: bool,
    rtss_owner: String,
    rtss_slot: u32,
    rtss_clear_all: bool,
    sectors_shp: Option<PathBuf>,
    sector_track_field: String,
    sector_id_field: String,
    timing_db_path: PathBuf,
    sector_cross_cooldown_ms: u64,
    sector_search_radius_m: f64,
    track_keep_max_dist_m: f64,
    track_switch_min_gain: f64,
    track_lock_after_sec: f64,
}

fn parse_args(args: Vec<String>) -> Result<CliConfig, Box<dyn std::error::Error>> {
    if args.len() < 2 {
        print_usage();
        return Err("Missing arguments".into());
    }

    let mut refs: Vec<PathBuf> = Vec::new();
    let mut input: Option<PathBuf> = None;
    let mut live = false;
    let mut downsample = 10usize;
    let mut coarse_buffer_m = 30.0f64;
    let mut coarse_required_ratio = 0.5f64;
    let mut history_points = 200usize;
    let mut live_rate_hz = 5u64;
    let mut min_ref_spacing_m = 2.0f64;
    let mut labels_path: Option<PathBuf> = None;
    let mut overlay_file: Option<PathBuf> = None;
    let mut rtss = false;
    let mut rtss_owner = "acr_track_match".to_string();
    let mut rtss_slot = 0u32;
    let mut rtss_clear_all = false;
    let mut sectors_shp: Option<PathBuf> = None;
    let mut sector_track_field = "src_layer".to_string();
    let mut sector_id_field = "seg_id".to_string();
    let mut timing_db_path: Option<PathBuf> = None;
    let mut sector_cross_cooldown_ms = 500u64;
    let mut sector_search_radius_m = 25.0f64;
    let mut track_keep_max_dist_m = 15.0f64;
    let mut track_switch_min_gain = 0.8f64;
    let mut track_lock_after_sec = 10.0f64;

    let mut i = 1;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "--refs" => {
                let next = args.get(i + 1).ok_or("--refs needs comma-separated paths")?;
                refs = next
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(PathBuf::from)
                    .collect();
                i += 1;
            }
            "--input" => {
                let next = args.get(i + 1).ok_or("--input needs a .rkyv path")?;
                input = Some(PathBuf::from(next));
                i += 1;
            }
            "--live" => live = true,
            "--downsample" => {
                let v = args
                    .get(i + 1)
                    .ok_or("--downsample needs integer")?
                    .parse::<usize>()?;
                if v == 0 {
                    return Err("--downsample must be >= 1".into());
                }
                downsample = v;
                i += 1;
            }
            "--buffer" => {
                coarse_buffer_m = args
                    .get(i + 1)
                    .ok_or("--buffer needs meters value")?
                    .parse::<f64>()?;
                i += 1;
            }
            "--required-ratio" => {
                coarse_required_ratio = args
                    .get(i + 1)
                    .ok_or("--required-ratio needs value between 0 and 1")?
                    .parse::<f64>()?;
                i += 1;
            }
            "--history-points" => {
                history_points = args
                    .get(i + 1)
                    .ok_or("--history-points needs integer")?
                    .parse::<usize>()?;
                i += 1;
            }
            "--rate" => {
                live_rate_hz = args
                    .get(i + 1)
                    .ok_or("--rate needs integer Hz")?
                    .parse::<u64>()?
                    .max(1);
                i += 1;
            }
            "--min-ref-spacing" => {
                min_ref_spacing_m = args
                    .get(i + 1)
                    .ok_or("--min-ref-spacing needs meters value")?
                    .parse::<f64>()?;
                i += 1;
            }
            "--labels" => {
                labels_path = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--labels needs a TOML path")?,
                ));
                i += 1;
            }
            "--overlay-file" => {
                overlay_file = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--overlay-file needs a path")?,
                ));
                i += 1;
            }
            "--rtss" => rtss = true,
            "--rtss-owner" => {
                rtss_owner = args
                    .get(i + 1)
                    .ok_or("--rtss-owner needs a string")?
                    .clone();
                i += 1;
            }
            "--rtss-slot" => {
                rtss_slot = args
                    .get(i + 1)
                    .ok_or("--rtss-slot needs integer")?
                    .parse::<u32>()?;
                i += 1;
            }
            "--rtss-clear-all" => rtss_clear_all = true,
            "--sectors-shp" => {
                sectors_shp = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--sectors-shp needs .shp path")?,
                ));
                i += 1;
            }
            "--sector-track-field" => {
                sector_track_field = args
                    .get(i + 1)
                    .ok_or("--sector-track-field needs field name")?
                    .to_string();
                i += 1;
            }
            "--sector-id-field" => {
                sector_id_field = args
                    .get(i + 1)
                    .ok_or("--sector-id-field needs field name")?
                    .to_string();
                i += 1;
            }
            "--timing-db" => {
                timing_db_path = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--timing-db needs path")?,
                ));
                i += 1;
            }
            "--sector-cooldown-ms" => {
                sector_cross_cooldown_ms = args
                    .get(i + 1)
                    .ok_or("--sector-cooldown-ms needs integer")?
                    .parse::<u64>()?;
                i += 1;
            }
            "--sector-radius" => {
                sector_search_radius_m = args
                    .get(i + 1)
                    .ok_or("--sector-radius needs meters value")?
                    .parse::<f64>()?;
                i += 1;
            }
            "--track-keep-max-dist" => {
                track_keep_max_dist_m = args
                    .get(i + 1)
                    .ok_or("--track-keep-max-dist needs meters value")?
                    .parse::<f64>()?;
                i += 1;
            }
            "--track-switch-min-gain" => {
                track_switch_min_gain = args
                    .get(i + 1)
                    .ok_or("--track-switch-min-gain needs score delta")?
                    .parse::<f64>()?;
                i += 1;
            }
            "--track-lock-after-sec" => {
                track_lock_after_sec = args
                    .get(i + 1)
                    .ok_or("--track-lock-after-sec needs seconds value")?
                    .parse::<f64>()?;
                i += 1;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    if refs.is_empty() {
        return Err("Need --refs refA,refB,refC".into());
    }
    if !live && input.is_none() {
        return Err("Need --input <file.rkyv> for offline mode".into());
    }
    if !(0.0..=1.0).contains(&coarse_required_ratio) {
        return Err("--required-ratio must be between 0 and 1".into());
    }

    let overlay_file = overlay_file.unwrap_or_else(|| {
        let cfg = config::load_config();
        config::resolve_notes_dir(&cfg.recorder).join("acr_detected_track.txt")
    });
    let timing_db_path = timing_db_path.unwrap_or_else(|| {
        let cfg = config::load_config();
        config::resolve_notes_dir(&cfg.recorder).join("timing.db")
    });

    Ok(CliConfig {
        refs,
        input,
        live,
        downsample,
        coarse_buffer_m,
        coarse_required_ratio,
        history_points,
        live_rate_hz,
        min_ref_spacing_m,
        labels_path,
        overlay_file,
        rtss,
        rtss_owner,
        rtss_slot,
        rtss_clear_all,
        sectors_shp,
        sector_track_field,
        sector_id_field,
        timing_db_path,
        sector_cross_cooldown_ms,
        sector_search_radius_m,
        track_keep_max_dist_m,
        track_switch_min_gain,
        track_lock_after_sec,
    })
}

fn print_usage() {
    eprintln!("Usage: acr_track_match --refs A.rkyv,B.points.shp,C.rkyv|reference_tracks [--input current.rkyv | --live]");
    eprintln!("       --downsample N       Reference/query downsample step (default: 10)");
    eprintln!("       --buffer M           Coarse corridor radius in meters (default: 30)");
    eprintln!("       --required-ratio R   Coarse inlier ratio [0..1] (default: 0.5)");
    eprintln!("       --history-points N   Live history size (default: 200)");
    eprintln!("       --rate HZ            Live evaluation rate (default: 5)");
    eprintln!("       --min-ref-spacing M  Minimum spacing for loaded reference points (default: 2.0m)");
    eprintln!("       --labels FILE.toml   Optional labels mapping for reference files");
    eprintln!("       --overlay-file PATH  Write live detection message to file");
    eprintln!("       --rtss                 Also push message to RTSS OSD (Windows)");
    eprintln!("       --rtss-owner NAME      RTSS OSD owner id (default: acr_track_match)");
    eprintln!("       --rtss-slot N          Force RTSS slot N (0 = auto, default: 0)");
    eprintln!("       --rtss-clear-all       Clear all RTSS slots once at startup (careful: clears other OSD sources)");
    eprintln!("       --sectors-shp FILE.shp Optional sector boundaries LineString SHP (timing)");
    eprintln!("       --sector-track-field F Track field in sectors SHP (default: src_layer)");
    eprintln!("       --sector-id-field F    Sector id field in sectors SHP (default: seg_id)");
    eprintln!("       --timing-db PATH       Separate SQLite timing DB path (default: notes_dir/timing.db)");
    eprintln!("       --sector-cooldown-ms N Ignore re-trigger for same sector N ms (default: 500)");
    eprintln!("       --sector-radius M      Candidate search radius around player segment (default: 25m)");
    eprintln!("       --track-keep-max-dist M Keep current track while its mean_dist <= M (default: 15m)");
    eprintln!("       --track-switch-min-gain G Switch only if new score is better by >= G (default: 0.8)");
    eprintln!("       --track-lock-after-sec S Lock selected track after S seconds stable match (default: 10)");
}

#[derive(Debug, Deserialize, Default)]
struct TrackLabelsFile {
    #[serde(default)]
    labels: std::collections::HashMap<String, String>,
}

fn resolve_reference_files(ref_inputs: &[PathBuf]) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    for p in ref_inputs {
        if p.is_dir() {
            for entry in std::fs::read_dir(p)? {
                let path = entry?.path();
                if path
                    .extension()
                    .map(|e| e == "rkyv" || e == "shp")
                    .unwrap_or(false)
                {
                    out.push(path);
                }
            }
        } else {
            out.push(p.clone());
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn load_references(
    ref_paths: &[PathBuf],
    downsample: usize,
    min_ref_spacing_m: f64,
    labels: &std::collections::HashMap<String, String>,
) -> Result<Vec<ReferenceTrack>, Box<dyn std::error::Error>> {
    let mut refs = Vec::new();
    for p in ref_paths {
        let loaded = if p.extension().map(|e| e == "shp").unwrap_or(false) {
            load_points_from_shp(p)?
        } else if p.extension().map(|e| e == "rkyv").unwrap_or(false) {
            load_points_from_rkyv(p, downsample)?
        } else {
            return Err(format!("Unsupported reference file: {}", p.display()).into());
        };
        let loaded = thin_points_by_spacing(&loaded, min_ref_spacing_m);
        if loaded.len() < 5 {
            return Err(format!("Reference too short: {}", p.display()).into());
        }
        refs.push(ReferenceTrack {
            name: labels
                .get(
                    p.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown"),
                )
                .cloned()
                .unwrap_or_else(|| p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()),
            headings: compute_headings(&loaded),
            points: loaded,
        });
    }
    Ok(refs)
}

fn thin_points_by_spacing(points: &[Point2], min_spacing_m: f64) -> Vec<Point2> {
    if points.is_empty() || min_spacing_m <= 0.0 {
        return points.to_vec();
    }
    let mut out = Vec::with_capacity(points.len());
    let mut last = points[0];
    out.push(last);
    for &p in points.iter().skip(1) {
        if dist(last, p) >= min_spacing_m {
            out.push(p);
            last = p;
        }
    }
    if let Some(&tail) = points.last() {
        if out.last().map_or(true, |v| dist(*v, tail) > 0.1) {
            out.push(tail);
        }
    }
    out
}

fn load_points_from_rkyv(
    path: &Path,
    downsample: usize,
) -> Result<Vec<Point2>, Box<dyn std::error::Error>> {
    let graphics_path = path.with_extension("graphics.rkyv");
    let (_, g) = rkyv_reader::read_graphics_rkyv(&graphics_path)?;
    let pts = g
        .iter()
        .enumerate()
        .step_by(downsample)
        .map(|(_, r)| Point2 {
            x: r.car_coordinates_x as f64,
            z: r.car_coordinates_z as f64,
        })
        .collect();
    Ok(pts)
}

fn load_points_from_shp(path: &Path) -> Result<Vec<Point2>, Box<dyn std::error::Error>> {
    let mut reader = shapefile::Reader::from_path(path)?;
    let mut pts = Vec::new();
    for item in reader.iter_shapes_and_records() {
        let (shape, _) = item?;
        if let shapefile::Shape::Point(p) = shape {
            pts.push(Point2 { x: p.x, z: p.y });
        }
    }
    Ok(pts)
}

fn compute_headings(points: &[Point2]) -> Vec<f64> {
    let mut out = vec![0.0; points.len()];
    if points.len() < 2 {
        return out;
    }
    for i in 0..points.len() - 1 {
        out[i] = (points[i + 1].z - points[i].z).atan2(points[i + 1].x - points[i].x);
    }
    out[points.len() - 1] = out[points.len() - 2];
    out
}

fn run_offline(
    refs: &[ReferenceTrack],
    input: &Path,
    cfg: &CliConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let query = load_points_from_rkyv(input, cfg.downsample)?;
    let scores = match_tracks(&query, refs, cfg);
    print_scores(&scores);
    Ok(())
}

fn run_live(refs: &[ReferenceTrack], cfg: &CliConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut acc = ACCSharedMemory::new()?;
    let timing_conn = acr_recorder::timing_db::open_or_create(&cfg.timing_db_path)?;
    let sector_sets = if let Some(sectors_path) = &cfg.sectors_shp {
        load_sector_sets_from_shp(
            sectors_path,
            &cfg.sector_track_field,
            &cfg.sector_id_field,
            refs,
        )?
    } else {
        HashMap::new()
    };
    let mut history: VecDeque<Point2> = VecDeque::with_capacity(cfg.history_points + 10);
    let eval_interval = Duration::from_millis((1000 / cfg.live_rate_hz.max(1)) as u64);
    let mut last_eval = Instant::now();
    let mut last_no_data_log = Instant::now();
    let mut last_pt: Option<Point2> = None;
    let mut total_drive_m = 0.0f64;
    let mut timing_state: Option<LiveTimingState> = None;
    let mut active_track_name: Option<String> = None;
    let mut latest_timing_line: Option<(String, Instant)> = None;
    let mut sector_status_line: Option<(String, Instant)> = None;
    let mut detected_track_line: Option<(String, Instant)> = None;
    let mut stable_selected: Option<(String, Instant)> = None;
    let mut locked_track: Option<String> = None;
    let mut locked_car_model: Option<String> = None;
    let mut last_sector_wait_log = Instant::now();
    let mut last_overlay_msg: String = "detecting track...".to_string();
    let overlay_dir = cfg
        .overlay_file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let _ = std::fs::create_dir_all(&overlay_dir);
    #[cfg(windows)]
    {
        if cfg.rtss {
            // Always release our own owner on startup to avoid stale slot artifacts from prior runs.
            let _ = acr_recorder::rtss_osd::release(&cfg.rtss_owner);
            if cfg.rtss_clear_all {
                match acr_recorder::rtss_osd::clear_all() {
                    Ok(()) => eprintln!("RTSS cleanup: cleared all OSD slots."),
                    Err(e) => eprintln!("RTSS cleanup failed: {}", e),
                }
            }
        }
    }
    push_live_overlay(cfg, &last_overlay_msg)?;
    eprintln!("live mode started; waiting for ACC shared memory...");

    while RUNNING.load(Ordering::Relaxed) {
        if let Some(data) = acc.read_shared_memory()? {
            let car_model_now = data.statics.car_model.trim().to_string();
            let speed_kmh_now = data.physics.speed_kmh as f64;
            if let Some(lock_car) = &locked_car_model {
                if !car_model_now.is_empty() && car_model_now != *lock_car {
                    eprintln!(
                        "unlocking track lock due to car change: '{}' -> '{}'",
                        lock_car, car_model_now
                    );
                    locked_track = None;
                    locked_car_model = None;
                    stable_selected = None;
                }
            }
            if locked_track.is_some() && speed_kmh_now < 2.0 {
                eprintln!(
                    "unlocking track lock due to low speed: {:.1} km/h",
                    speed_kmh_now
                );
                locked_track = None;
                locked_car_model = None;
                stable_selected = None;
            }
            if last_no_data_log.elapsed() >= Duration::from_secs(3) {
                eprintln!("ACC shared memory connected.");
            }
            last_no_data_log = Instant::now();
            let default_coords = acc_shared_memory_rs::datatypes::Vector3f::new(0.0, 0.0, 0.0);
            let player_coords = data
                .graphics
                .car_coordinates
                .iter()
                .zip(&data.graphics.car_id)
                .find(|&(_, &id)| id == data.graphics.player_car_id)
                .map(|(coords, _)| coords)
                .unwrap_or(&default_coords);
            let p = Point2 {
                x: player_coords.x as f64,
                z: player_coords.z as f64,
            };
            if let Some(lp) = last_pt {
                total_drive_m += dist(lp, p);
            }
            if last_pt.map_or(true, |lp| dist(lp, p) > 0.05) {
                history.push_back(p);
                if history.len() > cfg.history_points {
                    history.pop_front();
                }
                if let Some(track_name) = &active_track_name {
                    if let Some(set) = sector_sets.get(track_name) {
                        if timing_state.is_none() {
                            timing_state = Some(LiveTimingState::new(set.ring_ids.clone()));
                        }
                        if let Some(state) = timing_state.as_mut() {
                            if let Some(lp) = last_pt {
                                if let Some((cross_idx, _t)) =
                                    first_crossed_sector(lp, p, &set.boundaries, cfg.sector_search_radius_m)
                                {
                                    let now = Instant::now();
                                    if state
                                        .cooldown_until
                                        .get(&cross_idx)
                                        .map_or(false, |until| now < *until)
                                    {
                                        // still cooling down for this sector, ignore
                                    } else {
                                        state.cooldown_until.insert(
                                            cross_idx,
                                            now + Duration::from_millis(cfg.sector_cross_cooldown_ms),
                                        );
                                        match state.tracker.observe(cross_idx) {
                                            SectorPassEvent::Anchored { sector } => {
                                                state.last_anchor_t_sec = Some(data.graphics.clock as f64);
                                                state.last_anchor_instant = Some(Instant::now());
                                                state.last_anchor_drive_m = Some(total_drive_m);
                                                state.last_sector_idx = Some(sector);
                                            }
                                            SectorPassEvent::Step { from, to, direction } => {
                                                let now_t = data.graphics.clock as f64;
                                                let now_inst = Instant::now();
                                                if let (Some(prev_t), Some(prev_m)) =
                                                    (state.last_anchor_t_sec, state.last_anchor_drive_m)
                                                {
                                                    let dt = state
                                                        .last_anchor_instant
                                                        .map(|t| now_inst.duration_since(t).as_secs_f64())
                                                        .unwrap_or_else(|| {
                                                            let mut x = now_t - prev_t;
                                                            if x < 0.0 {
                                                                x += 24.0 * 3600.0;
                                                            }
                                                            x
                                                        });
                                                    let dist_m = (total_drive_m - prev_m).max(0.0);
                                                    if dt > 0.05 {
                                                        let from_sector_id = state.ring_ids[from];
                                                        let to_sector_id = state.ring_ids[to];
                                                        let direction_s = match direction {
                                                            SectorTravelDirection::Increasing => "inc",
                                                            SectorTravelDirection::Decreasing => "dec",
                                                        };
                                                        let car_model =
                                                            data.statics.car_model.trim();
                                                        let car_model = if car_model.is_empty() {
                                                            "unknown_car"
                                                        } else {
                                                            car_model
                                                        };

                                                        let rec = acr_recorder::timing_db::SplitRecord {
                                                            track_name,
                                                            car_model,
                                                            direction: direction_s,
                                                            from_sector: from_sector_id,
                                                            to_sector: to_sector_id,
                                                            duration_sec: dt,
                                                            distance_m: dist_m,
                                                        };
                                                        let _ = acr_recorder::timing_db::insert_split(
                                                            &timing_conn,
                                                            &rec,
                                                        );
                                                        let best = acr_recorder::timing_db::best_time(
                                                            &timing_conn,
                                                            track_name,
                                                            car_model,
                                                            direction_s,
                                                            from_sector_id,
                                                            to_sector_id,
                                                        )
                                                        .ok()
                                                        .flatten();
                                                        let delta = best.map(|b| dt - b).unwrap_or(0.0);
                                                        let sign = if delta >= 0.0 { "+" } else { "-" };
                                                        let line = format!(
                                                            "sector [{}]-[{}]: {:.3}s ({}{:0.3}s)",
                                                            from_sector_id,
                                                            to_sector_id,
                                                            dt,
                                                            sign,
                                                            delta.abs()
                                                        );
                                                        eprintln!("{line}");
                                                        latest_timing_line = Some((line.clone(), Instant::now()));
                                                        if active_track_name.is_some() {
                                                            let immediate = line;
                                                            if immediate != last_overlay_msg {
                                                                let _ = push_live_overlay(cfg, &immediate);
                                                                last_overlay_msg = immediate;
                                                            }
                                                        }
                                                    }
                                                }
                                                eprintln!("sector passed [{}]", state.ring_ids[to]);
                                                if active_track_name.is_some() {
                                                    let passed_line = format!("sector passed [{}]", state.ring_ids[to]);
                                                    sector_status_line = Some((passed_line.clone(), Instant::now()));
                                                    let immediate = passed_line;
                                                    if immediate != last_overlay_msg {
                                                        let _ = push_live_overlay(cfg, &immediate);
                                                        last_overlay_msg = immediate;
                                                    }
                                                }
                                                state.last_anchor_t_sec = Some(now_t);
                                                state.last_anchor_instant = Some(now_inst);
                                                state.last_anchor_drive_m = Some(total_drive_m);
                                                state.last_sector_idx = Some(to);
                                            }
                                            SectorPassEvent::NoStep { .. }
                                            | SectorPassEvent::Unexpected { .. }
                                            | SectorPassEvent::DirectionConflict { .. } => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                last_pt = Some(p);
            }
            if history.len() > 20 && last_eval.elapsed() >= eval_interval {
                if let Some(locked_name) = locked_track.as_deref() {
                    if active_track_name.as_deref() != Some(locked_name) {
                        active_track_name = Some(locked_name.to_string());
                    }
                    if timing_state.is_none() {
                        if let Some(s) = sector_sets.get(locked_name) {
                            timing_state = Some(LiveTimingState::new(s.ring_ids.clone()));
                        }
                    }
                    if last_sector_wait_log.elapsed() >= Duration::from_secs(5) {
                        eprintln!("track locked: {}", locked_name);
                        last_sector_wait_log = Instant::now();
                    }
                    let msg = if let Some((line, ts)) = &latest_timing_line {
                        if ts.elapsed() <= Duration::from_secs(8) {
                            line.to_string()
                        } else if let Some((sline, sts)) = &sector_status_line {
                            if sts.elapsed() <= Duration::from_secs(8) {
                                sline.to_string()
                            } else {
                                format!("track locked {}", locked_name)
                            }
                        } else {
                            format!("track locked {}", locked_name)
                        }
                    } else if let Some((sline, sts)) = &sector_status_line {
                        if sts.elapsed() <= Duration::from_secs(8) {
                            sline.to_string()
                        } else {
                            format!("track locked {}", locked_name)
                        }
                    } else {
                        format!("track locked {}", locked_name)
                    };
                    if msg != last_overlay_msg {
                        push_live_overlay(cfg, &msg)?;
                        last_overlay_msg = msg;
                    }
                    last_eval = Instant::now();
                    continue;
                }

                let query: Vec<Point2> = history.iter().copied().collect();
                let scores = match_tracks(&query, refs, cfg);
                if let Some(best) = scores.first() {
                    // Track hysteresis: keep current track while still plausible and
                    // only switch when candidate is clearly better.
                    let selected = if best.coarse_pass {
                        if let Some(active_name) = active_track_name.as_deref() {
                            if let Some(active_score) = scores.iter().find(|s| s.name == active_name) {
                                if active_score.coarse_pass
                                    && active_score.mean_dist_m <= cfg.track_keep_max_dist_m
                                    && best.name != active_name
                                {
                                    let gain = active_score.final_score - best.final_score;
                                    if gain < cfg.track_switch_min_gain {
                                        active_score
                                    } else {
                                        best
                                    }
                                } else {
                                    best
                                }
                            } else {
                                best
                            }
                        } else {
                            best
                        }
                    } else {
                        best
                    };

                    if selected.coarse_pass {
                        if let Some((name, since)) = &stable_selected {
                            if name == &selected.name {
                                if since.elapsed().as_secs_f64() >= cfg.track_lock_after_sec {
                                    if locked_track.as_deref() != Some(selected.name.as_str()) {
                                        locked_track = Some(selected.name.clone());
                                        locked_car_model = if car_model_now.is_empty() {
                                            None
                                        } else {
                                            Some(car_model_now.clone())
                                        };
                                        eprintln!(
                                            "track locked after {:.1}s stable: {} (car={})",
                                            cfg.track_lock_after_sec,
                                            selected.name,
                                            locked_car_model.as_deref().unwrap_or("unknown")
                                        );
                                    }
                                }
                            } else {
                                stable_selected = Some((selected.name.clone(), Instant::now()));
                            }
                        } else {
                            stable_selected = Some((selected.name.clone(), Instant::now()));
                        }
                        if active_track_name.as_deref() != Some(selected.name.as_str()) {
                            active_track_name = Some(selected.name.clone());
                            timing_state = if let Some(s) = sector_sets.get(&selected.name) {
                                let line = "waiting for sector passing...".to_string();
                                eprintln!("{} ({})", line, selected.name);
                                sector_status_line = Some((line, Instant::now()));
                                detected_track_line =
                                    Some((format!("detected track {}", selected.name), Instant::now()));
                                Some(LiveTimingState::new(s.ring_ids.clone()))
                            } else {
                                let line = "no sector set for detected track".to_string();
                                eprintln!("{} ({})", line, selected.name);
                                sector_status_line = Some((line, Instant::now()));
                                detected_track_line =
                                    Some((format!("detected track {}", selected.name), Instant::now()));
                                None
                            };
                            last_sector_wait_log = Instant::now();
                        }
                    } else {
                        stable_selected = None;
                        active_track_name = None;
                        timing_state = None;
                        sector_status_line = None;
                        detected_track_line = None;
                    }
                    let base_msg = if best.coarse_pass {
                        "".to_string()
                    } else {
                        "detecting track...".to_string()
                    };
                    if best.coarse_pass && timing_state.is_some() && latest_timing_line.is_none() {
                        if last_sector_wait_log.elapsed() >= Duration::from_secs(3) {
                            eprintln!("waiting for sector passing...");
                            last_sector_wait_log = Instant::now();
                        }
                    }
                    let msg = if let Some((line, ts)) = &latest_timing_line {
                        if ts.elapsed() <= Duration::from_secs(8) {
                            line.to_string()
                        } else if let Some((sline, sts)) = &sector_status_line {
                            if sts.elapsed() <= Duration::from_secs(8) {
                                sline.to_string()
                            } else if let Some((dline, dts)) = &detected_track_line {
                                if dts.elapsed() <= Duration::from_secs(5) {
                                    dline.to_string()
                                } else {
                                    base_msg
                                }
                            } else {
                                base_msg
                            }
                        } else {
                            base_msg
                        }
                    } else if let Some((sline, sts)) = &sector_status_line {
                        if sts.elapsed() <= Duration::from_secs(8) {
                            sline.to_string()
                        } else if let Some((dline, dts)) = &detected_track_line {
                            if dts.elapsed() <= Duration::from_secs(5) {
                                dline.to_string()
                            } else {
                                base_msg
                            }
                        } else {
                            base_msg
                        }
                    } else if let Some((dline, dts)) = &detected_track_line {
                        if dts.elapsed() <= Duration::from_secs(5) {
                            dline.to_string()
                        } else {
                            base_msg
                        }
                    } else {
                        base_msg
                    };
                    if msg != last_overlay_msg {
                        push_live_overlay(cfg, &msg)?;
                        last_overlay_msg = msg;
                    }
                    eprintln!(
                        "best={} sel={} score={:.2} dist={:.2}m coarse={:.0}%",
                        best.name,
                        selected.name,
                        selected.final_score,
                        selected.mean_dist_m,
                        selected.coarse_inlier_ratio * 100.0
                    );
                }
                last_eval = Instant::now();
            }
        } else {
            if last_no_data_log.elapsed() >= Duration::from_secs(3) {
                eprintln!("waiting for ACC shared memory data...");
                last_no_data_log = Instant::now();
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    Ok(())
}

fn field_value_to_string(v: Option<&FieldValue>) -> Option<String> {
    match v? {
        FieldValue::Character(Some(s)) => Some(s.trim().to_string()),
        FieldValue::Numeric(Some(n)) => Some(format!("{n:.0}")),
        FieldValue::Float(Some(f)) => Some(format!("{f:.0}")),
        FieldValue::Integer(i) => Some(i.to_string()),
        FieldValue::Double(d) => Some(format!("{d:.0}")),
        FieldValue::Logical(Some(b)) => Some(if *b { "1".into() } else { "0".into() }),
        _ => None,
    }
}

fn field_value_to_i32(v: Option<&FieldValue>) -> Option<i32> {
    match v? {
        FieldValue::Numeric(Some(n)) => Some(*n as i32),
        FieldValue::Float(Some(f)) => Some(*f as i32),
        FieldValue::Integer(i) => Some(*i),
        FieldValue::Double(d) => Some(*d as i32),
        FieldValue::Character(Some(s)) => s.trim().parse::<i32>().ok(),
        _ => None,
    }
}

fn normalize_track_key(s: &str) -> String {
    s.trim().to_lowercase().replace(' ', "_")
}

fn load_sector_sets_from_shp(
    shp_path: &Path,
    track_field: &str,
    sector_id_field: &str,
    refs: &[ReferenceTrack],
) -> Result<HashMap<String, SectorSet>, Box<dyn std::error::Error>> {
    let mut grouped: HashMap<String, Vec<SectorBoundary>> = HashMap::new();
    let mut reader = shapefile::Reader::from_path(shp_path)?;
    for item in reader.iter_shapes_and_records() {
        let (shape, rec) = item?;
        let track_name = field_value_to_string(rec.get(track_field))
            .ok_or_else(|| format!("Missing or invalid '{track_field}' in sectors SHP"))?;
        let sector_id = field_value_to_i32(rec.get(sector_id_field))
            .ok_or_else(|| format!("Missing or invalid '{sector_id_field}' in sectors SHP"))?;
        let (a, b) = match shape {
            shapefile::Shape::Polyline(pl) => {
                let first_part = pl.parts().first();
                if let Some(part) = first_part {
                    if part.len() < 2 {
                        continue;
                    }
                    let pa = part.first().unwrap();
                    let pb = part.last().unwrap();
                    (
                        Point2 { x: pa.x, z: pa.y },
                        Point2 { x: pb.x, z: pb.y },
                    )
                } else {
                    continue;
                }
            }
            shapefile::Shape::PolylineM(pl) => {
                let first_part = pl.parts().first();
                if let Some(part) = first_part {
                    if part.len() < 2 {
                        continue;
                    }
                    let pa = part.first().unwrap();
                    let pb = part.last().unwrap();
                    (
                        Point2 { x: pa.x, z: pa.y },
                        Point2 { x: pb.x, z: pb.y },
                    )
                } else {
                    continue;
                }
            }
            shapefile::Shape::PolylineZ(pl) => {
                let first_part = pl.parts().first();
                if let Some(part) = first_part {
                    if part.len() < 2 {
                        continue;
                    }
                    let pa = part.first().unwrap();
                    let pb = part.last().unwrap();
                    (
                        Point2 { x: pa.x, z: pa.y },
                        Point2 { x: pb.x, z: pb.y },
                    )
                } else {
                    continue;
                }
            }
            _ => continue,
        };
        grouped
            .entry(normalize_track_key(&track_name))
            .or_default()
            .push(SectorBoundary { sector_id, a, b });
    }

    let mut out = HashMap::new();
    for r in refs {
        let k = normalize_track_key(&r.name);
        if let Some(bounds) = grouped.get(&k) {
            let mut ids: Vec<i32> = bounds.iter().map(|b| b.sector_id).collect();
            ids.sort();
            ids.dedup();
            let id_to_index = ids
                .iter()
                .enumerate()
                .map(|(i, v)| (*v, i))
                .collect::<HashMap<_, _>>();
            let mut boundaries = Vec::new();
            for b in bounds {
                if let Some(idx) = id_to_index.get(&b.sector_id) {
                    boundaries.push(SectorBoundary {
                        sector_id: *idx as i32,
                        a: b.a,
                        b: b.b,
                    });
                }
            }
            out.insert(
                r.name.clone(),
                SectorSet {
                    boundaries,
                    ring_ids: ids,
                },
            );
        }
    }

    if out.is_empty() {
        eprintln!(
            "No matching sector boundaries loaded from {} (track field='{}').",
            shp_path.display(),
            track_field
        );
    } else {
        eprintln!(
            "Loaded sector boundaries for {} detected tracks from {}",
            out.len(),
            shp_path.display()
        );
    }
    Ok(out)
}

fn first_crossed_sector(
    p0: Point2,
    p1: Point2,
    boundaries: &[SectorBoundary],
    search_radius_m: f64,
) -> Option<(usize, f64)> {
    let mut best: Option<(usize, f64)> = None;
    for b in boundaries {
        let center = Point2 {
            x: (b.a.x + b.b.x) * 0.5,
            z: (b.a.z + b.b.z) * 0.5,
        };
        let d0 = dist(p0, center);
        let d1 = dist(p1, center);
        if d0 > search_radius_m && d1 > search_radius_m {
            continue;
        }
        if let Some(t) = segment_intersection_t(p0, p1, b.a, b.b) {
            let idx = b.sector_id as usize;
            if best.map_or(true, |(_, bt)| t < bt) {
                best = Some((idx, t));
            }
        }
    }
    best
}

fn segment_intersection_t(p0: Point2, p1: Point2, q0: Point2, q1: Point2) -> Option<f64> {
    let r = Point2 {
        x: p1.x - p0.x,
        z: p1.z - p0.z,
    };
    let s = Point2 {
        x: q1.x - q0.x,
        z: q1.z - q0.z,
    };
    let rxs = r.x * s.z - r.z * s.x;
    if rxs.abs() < 1e-9 {
        return None;
    }
    let qp = Point2 {
        x: q0.x - p0.x,
        z: q0.z - p0.z,
    };
    let t = (qp.x * s.z - qp.z * s.x) / rxs;
    let u = (qp.x * r.z - qp.z * r.x) / rxs;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some(t)
    } else {
        None
    }
}

fn match_tracks(query: &[Point2], refs: &[ReferenceTrack], cfg: &CliConfig) -> Vec<MatchScore> {
    let query_headings = compute_headings(query);
    let mut out = Vec::with_capacity(refs.len());
    for r in refs {
        let mut inliers = 0usize;
        let mut d_sum = 0.0f64;
        let mut h_sum = 0.0f64;
        let mut n = 0usize;
        for i in (0..query.len()).step_by(2) {
            let (nearest_idx, d) = nearest_point_idx(query[i], &r.points);
            if d <= cfg.coarse_buffer_m {
                inliers += 1;
            }
            d_sum += d;
            let hd = angle_diff(query_headings[i], r.headings[nearest_idx]).abs();
            h_sum += hd;
            n += 1;
        }
        let coarse_ratio = if n == 0 { 0.0 } else { inliers as f64 / n as f64 };
        let coarse_pass = coarse_ratio >= cfg.coarse_required_ratio;
        let mean_dist = if n == 0 { f64::INFINITY } else { d_sum / n as f64 };
        let mean_heading = if n == 0 { f64::INFINITY } else { h_sum / n as f64 };
        let coarse_penalty = if coarse_pass { 0.0 } else { 10_000.0 };
        let final_score = mean_dist + (mean_heading * 25.0) + coarse_penalty;
        out.push(MatchScore {
            name: r.name.clone(),
            coarse_pass,
            coarse_inlier_ratio: coarse_ratio,
            mean_dist_m: mean_dist,
            mean_heading_diff_rad: mean_heading,
            final_score,
        });
    }
    out.sort_by(|a, b| a.final_score.partial_cmp(&b.final_score).unwrap_or(std::cmp::Ordering::Equal));
    out
}

fn print_scores(scores: &[MatchScore]) {
    eprintln!("Track matching results (best first):");
    for s in scores {
        eprintln!(
            "  {:<24} score={:8.3} dist={:6.2}m heading={:5.3}rad coarse={:.0}% {}",
            s.name,
            s.final_score,
            s.mean_dist_m,
            s.mean_heading_diff_rad,
            s.coarse_inlier_ratio * 100.0,
            if s.coarse_pass { "PASS" } else { "FAIL" }
        );
    }
}

fn nearest_point_idx(p: Point2, pts: &[Point2]) -> (usize, f64) {
    let mut best_i = 0usize;
    let mut best_d = f64::INFINITY;
    for (i, rp) in pts.iter().enumerate() {
        let d = dist(p, *rp);
        if d < best_d {
            best_d = d;
            best_i = i;
        }
    }
    (best_i, best_d)
}

fn dist(a: Point2, b: Point2) -> f64 {
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    (dx * dx + dz * dz).sqrt()
}

fn angle_diff(a: f64, b: f64) -> f64 {
    let mut d = a - b;
    while d > std::f64::consts::PI {
        d -= 2.0 * std::f64::consts::PI;
    }
    while d < -std::f64::consts::PI {
        d += 2.0 * std::f64::consts::PI;
    }
    d
}

fn ctrlc_handler() {
    ctrlc::set_handler(|| {
        RUNNING.store(false, Ordering::Relaxed);
    })
    .expect("could not set Ctrl+C handler");
}

fn load_labels(cfg: &CliConfig) -> Result<std::collections::HashMap<String, String>, Box<dyn std::error::Error>> {
    if let Some(path) = &cfg.labels_path {
        return parse_labels_file(path);
    }
    // Auto-detect labels file in any reference directory.
    for r in &cfg.refs {
        if r.is_dir() {
            let p = r.join("track_labels.toml");
            if p.exists() {
                return parse_labels_file(&p);
            }
        }
    }
    Ok(std::collections::HashMap::new())
}

fn parse_labels_file(path: &Path) -> Result<std::collections::HashMap<String, String>, Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(path)?;
    let parsed: TrackLabelsFile = toml::from_str(&raw)?;
    Ok(parsed.labels)
}

/// Write overlay text atomically: temp file in same directory, then replace target.
/// Avoids readers (e.g. RTSS) seeing a half-written file on Windows.
fn write_overlay_atomic(path: &Path, contents: &str) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("acr_detected_track.txt");
    let tmp = dir.join(format!("{}.tmp", name));
    std::fs::write(&tmp, contents)?;
    // On Windows, rename does not replace an existing destination.
    let _ = std::fs::remove_file(path);
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn push_live_overlay(cfg: &CliConfig, msg: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _ = write_overlay_atomic(&cfg.overlay_file, msg);
    #[cfg(windows)]
    {
        if cfg.rtss {
            let safe = sanitize_for_rtss(msg);
            if let Err(e) = acr_recorder::rtss_osd::update(&cfg.rtss_owner, &safe, cfg.rtss_slot) {
                eprintln!("RTSS update failed: {}", e);
            }
        }
    }
    Ok(())
}

fn sanitize_for_rtss(msg: &str) -> String {
    // Avoid characters RTSS may interpret as formatting/layout separators.
    let mut out = String::with_capacity(msg.len());
    for ch in msg.chars() {
        let mapped = match ch {
            '|' => ' ',
            '[' => '(',
            ']' => ')',
            '\r' | '\n' | '\t' => ' ',
            c if c.is_ascii() && !c.is_ascii_control() => c,
            _ => '?',
        };
        out.push(mapped);
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
