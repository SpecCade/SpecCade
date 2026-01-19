//! Output record types for analysis, inspection, and comparison commands.

use super::{JsonError, JsonWarning};
use serde::{Deserialize, Serialize};

/// JSON output for the `analyze` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeOutput {
    /// Whether analysis succeeded
    pub success: bool,
    /// Errors encountered during analysis
    pub errors: Vec<JsonError>,
    /// Analysis result (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<AnalyzeResult>,
}

/// Analysis result details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeResult {
    /// Input file path
    pub input: String,
    /// Asset type analyzed (audio/texture)
    pub asset_type: String,
    /// BLAKE3 hash of the input file
    pub input_hash: String,
    /// Extracted metrics (structure depends on asset type)
    pub metrics: std::collections::BTreeMap<String, serde_json::Value>,
    /// Fixed-dimension feature embedding for similarity search (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f64>>,
}

impl AnalyzeOutput {
    /// Creates a successful analyze output.
    pub fn success(result: AnalyzeResult) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            result: Some(result),
        }
    }

    /// Creates a failed analyze output.
    pub fn failure(errors: Vec<JsonError>) -> Self {
        Self {
            success: false,
            errors,
            result: None,
        }
    }
}

/// Result for a single file in batch mode (either success or error).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalyzeItem {
    /// Input file path
    pub input: String,
    /// Whether analysis succeeded
    pub success: bool,
    /// Asset type analyzed (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_type: Option<String>,
    /// BLAKE3 hash of the input file (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_hash: Option<String>,
    /// Extracted metrics (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<std::collections::BTreeMap<String, serde_json::Value>>,
    /// Fixed-dimension feature embedding (if requested and successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f64>>,
    /// Error information (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonError>,
}

impl BatchAnalyzeItem {
    /// Creates a successful batch item from an AnalyzeResult.
    pub fn success(result: AnalyzeResult) -> Self {
        Self {
            input: result.input,
            success: true,
            asset_type: Some(result.asset_type),
            input_hash: Some(result.input_hash),
            metrics: Some(result.metrics),
            embedding: result.embedding,
            error: None,
        }
    }

    /// Creates a failed batch item.
    pub fn failure(input: String, error: JsonError) -> Self {
        Self {
            input,
            success: false,
            asset_type: None,
            input_hash: None,
            metrics: None,
            embedding: None,
            error: Some(error),
        }
    }
}

/// Summary statistics for batch analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalyzeSummary {
    /// Total number of files processed
    pub total: usize,
    /// Number of successfully analyzed files
    pub succeeded: usize,
    /// Number of failed files
    pub failed: usize,
}

/// JSON output for batch analyze command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalyzeOutput {
    /// Whether the overall batch operation completed (always true unless catastrophic failure)
    pub success: bool,
    /// Individual file results
    pub results: Vec<BatchAnalyzeItem>,
    /// Summary statistics
    pub summary: BatchAnalyzeSummary,
}

impl BatchAnalyzeOutput {
    /// Creates a new batch output from results.
    pub fn new(results: Vec<BatchAnalyzeItem>) -> Self {
        let total = results.len();
        let succeeded = results.iter().filter(|r| r.success).count();
        let failed = total - succeeded;

        Self {
            success: true,
            results,
            summary: BatchAnalyzeSummary {
                total,
                succeeded,
                failed,
            },
        }
    }
}

/// JSON output for the `inspect` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectOutput {
    /// Whether inspection succeeded
    pub success: bool,
    /// Errors encountered during inspection
    pub errors: Vec<JsonError>,
    /// Warnings from validation
    pub warnings: Vec<JsonWarning>,
    /// Inspection result (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<InspectResult>,
    /// Canonical spec hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_hash: Option<String>,
    /// BLAKE3 hash of the source file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
}

