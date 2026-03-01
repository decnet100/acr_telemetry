//! Configuration loading for acr_recorder, acr_export, and acr_telemetry_bridge.

use std::path::{Path, PathBuf};

use crate::color_config::ColorConfig;

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub recorder: RecorderConfig,
    #[serde(default)]
    pub export: ExportConfig,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BridgeConfig {
    /// Update rate in Hz (1–60).
    #[serde(default = "default_bridge_rate")]
    pub rate_hz: u64,
    /// UDP target: "HOST:PORT" (e.g. "192.168.1.255:9000" for broadcast). Empty = disabled.
    #[serde(default)]
    pub udp_target: Option<String>,
    /// HTTP server address (e.g. "0.0.0.0:8080"). Empty string = disabled.
    #[serde(default = "default_http_addr")]
    pub http_addr: String,
    /// Temperature unit: "c" (Celsius), "f" (Fahrenheit), "k" (Kelvin). ACC uses Kelvin internally.
    #[serde(default = "default_temperature_unit")]
    pub temperature_unit: String,
    /// Dashboard slots: list of field IDs or { field, label? }. Order = display order.
    /// Fields: water_temp, road_temp, air_temp, fuel, tyre_fl, tyre_fr, tyre_rl, tyre_rr,
    /// brake_fl, brake_fr, brake_rl, brake_rr, speed_kmh, gear, rpm
    #[serde(default = "default_dashboard_slots")]
    pub dashboard_slots: Vec<DashboardSlot>,
    /// Dashboard coloring: palette and per-field thresholds. Omit to use defaults or load from telemetry_color.toml (fallback).
    #[serde(default)]
    pub telemetry_colors: Option<ColorConfig>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum DashboardSlot {
    Field(String),
    WithLabel { field: String, label: Option<String> },
}

impl DashboardSlot {
    pub fn field_id(&self) -> &str {
        match self {
            Self::Field(s) => s,
            Self::WithLabel { field, .. } => field,
        }
    }
    pub fn label(&self) -> Option<&str> {
        match self {
            Self::Field(_) => None,
            Self::WithLabel { label, .. } => label.as_deref(),
        }
    }
}

fn default_dashboard_slots() -> Vec<DashboardSlot> {
    vec![
        DashboardSlot::Field("water_temp".into()),
        DashboardSlot::Field("road_temp".into()),
        DashboardSlot::Field("air_temp".into()),
        DashboardSlot::Field("fuel".into()),
        DashboardSlot::Field("tyre_fl".into()),
        DashboardSlot::Field("tyre_fr".into()),
        DashboardSlot::Field("tyre_rl".into()),
        DashboardSlot::Field("tyre_rr".into()),
        DashboardSlot::Field("brake_fl".into()),
        DashboardSlot::Field("brake_fr".into()),
        DashboardSlot::Field("brake_rl".into()),
        DashboardSlot::Field("brake_rr".into()),
        DashboardSlot::Field("speed_kmh".into()),
        DashboardSlot::Field("gear".into()),
        DashboardSlot::Field("rpm".into()),
    ]
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            rate_hz: default_bridge_rate(),
            udp_target: None,
            http_addr: default_http_addr(),
            temperature_unit: default_temperature_unit(),
            dashboard_slots: default_dashboard_slots(),
            telemetry_colors: None,
        }
    }
}


fn default_bridge_rate() -> u64 {
    5
}

fn default_http_addr() -> String {
    "0.0.0.0:8080".into()
}

