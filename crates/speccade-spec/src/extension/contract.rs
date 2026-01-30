//! I/O contract types for extension communication.
//!
//! This module defines the protocol for communication between SpecCade and external extensions.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::DeterminismLevel;

/// Output manifest written by an extension after generation.
///
/// Extensions must write this manifest to `{out_dir}/manifest.json` after completing generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionOutputManifest {
    /// Schema version for the output manifest (currently 1).
    #[serde(default = "default_manifest_version")]
    pub manifest_version: u32,
    /// Whether generation succeeded.
    pub success: bool,
    /// List of output files produced.
    #[serde(default)]
    pub output_files: Vec<ExtensionOutputFile>,
    /// Determinism report for this generation.
    pub determinism_report: DeterminismReport,
    /// List of errors that occurred.
    #[serde(default)]
    pub errors: Vec<ExtensionErrorEntry>,
    /// List of warnings.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Generation duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Extension version that produced this output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_version: Option<String>,
    /// Additional metadata (extension-specific).
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

fn default_manifest_version() -> u32 {
    1
}

impl ExtensionOutputManifest {
    /// Creates a new successful output manifest.
    pub fn success(
        output_files: Vec<ExtensionOutputFile>,
        determinism_report: DeterminismReport,
    ) -> Self {
        Self {
            manifest_version: 1,
            success: true,
            output_files,
            determinism_report,
            errors: Vec::new(),
            warnings: Vec::new(),
            duration_ms: None,
            extension_version: None,
            metadata: serde_json::Map::new(),
        }
    }

    /// Creates a failed output manifest.
    pub fn failure(
        errors: Vec<ExtensionErrorEntry>,
        determinism_report: DeterminismReport,
    ) -> Self {
        Self {
            manifest_version: 1,
            success: false,
            output_files: Vec::new(),
            determinism_report,
            errors,
            warnings: Vec::new(),
            duration_ms: None,
            extension_version: None,
            metadata: serde_json::Map::new(),
        }
    }

    /// Adds a warning to the manifest.
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Sets the duration.
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Sets the extension version.
    pub fn with_extension_version(mut self, version: impl Into<String>) -> Self {
        self.extension_version = Some(version.into());
        self
    }
}

/// Description of a single output file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionOutputFile {
    /// Relative path within the output directory.
    pub path: String,
    /// BLAKE3 hash of the file contents (hex-encoded).
    pub hash: String,
    /// File size in bytes.
    pub size: u64,
    /// Output kind (primary, metadata, preview).
    #[serde(default = "default_output_kind")]
    pub kind: String,
    /// File format (png, wav, json, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

fn default_output_kind() -> String {
    "primary".to_string()
}

impl ExtensionOutputFile {
    /// Creates a new output file descriptor.
    pub fn new(path: impl Into<String>, hash: impl Into<String>, size: u64) -> Self {
        Self {
            path: path.into(),
            hash: hash.into(),
            size,
            kind: "primary".to_string(),
            format: None,
        }
    }

    /// Creates a primary output file.
    pub fn primary(
        path: impl Into<String>,
        hash: impl Into<String>,
        size: u64,
        format: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            hash: hash.into(),
            size,
            kind: "primary".to_string(),
            format: Some(format.into()),
        }
    }

    /// Creates a metadata output file.
    pub fn metadata(path: impl Into<String>, hash: impl Into<String>, size: u64) -> Self {
        Self {
            path: path.into(),
            hash: hash.into(),
            size,
            kind: "metadata".to_string(),
            format: Some("json".to_string()),
        }
    }
}

/// Determinism report from an extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeterminismReport {
    /// Hash of the input spec (for verification).
    pub input_hash: String,
    /// Combined hash of all output files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_hash: Option<String>,
    /// Declared determinism tier.
    pub tier: u8,
    /// Declared determinism level.
    pub determinism: DeterminismLevel,
    /// Seed used for generation.
    pub seed: u64,
    /// Whether the extension claims this run was deterministic.
    #[serde(default = "default_true")]
    pub deterministic: bool,
    /// If non-deterministic, explanation of why.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_determinism_reason: Option<String>,
}

fn default_true() -> bool {
    true
}

impl DeterminismReport {
    /// Creates a new determinism report for a Tier 1 (byte-identical) extension.
    pub fn tier1(input_hash: impl Into<String>, output_hash: impl Into<String>, seed: u64) -> Self {
        Self {
            input_hash: input_hash.into(),
            output_hash: Some(output_hash.into()),
            tier: 1,
            determinism: DeterminismLevel::ByteIdentical,
            seed,
            deterministic: true,
            non_determinism_reason: None,
        }
    }

    /// Creates a new determinism report for a Tier 2 (semantic equivalent) extension.
    pub fn tier2(input_hash: impl Into<String>, seed: u64) -> Self {
        Self {
            input_hash: input_hash.into(),
            output_hash: None,
            tier: 2,
            determinism: DeterminismLevel::SemanticEquivalent,
            seed,
            deterministic: true,
            non_determinism_reason: None,
        }
    }

    /// Creates a new determinism report for a non-deterministic extension.
    pub fn non_deterministic(
        input_hash: impl Into<String>,
        seed: u64,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            input_hash: input_hash.into(),
            output_hash: None,
            tier: 3,
            determinism: DeterminismLevel::NonDeterministic,
            seed,
            deterministic: false,
            non_determinism_reason: Some(reason.into()),
        }
    }
}

/// An error entry in the output manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionErrorEntry {
    /// Error code (extension-specific).
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Additional context (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

impl ExtensionErrorEntry {
    /// Creates a new error entry.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            context: None,
        }
    }

    /// Adds context to the error.
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }
}

