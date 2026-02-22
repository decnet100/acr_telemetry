//! Load telemetry_color.toml for dashboard threshold-based coloring.

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ColorConfig {
    #[serde(default = "default_colors")]
    pub colors: ColorPalette,
    #[serde(default)]
    pub fields: HashMap<String, FieldThresholds>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ColorPalette {
    #[serde(default = "default_very_low")]
    pub very_low: String,
    #[serde(default = "default_low")]
    pub low: String,
    #[serde(default = "default_normal")]
    pub normal: String,
    #[serde(default = "default_high")]
    pub high: String,
    #[serde(default = "default_very_high")]
    pub very_high: String,
    #[serde(default = "default_ignore")]
    pub ignore: String,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct FieldThresholds {
    pub very_low: Option<f64>,
    pub low: Option<f64>,
    pub normal: Option<f64>,
    pub high: Option<f64>,
    pub very_high: Option<f64>,
}

fn default_colors() -> ColorPalette {
    ColorPalette {
        very_low: default_very_low(),
        low: default_low(),
        normal: default_normal(),
        high: default_high(),
        very_high: default_very_high(),
        ignore: default_ignore(),
    }
}

fn default_very_low() -> String {
    "#1e3a5f".into()
}
fn default_low() -> String {
    "#7dd3fc".into()
}
fn default_normal() -> String {
    "#fde047".into()
}
fn default_high() -> String {
    "#fb923c".into()
}
fn default_very_high() -> String {
    "#ef4444".into()
}
fn default_ignore() -> String {
    "#ffffff".into()
}

fn color_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("telemetry_color.toml"));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("telemetry_color.toml"));
    }
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("acr_recorder").join("telemetry_color.toml"));
    }
    paths
}

/// Load telemetry_color.toml from CWD or ~/.config/acr_recorder/.
pub fn load_color_config() -> ColorConfig {
    for path in color_config_paths() {
        if path.exists() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                match toml::from_str(&s) {
                    Ok(cfg) => return cfg,
                    Err(e) => eprintln!("telemetry_color.toml parse error in {}: {}", path.display(), e),
                }
            }
        }
    }
    ColorConfig::default()
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            colors: default_colors(),
            fields: default_field_thresholds(),
        }
    }
}

fn default_field_thresholds() -> HashMap<String, FieldThresholds> {
    let mut m = HashMap::new();
    // Water/coolant: °C, optimal ~80–95
    m.insert("water_temp".into(), FieldThresholds {
        very_low: Some(0.0),
        low: Some(60.0),
        normal: Some(80.0),
        high: Some(95.0),
        very_high: Some(120.0),
    });
    m.insert("road_temp".into(), FieldThresholds {
        very_low: Some(0.0),
        low: Some(15.0),
        normal: Some(25.0),
        high: Some(40.0),
        very_high: Some(55.0),
    });
    m.insert("air_temp".into(), FieldThresholds {
        very_low: Some(-10.0),
        low: Some(10.0),
        normal: Some(20.0),
        high: Some(35.0),
        very_high: Some(45.0),
    });
    m.insert("fuel".into(), FieldThresholds {
        very_low: Some(0.0),
        low: Some(5.0),
        normal: Some(20.0),
        high: Some(50.0),
        very_high: Some(80.0),
    });
    for f in ["tyre_fl", "tyre_fr", "tyre_rl", "tyre_rr"] {
        m.insert(f.into(), FieldThresholds {
            very_low: Some(50.0),
            low: Some(70.0),
            normal: Some(85.0),
            high: Some(100.0),
            very_high: Some(120.0),
        });
    }
    for f in ["brake_fl", "brake_fr", "brake_rl", "brake_rr"] {
        m.insert(f.into(), FieldThresholds {
            very_low: None,
            low: Some(200.0),
            normal: Some(300.0),
            high: Some(500.0),
            very_high: Some(700.0),
        });
    }
    m.insert("speed_kmh".into(), FieldThresholds {
        very_low: Some(0.0),
        low: Some(50.0),
        normal: Some(100.0),
        high: Some(200.0),
        very_high: Some(300.0),
    });
    m.insert("gear".into(), FieldThresholds::default()); // no thresholds = ignore
    m.insert("rpm".into(), FieldThresholds {
        very_low: Some(0.0),
        low: Some(2000.0),
        normal: Some(4000.0),
        high: Some(6000.0),
        very_high: Some(8000.0),
    });
    m
}
