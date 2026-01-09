//! Report types and writer for SpecCade generation and validation results.
//!
//! This module provides types for creating standardized reports as defined in
//! RFC-0001 Section 4. Reports document the results of `speccade generate` or
//! `speccade validate` operations, including errors, warnings, and output metadata.

use crate::error::{ValidationError, ValidationWarning};
use crate::output::{OutputFormat, OutputKind};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Report schema version (always 1 for this RFC).
pub const REPORT_VERSION: u32 = 1;

/// A complete report for a generation or validation operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    /// Report schema version (always 1).
    pub report_version: u32,
    /// Hex-encoded BLAKE3 hash of the canonicalized spec.
    pub spec_hash: String,
    /// Whether the operation succeeded without errors.
    pub ok: bool,
    /// List of errors that occurred.
    pub errors: Vec<ReportError>,
    /// List of warnings that were generated.
    pub warnings: Vec<ReportWarning>,
    /// List of output artifacts produced.
    pub outputs: Vec<OutputResult>,
    /// Total execution time in milliseconds.
    pub duration_ms: u64,
    /// Backend identifier and version (e.g., "speccade-backend-audio v0.1.0").
    pub backend_version: String,
    /// Rust target triple (e.g., "x86_64-pc-windows-msvc").
    pub target_triple: String,
}

impl Report {
    /// Creates a new report builder.
    pub fn builder(spec_hash: String, backend_version: String) -> ReportBuilder {
        ReportBuilder::new(spec_hash, backend_version)
    }

    /// Serializes the report to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serializes the report to pretty-printed JSON.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Generates the standard report filename for a given asset ID.
    ///
    /// # Example
    ///
    /// ```
    /// use speccade_spec::report::Report;
    ///
    /// let filename = Report::filename("laser-blast-01");
    /// assert_eq!(filename, "laser-blast-01.report.json");
    /// ```
    pub fn filename(asset_id: &str) -> String {
        format!("{}.report.json", asset_id)
    }

    /// Parses a report from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Error entry in a report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportError {
    /// Error code (e.g., "E001").
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// JSON path to the problematic field (e.g., "outputs[0].path").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ReportError {
    /// Creates a new report error.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: None,
        }
    }

    /// Creates a new report error with a JSON path.
    pub fn with_path(
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: Some(path.into()),
        }
    }

    /// Converts a ValidationError to a ReportError.
    pub fn from_validation_error(err: &ValidationError) -> Self {
        Self {
            code: err.code.code().to_string(),
            message: err.message.clone(),
            path: err.path.clone(),
        }
    }
}

/// Warning entry in a report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportWarning {
    /// Warning code (e.g., "W001").
    pub code: String,
    /// Human-readable warning message.
    pub message: String,
    /// JSON path to the problematic field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ReportWarning {
    /// Creates a new report warning.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: None,
        }
    }

    /// Creates a new report warning with a JSON path.
    pub fn with_path(
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: Some(path.into()),
        }
    }

    /// Converts a ValidationWarning to a ReportWarning.
    pub fn from_validation_warning(warn: &ValidationWarning) -> Self {
        Self {
            code: warn.code.code().to_string(),
            message: warn.message.clone(),
            path: warn.path.clone(),
        }
    }
}

/// Result entry for a single output artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputResult {
    /// The kind of output (primary, metadata, preview).
    pub kind: OutputKind,
    /// The file format.
    pub format: OutputFormat,
    /// Relative path where the artifact was written.
    pub path: PathBuf,
    /// Hex-encoded BLAKE3 hash (for Tier 1 outputs only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Validation metrics (for Tier 2 outputs only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<OutputMetrics>,
}

impl OutputResult {
    /// Creates a new Tier 1 output result with a hash.
    pub fn tier1(
        kind: OutputKind,
        format: OutputFormat,
        path: PathBuf,
        hash: String,
    ) -> Self {
        Self {
            kind,
            format,
            path,
            hash: Some(hash),
            metrics: None,
        }
    }

