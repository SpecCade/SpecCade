//! Pattern definitions for tracker modules.

use serde::{Deserialize, Serialize};

use super::effects::PatternEffect;

/// Pattern definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackerPattern {
    /// Number of rows in the pattern.
    pub rows: u16,
    /// Note data.
    pub data: Vec<PatternNote>,
}

/// A single note event in a pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternNote {
    /// Row number (0-indexed).
    pub row: u16,
    /// Channel number (0-indexed).
    pub channel: u8,
    /// Note name (e.g., "C4", "---" for note off, "..." for no note).
    pub note: String,
    /// Instrument index.
    pub instrument: u8,
    /// Volume (0-64, optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u8>,
    /// Effect command (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<PatternEffect>,
}
