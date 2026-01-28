//! Feature coverage tracking command implementation
//!
//! Generates coverage reports showing which stdlib features have golden examples.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// A function from the stdlib
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdlibFunction {
    pub name: String,
    pub category: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub params: Vec<StdlibParam>,
}

/// A parameter from a stdlib function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdlibParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub enum_values: Option<Vec<String>>,
}

/// Feature inventory loaded from stdlib.snapshot.json
#[derive(Debug, Clone)]
pub struct FeatureInventory {
    pub functions: Vec<StdlibFunction>,
    /// Map of enum name -> possible values (extracted from function params)
    pub enums: HashMap<String, Vec<String>>,
}

/// Stdlib snapshot JSON structure
#[derive(Debug, Deserialize)]
struct StdlibSnapshot {
    functions: Vec<StdlibFunction>,
}

/// Load feature inventory from stdlib.snapshot.json
///
/// # Arguments
/// * `base_path` - Optional base path to the project root. If None, uses current directory.
pub fn load_feature_inventory_from(base_path: Option<&Path>) -> Result<FeatureInventory> {
    let snapshot_path = match base_path {
        Some(base) => base.join("golden/stdlib/stdlib.snapshot.json"),
        None => PathBuf::from("golden/stdlib/stdlib.snapshot.json"),
    };

    let content = fs::read_to_string(&snapshot_path)
        .with_context(|| format!("Failed to read {}", snapshot_path.display()))?;

    let snapshot: StdlibSnapshot = serde_json::from_str(&content)
        .with_context(|| "Failed to parse stdlib.snapshot.json")?;

    // Extract enums from function parameters
    let mut enums: HashMap<String, Vec<String>> = HashMap::new();

    for func in &snapshot.functions {
        for param in &func.params {
            if let Some(ref values) = param.enum_values {
                // Use param name as enum name (e.g., "waveform", "filter_type")
                let enum_name = param.name.clone();
                let entry = enums.entry(enum_name).or_default();
                for value in values {
                    if !entry.contains(value) {
                        entry.push(value.clone());
                    }
                }
            }
        }
    }

    Ok(FeatureInventory {
        functions: snapshot.functions,
        enums,
    })
}

/// Load feature inventory from stdlib.snapshot.json (from current directory)
pub fn load_feature_inventory() -> Result<FeatureInventory> {
    load_feature_inventory_from(None)
}

/// Run the coverage generate subcommand
///
/// # Arguments
/// * `strict` - If true, exit with code 1 when coverage < 100%
/// * `output` - Optional output path (default: docs/coverage/feature-coverage.yaml)
pub fn run_generate(strict: bool, output: Option<&str>) -> Result<ExitCode> {
    let output_path = output.unwrap_or("docs/coverage/feature-coverage.yaml");
    println!("coverage generate: output={}, strict={}", output_path, strict);
    println!("Not yet implemented");
    Ok(ExitCode::SUCCESS)
}

/// Run the coverage report subcommand
pub fn run_report() -> Result<ExitCode> {
    println!("coverage report");
    println!("Not yet implemented");
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Get the project root for tests
    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
    }

    #[test]
    fn test_load_feature_inventory() {
        let inventory = load_feature_inventory_from(Some(&project_root())).unwrap();

        // Should have functions from stdlib
        assert!(!inventory.functions.is_empty(), "expected some functions");

        // Should have categories
        let categories: std::collections::HashSet<_> = inventory
            .functions
            .iter()
            .map(|f| f.category.as_str())
            .collect();
        assert!(categories.contains("audio"), "expected audio category");

        // Should have specific known functions
        let func_names: std::collections::HashSet<_> = inventory
            .functions
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert!(func_names.contains("oscillator"), "expected oscillator function");
    }
}
