//! Lint rule trait and domain-specific rule modules.

use crate::report::{AssetType, LintIssue, Severity};
use speccade_spec::Spec;
use std::path::Path;

pub mod audio;
pub mod mesh;
pub mod music;
pub mod texture;

/// Data needed to analyze an asset.
pub struct AssetData<'a> {
    /// Path to the asset file.
    pub path: &'a Path,
    /// Raw file contents.
    pub bytes: &'a [u8],
}

/// A lint rule that can analyze assets and report issues.
pub trait LintRule: Send + Sync {
    /// Unique identifier (e.g., "audio/clipping", "mesh/non-manifold").
    fn id(&self) -> &'static str;

    /// Human-readable description.
    fn description(&self) -> &'static str;

    /// Which asset types this rule applies to.
    fn applies_to(&self) -> &[AssetType];

    /// Default severity (can be overridden by config).
    fn default_severity(&self) -> Severity;

    /// Run the check, return issues found.
    ///
    /// The optional `spec` parameter allows rules to reference source locations
    /// for LLM-actionable fixes.
    fn check(&self, asset: &AssetData, spec: Option<&Spec>) -> Vec<LintIssue>;
}

/// Errors that can occur during linting.
#[derive(Debug, thiserror::Error)]
pub enum LintError {
    /// Failed to read the asset file.
    #[error("failed to read asset: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to parse the asset format.
    #[error("failed to parse asset: {0}")]
    Parse(String),

    /// The asset type is not supported by this rule.
    #[error("unsupported asset type: {0}")]
    UnsupportedAsset(String),
}