    /// Creates a new Tier 2 output result with metrics.
    pub fn tier2(
        kind: OutputKind,
        format: OutputFormat,
        path: PathBuf,
        metrics: OutputMetrics,
    ) -> Self {
        Self {
            kind,
            format,
            path,
            hash: None,
            metrics: Some(metrics),
        }
    }
}

/// Validation metrics for Tier 2 outputs (GLB meshes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputMetrics {
    /// Number of triangles in the mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triangle_count: Option<u32>,
    /// Bounding box of the mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    /// Number of UV islands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_island_count: Option<u32>,
    /// Number of bones in the skeleton.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bone_count: Option<u32>,
    /// Number of material slots.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_slot_count: Option<u32>,
    /// Maximum number of bone influences per vertex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bone_influences: Option<u32>,
    /// Number of animation frames.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation_frame_count: Option<u32>,
    /// Duration of animation in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation_duration_seconds: Option<f32>,
}

impl OutputMetrics {
    /// Creates a new empty metrics object.
    pub fn new() -> Self {
        Self {
            triangle_count: None,
            bounding_box: None,
            uv_island_count: None,
            bone_count: None,
            material_slot_count: None,
            max_bone_influences: None,
            animation_frame_count: None,
            animation_duration_seconds: None,
        }
    }

    /// Sets the triangle count.
    pub fn with_triangle_count(mut self, count: u32) -> Self {
        self.triangle_count = Some(count);
        self
    }

    /// Sets the bounding box.
    pub fn with_bounding_box(mut self, bbox: BoundingBox) -> Self {
        self.bounding_box = Some(bbox);
        self
    }

    /// Sets the UV island count.
    pub fn with_uv_island_count(mut self, count: u32) -> Self {
        self.uv_island_count = Some(count);
        self
    }

    /// Sets the bone count.
    pub fn with_bone_count(mut self, count: u32) -> Self {
        self.bone_count = Some(count);
        self
    }

    /// Sets the material slot count.
    pub fn with_material_slot_count(mut self, count: u32) -> Self {
        self.material_slot_count = Some(count);
        self
    }

    /// Sets the maximum bone influences.
    pub fn with_max_bone_influences(mut self, max: u32) -> Self {
        self.max_bone_influences = Some(max);
        self
    }

    /// Sets the animation frame count.
    pub fn with_animation_frame_count(mut self, count: u32) -> Self {
        self.animation_frame_count = Some(count);
        self
    }

    /// Sets the animation duration.
    pub fn with_animation_duration_seconds(mut self, duration: f32) -> Self {
        self.animation_duration_seconds = Some(duration);
        self
    }
}

impl Default for OutputMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Minimum corner (x, y, z).
    pub min: [f32; 3],
    /// Maximum corner (x, y, z).
    pub max: [f32; 3],
}

impl BoundingBox {
    /// Creates a new bounding box.
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }
}

/// Builder for creating reports ergonomically.
pub struct ReportBuilder {
    spec_hash: String,
    ok: bool,
    errors: Vec<ReportError>,
    warnings: Vec<ReportWarning>,
    outputs: Vec<OutputResult>,
    duration_ms: u64,
    backend_version: String,
    target_triple: String,
}

impl ReportBuilder {
    /// Creates a new report builder.
    ///
    /// # Arguments
    ///
    /// * `spec_hash` - Hex-encoded BLAKE3 hash of the spec
    /// * `backend_version` - Backend identifier and version
    ///
    /// # Example
    ///
    /// ```
    /// use speccade_spec::report::ReportBuilder;
    ///
    /// let report = ReportBuilder::new(
    ///     "a1b2c3d4...".to_string(),
    ///     "speccade-backend-audio v0.1.0".to_string()
    /// )
    /// .ok(true)
    /// .duration_ms(1234)
    /// .build();
    /// ```
    pub fn new(spec_hash: String, backend_version: String) -> Self {
        Self {
            spec_hash,
            ok: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            outputs: Vec::new(),
            duration_ms: 0,
            backend_version,
            target_triple: Self::detect_target_triple(),
        }
    }