/// Inspection result details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectResult {
    /// Asset ID from the spec
    pub asset_id: String,
    /// Asset type
    pub asset_type: String,
    /// Source format (json/starlark)
    pub source_kind: String,
    /// Recipe kind
    pub recipe_kind: String,
    /// Output directory for intermediates
    pub out_dir: String,
    /// Intermediate artifact paths
    pub intermediates: Vec<IntermediateFile>,
    /// Final output paths (from spec outputs)
    pub final_outputs: Vec<IntermediateFile>,
    /// Expanded params JSON path (for compose specs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expanded_params_path: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// An intermediate file produced during inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediateFile {
    /// Identifier for this intermediate (node id, layer name, etc.)
    pub id: String,
    /// File format (png, json, etc.)
    pub format: String,
    /// Path to the file (relative to out_dir)
    pub path: String,
    /// BLAKE3 hash of the file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl InspectOutput {
    /// Creates a successful inspect output.
    pub fn success(
        result: InspectResult,
        spec_hash: String,
        source_hash: String,
        warnings: Vec<JsonWarning>,
    ) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            warnings,
            result: Some(result),
            spec_hash: Some(spec_hash),
            source_hash: Some(source_hash),
        }
    }

    /// Creates a failed inspect output.
    pub fn failure(
        errors: Vec<JsonError>,
        warnings: Vec<JsonWarning>,
        spec_hash: Option<String>,
        source_hash: Option<String>,
    ) -> Self {
        Self {
            success: false,
            errors,
            warnings,
            result: None,
            spec_hash,
            source_hash,
        }
    }
}

/// JSON output for the `compare` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareOutput {
    /// Whether the comparison succeeded
    pub success: bool,
    /// Errors encountered during comparison
    pub errors: Vec<JsonError>,
    /// Comparison result (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<CompareResult>,
}

/// Comparison result details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResult {
    /// Path to file A
    pub path_a: String,
    /// Path to file B
    pub path_b: String,
    /// Asset type (audio/texture)
    pub asset_type: String,
    /// BLAKE3 hash of file A
    pub hash_a: String,
    /// BLAKE3 hash of file B
    pub hash_b: String,
    /// Whether files are byte-identical
    pub identical: bool,
    /// Comparison metrics (type-specific)
    pub metrics: CompareMetrics,
}

/// Type-specific comparison metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CompareMetrics {
    /// Texture comparison metrics
    #[serde(rename = "texture")]
    Texture(TextureCompareMetrics),
    /// Audio comparison metrics
    #[serde(rename = "audio")]
    Audio(AudioCompareMetrics),
    /// Mesh comparison metrics (byte-identical only for now)
    #[serde(rename = "mesh")]
    Mesh(MeshCompareMetrics),
}

/// Mesh comparison metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshCompareMetrics {
    /// Placeholder - mesh comparison not yet implemented beyond byte-identical check
    pub byte_identical_only: bool,
}

/// Texture comparison metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureCompareMetrics {
    /// SSIM (Structural Similarity Index), range [0, 1] where 1 = identical
    pub ssim: f64,
    /// Mean DeltaE (CIE76) color difference
    pub delta_e_mean: f64,
    /// Maximum DeltaE color difference
    pub delta_e_max: f64,
    /// Histogram difference metrics
    pub histogram_diff: HistogramDiffMetrics,
}

/// Histogram difference for compare output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramDiffMetrics {
    /// Red/grayscale channel mean difference
    pub red: f64,
    /// Green channel mean difference (None for grayscale)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub green: Option<f64>,
    /// Blue channel mean difference (None for grayscale)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blue: Option<f64>,
    /// Alpha channel mean difference (None if no alpha)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha: Option<f64>,
}

/// Audio comparison metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCompareMetrics {
    /// Spectral centroid correlation coefficient [-1, 1]
    pub spectral_correlation: f64,
    /// RMS level difference in dB (A - B)
    pub rms_delta_db: f64,
    /// Peak level difference in dB (A - B)
    pub peak_delta_db: f64,
    /// Loudness difference as percentage ((A - B) / B * 100)
    pub loudness_delta_percent: f64,
}

impl CompareOutput {
    /// Creates a successful compare output.
    pub fn success(result: CompareResult) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            result: Some(result),
        }
    }

    /// Creates a failed compare output.
    pub fn failure(errors: Vec<JsonError>) -> Self {
        Self {
            success: false,
            errors,
            result: None,
        }
    }
}
