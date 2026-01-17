//! Input abstraction for loading specs from JSON or Starlark sources.
//!
//! This module provides a unified interface for loading SpecCade specs from
//! different source formats. It dispatches by file extension and returns
//! a consistent result type with source provenance information.

use serde::{Deserialize, Serialize};
use speccade_spec::Spec;
use std::path::{Path, PathBuf};

#[cfg(feature = "starlark")]
use crate::compiler;

/// Recognized JSON extensions.
pub const JSON_EXTENSIONS: &[&str] = &["json"];

/// Recognized Starlark extensions.
pub const STARLARK_EXTENSIONS: &[&str] = &["star", "bzl"];

/// Identifies the source format of a spec file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    /// JSON spec file (existing format).
    Json,
    /// Starlark spec file (new format).
    Starlark,
}

impl SourceKind {
    /// Returns the string representation for reports.
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceKind::Json => "json",
            SourceKind::Starlark => "starlark",
        }
    }
}

impl std::fmt::Display for SourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A warning generated during Starlark compilation.
#[derive(Debug, Clone)]
pub struct CompileWarning {
    /// Warning message.
    pub message: String,
    /// Source location (line:column) if available.
    pub location: Option<String>,
}

impl CompileWarning {
    /// Creates a new compile warning.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
        }
    }

    /// Creates a new compile warning with location.
    pub fn with_location(message: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: Some(location.into()),
        }
    }
}

/// Result of loading and compiling a spec from any supported format.
#[derive(Debug)]
pub struct LoadResult {
    /// The canonical spec IR.
    pub spec: Spec,
    /// Source format.
    pub source_kind: SourceKind,
    /// BLAKE3 hash of the source file content (hex string).
    pub source_hash: String,
    /// Warnings from compilation (Starlark only; empty for JSON).
    pub warnings: Vec<CompileWarning>,
}

/// Errors that can occur during spec loading.
#[derive(Debug)]
pub enum InputError {
    /// File could not be read.
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Unknown file extension.
    UnknownExtension { extension: Option<String> },

    /// Starlark feature not enabled.
    #[cfg(not(feature = "starlark"))]
    StarlarkNotEnabled,

    /// JSON parsing failed.
    JsonParse { message: String },

    /// Starlark compilation failed.
    #[cfg(feature = "starlark")]
    StarlarkCompile { message: String },

    /// Starlark evaluation timed out.
    #[cfg(feature = "starlark")]
    Timeout { seconds: u64 },

    /// Starlark output is not a valid spec.
    InvalidSpec { message: String },
}

impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::FileRead { path, source } => {
                write!(f, "failed to read file '{}': {}", path.display(), source)
            }
            InputError::UnknownExtension { extension } => match extension {
                Some(ext) => write!(
                    f,
                    "unknown file extension '.{}' (expected .json or .star)",
                    ext
                ),
                None => write!(f, "file has no extension (expected .json or .star)"),
            },
            #[cfg(not(feature = "starlark"))]
            InputError::StarlarkNotEnabled => {
                write!(
                    f,
                    "Starlark support is not enabled. Rebuild with --features starlark"
                )
            }
            InputError::JsonParse { message } => {
                write!(f, "JSON parse error: {}", message)
            }
            #[cfg(feature = "starlark")]
            InputError::StarlarkCompile { message } => {
                write!(f, "Starlark error: {}", message)
            }
            #[cfg(feature = "starlark")]
            InputError::Timeout { seconds } => {
                write!(f, "Starlark evaluation timed out after {}s", seconds)
            }
            InputError::InvalidSpec { message } => {
                write!(f, "invalid spec: {}", message)
            }
        }
    }
}

impl std::error::Error for InputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InputError::FileRead { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Load a spec from a file path, dispatching by extension.
///
/// # Arguments
/// * `path` - Path to the spec file (.json or .star)
///
/// # Returns
/// * `Ok(LoadResult)` - Successfully loaded and parsed spec
/// * `Err(InputError)` - File read, parse, or compilation error
///
/// # Example
/// ```no_run
/// use std::path::Path;
/// use speccade_cli::input::load_spec;
///
/// let result = load_spec(Path::new("spec.json")).unwrap();
/// println!("Loaded {} spec", result.source_kind.as_str());
/// ```
pub fn load_spec(path: &Path) -> Result<LoadResult, InputError> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    match extension.as_deref() {
        Some(ext) if JSON_EXTENSIONS.contains(&ext) => load_json_spec(path),
        #[cfg(feature = "starlark")]
        Some(ext) if STARLARK_EXTENSIONS.contains(&ext) => load_starlark_spec(path),
        #[cfg(not(feature = "starlark"))]
        Some(ext) if STARLARK_EXTENSIONS.contains(&ext) => Err(InputError::StarlarkNotEnabled),
        _ => Err(InputError::UnknownExtension { extension }),
    }
}