    /// Sets the ok status.
    pub fn ok(mut self, ok: bool) -> Self {
        self.ok = ok;
        self
    }

    /// Adds an error to the report.
    pub fn error(mut self, error: ReportError) -> Self {
        self.errors.push(error);
        self.ok = false;
        self
    }

    /// Adds multiple errors to the report.
    pub fn errors(mut self, errors: Vec<ReportError>) -> Self {
        if !errors.is_empty() {
            self.ok = false;
        }
        self.errors.extend(errors);
        self
    }

    /// Adds errors from ValidationErrors.
    pub fn validation_errors(mut self, errors: &[ValidationError]) -> Self {
        if !errors.is_empty() {
            self.ok = false;
            self.errors.extend(
                errors.iter().map(ReportError::from_validation_error)
            );
        }
        self
    }

    /// Adds a warning to the report.
    pub fn warning(mut self, warning: ReportWarning) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Adds multiple warnings to the report.
    pub fn warnings(mut self, warnings: Vec<ReportWarning>) -> Self {
        self.warnings.extend(warnings);
        self
    }

    /// Adds warnings from ValidationWarnings.
    pub fn validation_warnings(mut self, warnings: &[ValidationWarning]) -> Self {
        self.warnings.extend(
            warnings.iter().map(ReportWarning::from_validation_warning)
        );
        self
    }

    /// Adds an output to the report.
    pub fn output(mut self, output: OutputResult) -> Self {
        self.outputs.push(output);
        self
    }

    /// Adds multiple outputs to the report.
    pub fn outputs(mut self, outputs: Vec<OutputResult>) -> Self {
        self.outputs.extend(outputs);
        self
    }

    /// Sets the execution duration in milliseconds.
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Sets the target triple.
    pub fn target_triple(mut self, triple: String) -> Self {
        self.target_triple = triple;
        self
    }

    /// Builds the final report.
    pub fn build(self) -> Report {
        Report {
            report_version: REPORT_VERSION,
            spec_hash: self.spec_hash,
            ok: self.ok,
            errors: self.errors,
            warnings: self.warnings,
            outputs: self.outputs,
            duration_ms: self.duration_ms,
            backend_version: self.backend_version,
            target_triple: self.target_triple,
        }
    }