fn default_temperature_unit() -> String {
    "c".into()
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RecorderConfig {
    /// Directory for raw .rkyv recordings (relative to CWD or absolute).
    #[serde(default = "default_raw_output_dir")]
    pub raw_output_dir: String,
    /// Path to stop file. Creating this file signals acr_recorder to exit.
    /// Relative to CWD or absolute. Empty = %APPDATA%/acr_telemetry/acr_stop (Windows) or ~/.config/acr_telemetry/acr_stop (Linux).
    #[serde(default)]
    pub stop_file_path: Option<String>,
    /// Directory for acr_notes and acr_elapsed_secs (batch scripts). Empty = %APPDATA%/acr_telemetry.
    #[serde(default)]
    pub notes_dir: Option<String>,
    /// Record GraphicsMap data (~60 Hz) alongside physics. Used for Grafana (e.g. distance_traveled).
    #[serde(default = "default_record_graphics")]
    pub record_graphics: bool,
}

fn default_record_graphics() -> bool {
    true
}

impl Default for RecorderConfig {
    fn default() -> Self {
        Self {
            raw_output_dir: default_raw_output_dir(),
            stop_file_path: None,
            notes_dir: None,
            record_graphics: true,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExportConfig {
    /// Default export method: "csv" or "sqlite"
    #[serde(default = "default_export_method")]
    pub default_method: String,
    /// Default path for SQLite database (relative to CWD or absolute).
    #[serde(default = "default_sqlite_path")]
    pub sqlite_db_path: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            default_method: default_export_method(),
            sqlite_db_path: default_sqlite_path(),
        }
    }
}

fn default_raw_output_dir() -> String {
    "telemetry_raw".into()
}

fn default_export_method() -> String {
    "csv".into()
}

fn default_sqlite_path() -> String {
    "telemetry.db".into()
}

/// Load config from standard locations:
/// 1. ./acr_recorder.toml (current working directory)
/// 2. ~/.config/acr_recorder/config.toml
pub fn load_config() -> Config {
    let paths = config_paths();
    for path in paths {
        if path.exists() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                match toml::from_str(&s) {
                    Ok(cfg) => return cfg,
                    Err(e) => eprintln!("Config parse error in {}: {}", path.display(), e),
                }
            }
        }
    }
    Config::default()
}

fn config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("acr_recorder.toml"));
    }
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("acr_recorder").join("config.toml"));
    }
    paths
}

fn bridge_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("acr_telemetry_bridge.toml"));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("acr_telemetry_bridge.toml"));
    }
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("acr_recorder").join("acr_telemetry_bridge.toml"));
    }
    paths
}

/// Load bridge config from acr_telemetry_bridge.toml.
/// Paths: ./acr_telemetry_bridge.toml, ~/.config/acr_recorder/acr_telemetry_bridge.toml
pub fn load_bridge_config() -> BridgeConfig {
    for path in bridge_config_paths() {
        if path.exists() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                match toml::from_str(&s) {
                    Ok(cfg) => return cfg,
                    Err(e) => eprintln!("Config parse error in {}: {}", path.display(), e),
                }
            }
        }
    }
    BridgeConfig::default()
}

/// Path to the stop file. Creating this file signals acr_recorder to exit.
/// Uses config if set, else platform default (config_dir/acr_telemetry/acr_stop).
pub fn resolve_stop_file_path(cfg: &RecorderConfig) -> PathBuf {
    match &cfg.stop_file_path {
        Some(s) if !s.is_empty() => resolve_path(s),
        _ => dirs::config_dir()
            .map(|d| d.join("acr_telemetry").join("acr_stop"))
            .unwrap_or_else(|| PathBuf::from(".acr_stop")),
    }
}

/// Directory for acr_notes and acr_elapsed_secs (used by batch scripts). Uses config if set, else %APPDATA%/acr_telemetry.
pub fn resolve_notes_dir(cfg: &RecorderConfig) -> PathBuf {
    match &cfg.notes_dir {
        Some(s) if !s.is_empty() => resolve_path(s),
        _ => dirs::config_dir()
            .map(|d| d.join("acr_telemetry"))
            .unwrap_or_else(|| PathBuf::from(".")),
    }
}

/// Resolve a path (relative or absolute). Relative paths are resolved against CWD.
pub fn resolve_path(s: &str) -> PathBuf {
    let p = Path::new(s);
    if p.is_absolute() {
        p.to_path_buf()
    } else if let Ok(cwd) = std::env::current_dir() {
        cwd.join(p)
    } else {
        p.to_path_buf()
    }
}
