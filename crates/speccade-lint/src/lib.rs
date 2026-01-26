//! Semantic quality lint system for SpecCade.
//!
//! Detects perceptual problems in generated assets and provides
//! LLM-actionable fix suggestions.
//!
//! # Example
//!
//! ```no_run
//! use speccade_lint::RuleRegistry;
//! use std::path::Path;
//!
//! let registry = RuleRegistry::default_rules();
//! let report = registry.lint(Path::new("sound.wav"), None).unwrap();
//!
//! if !report.ok {
//!     for issue in &report.errors {
//!         eprintln!("ERROR: {} - {}", issue.rule_id, issue.message);
//!     }
//! }
//! ```

pub mod registry;
pub mod report;
pub mod rules;

pub use registry::{RuleMetadata, RuleRegistry};
pub use report::{AssetType, LintIssue, LintReport, LintSummary, Severity};
pub use rules::{AssetData, LintError, LintRule};
