//! Starlark compiler module.
//!
//! This module provides functionality for compiling Starlark source files
//! into canonical SpecCade Spec IR. The compiler:
//!
//! - Parses Starlark source code
//! - Evaluates the code with safety limits (timeout, no recursion)
//! - Converts the resulting Starlark dict to JSON
//! - Validates and parses the JSON as a Spec
//!
//! # Safety
//!
//! The compiler enforces several safety measures:
//!
//! - **Timeout**: Evaluation is limited to prevent infinite loops
//! - **No external loads**: The `load()` statement is disabled
//! - **Standard dialect**: Only standard Starlark features are enabled
//!
//! # Example
//!
//! ```ignore
//! use speccade_cli::compiler::{compile, CompilerConfig};
//!
//! let source = r#"
//! {
//!     "spec_version": 1,
//!     "asset_id": "test-01",
//!     "asset_type": "audio",
//!     "license": "CC0-1.0",
//!     "seed": 42,
//!     "outputs": [
//!         {"kind": "primary", "format": "wav", "path": "test.wav"}
//!     ]
//! }
//! "#;
//!
//! let config = CompilerConfig::default();
//! let result = compile("spec.star", source, &config)?;
//! println!("Asset ID: {}", result.spec.asset_id);
//! ```

mod convert;
mod error;
mod eval;
pub mod stdlib;

pub use error::CompileError;

use speccade_spec::Spec;

/// Current Starlark stdlib version.
/// Increment when stdlib changes affect output.
pub const STDLIB_VERSION: &str = "0.1.0";

/// Default Starlark evaluation timeout in seconds.
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 30;

/// Configuration for the Starlark compiler.
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Maximum evaluation time in seconds (default: 30).
    pub timeout_seconds: u64,
    /// Whether to enable Starlark `load()` statements (default: false).
    pub enable_load: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: DEFAULT_TIMEOUT_SECONDS,
            enable_load: false,
        }
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

/// Result of Starlark compilation.
#[derive(Debug)]
pub struct CompileResult {
    /// The parsed spec.
    pub spec: Spec,
    /// Compilation warnings.
    pub warnings: Vec<CompileWarning>,
}

/// Compile Starlark source to a canonical Spec.
///
/// # Arguments
/// * `filename` - Filename for error messages
/// * `source` - Starlark source code
/// * `config` - Compiler configuration
///
/// # Returns
/// * `Ok(CompileResult)` - Successfully compiled spec
/// * `Err(CompileError)` - Compilation failed
///
/// # Example
///
/// ```ignore
/// use speccade_cli::compiler::{compile, CompilerConfig};
///
/// let config = CompilerConfig::default();
/// let result = compile("spec.star", source, &config)?;
/// ```
pub fn compile(
    filename: &str,
    source: &str,
    config: &CompilerConfig,
) -> Result<CompileResult, CompileError> {
    eval::eval_with_timeout(filename, source, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CompilerConfig::default();
        assert_eq!(config.timeout_seconds, DEFAULT_TIMEOUT_SECONDS);
        assert!(!config.enable_load);
    }

    #[test]
    fn test_stdlib_version() {
        // Ensure the version string is valid semver-ish
        assert!(STDLIB_VERSION.contains('.'));
        assert!(!STDLIB_VERSION.is_empty());
    }

    #[test]
    fn test_compile_minimal() {
        let source = r#"
{
    "spec_version": 1,
    "asset_id": "compile-test-01",
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
}
"#;
        let config = CompilerConfig::default();
        let result = compile("test.star", source, &config).unwrap();
        assert_eq!(result.spec.asset_id, "compile-test-01");
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_compile_with_starlark_features() {
        let source = r#"
# Use Starlark features
base_id = "feature-test"
version = 1

def make_output(name):
    return {
        "kind": "primary",
        "format": "wav",
        "path": "sounds/" + name + ".wav"
    }

outputs = [make_output("laser"), make_output("explosion")]

{
    "spec_version": 1,
    "asset_id": base_id + "-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": outputs
}
"#;
        let config = CompilerConfig::default();
        let result = compile("test.star", source, &config).unwrap();
        assert_eq!(result.spec.asset_id, "feature-test-01");
        assert_eq!(result.spec.outputs.len(), 2);
    }

    #[test]
    fn test_compile_warning_new() {
        let warning = CompileWarning::new("test warning");
        assert_eq!(warning.message, "test warning");
        assert!(warning.location.is_none());
    }

    #[test]
    fn test_compile_warning_with_location() {
        let warning = CompileWarning::with_location("test warning", "line 10");
        assert_eq!(warning.message, "test warning");
        assert_eq!(warning.location, Some("line 10".to_string()));
    }
}
