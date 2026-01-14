//! Error types and reporting infrastructure for determinism testing.

use std::fmt;
use std::path::PathBuf;

use crate::determinism::core::DeterminismResult;

/// Error type for determinism testing.
#[derive(Debug, Clone)]
pub enum DeterminismError {
    /// Spec file was not found.
    SpecNotFound,
    /// IO error reading spec file.
    IoError(String),
    /// Error parsing spec JSON.
    ParseError(String),
    /// Asset type not supported for automated determinism testing.
    UnsupportedAssetType(String),
    /// Generation failed.
    GenerationFailed(String),
}

impl fmt::Display for DeterminismError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SpecNotFound => write!(f, "Spec file not found"),
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::UnsupportedAssetType(t) => write!(f, "Unsupported asset type: {}", t),
            Self::GenerationFailed(e) => write!(f, "Generation failed: {}", e),
        }
    }
}

impl std::error::Error for DeterminismError {}

/// Entry in a determinism report for a single spec.
#[derive(Debug)]
pub struct DeterminismReportEntry {
    /// Path to the spec file.
    pub spec_path: PathBuf,
    /// Result of the determinism test.
    pub result: Result<DeterminismResult, DeterminismError>,
}

impl DeterminismReportEntry {
    /// Check if this entry passed determinism verification.
    pub fn passed(&self) -> bool {
        matches!(&self.result, Ok(r) if r.is_deterministic)
    }
}

/// Report of determinism tests across multiple specs.
#[derive(Debug, Default)]
pub struct DeterminismReport {
    /// Individual spec test entries.
    pub entries: Vec<DeterminismReportEntry>,
}

impl DeterminismReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add an entry to the report.
    pub fn add_entry(&mut self, entry: DeterminismReportEntry) {
        self.entries.push(entry);
    }

    /// Check if all specs passed determinism verification.
    pub fn all_deterministic(&self) -> bool {
        self.entries.iter().all(|e| e.passed())
    }

    /// Get number of passed tests.
    pub fn passed_count(&self) -> usize {
        self.entries.iter().filter(|e| e.passed()).count()
    }

    /// Get number of failed tests.
    pub fn failed_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.passed()).count()
    }

    /// Get total number of tests.
    pub fn total_count(&self) -> usize {
        self.entries.len()
    }

    /// Get all failed entries.
    pub fn failures(&self) -> Vec<&DeterminismReportEntry> {
        self.entries.iter().filter(|e| !e.passed()).collect()
    }

    /// Panic if any tests failed, with detailed report.
    pub fn assert_all_deterministic(&self) {
        if !self.all_deterministic() {
            let mut msg = format!(
                "Determinism verification failed!\n\
                 Passed: {}/{}\n\n\
                 Failures:\n",
                self.passed_count(),
                self.total_count()
            );

            for entry in self.failures() {
                msg.push_str(&format!("  - {:?}\n", entry.spec_path));
                match &entry.result {
                    Ok(result) => {
                        if let Some(diff) = &result.diff_info {
                            msg.push_str(&format!("    {}\n", diff));
                        }
                    }
                    Err(e) => {
                        msg.push_str(&format!("    Error: {}\n", e));
                    }
                }
            }

            panic!("{}", msg);
        }
    }
}

impl fmt::Display for DeterminismReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Determinism Report: {}/{} passed",
            self.passed_count(),
            self.total_count()
        )?;

        for entry in &self.entries {
            let status = if entry.passed() { "PASS" } else { "FAIL" };
            writeln!(f, "  [{}] {:?}", status, entry.spec_path)?;

            if !entry.passed() {
                match &entry.result {
                    Ok(result) => {
                        if let Some(diff) = &result.diff_info {
                            writeln!(f, "        {}", diff)?;
                        }
                    }
                    Err(e) => {
                        writeln!(f, "        Error: {}", e)?;
                    }
                }
            }
        }

        Ok(())
    }
}
