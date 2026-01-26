//! Report types and writer for SpecCade generation and validation results.
//!
//! This module provides types for creating standardized reports as defined in
//! RFC-0001 Section 4. Reports document the results of `speccade generate` or
//! `speccade validate` operations, including errors, warnings, and output metadata.

mod builder;
mod error;
mod lint;
mod output;
mod structural;
mod timing;

#[cfg(test)]
mod tests;

pub use builder::ReportBuilder;
pub use error::{ReportError, ReportWarning};
pub use lint::{LintIssueData, LintReportData};
pub use output::{
    BakedMapInfo, BakingMetrics, BoundingBox, CollisionBoundingBox, CollisionMeshMetrics,
    NavmeshMetrics, OutputMetrics, OutputResult, StaticMeshLodLevelMetrics,
};
pub use structural::{
    AspectRatios, BoneCoverageInfo, BonePairSymmetry, ComponentAdjacency, ComponentInfo,
    ComponentMetrics, GeometryMetrics, ScaleReference, SkeletalStructureMetrics, StructuralMetrics,
    SymmetryMetrics,
};
pub use timing::StageTiming;

use crate::spec::AssetType;
use serde::{Deserialize, Serialize};

/// Report schema version (always 1 for this RFC).
pub const REPORT_VERSION: u32 = 1;

/// A complete report for a generation or validation operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    /// Report schema version (always 1).
    pub report_version: u32,
    /// Hex-encoded BLAKE3 hash of the canonicalized spec.
    pub spec_hash: String,
    /// Optional hash of the *unexpanded* spec (when generating a derived variant).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_spec_hash: Option<String>,
    /// Variant identifier for this run (if the spec was expanded as a variant).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_id: Option<String>,
    /// Asset ID from the spec (convenience/provenance).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    /// Asset type from the spec (convenience/provenance).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_type: Option<AssetType>,
    /// License identifier from the spec (convenience/provenance).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Seed used for this run (convenience/provenance).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u32>,
    /// Recipe kind from the spec (if present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_kind: Option<String>,
    /// Canonical hash of the recipe (kind + params), if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_hash: Option<String>,
    /// Source file format ("json" or "starlark").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<String>,
    /// BLAKE3 hash of the source file content (before compilation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
    /// Starlark stdlib version (for Starlark sources; cache invalidation key).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdlib_version: Option<String>,
    /// Whether the operation succeeded without errors.
    pub ok: bool,
    /// List of errors that occurred.
    pub errors: Vec<ReportError>,
    /// List of warnings that were generated.
    pub warnings: Vec<ReportWarning>,
    /// List of output artifacts produced.
    pub outputs: Vec<OutputResult>,
    /// Semantic quality lint report for generated outputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lint: Option<LintReportData>,
    /// Per-stage timing breakdown (only present when --profile is used).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stages: Option<Vec<StageTiming>>,
    /// Total execution time in milliseconds.
    pub duration_ms: u64,
    /// Backend identifier and version (e.g., "speccade-backend-audio v0.1.0").
    pub backend_version: String,
    /// Rust target triple (e.g., "x86_64-pc-windows-msvc").
    pub target_triple: String,
    /// Git commit hash of the toolchain/backend producing this report (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    /// Whether the toolchain/backend had uncommitted changes when built (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_dirty: Option<bool>,
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
