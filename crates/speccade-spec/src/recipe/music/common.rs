//! Common music types: tracker format.

use serde::{Deserialize, Serialize};

/// Tracker module format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum TrackerFormat {
    /// FastTracker II Extended Module format.
    #[default]
    Xm,
    /// Impulse Tracker format.
    It,
}

impl TrackerFormat {
    /// Returns the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            TrackerFormat::Xm => "xm",
            TrackerFormat::It => "it",
        }
    }
}
