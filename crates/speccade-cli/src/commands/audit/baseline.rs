//! Baseline file handling for audio audit.
//!
//! Manages `.audit.json` baseline files that store expected audio metrics.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::analysis::audio;

fn default_schema_version() -> u32 {
    1
}

/// Baseline metrics stored in .audit.json files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioBaseline {
    /// Schema version for future compatibility
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// Expected peak_db value
    pub peak_db: f64,
    /// Expected rms_db value
    pub rms_db: f64,
    /// Expected dc_offset value
    pub dc_offset: f64,
    /// Expected clipping status
    pub clipping_detected: bool,
    /// BLAKE3 hash of the audio file when baseline was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_hash: Option<String>,
}

impl AudioBaseline {
    /// Create a baseline from audio metrics.
    pub fn from_metrics(metrics: &audio::AudioMetrics, file_hash: Option<String>) -> Self {
        Self {
            schema_version: 1,
            peak_db: metrics.quality.peak_db,
            rms_db: metrics.quality.rms_db,
            dc_offset: metrics.quality.dc_offset,
            clipping_detected: metrics.quality.clipping_detected,
            file_hash,
        }
    }

    /// Load baseline from a file.
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read baseline file: {}", path.display()))?;
        let baseline: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse baseline file: {}", path.display()))?;
        Ok(baseline)
    }

    /// Save baseline to a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self).context("Failed to serialize baseline")?;
        fs::write(path, json)
            .with_context(|| format!("Failed to write baseline file: {}", path.display()))?;
        Ok(())
    }
}

/// Get the baseline path for a WAV file.
///
/// For a file `audio.wav`, returns `audio.audit.json`.
pub fn baseline_path(wav_path: &Path) -> PathBuf {
    let mut baseline = wav_path.to_path_buf();
    baseline.set_extension("audit.json");
    baseline
}
