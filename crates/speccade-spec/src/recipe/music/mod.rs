//! Music/tracker recipe types.

mod arrangement;
mod common;
mod compose;
mod effects;
mod instrument;
mod pattern;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_advanced;

pub use arrangement::*;
pub use common::*;
pub use compose::*;
pub use effects::*;
pub use instrument::*;
pub use pattern::*;

use serde::{Deserialize, Serialize};

/// Parameters for the `music.tracker_song_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MusicTrackerSongV1Params {
    /// Song internal name (used in IT/XM module).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Song display title (for metadata).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Tracker format: "xm" or "it".
    pub format: TrackerFormat,
    /// Beats per minute (32-255).
    pub bpm: u16,
    /// Tracker speed (ticks per row, 1-31).
    pub speed: u8,
    /// Number of channels.
    ///
    /// - XM: 1-32
    /// - IT: 1-64
    pub channels: u8,
    /// Whether the song should loop.
    ///
    /// XM uses `restart_position`; IT encodes a terminal position jump effect.
    #[serde(default)]
    pub r#loop: bool,
    /// Restart position (order-table index) to jump to when looping.
    ///
    /// Used by XM directly and by IT loop jump insertion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restart_position: Option<u16>,
    /// Instrument definitions.
    #[serde(default)]
    pub instruments: Vec<TrackerInstrument>,
    /// Pattern definitions.
    #[serde(default)]
    pub patterns: std::collections::HashMap<String, TrackerPattern>,
    /// Song arrangement (order of patterns).
    #[serde(default)]
    pub arrangement: Vec<ArrangementEntry>,
    /// Automation definitions (volume fades, tempo changes).
    #[serde(default)]
    pub automation: Vec<AutomationEntry>,
    /// IT-specific options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub it_options: Option<ItOptions>,
}
