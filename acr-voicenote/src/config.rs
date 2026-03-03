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
    /// Output file path (lines: ISO timestamp + tab + transcript). Ignored when notes_dir is set.
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
}

fn default_model() -> String {
    "tiny".to_string()
}

impl Config {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let cfg: Config = toml::from_str(&s)?;
        Ok(cfg)
    }

    /// Search for config.toml in CWD, ~/.config/acr-voicenote/
    pub fn discover() -> Option<PathBuf> {
        let cwd = std::path::Path::new("config.toml");
        if cwd.exists() {
            return Some(cwd.to_path_buf());
        }
        if let Some(config_dir) = dirs::config_dir() {
            let p = config_dir.join("acr-voicenote/config.toml");
            if p.exists() {
                return Some(p);
            }
        }
        None
    }
}
