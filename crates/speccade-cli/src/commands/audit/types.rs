//! Type definitions for the audio audit command.
//!
//! Contains all serializable types for audit results, violations, and configuration.

use serde::{Deserialize, Serialize};

use crate::analysis::audio;

use super::super::json_output::JsonError;

/// Default tolerance configuration for audio metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTolerances {
    /// Maximum allowed peak_db (default: 0.0 dB, no clipping)
    pub max_peak_db: f64,
    /// Maximum allowed absolute dc_offset (default: 0.05)
    pub max_dc_offset: f64,
    /// Whether clipping is allowed (default: false)
    pub allow_clipping: bool,
    /// Maximum allowed delta for peak_db when comparing to baseline
    pub peak_db_delta: f64,
    /// Maximum allowed delta for rms_db when comparing to baseline
    pub rms_db_delta: f64,
    /// Maximum allowed delta for dc_offset when comparing to baseline
    pub dc_offset_delta: f64,
}

impl Default for AuditTolerances {
    fn default() -> Self {
        Self {
            max_peak_db: 0.0,
            max_dc_offset: 0.05,
            allow_clipping: false,
            peak_db_delta: 0.5,
            rms_db_delta: 1.0,
            dc_offset_delta: 0.01,
        }
    }
}

impl AuditTolerances {
    /// Parse tolerances from a JSON file.
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        use anyhow::Context;
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read tolerances file: {}", path.display()))?;
        let tolerances: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse tolerances file: {}", path.display()))?;
        Ok(tolerances)
    }
}

/// Result of auditing a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFileResult {
    /// Path to the audio file
    pub path: String,
    /// Whether the audit passed
    pub passed: bool,
    /// List of violations found
    pub violations: Vec<AuditViolation>,
    /// Current metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<AuditMetrics>,
    /// Baseline metrics (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<AuditMetrics>,
    /// Error message (if analysis failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Simplified metrics for audit output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetrics {
    pub peak_db: f64,
    pub rms_db: f64,
    pub dc_offset: f64,
    pub clipping_detected: bool,
}

impl From<&audio::AudioMetrics> for AuditMetrics {
    fn from(m: &audio::AudioMetrics) -> Self {
        Self {
            peak_db: m.quality.peak_db,
            rms_db: m.quality.rms_db,
            dc_offset: m.quality.dc_offset,
            clipping_detected: m.quality.clipping_detected,
        }
    }
}

/// A single audit violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditViolation {
    /// Type of violation
    pub kind: ViolationKind,
    /// Description of the violation
    pub message: String,
    /// Current value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<f64>,
    /// Expected/threshold value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<f64>,
    /// Delta from baseline (for regression checks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<f64>,
}

/// Type of audit violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationKind {
    /// Peak level exceeds threshold
    PeakExceeded,
    /// DC offset exceeds threshold
    DcOffsetExceeded,
    /// Clipping detected when not allowed
    ClippingDetected,
    /// Peak level regression from baseline
    PeakRegression,
    /// RMS level regression from baseline
    RmsRegression,
    /// DC offset regression from baseline
    DcOffsetRegression,
    /// Clipping status changed from baseline
    ClippingRegression,
}

/// Summary of audit results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    /// Total files audited
    pub total: usize,
    /// Files that passed
    pub passed: usize,
    /// Files that failed
    pub failed: usize,
    /// Files that had errors during analysis
    pub errors: usize,
    /// Files with baselines
    pub with_baseline: usize,
    /// Files without baselines
    pub without_baseline: usize,
}

/// JSON output for the audit command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditOutput {
    /// Whether the overall audit passed (all files passed)
    pub success: bool,
    /// Individual file results
    pub results: Vec<AuditFileResult>,
    /// Summary statistics
    pub summary: AuditSummary,
    /// Tolerances used for this audit
    pub tolerances: AuditTolerances,
    /// Errors encountered
    pub errors: Vec<JsonError>,
}

impl AuditOutput {
    /// Create output from results.
    pub fn from_results(
        results: Vec<AuditFileResult>,
        tolerances: AuditTolerances,
        errors: Vec<JsonError>,
    ) -> Self {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results
            .iter()
            .filter(|r| !r.passed && r.error.is_none())
            .count();
        let error_count = results.iter().filter(|r| r.error.is_some()).count();
        let with_baseline = results.iter().filter(|r| r.baseline.is_some()).count();
        let without_baseline = total - with_baseline;

        let success = failed == 0 && error_count == 0 && errors.is_empty();

        Self {
            success,
            results,
            summary: AuditSummary {
                total,
                passed,
                failed,
                errors: error_count,
                with_baseline,
                without_baseline,
            },
            tolerances,
            errors,
        }
    }
}
