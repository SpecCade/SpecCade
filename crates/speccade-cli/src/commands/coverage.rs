//! Feature coverage tracking command implementation
//!
//! Generates coverage reports showing which stdlib features have golden examples.

use anyhow::{Context, Result};
use glob::glob;
use regex::Regex;
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
        Some(base) => base.join("stdlib/stdlib.snapshot.json"),
        None => PathBuf::from("stdlib/stdlib.snapshot.json"),
    };

    let content = fs::read_to_string(&snapshot_path)
        .with_context(|| format!("Failed to read {}", snapshot_path.display()))?;

    let snapshot: StdlibSnapshot =
        serde_json::from_str(&content).with_context(|| "Failed to parse stdlib.snapshot.json")?;

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

/// A location where a feature is used
#[derive(Debug, Clone, Serialize)]
pub struct UsageLocation {
    pub file: String,
    pub line: Option<u32>,
}

/// Usages found in Starlark spec files
#[derive(Debug, Default)]
pub struct StarlarkUsages {
    /// Function usages (function_name -> locations)
    pub function_usages: HashMap<String, Vec<UsageLocation>>,
    /// Enum value usages (param_name -> value -> locations)
    pub enum_usages: HashMap<String, HashMap<String, Vec<UsageLocation>>>,
}

/// Scan specs/**/*.star files for function and enum usages
///
/// # Arguments
/// * `base_path` - Optional base path to the project root. If None, uses current directory.
/// * `inventory` - The feature inventory containing known enum values to scan for.
pub fn scan_starlark_usages_from(
    base_path: Option<&Path>,
    inventory: &FeatureInventory,
) -> Result<StarlarkUsages> {
    let mut usages = StarlarkUsages::default();

    let pattern = match base_path {
        Some(base) => base.join("specs/**/*.star").to_string_lossy().to_string(),
        None => "specs/**/*.star".to_string(),
    };

    let entries: Vec<_> = glob(&pattern)
        .with_context(|| format!("Invalid glob pattern: {}", pattern))?
        .filter_map(|e| e.ok())
        .collect();

    // Find function calls: word boundary + function name + optional whitespace + (
    let call_re = Regex::new(r"\b([a-z_][a-z0-9_]*)\s*\(").expect("valid regex");

    // Find string literals: "some_value"
    let string_re = Regex::new(r#""([a-z_][a-z0-9_]*)""#).expect("valid regex");

    // Build reverse lookup: enum_value -> [(param_name, ...)]
    // A single value like "sine" may appear in multiple enums (waveform, shape, etc.)
    let mut value_to_params: HashMap<&str, Vec<&str>> = HashMap::new();
    for (param_name, values) in &inventory.enums {
        for value in values {
            value_to_params
                .entry(value.as_str())
                .or_default()
                .push(param_name.as_str());
        }
    }

    for path in entries {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let file_str = path.to_string_lossy().replace('\\', "/");

        for (line_num, line) in content.lines().enumerate() {
            // Skip comment lines
            if line.trim().starts_with('#') {
                continue;
            }

            let location = UsageLocation {
                file: file_str.clone(),
                line: Some((line_num + 1) as u32),
            };

            // Find function calls
            for cap in call_re.captures_iter(line) {
                let func_name = cap.get(1).unwrap().as_str().to_string();
                usages
                    .function_usages
                    .entry(func_name)
                    .or_default()
                    .push(location.clone());
            }

            // Find string literals that match known enum values
            for cap in string_re.captures_iter(line) {
                let value = cap.get(1).unwrap().as_str();
                if let Some(param_names) = value_to_params.get(value) {
                    // Record this usage for all params that have this enum value
                    for param_name in param_names {
                        usages
                            .enum_usages
                            .entry((*param_name).to_string())
                            .or_default()
                            .entry(value.to_string())
                            .or_default()
                            .push(location.clone());
                    }
                }
            }
        }
    }

    Ok(usages)
}

/// Scan specs/**/*.star files for function usages (from current directory)
pub fn scan_starlark_usages(inventory: &FeatureInventory) -> Result<StarlarkUsages> {
    scan_starlark_usages_from(None, inventory)
}

/// Usages found in JSON spec files
#[derive(Debug, Default)]
pub struct JsonSpecUsages {
    /// Function usages (if spec was generated from Starlark)
    pub function_usages: HashMap<String, Vec<UsageLocation>>,
    /// Recipe kind usages (e.g., "audio_v1.synthesis.oscillator")
    pub recipe_usages: HashMap<String, Vec<UsageLocation>>,
    /// Enum value usages (param_name -> value -> locations)
    pub enum_usages: HashMap<String, HashMap<String, Vec<UsageLocation>>>,
}

/// Scan specs/**/*.star for recipe features (legacy JSON support removed)
///
/// # Arguments
/// * `base_path` - Optional base path to the project root. If None, uses current directory.
pub fn scan_json_spec_usages_from(_base_path: Option<&Path>) -> Result<JsonSpecUsages> {
    // JSON specs are no longer used - all specs are now Starlark
    // This function returns empty usages for backwards compatibility
    Ok(JsonSpecUsages::default())
}

/// Scan for legacy JSON spec usages (no longer used - all specs are Starlark)
pub fn scan_json_spec_usages() -> Result<JsonSpecUsages> {
    scan_json_spec_usages_from(None)
}

/// Coverage report summary
#[derive(Debug, Clone, Serialize)]
pub struct CoverageSummary {
    pub total_features: u32,
    pub covered: u32,
    pub uncovered: u32,
    pub coverage_percent: f64,
}

/// Coverage info for a single function
#[derive(Debug, Clone, Serialize)]
pub struct FunctionCoverage {
    pub name: String,
    pub covered: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub enum_coverage: HashMap<String, HashMap<String, EnumValueCoverage>>,
}

/// Coverage info for a single enum value
#[derive(Debug, Clone, Serialize)]
pub struct EnumValueCoverage {
    pub covered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

/// Full coverage report
#[derive(Debug, Clone, Serialize)]
pub struct CoverageReport {
    pub schema_version: u32,
    pub generated_at: String,
    pub summary: CoverageSummary,
    /// Map of category -> functions
    pub stdlib: HashMap<String, Vec<FunctionCoverage>>,
    /// Features with no coverage
    pub uncovered_features: Vec<String>,
}

/// Generate the full coverage report
///
/// # Arguments
/// * `base_path` - Optional base path to the project root. If None, uses current directory.
pub fn generate_coverage_report_from(base_path: Option<&Path>) -> Result<CoverageReport> {
    // Load inventory and scan usages
    let inventory = load_feature_inventory_from(base_path)?;
    let starlark_usages = scan_starlark_usages_from(base_path, &inventory)?;

    let mut stdlib: HashMap<String, Vec<FunctionCoverage>> = HashMap::new();
    let mut uncovered_features: Vec<String> = Vec::new();
    let mut total_features = 0u32;
    let mut covered = 0u32;

    // Process each function
    for func in &inventory.functions {
        total_features += 1;

        // Find examples from Starlark
        let mut examples: Vec<String> = Vec::new();

        if let Some(usages) = starlark_usages.function_usages.get(&func.name) {
            for usage in usages.iter().take(3) {
                // Limit to 3 examples
                let example = if let Some(line) = usage.line {
                    format!("{}:{}", usage.file, line)
                } else {
                    usage.file.clone()
                };
                if !examples.contains(&example) {
                    examples.push(example);
                }
            }
        }

        let is_covered = !examples.is_empty();
        if is_covered {
            covered += 1;
        } else {
            uncovered_features.push(format!("function:{}", func.name));
        }

        // Check enum coverage for this function's params
        let mut enum_coverage: HashMap<String, HashMap<String, EnumValueCoverage>> = HashMap::new();

        for param in &func.params {
            if let Some(ref enum_values) = param.enum_values {
                let mut value_coverage: HashMap<String, EnumValueCoverage> = HashMap::new();

                for value in enum_values {
                    total_features += 1;

                    // Check if this enum value is used in Starlark specs
                    let example = starlark_usages
                        .enum_usages
                        .get(&param.name)
                        .and_then(|values| values.get(value))
                        .and_then(|locs| locs.first())
                        .map(|loc| {
                            if let Some(line) = loc.line {
                                format!("{}:{}", loc.file, line)
                            } else {
                                loc.file.clone()
                            }
                        });

                    let value_is_covered = example.is_some();
                    if value_is_covered {
                        covered += 1;
                    } else {
                        uncovered_features.push(format!("enum:{}::{}", param.name, value));
                    }

                    value_coverage.insert(
                        value.clone(),
                        EnumValueCoverage {
                            covered: value_is_covered,
                            example,
                        },
                    );
                }

                if !value_coverage.is_empty() {
                    enum_coverage.insert(param.name.clone(), value_coverage);
                }
            }
        }

        let func_coverage = FunctionCoverage {
            name: func.name.clone(),
            covered: is_covered,
            examples,
            enum_coverage,
        };

        stdlib
            .entry(func.category.clone())
            .or_default()
            .push(func_coverage);
    }

    // Sort functions within each category
    for funcs in stdlib.values_mut() {
        funcs.sort_by(|a, b| a.name.cmp(&b.name));
    }

    let coverage_percent = if total_features > 0 {
        (covered as f64 / total_features as f64) * 100.0
    } else {
        100.0
    };

    Ok(CoverageReport {
        schema_version: 1,
        // Deterministic value so `coverage generate` doesn't dirty worktrees.
        generated_at: "1970-01-01T00:00:00Z".to_string(),
        summary: CoverageSummary {
            total_features,
            covered,
            uncovered: total_features - covered,
            coverage_percent,
        },
        stdlib,
        uncovered_features,
    })
}

/// Generate the full coverage report (from current directory)
pub fn generate_coverage_report() -> Result<CoverageReport> {
    generate_coverage_report_from(None)
}

/// Run the coverage generate subcommand
///
/// # Arguments
/// * `strict` - If true, exit with code 1 when coverage < 100%
/// * `output` - Optional output path (default: docs/coverage/feature-coverage.yaml)
pub fn run_generate(strict: bool, output: Option<&str>) -> Result<ExitCode> {
    use colored::Colorize;

    let output_path = output.unwrap_or("docs/coverage/feature-coverage.yaml");

    println!("{}", "Generating feature coverage report...".cyan().bold());

    let report = generate_coverage_report()?;

    // Create output directory if needed
    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    // Write YAML
    let yaml = serde_yaml::to_string(&report)?;
    fs::write(output_path, &yaml)?;

    println!(
        "{} Coverage report written to: {}",
        "OK".green().bold(),
        output_path
    );
    println!();
    print_summary(&report.summary);

    if !report.uncovered_features.is_empty() {
        println!();
        println!("{}", "Uncovered features:".red().bold());
        for feature in report.uncovered_features.iter().take(20) {
            println!("  - {}", feature);
        }
        if report.uncovered_features.len() > 20 {
            println!("  ... and {} more", report.uncovered_features.len() - 20);
        }
    }

    if strict && report.summary.uncovered > 0 {
        println!();
        println!(
            "{} {} features have no golden examples.",
            "BLOCKING:".red().bold(),
            report.summary.uncovered
        );
        println!("Create examples in specs/.");
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}

fn print_summary(summary: &CoverageSummary) {
    use colored::Colorize;

    println!("{}", "Coverage Summary".bold());
    println!("  Total features: {}", summary.total_features);
    println!(
        "  Covered: {} ({:.1}%)",
        summary.covered, summary.coverage_percent
    );
    println!("  Uncovered: {}", summary.uncovered);
}

/// Run the coverage report subcommand
pub fn run_report() -> Result<ExitCode> {
    use colored::Colorize;

    println!("{}", "Feature Coverage Report".cyan().bold());
    println!("{}", "=======================".cyan());
    println!();

    let report = generate_coverage_report()?;

    print_summary(&report.summary);
    println!();

    // Print by category
    println!("{}", "By Category:".bold());

    let mut categories: Vec<_> = report.stdlib.keys().collect();
    categories.sort();

    for category in categories {
        let funcs = &report.stdlib[category];
        let cat_covered = funcs.iter().filter(|f| f.covered).count();
        let cat_total = funcs.len();
        let cat_percent = if cat_total > 0 {
            (cat_covered as f64 / cat_total as f64) * 100.0
        } else {
            100.0
        };

        let status = if cat_covered == cat_total {
            "OK".green()
        } else {
            "MISSING".red()
        };

        println!(
            "  {}: {}/{} ({:.0}%) {}",
            category, cat_covered, cat_total, cat_percent, status
        );
    }

    if !report.uncovered_features.is_empty() {
        println!();
        println!("{}", "Uncovered Features:".red().bold());
        for feature in report.uncovered_features.iter().take(20) {
            println!("  - {}", feature);
        }
        if report.uncovered_features.len() > 20 {
            println!("  ... and {} more", report.uncovered_features.len() - 20);
        }
    }

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
        assert!(
            func_names.contains("oscillator"),
            "expected oscillator function"
        );
    }

    #[test]
    fn test_scan_starlark_usages() {
        let inventory = load_feature_inventory_from(Some(&project_root())).unwrap();
        let usages = scan_starlark_usages_from(Some(&project_root()), &inventory).unwrap();

        // Should find function usages in specs/**/*.star files
        assert!(
            !usages.function_usages.is_empty(),
            "expected some function usages"
        );

        // Should find oscillator usage (common in audio tests)
        let osc_usages = usages.function_usages.get("oscillator");
        assert!(osc_usages.is_some(), "expected oscillator usage");
        assert!(
            !osc_usages.unwrap().is_empty(),
            "expected oscillator examples"
        );

        // Should find enum usages
        assert!(
            !usages.enum_usages.is_empty(),
            "expected some enum value usages"
        );

        // Should find "sine" waveform usage (common in audio tests)
        let waveform_usages = usages.enum_usages.get("waveform");
        assert!(waveform_usages.is_some(), "expected waveform enum usages");
        let sine_usages = waveform_usages.unwrap().get("sine");
        assert!(sine_usages.is_some(), "expected sine waveform usage");
        assert!(!sine_usages.unwrap().is_empty(), "expected sine examples");
    }

    #[test]
    fn test_scan_json_spec_usages() {
        // JSON specs are no longer used - all specs are now Starlark
        let usages = scan_json_spec_usages_from(Some(&project_root())).unwrap();

        // Returns empty (legacy function kept for compatibility)
        assert!(usages.function_usages.is_empty());
        assert!(usages.recipe_usages.is_empty());
        assert!(usages.enum_usages.is_empty());
    }

    #[test]
    fn test_generate_coverage_report() {
        let report = generate_coverage_report_from(Some(&project_root())).unwrap();

        // Should have summary
        assert!(report.summary.total_features > 0, "expected some features");

        // Should have stdlib section
        assert!(!report.stdlib.is_empty(), "expected stdlib coverage");

        // Coverage should be a valid percentage
        assert!(report.summary.coverage_percent >= 0.0);
        assert!(report.summary.coverage_percent <= 100.0);
    }
}
