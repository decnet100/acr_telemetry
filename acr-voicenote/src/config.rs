//! Configuration from config.toml

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub output: OutputConfig,
    pub speech: SpeechConfig,
    pub whisper: WhisperConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OutputConfig {
    /// Output file path (fallback when notes_dir not set; with default notes_dir this is unused).
    #[allow(dead_code)]
    pub file_path: PathBuf,
    /// For ACR integration: write to notes_dir/acr_notes (same path as acr_recorder notes_dir).
    /// When set, overrides file_path for the main output.
    #[serde(default)]
    pub notes_dir: Option<PathBuf>,
    /// Optional UDP target
    #[serde(default)]
    pub udp: Option<UdpConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UdpConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SpeechConfig {
    /// Language: "de", "en", "auto"
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "auto".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct WhisperConfig {
    /// Model: tiny, base, small, tiny.en, base.en, ...
    #[serde(default = "default_model")]
    pub model: String,
    /// no_speech probability threshold (default 0.4). Lower = more aggressive silence filtering.
    #[serde(default = "default_no_speech_threshold")]
    pub no_speech_threshold: f64,
    /// avg_logprob threshold (default -0.5). Higher (less negative) = reject low-confidence transcriptions more.
    #[serde(default = "default_logprob_threshold")]
    pub logprob_threshold: f64,
}

fn default_model() -> String {
    "tiny".to_string()
}
fn default_no_speech_threshold() -> f64 {
    0.4
}
fn default_logprob_threshold() -> f64 {
    -0.5
}

/// Default notes directory (same as acr_recorder): %APPDATA%/acr_telemetry (Windows) or ~/.config/acr_telemetry (Linux).
pub fn default_notes_dir() -> PathBuf {
    dirs::config_dir()
        .map(|d| d.join("acr_telemetry"))
        .unwrap_or_else(|| PathBuf::from("."))
}

impl Config {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let cfg: Config = toml::from_str(&s)?;
        Ok(cfg)
    }

    /// Search for config.toml in CWD, ~/.config/acr_voicenote/
    pub fn discover() -> Option<PathBuf> {
        let cwd = std::path::Path::new("config.toml");
        if cwd.exists() {
            return Some(cwd.to_path_buf());
        }
        if let Some(config_dir) = dirs::config_dir() {
            let p = config_dir.join("acr_voicenote/config.toml");
            if p.exists() {
                return Some(p);
            }
        }
        None
    }
}
