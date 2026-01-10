//! Effect types for tracker modules.

use serde::{Deserialize, Serialize};

/// Pattern effect command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternEffect {
    /// Effect type (e.g., "vibrato", "volume_slide").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Effect parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<u8>,
    /// Effect X/Y nibbles (alternative to param).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_xy: Option<(u8, u8)>,
}

/// IT-specific module options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItOptions {
    /// Stereo output flag.
    #[serde(default = "default_stereo")]
    pub stereo: bool,
    /// Global volume (0-128).
    #[serde(default = "default_global_volume")]
    pub global_volume: u8,
    /// Mix volume (0-128).
    #[serde(default = "default_mix_volume")]
    pub mix_volume: u8,
}

fn default_stereo() -> bool {
    true
}

fn default_global_volume() -> u8 {
    128
}

fn default_mix_volume() -> u8 {
    48
}

impl Default for ItOptions {
    fn default() -> Self {
        Self {
            stereo: true,
            global_volume: 128,
            mix_volume: 48,
        }
    }
}

/// Automation entry for volume fades and tempo changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutomationEntry {
    /// Volume fade automation.
    VolumeFade {
        /// Target pattern name.
        pattern: String,
        /// Target channel (0-indexed).
        #[serde(default)]
        channel: u8,
        /// Start row.
        #[serde(default)]
        start_row: u16,
        /// End row.
        end_row: u16,
        /// Start volume (0-64).
        #[serde(default)]
        start_vol: u8,
        /// End volume (0-64).
        end_vol: u8,
    },
    /// Tempo change automation.
    TempoChange {
        /// Target pattern name.
        pattern: String,
        /// Row for tempo change.
        #[serde(default)]
        row: u16,
        /// New BPM (32-255).
        bpm: u8,
    },
}
