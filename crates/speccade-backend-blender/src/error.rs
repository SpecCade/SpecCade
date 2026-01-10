//! Error types for the Blender backend.

use speccade_spec::BackendError;
use std::path::PathBuf;
use thiserror::Error;

/// Result type for Blender backend operations.
pub type BlenderResult<T> = Result<T, BlenderError>;

/// Errors that can occur during Blender backend operations.
#[derive(Debug, Error)]
pub enum BlenderError {
    /// Blender executable not found.
    #[error("Blender executable not found. Ensure Blender is installed and in PATH, or set BLENDER_PATH environment variable")]
    BlenderNotFound,

    /// Failed to spawn Blender process.
    #[error("Failed to spawn Blender process: {0}")]
    SpawnFailed(#[source] std::io::Error),

    /// Blender process timed out.
    #[error("Blender process timed out after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },

    /// Blender process exited with non-zero status.
    #[error("Blender process exited with status {exit_code}: {stderr}")]
    ProcessFailed { exit_code: i32, stderr: String },

    /// Failed to write spec file for Blender.
    #[error("Failed to write spec file: {0}")]
    WriteSpecFailed(#[source] std::io::Error),

    /// Failed to read report from Blender.
    #[error("Failed to read Blender report from {path}: {source}")]
    ReadReportFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse report JSON from Blender.
    #[error("Failed to parse Blender report: {0}")]
    ParseReportFailed(#[source] serde_json::Error),

    /// Blender reported an error.
    #[error("Blender generation failed: {message}")]
    GenerationFailed { message: String },

    /// Invalid recipe kind for this backend.
    #[error("Invalid recipe kind '{kind}' for Blender backend. Expected one of: static_mesh.blender_primitives_v1, skeletal_mesh.blender_rigged_mesh_v1, skeletal_animation.blender_clip_v1")]
    InvalidRecipeKind { kind: String },

    /// Missing recipe in spec.
    #[error("Spec is missing recipe definition")]
    MissingRecipe,

    /// Failed to serialize spec to JSON.
    #[error("Failed to serialize spec: {0}")]
    SerializeFailed(#[source] serde_json::Error),

    /// Failed to deserialize recipe params.
    #[error("Failed to deserialize recipe params: {0}")]
    DeserializeParamsFailed(#[source] serde_json::Error),

    /// Output file not found after generation.
    #[error("Expected output file not found: {path}")]
    OutputNotFound { path: PathBuf },

    /// Python entrypoint script not found.
    #[error("Python entrypoint script not found at: {path}")]
    EntrypointNotFound { path: PathBuf },

    /// IO error during file operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Metrics validation failed.
    #[error("Metrics validation failed: {message}")]
    MetricsValidationFailed { message: String },

    /// Constraint violation.
    #[error("Constraint violation: {message}")]
    ConstraintViolation { message: String },
}

impl BlenderError {
    /// Creates a new process failed error.
    pub fn process_failed(exit_code: i32, stderr: impl Into<String>) -> Self {
        Self::ProcessFailed {
            exit_code,
            stderr: stderr.into(),
        }
    }

    /// Creates a new generation failed error.
    pub fn generation_failed(message: impl Into<String>) -> Self {
        Self::GenerationFailed {
            message: message.into(),
        }
    }

    /// Creates a new constraint violation error.
    pub fn constraint_violation(message: impl Into<String>) -> Self {
        Self::ConstraintViolation {
            message: message.into(),
        }
    }

    /// Creates a new metrics validation failed error.
    pub fn metrics_validation_failed(message: impl Into<String>) -> Self {
        Self::MetricsValidationFailed {
            message: message.into(),
        }
    }
}

impl BackendError for BlenderError {
    fn code(&self) -> &'static str {
        match self {
            BlenderError::BlenderNotFound => "BLENDER_001",
            BlenderError::SpawnFailed(_) => "BLENDER_002",
            BlenderError::Timeout { .. } => "BLENDER_003",
            BlenderError::ProcessFailed { .. } => "BLENDER_004",
            BlenderError::WriteSpecFailed(_) => "BLENDER_005",
            BlenderError::ReadReportFailed { .. } => "BLENDER_006",
            BlenderError::ParseReportFailed(_) => "BLENDER_007",
            BlenderError::GenerationFailed { .. } => "BLENDER_008",
            BlenderError::InvalidRecipeKind { .. } => "BLENDER_009",
            BlenderError::MissingRecipe => "BLENDER_010",
            BlenderError::SerializeFailed(_) => "BLENDER_011",
            BlenderError::DeserializeParamsFailed(_) => "BLENDER_012",
            BlenderError::OutputNotFound { .. } => "BLENDER_013",
            BlenderError::EntrypointNotFound { .. } => "BLENDER_014",
            BlenderError::Io(_) => "BLENDER_015",
            BlenderError::MetricsValidationFailed { .. } => "BLENDER_016",
            BlenderError::ConstraintViolation { .. } => "BLENDER_017",
        }
    }

    fn category(&self) -> &'static str {
        "blender"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BlenderError::BlenderNotFound;
        assert!(err.to_string().contains("Blender executable not found"));

        let err = BlenderError::Timeout { timeout_secs: 300 };
        assert!(err.to_string().contains("300 seconds"));

        let err = BlenderError::process_failed(1, "something went wrong");
        assert!(err.to_string().contains("something went wrong"));
    }

    #[test]
    fn test_constraint_violation() {
        let err = BlenderError::constraint_violation("Max triangles exceeded: 1000 > 500");
        assert!(err.to_string().contains("Max triangles exceeded"));
    }
}
