use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Bars,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    Solid,
    Gradient,
}

pub type RgbColor = (u8, u8, u8);

const DEFAULT_CONFIG: &str = 
r##"[general]
framerate = 60

[input]
# Number of frequency buckets read from cava (2–8 recommended)
driver_bars = 16

# Reverse frequency order — puts high frequencies at center in mirror mode
reverse_frequencies = false

[physics]
# Internal simulation resolution — more points = smoother wave but slower traversal
# (traversal time ≈ points / (60 * sqrt(wave_speed)) seconds)
points = 64

# How fast the wave travels across the bars (0.05–0.45, keep below 0.5)
wave_speed = 0.3

# Energy retained per frame — closer to 1.0 = longer decay (0.9–0.99)
damping = 0.97

# How much energy walls reflect back (0.0 = fully absorbing, 1.0 = fully reflecting)
wall_damping = 0.3

# Minimum rate-of-change to trigger an impulse (filters noise)
impulse_threshold = 0.05

# Force multiplier on detected transients
impulse_strength = 10

# Gaussian spread of each impulse in physics points (larger = wider bell curve)
impulse_spread = 2.0

# How gradually displaced volume equalises across the wave (0.0=never, 1.0=instant)
# Low values like 0.02 give a slow elastic feel over ~50 frames
compressibility = 0.02

# Minimum RMS level before any impulses fire (silence gate)
min_rms = 0.05

[output]
# Display mode: bars (more modes coming)
mode = "bars"

# Mirror the wave symmetrically (true/false)
mirror = true

# Visual amplification — 1.0 = natural loudness, >1 amplifies, <1 quiets
visual_gain = 1

# Color mode: gradient or solid
color_mode = "gradient"

# color1 = resting color (solid mode), or gradient start (gradient mode)
# color2 = peak displacement color (gradient mode only)
# Format: hex RGB e.g. #cdd6f4
color1 = "#cdd6f4"
color2 = "#bac2de"
"##;

fn parse_color(s: &str) -> Option<RgbColor> {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub input: InputConfig,
    pub physics: PhysicsConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Number of frames per second for the simulation and UI
    pub framerate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Number of frequency buckets read from cava (2–8 recommended)
    pub driver_bars: usize,
    /// Reverse frequency order — puts high frequencies at center in mirror mode
    pub reverse_frequencies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Internal simulation resolution — more points = smoother wave but slower traversal
    pub points: usize,
    /// How fast the wave travels across the bars (0.05–0.45, keep below 0.5)
    pub wave_speed: f32,
    /// Energy retained per frame — closer to 1.0 = longer decay (0.9–0.99)
    pub damping: f32,
    /// How much energy walls reflect back (0.0 = fully absorbing, 1.0 = fully reflecting)
    pub wall_damping: f32,
    /// Minimum rate-of-change to trigger an impulse (filters noise)
    pub impulse_threshold: f32,
    /// Force multiplier on detected transients
    pub impulse_strength: f32,
    /// Gaussian spread of each impulse in physics points (larger = wider bell curve)
    pub impulse_spread: f32,
    /// How gradually displaced volume equalises across the wave (0.0=never, 1.0=instant)
    pub compressibility: f32,
    /// Minimum RMS level before any impulses fire (silence gate)
    pub min_rms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Display mode: bars (more modes coming)
    pub mode: Mode,
    /// Mirror the wave symmetrically (true/false)
    pub mirror: bool,
    /// Visual amplification — 1.0 = natural loudness, >1 amplifies, <1 quiets
    pub visual_gain: f32,
    /// Color mode: gradient or solid
    pub color_mode: ColorMode,
    /// Resting color (solid mode), or gradient start (gradient mode)
    pub color1: String,
    /// Peak displacement color (gradient mode only)
    pub color2: String,
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).expect("invalid default config template")
    }
}

impl Config {
    pub fn load(custom_path: Option<PathBuf>) -> Result<Self> {
        if let Some(path) = custom_path {
            // Manual load from custom path
            if !path.exists() {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, DEFAULT_CONFIG)?;
            }
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            // Standard confy load
            let path = confy::get_configuration_file_path("cawave", Some("config"))?;
            if !path.exists() {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, DEFAULT_CONFIG)?;
            }
            confy::load("cawave", Some("config")).map_err(Into::into)
        }
    }

    pub fn color1_rgb(&self) -> RgbColor {
        parse_color(&self.output.color1).unwrap_or((205, 214, 244))
    }

    pub fn color2_rgb(&self) -> RgbColor {
        parse_color(&self.output.color2).unwrap_or((186, 194, 222))
    }
}