/// Errors that can occur during extension execution.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtensionError {
    /// The extension process failed to start.
    SpawnFailed(String),
    /// The extension timed out.
    Timeout { timeout_seconds: u64 },
    /// The extension exited with a non-zero code.
    NonZeroExit { code: i32, stderr: String },
    /// The output manifest is missing.
    ManifestMissing,
    /// The output manifest is invalid JSON.
    ManifestInvalid(String),
    /// The output manifest failed validation.
    ManifestValidation(String),
    /// An output file is missing.
    OutputFileMissing(PathBuf),
    /// An output file hash mismatch.
    OutputHashMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    /// Determinism tier mismatch.
    TierMismatch { declared: u8, expected: u8 },
    /// Input hash mismatch (spec was modified).
    InputHashMismatch { expected: String, actual: String },
    /// The extension reported an error.
    ExtensionReportedError(Vec<ExtensionErrorEntry>),
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SpawnFailed(msg) => write!(f, "Failed to spawn extension: {}", msg),
            Self::Timeout { timeout_seconds } => {
                write!(f, "Extension timed out after {} seconds", timeout_seconds)
            }
            Self::NonZeroExit { code, stderr } => {
                write!(f, "Extension exited with code {}: {}", code, stderr)
            }
            Self::ManifestMissing => write!(f, "Extension did not produce manifest.json"),
            Self::ManifestInvalid(msg) => write!(f, "Invalid manifest.json: {}", msg),
            Self::ManifestValidation(msg) => write!(f, "Manifest validation failed: {}", msg),
            Self::OutputFileMissing(path) => write!(f, "Output file missing: {}", path.display()),
            Self::OutputHashMismatch {
                path,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Hash mismatch for {}: expected {}, got {}",
                    path.display(),
                    expected,
                    actual
                )
            }
            Self::TierMismatch { declared, expected } => {
                write!(
                    f,
                    "Tier mismatch: extension declared tier {}, manifest reports tier {}",
                    expected, declared
                )
            }
            Self::InputHashMismatch { expected, actual } => {
                write!(
                    f,
                    "Input hash mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            Self::ExtensionReportedError(errors) => {
                let msgs: Vec<_> = errors
                    .iter()
                    .map(|e| format!("[{}] {}", e.code, e.message))
                    .collect();
                write!(f, "Extension reported errors: {}", msgs.join("; "))
            }
        }
    }
}

impl std::error::Error for ExtensionError {}

/// Validation errors for output manifests.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputManifestValidationError {
    /// Manifest version is unsupported.
    UnsupportedVersion(u32),
    /// No output files in a successful manifest.
    NoOutputFiles,
    /// Output file path is invalid.
    InvalidOutputPath(String),
    /// Output file hash is invalid.
    InvalidOutputHash(String),
    /// Determinism report is missing.
    MissingDeterminismReport,
    /// Tier mismatch in determinism report.
    DeterminismTierMismatch {
        tier: u8,
        determinism: DeterminismLevel,
    },
    /// Tier 1 extension missing output hash.
    MissingOutputHash,
}

impl std::fmt::Display for OutputManifestValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedVersion(v) => write!(f, "Unsupported manifest version: {}", v),
            Self::NoOutputFiles => write!(f, "Successful manifest has no output files"),
            Self::InvalidOutputPath(path) => write!(f, "Invalid output path: {}", path),
            Self::InvalidOutputHash(hash) => write!(f, "Invalid output hash: {}", hash),
            Self::MissingDeterminismReport => write!(f, "Missing determinism report"),
            Self::DeterminismTierMismatch { tier, determinism } => {
                write!(
                    f,
                    "Tier {} doesn't match determinism level {}",
                    tier, determinism
                )
            }
            Self::MissingOutputHash => write!(f, "Tier 1 extension must provide output_hash"),
        }
    }
}

impl std::error::Error for OutputManifestValidationError {}

/// Validates an extension output manifest.
pub fn validate_output_manifest(
    manifest: &ExtensionOutputManifest,
) -> Result<(), Vec<OutputManifestValidationError>> {
    let mut errors = Vec::new();

    // Check version
    if manifest.manifest_version != 1 {
        errors.push(OutputManifestValidationError::UnsupportedVersion(
            manifest.manifest_version,
        ));
    }

    // Check output files for successful runs
    if manifest.success && manifest.output_files.is_empty() {
        errors.push(OutputManifestValidationError::NoOutputFiles);
    }

    // Validate output file paths and hashes
    for file in &manifest.output_files {
        if file.path.is_empty() || file.path.contains("..") || file.path.starts_with('/') {
            errors.push(OutputManifestValidationError::InvalidOutputPath(
                file.path.clone(),
            ));
        }
        if file.hash.len() != 64 || !file.hash.chars().all(|c| c.is_ascii_hexdigit()) {
            errors.push(OutputManifestValidationError::InvalidOutputHash(
                file.hash.clone(),
            ));
        }
    }

    // Validate determinism report
    if let Err(det_errors) = validate_determinism_report(&manifest.determinism_report) {
        errors.extend(det_errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validates a determinism report.
pub fn validate_determinism_report(
    report: &DeterminismReport,
) -> Result<(), Vec<OutputManifestValidationError>> {
    let mut errors = Vec::new();

    // Check tier matches determinism level
    let expected_tier = report.determinism.tier();
    if report.tier != expected_tier {
        errors.push(OutputManifestValidationError::DeterminismTierMismatch {
            tier: report.tier,
            determinism: report.determinism,
        });
    }

    // Tier 1 must have output hash
    if report.determinism == DeterminismLevel::ByteIdentical && report.output_hash.is_none() {
        errors.push(OutputManifestValidationError::MissingOutputHash);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
