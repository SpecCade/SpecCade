//! Error types for spec validation and processing.

use thiserror::Error;

/// Error codes for spec validation as defined in RFC-0001.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // Contract errors (E001-E009)
    /// E001: Unsupported spec_version
    UnsupportedSpecVersion,
    /// E002: Invalid asset_id format
    InvalidAssetId,
    /// E003: Unknown asset type
    UnknownAssetType,
    /// E004: Seed out of valid range
    SeedOutOfRange,
    /// E005: No outputs declared
    NoOutputs,
    /// E006: No primary output declared
    NoPrimaryOutput,
    /// E007: Duplicate output path
    DuplicateOutputPath,
    /// E008: Unsafe output path (traversal)
    UnsafeOutputPath,
    /// E009: Output path extension does not match format
    PathFormatMismatch,

    // Recipe errors (E010-E016)
    /// E010: Recipe required for generate command
    MissingRecipe,
    /// E011: Recipe kind incompatible with asset type
    RecipeAssetTypeMismatch,
    /// E012: Invalid recipe params
    InvalidRecipeParams,
    /// E013: Backend not available
    BackendNotAvailable,
    /// E014: Backend execution failed
    BackendExecutionFailed,
    /// E015: Output validation failed
    OutputValidationFailed,
    /// E016: Recipe kind is not supported by this generator version
    UnsupportedRecipeKind,

    // Legacy packed output errors (E020-E023)
    /// E020: Packed output channel references an unknown map key
    PackedChannelsUnknownMapKey,
    /// E021: Packed output is missing a channels mapping
    PackedOutputMissingChannels,
    /// E022: Packed output has an invalid format (must be png)
    PackedOutputInvalidFormat,
    /// E023: Packed texture recipe declared but no packed outputs were provided
    NoPackedOutputs,
}

impl ErrorCode {
    /// Returns the error code string (e.g., "E001").
    pub fn code(&self) -> &'static str {
        match self {
            ErrorCode::UnsupportedSpecVersion => "E001",
            ErrorCode::InvalidAssetId => "E002",
            ErrorCode::UnknownAssetType => "E003",
            ErrorCode::SeedOutOfRange => "E004",
            ErrorCode::NoOutputs => "E005",
            ErrorCode::NoPrimaryOutput => "E006",
            ErrorCode::DuplicateOutputPath => "E007",
            ErrorCode::UnsafeOutputPath => "E008",
            ErrorCode::PathFormatMismatch => "E009",
            ErrorCode::MissingRecipe => "E010",
            ErrorCode::RecipeAssetTypeMismatch => "E011",
            ErrorCode::InvalidRecipeParams => "E012",
            ErrorCode::BackendNotAvailable => "E013",
            ErrorCode::BackendExecutionFailed => "E014",
            ErrorCode::OutputValidationFailed => "E015",
            ErrorCode::UnsupportedRecipeKind => "E016",
            ErrorCode::PackedChannelsUnknownMapKey => "E020",
            ErrorCode::PackedOutputMissingChannels => "E021",
            ErrorCode::PackedOutputInvalidFormat => "E022",
            ErrorCode::NoPackedOutputs => "E023",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Warning codes for spec validation as defined in RFC-0001.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarningCode {
    /// W001: Missing license information
    MissingLicense,
    /// W002: Missing description
    MissingDescription,
    /// W003: Seed near overflow boundary
    SeedNearOverflow,
    /// W004: Unused recipe params
    UnusedRecipeParams,
}

impl WarningCode {
    /// Returns the warning code string (e.g., "W001").
    pub fn code(&self) -> &'static str {
        match self {
            WarningCode::MissingLicense => "W001",
            WarningCode::MissingDescription => "W002",
            WarningCode::SeedNearOverflow => "W003",
            WarningCode::UnusedRecipeParams => "W004",
        }
    }
}

impl std::fmt::Display for WarningCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// A validation error with code, message, and optional JSON path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// The error code.
    pub code: ErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// JSON path to the problematic field (e.g., "outputs\[0\].path").
    pub path: Option<String>,
}

impl ValidationError {
    /// Creates a new validation error.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            path: None,
        }
    }

    /// Creates a new validation error with a JSON path.
    pub fn with_path(code: ErrorCode, message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            path: Some(path.into()),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref path) = self.path {
            write!(f, "{}: {} (at {})", self.code, self.message, path)
        } else {
            write!(f, "{}: {}", self.code, self.message)
        }
    }
}

impl std::error::Error for ValidationError {}

/// A validation warning with code, message, and optional JSON path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationWarning {
    /// The warning code.
    pub code: WarningCode,
    /// Human-readable warning message.
    pub message: String,
    /// JSON path to the problematic field.
    pub path: Option<String>,
}

impl ValidationWarning {
    /// Creates a new validation warning.
    pub fn new(code: WarningCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            path: None,
        }
    }

    /// Creates a new validation warning with a JSON path.
    pub fn with_path(
        code: WarningCode,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            path: Some(path.into()),
        }
    }
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref path) = self.path {
            write!(f, "{}: {} (at {})", self.code, self.message, path)
        } else {
            write!(f, "{}: {}", self.code, self.message)
        }
    }
}

/// Top-level error type for spec operations.
#[derive(Debug, Error)]
pub enum SpecError {
    /// Spec validation failed with one or more errors.
    #[error("spec validation failed with {0} error(s)")]
    ValidationFailed(usize),

