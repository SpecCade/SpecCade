//! Test fixture utilities for golden test files.

use std::fs;
use std::path::PathBuf;

/// Paths to golden test fixtures in the repository.
pub struct GoldenFixtures;

impl GoldenFixtures {
    /// Get the path to the golden speccade specs directory.
    pub fn speccade_specs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("golden")
            .join("speccade")
            .join("specs")
    }

    /// Get the path to the expected hashes directory.
    pub fn expected_hashes_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("golden")
            .join("speccade")
            .join("expected")
            .join("hashes")
    }

    /// Get the path to the expected metrics directory.
    pub fn expected_metrics_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("golden")
            .join("speccade")
            .join("expected")
            .join("metrics")
    }

    /// Check if golden fixtures exist.
    pub fn exists() -> bool {
        Self::speccade_specs_dir().exists()
    }

    /// List all canonical spec files in a category.
    pub fn list_speccade_specs(asset_type: &str) -> Vec<PathBuf> {
        let type_dir = Self::speccade_specs_dir().join(asset_type);
        if !type_dir.exists() {
            return Vec::new();
        }

        let mut specs: Vec<PathBuf> = fs::read_dir(&type_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
            .filter(|p| {
                !p.file_name()
                    .map(|name| name.to_string_lossy().contains(".report."))
                    .unwrap_or(false)
            })
            .collect();

        // Deterministic ordering: sort by filename.
        specs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        specs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_fixtures_paths() {
        let speccade = GoldenFixtures::speccade_specs_dir();
        println!("Speccade specs dir: {:?}", speccade);
    }
}