/// Load a spec from a JSON file.
fn load_json_spec(path: &Path) -> Result<LoadResult, InputError> {
    // Read file content
    let content = std::fs::read_to_string(path).map_err(|e| InputError::FileRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    // Compute source hash
    let source_hash = blake3::hash(content.as_bytes()).to_hex().to_string();

    // Parse JSON
    let spec = Spec::from_json(&content).map_err(|e| InputError::JsonParse {
        message: e.to_string(),
    })?;

    Ok(LoadResult {
        spec,
        source_kind: SourceKind::Json,
        source_hash,
        warnings: Vec::new(),
    })
}

/// Load a spec from a Starlark file.
#[cfg(feature = "starlark")]
fn load_starlark_spec(path: &Path) -> Result<LoadResult, InputError> {
    // Read file content
    let content = std::fs::read_to_string(path).map_err(|e| InputError::FileRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    // Compute source hash (on raw Starlark source)
    let source_hash = blake3::hash(content.as_bytes()).to_hex().to_string();

    // Get filename for error messages
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.star");

    // Compile Starlark to Spec
    let config = compiler::CompilerConfig::default();
    let compile_result =
        compiler::compile(filename, &content, &config).map_err(|e| match e {
            compiler::CompileError::Timeout { seconds } => InputError::Timeout { seconds },
            other => InputError::StarlarkCompile {
                message: other.to_string(),
            },
        })?;

    // Convert compiler warnings to input warnings
    let warnings = compile_result
        .warnings
        .into_iter()
        .map(|w| CompileWarning {
            message: w.message,
            location: w.location,
        })
        .collect();

    Ok(LoadResult {
        spec: compile_result.spec,
        source_kind: SourceKind::Starlark,
        source_hash,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_kind_as_str() {
        assert_eq!(SourceKind::Json.as_str(), "json");
        assert_eq!(SourceKind::Starlark.as_str(), "starlark");
    }

    #[test]
    fn test_source_kind_display() {
        assert_eq!(format!("{}", SourceKind::Json), "json");
        assert_eq!(format!("{}", SourceKind::Starlark), "starlark");
    }

    #[test]
    fn test_load_json_spec() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("test.json");

        let spec_json = r#"{
            "spec_version": 1,
            "asset_id": "test-asset-01",
            "asset_type": "audio",
            "license": "CC0-1.0",
            "seed": 42,
            "outputs": [
                {
                    "kind": "primary",
                    "format": "wav",
                    "path": "sounds/test.wav"
                }
            ]
        }"#;

        std::fs::write(&spec_path, spec_json).unwrap();

        let result = load_spec(&spec_path).unwrap();
        assert_eq!(result.source_kind, SourceKind::Json);
        assert_eq!(result.spec.asset_id, "test-asset-01");
        assert!(!result.source_hash.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_load_unknown_extension() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("test.yaml");
        std::fs::write(&spec_path, "key: value").unwrap();

        let result = load_spec(&spec_path);
        assert!(matches!(
            result,
            Err(InputError::UnknownExtension { extension: Some(ref ext) }) if ext == "yaml"
        ));
    }

    #[test]
    fn test_load_file_not_found() {
        let result = load_spec(Path::new("/nonexistent/spec.json"));
        assert!(matches!(result, Err(InputError::FileRead { .. })));
    }

    #[test]
    fn test_load_invalid_json() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("invalid.json");
        std::fs::write(&spec_path, "{ invalid json }").unwrap();

        let result = load_spec(&spec_path);
        assert!(matches!(result, Err(InputError::JsonParse { .. })));
    }

    #[test]
    fn test_compile_warning_new() {
        let warning = CompileWarning::new("test warning");
        assert_eq!(warning.message, "test warning");
        assert!(warning.location.is_none());
    }

    #[test]
    fn test_compile_warning_with_location() {
        let warning = CompileWarning::with_location("test warning", "line 1, column 5");
        assert_eq!(warning.message, "test warning");
        assert_eq!(warning.location, Some("line 1, column 5".to_string()));
    }
}
