//! Notes and annotations: acr_stop signals stop; acr_notes and acr_<field> are read on stop,
//! merged with parsed annotations (from #marker X# and [elapsed Ys] in acr_notes), and written
//! as a single <stem>.notes.json file. Grafana can use the annotations table for annotation layers.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Max size we read per file (avoid DoS from a buggy or malicious writer).
const NOTES_MAX_BYTES: usize = 64 * 1024;

/// Field names that have corresponding acr_<field> files (same as recording_notes columns except "notes").
pub const RECORDING_NOTES_FIELDS: &[&str] = &[
    "laptime",
    "result",
    "driver_impression",
    "tested_parameters",
    "conditions",
    "setup_notes",
    "session_goal",
    "incident",
];

const NOTES_FILENAME: &str = "acr_notes";
const ELAPSED_FILENAME: &str = "acr_elapsed_secs";

/// One annotation: point in time (time_offset_sec) or range (time_end_sec). Grafana uses time/timeEnd in ms.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Annotation {
    /// Seconds since recording start (used as 1000000000 + time_offset_sec for Grafana time axis).
    pub time_offset_sec: f64,
    /// Optional end of region (seconds); if null, annotation is a point.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_end_sec: Option<f64>,
    /// Display text.
    pub text: String,
    /// Tag for filtering (e.g. "good", "aborted").
    pub tag: String,
}

/// Root structure of <stem>.notes.json (written by recorder, read by acr_export).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordingNotesJson {
    pub recording_start_utc: String,
    pub recording_end_utc: String,
    /// Free-form notes text.
    pub notes: String,
    /// Field name -> content (from acr_<field> files).
    #[serde(default)]
    pub fields: HashMap<String, String>,
    /// Annotations for Grafana (point or range).
    #[serde(default)]
    pub annotations: Vec<Annotation>,
}

/// Call at recorder start: delete acr_notes, acr_elapsed_secs, and all acr_<field> in `dir`.
pub fn reset_notes_at_start(dir: &Path) -> std::io::Result<()> {
    let remove = |name: &str| {
        let p = dir.join(name);
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
    };
    remove(NOTES_FILENAME);
    remove(ELAPSED_FILENAME);
    for f in RECORDING_NOTES_FIELDS {
        remove(&format!("acr_{}", f));
    }
    Ok(())
}

/// Write current elapsed seconds to dir/acr_elapsed_secs for batch scripts.
pub fn write_elapsed_secs(dir: &Path, elapsed_secs: u64) -> std::io::Result<()> {
    let p = dir.join(ELAPSED_FILENAME);
    std::fs::write(p, elapsed_secs.to_string())
}

/// Read up to NOTES_MAX_BYTES from a path, return trimmed string.
fn read_file_trim(path: &Path) -> std::io::Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let mut f = std::fs::File::open(path)?;
    let mut raw = Vec::with_capacity(NOTES_MAX_BYTES.min(4096));
    let mut buf = [0u8; 4096];
    let mut total = 0usize;
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let take = (NOTES_MAX_BYTES - total).min(n);
        raw.extend_from_slice(&buf[..take]);
        total += take;
        if total >= NOTES_MAX_BYTES {
            break;
        }
    }
    let text = String::from_utf8_lossy(&raw).trim_end().to_string();
    Ok(if text.is_empty() { None } else { Some(text) })
}

/// Parse one line: extract [elapsed Ns] (-> time_offset_sec) and #marker TAG# (-> tag). Returns (time_offset_sec, tag, text) only when the line contains #marker (so pure notes lines are not turned into annotations).
fn parse_annotation_line(line: &str) -> Option<(f64, String, String)> {
    let line = line.trim();
    if line.is_empty() || !line.contains("#marker ") {
        return None;
    }
    let mut time_offset_sec = 0.0_f64;
    let mut tag = String::from("marker");

    // [elapsed Ns] or [elapsed N s]
    if let Some(start) = line.find("[elapsed ") {
        let rest = &line[start + 9..];
        let end = rest.find(']').unwrap_or(rest.len());
        let num_str = rest[..end].trim().trim_end_matches('s').trim();
        if let Ok(n) = num_str.parse::<f64>() {
            time_offset_sec = n;
        }
    }
    // #marker TAG#
    if let Some(start) = line.find("#marker ") {
        let rest = &line[start + 8..];
        let end = rest.find('#').unwrap_or(rest.len());
        tag = rest[..end].trim().to_string();
        if tag.is_empty() {
            tag = "marker".into();
        }
    }

    let text = tag.clone();
    Some((time_offset_sec, tag, text))
}

/// Called when recording stops: read acr_notes and acr_<field>, build RecordingNotesJson (with parsed annotations), write <stem>.notes.json, then delete all acr_* files.
pub fn save_notes_to_json(
    rkyv_path: &Path,
    notes_dir: &Path,
    recording_start_utc: &str,
    recording_end_utc: &str,
) -> std::io::Result<()> {
    let stem = rkyv_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("recording");
    let parent = rkyv_path.parent().unwrap_or(Path::new("."));

    let notes_path = notes_dir.join(NOTES_FILENAME);
    let notes_body = read_file_trim(&notes_path)?.unwrap_or_default();

    let mut annotations: Vec<Annotation> = Vec::new();
    for line in notes_body.lines() {
        if let Some((time_offset_sec, tag, text)) = parse_annotation_line(line) {
            annotations.push(Annotation {
                time_offset_sec,
                time_end_sec: None,
                text,
                tag,
            });
        }
    }

    let mut fields: HashMap<String, String> = HashMap::new();
    for field in RECORDING_NOTES_FIELDS {
        let src = notes_dir.join(format!("acr_{}", field));
        if let Ok(Some(text)) = read_file_trim(&src) {
            fields.insert((*field).to_string(), text);
        }
        let _ = std::fs::remove_file(&src);
    }

    let payload = RecordingNotesJson {
        recording_start_utc: recording_start_utc.to_string(),
        recording_end_utc: recording_end_utc.to_string(),
        notes: notes_body,
        fields,
        annotations,
    };

    let json_path = parent.join(format!("{}.notes.json", stem));
    let json_bytes = serde_json::to_string_pretty(&payload).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
    })?;
    std::fs::write(&json_path, json_bytes)?;

    let _ = std::fs::remove_file(notes_path);
    let _ = std::fs::remove_file(notes_dir.join(ELAPSED_FILENAME));
    Ok(())
}
