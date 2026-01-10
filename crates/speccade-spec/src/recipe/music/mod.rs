//! Music/tracker recipe types.

mod arrangement;
mod common;
mod effects;
mod instrument;
mod pattern;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_advanced;

pub use arrangement::*;
pub use common::*;
pub use effects::*;
pub use instrument::*;
pub use pattern::*;

use serde::{Deserialize, Serialize};

/// Parameters for the `music.tracker_song_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicTrackerSongV1Params {
    /// Tracker format: "xm" or "it".
    pub format: TrackerFormat,
    /// Beats per minute (30-300).
    pub bpm: u16,
    /// Tracker speed (ticks per row, 1-31).
    pub speed: u8,
    /// Number of channels (1-32).
    pub channels: u8,
    /// Whether the song should loop.
    #[serde(default)]
    pub r#loop: bool,
    /// Instrument definitions.
    pub instruments: Vec<TrackerInstrument>,
    /// Pattern definitions.
    pub patterns: std::collections::HashMap<String, TrackerPattern>,
    /// Song arrangement (order of patterns).
    pub arrangement: Vec<ArrangementEntry>,
    /// Automation definitions (volume fades, tempo changes).
    #[serde(default)]
    pub automation: Vec<AutomationEntry>,
    /// IT-specific options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub it_options: Option<ItOptions>,
}