    /// JSON parsing error.
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Canonicalization error.
    #[error("canonicalization error: {0}")]
    Canonicalization(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result of spec validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed (no errors).
    pub ok: bool,
    /// List of validation errors.
    pub errors: Vec<ValidationError>,
    /// List of validation warnings.
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Creates a successful validation result.
    pub fn success() -> Self {
        Self {
            ok: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Creates a successful validation result with warnings.
    pub fn success_with_warnings(warnings: Vec<ValidationWarning>) -> Self {
        Self {
            ok: true,
            errors: Vec::new(),
            warnings,
        }
    }

    /// Creates a failed validation result.
    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            ok: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Creates a failed validation result with warnings.
    pub fn failure_with_warnings(
        errors: Vec<ValidationError>,
        warnings: Vec<ValidationWarning>,
    ) -> Self {
        Self {
            ok: false,
            errors,
            warnings,
        }
    }

    /// Adds an error to the result.
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.ok = false;
    }

    /// Adds a warning to the result.
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Returns true if there are no errors.
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    /// Converts to a Result, returning Err if there are errors.
    pub fn into_result(self) -> Result<Vec<ValidationWarning>, Vec<ValidationError>> {
        if self.ok {
            Ok(self.warnings)
        } else {
            Err(self.errors)
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::success()
    }
}

/// Common trait for backend errors.
///
/// This trait provides a unified interface for error reporting across all
/// backends. Each backend error type implements this trait to enable:
/// - Consistent error codes for reporting
/// - Human-readable messages for users
/// - Integration with the unified `GenerationError` enum
///
/// # Example
///
/// ```ignore
/// use speccade_spec::error::BackendError;
///
/// fn handle_error<E: BackendError>(err: E) {
///     eprintln!("[{}] {}", err.code(), err.message());
/// }
/// ```
pub trait BackendError: std::error::Error {
    /// Get the error code for reporting.
    ///
    /// Returns a static string like "AUDIO_001", "TEXTURE_002", etc.
    /// These codes should be stable and can be used for programmatic
    /// error handling.
    fn code(&self) -> &'static str;

    /// Get a human-readable message describing the error.
    ///
    /// This is typically the same as `Display::fmt` but guaranteed to
    /// return an owned String for flexibility in error reporting.
    fn message(&self) -> String {
        self.to_string()
    }

    /// Get the error category for grouping related errors.
    ///
    /// Returns a category like "audio", "texture", "music", "blender".
    fn category(&self) -> &'static str;
}

/// A unified error type that can wrap any backend error.
///
/// This type provides a way to handle errors from different backends uniformly
/// without requiring the spec crate to depend on backend crates. It captures
/// the error code, message, and category from any `BackendError` implementor.
///
/// # Example
///
/// ```ignore
/// use speccade_spec::error::{GenerationError, BackendError};
///
/// fn handle_any_backend_error<E: BackendError + 'static>(err: E) -> GenerationError {
///     GenerationError::from_backend(err)
/// }
/// ```
#[derive(Debug)]
pub struct GenerationError {
    /// The error code (e.g., "AUDIO_001", "TEXTURE_002").
    pub code: &'static str,
    /// The human-readable error message.
    pub message: String,
    /// The error category (e.g., "audio", "texture", "music", "blender").
    pub category: &'static str,
    /// The underlying error, boxed for type erasure.
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl GenerationError {
    /// Create a `GenerationError` from any `BackendError` implementor.
    ///
    /// This is the primary way to create a `GenerationError` from backend-specific
    /// error types.
    pub fn from_backend<E: BackendError + Send + Sync + 'static>(err: E) -> Self {
        Self {
            code: err.code(),
            message: err.message(),
            category: err.category(),
            source: Some(Box::new(err)),
        }
    }

    /// Create a `GenerationError` with explicit values.
    ///
    /// This is useful when you need to create an error without an underlying
    /// backend error (e.g., for dispatch-level errors).
    pub fn new(code: &'static str, message: impl Into<String>, category: &'static str) -> Self {
        Self {
            code,
            message: message.into(),
            category,
            source: None,
        }
    }
}

impl std::fmt::Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for GenerationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(ErrorCode::UnsupportedSpecVersion.code(), "E001");
        assert_eq!(ErrorCode::InvalidAssetId.code(), "E002");
        assert_eq!(ErrorCode::PathFormatMismatch.code(), "E009");
        assert_eq!(ErrorCode::RecipeAssetTypeMismatch.code(), "E011");
    }

    #[test]
    fn test_warning_codes() {
        assert_eq!(WarningCode::MissingLicense.code(), "W001");
        assert_eq!(WarningCode::MissingDescription.code(), "W002");
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::new(
            ErrorCode::InvalidAssetId,
            "must start with lowercase letter",
        );
        assert_eq!(err.to_string(), "E002: must start with lowercase letter");

        let err_with_path = ValidationError::with_path(
            ErrorCode::UnsafeOutputPath,
            "contains '..'",
            "outputs[0].path",
        );
        assert_eq!(
            err_with_path.to_string(),
            "E008: contains '..' (at outputs[0].path)"
        );
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::success();
        assert!(result.is_ok());

        result.add_error(ValidationError::new(ErrorCode::NoOutputs, "no outputs"));
        assert!(!result.is_ok());
        assert_eq!(result.errors.len(), 1);
    }
}
