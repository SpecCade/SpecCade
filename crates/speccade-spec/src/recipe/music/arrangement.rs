//! Song arrangement and sequence types.

use serde::{Deserialize, Serialize};

/// Entry in the song arrangement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrangementEntry {
    /// Pattern name.
    pub pattern: String,
    /// Number of times to repeat (default: 1).
    #[serde(default = "default_repeat")]
    pub repeat: u16,
}

pub(crate) fn default_repeat() -> u16 {
    1
}
