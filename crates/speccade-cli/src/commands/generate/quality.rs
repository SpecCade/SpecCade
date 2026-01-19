//! Quality constraints for audio variation generation.

use crate::analysis::audio::AudioMetrics;
use crate::commands::json_output::VariationConstraints;

/// Quality constraints for variation generation.
#[derive(Debug, Clone)]
pub struct QualityConstraints {
    /// Maximum peak level in dB (None = no constraint)
    pub max_peak_db: Option<f64>,
    /// Maximum DC offset (None = no constraint)
    pub max_dc_offset: Option<f64>,
}

impl QualityConstraints {
    /// Creates quality constraints from CLI options.
    pub fn from_options(max_peak_db: Option<f64>, max_dc_offset: Option<f64>) -> Option<Self> {
        if max_peak_db.is_some() || max_dc_offset.is_some() {
            Some(Self {
                max_peak_db,
                max_dc_offset,
            })
        } else {
            None
        }
    }

    /// Checks if audio metrics pass the quality constraints.
    /// Returns Ok(()) if passed, Err(reason) if failed.
    pub fn check(&self, metrics: &AudioMetrics) -> Result<(), String> {
        if let Some(max_peak) = self.max_peak_db {
            if metrics.quality.peak_db > max_peak {
                return Err(format!(
                    "peak exceeded: {:.2} dB > {:.2} dB",
                    metrics.quality.peak_db, max_peak
                ));
            }
        }

        if let Some(max_dc) = self.max_dc_offset {
            let dc_abs = metrics.quality.dc_offset.abs();
            if dc_abs > max_dc {
                return Err(format!("DC offset exceeded: {:.6} > {:.6}", dc_abs, max_dc));
            }
        }

        Ok(())
    }

    /// Converts to JSON manifest format.
    pub fn to_manifest_constraints(&self) -> VariationConstraints {
        VariationConstraints {
            max_peak_db: self.max_peak_db,
            max_dc_offset: self.max_dc_offset,
        }
    }
}