    /// Detects the current target triple.
    fn detect_target_triple() -> String {
        // Use the target triple from the Rust standard library's build configuration
        #[cfg(target_arch = "x86_64")]
        const ARCH: &str = "x86_64";
        #[cfg(target_arch = "x86")]
        const ARCH: &str = "i686";
        #[cfg(target_arch = "aarch64")]
        const ARCH: &str = "aarch64";
        #[cfg(target_arch = "arm")]
        const ARCH: &str = "arm";
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "arm"
        )))]
        const ARCH: &str = "unknown";

        #[cfg(target_vendor = "pc")]
        const VENDOR: &str = "pc";
        #[cfg(target_vendor = "apple")]
        const VENDOR: &str = "apple";
        #[cfg(target_vendor = "unknown")]
        const VENDOR: &str = "unknown";
        #[cfg(not(any(
            target_vendor = "pc",
            target_vendor = "apple",
            target_vendor = "unknown"
        )))]
        const VENDOR: &str = "unknown";

        #[cfg(target_os = "windows")]
        const OS: &str = "windows";
        #[cfg(target_os = "linux")]
        const OS: &str = "linux";
        #[cfg(target_os = "macos")]
        const OS: &str = "darwin";
        #[cfg(not(any(
            target_os = "windows",
            target_os = "linux",
            target_os = "macos"
        )))]
        const OS: &str = "unknown";

        #[cfg(target_env = "msvc")]
        const ENV: &str = "msvc";
        #[cfg(target_env = "gnu")]
        const ENV: &str = "gnu";
        #[cfg(target_env = "")]
        const ENV: &str = "";
        #[cfg(not(any(
            target_env = "msvc",
            target_env = "gnu",
            target_env = ""
        )))]
        const ENV: &str = "";

        if ENV.is_empty() {
            format!("{}-{}-{}", ARCH, VENDOR, OS)
        } else {
            format!("{}-{}-{}-{}", ARCH, VENDOR, OS, ENV)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ErrorCode, WarningCode};

    #[test]
    fn test_report_builder() {
        let report = ReportBuilder::new(
            "a1b2c3d4e5f6".to_string(),
            "speccade-backend-audio v0.1.0".to_string(),
        )
        .ok(true)
        .duration_ms(1234)
        .build();

        assert_eq!(report.report_version, 1);
        assert_eq!(report.spec_hash, "a1b2c3d4e5f6");
        assert!(report.ok);
        assert_eq!(report.duration_ms, 1234);
        assert_eq!(report.backend_version, "speccade-backend-audio v0.1.0");
        assert!(!report.target_triple.is_empty());
    }

    #[test]
    fn test_report_builder_with_errors() {
        let error = ReportError::new("E001", "Unsupported spec version");
        let report = ReportBuilder::new(
            "abc123".to_string(),
            "test-backend v1.0".to_string(),
        )
        .error(error)
        .duration_ms(500)
        .build();

        assert!(!report.ok);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].code, "E001");
        assert_eq!(report.errors[0].message, "Unsupported spec version");
    }

    #[test]
    fn test_report_builder_with_warnings() {
        let warning = ReportWarning::new("W001", "Missing license");
        let report = ReportBuilder::new(
            "xyz789".to_string(),
            "test-backend v1.0".to_string(),
        )
        .warning(warning)
        .build();

        assert!(report.ok);
        assert_eq!(report.warnings.len(), 1);
        assert_eq!(report.warnings[0].code, "W001");
    }

    #[test]
    fn test_report_filename() {
        assert_eq!(Report::filename("laser-blast-01"), "laser-blast-01.report.json");
        assert_eq!(Report::filename("test"), "test.report.json");
    }

    #[test]
    fn test_report_serialization() {
        let report = ReportBuilder::new(
            "testhash".to_string(),
            "test-backend v1.0".to_string(),
        )
        .ok(true)
        .duration_ms(1000)
        .output(OutputResult::tier1(
            OutputKind::Primary,
            OutputFormat::Wav,
            PathBuf::from("sounds/test.wav"),
            "outputhash".to_string(),
        ))
        .build();

        let json = report.to_json().unwrap();
        assert!(json.contains("\"report_version\":1"));
        assert!(json.contains("\"spec_hash\":\"testhash\""));
        assert!(json.contains("\"ok\":true"));

        // Round-trip test
        let parsed = Report::from_json(&json).unwrap();
        assert_eq!(parsed.spec_hash, report.spec_hash);
        assert_eq!(parsed.ok, report.ok);
        assert_eq!(parsed.outputs.len(), 1);
    }

    #[test]
    fn test_report_pretty_json() {
        let report = ReportBuilder::new(
            "prettyhash".to_string(),
            "test-backend v1.0".to_string(),
        )
        .ok(true)
        .build();

        let pretty = report.to_json_pretty().unwrap();
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));
    }

    #[test]
    fn test_validation_error_conversion() {
        let val_err = ValidationError::with_path(
            ErrorCode::InvalidAssetId,
            "Invalid format",
            "asset_id",
        );

        let report_err = ReportError::from_validation_error(&val_err);
        assert_eq!(report_err.code, "E002");
        assert_eq!(report_err.message, "Invalid format");
        assert_eq!(report_err.path, Some("asset_id".to_string()));
    }

    #[test]
    fn test_validation_warning_conversion() {
        let val_warn = ValidationWarning::with_path(
            WarningCode::MissingLicense,
            "No license specified",
            "license",
        );

        let report_warn = ReportWarning::from_validation_warning(&val_warn);
        assert_eq!(report_warn.code, "W001");
        assert_eq!(report_warn.message, "No license specified");
        assert_eq!(report_warn.path, Some("license".to_string()));
    }

    #[test]
    fn test_output_result_tier1() {
        let result = OutputResult::tier1(
            OutputKind::Primary,
            OutputFormat::Wav,
            PathBuf::from("sounds/test.wav"),
            "abc123".to_string(),
        );

        assert_eq!(result.kind, OutputKind::Primary);
        assert_eq!(result.format, OutputFormat::Wav);
        assert_eq!(result.hash, Some("abc123".to_string()));
        assert!(result.metrics.is_none());
    }

    #[test]
    fn test_output_result_tier2() {
        let metrics = OutputMetrics::new()
            .with_triangle_count(1234)
            .with_bone_count(22);

        let result = OutputResult::tier2(
            OutputKind::Primary,
            OutputFormat::Glb,
            PathBuf::from("meshes/robot.glb"),
            metrics,
        );

        assert_eq!(result.kind, OutputKind::Primary);
        assert_eq!(result.format, OutputFormat::Glb);
        assert!(result.hash.is_none());
        assert!(result.metrics.is_some());

        let m = result.metrics.unwrap();
        assert_eq!(m.triangle_count, Some(1234));
        assert_eq!(m.bone_count, Some(22));
    }

    #[test]
    fn test_output_metrics_builder() {
        let bbox = BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 2.0, 1.0]);
        let metrics = OutputMetrics::new()
            .with_triangle_count(500)
            .with_bounding_box(bbox)
            .with_uv_island_count(4)
            .with_bone_count(22)
            .with_material_slot_count(2)
            .with_max_bone_influences(4)
            .with_animation_frame_count(30)
            .with_animation_duration_seconds(1.0);

        assert_eq!(metrics.triangle_count, Some(500));
        assert_eq!(metrics.uv_island_count, Some(4));
        assert_eq!(metrics.bone_count, Some(22));
        assert_eq!(metrics.material_slot_count, Some(2));
        assert_eq!(metrics.max_bone_influences, Some(4));
        assert_eq!(metrics.animation_frame_count, Some(30));
        assert_eq!(metrics.animation_duration_seconds, Some(1.0));
        assert!(metrics.bounding_box.is_some());
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new([-1.0, -2.0, -3.0], [1.0, 2.0, 3.0]);
        assert_eq!(bbox.min, [-1.0, -2.0, -3.0]);
        assert_eq!(bbox.max, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_report_error_with_path() {
        let err = ReportError::with_path("E007", "Duplicate path", "outputs[1].path");
        assert_eq!(err.code, "E007");
        assert_eq!(err.message, "Duplicate path");
        assert_eq!(err.path, Some("outputs[1].path".to_string()));
    }

    #[test]
    fn test_report_warning_with_path() {
        let warn = ReportWarning::with_path("W002", "Missing description", "description");
        assert_eq!(warn.code, "W002");
        assert_eq!(warn.message, "Missing description");
        assert_eq!(warn.path, Some("description".to_string()));
    }

    #[test]
    fn test_validation_errors_batch_conversion() {
        let errors = vec![
            ValidationError::new(ErrorCode::InvalidAssetId, "Bad asset ID"),
            ValidationError::new(ErrorCode::NoOutputs, "No outputs declared"),
        ];

        let report = ReportBuilder::new("hash".to_string(), "backend v1.0".to_string())
            .validation_errors(&errors)
            .build();

        assert!(!report.ok);
        assert_eq!(report.errors.len(), 2);
        assert_eq!(report.errors[0].code, "E002");
        assert_eq!(report.errors[1].code, "E005");
    }

    #[test]
    fn test_validation_warnings_batch_conversion() {
        let warnings = vec![
            ValidationWarning::new(WarningCode::MissingLicense, "No license"),
            ValidationWarning::new(WarningCode::MissingDescription, "No description"),
        ];

        let report = ReportBuilder::new("hash".to_string(), "backend v1.0".to_string())
            .validation_warnings(&warnings)
            .build();

        assert!(report.ok);
        assert_eq!(report.warnings.len(), 2);
        assert_eq!(report.warnings[0].code, "W001");
        assert_eq!(report.warnings[1].code, "W002");
    }
}
