# Feature Coverage System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a CLI-driven feature coverage system that auto-discovers all stdlib features, scans golden examples for usage, generates a machine-readable YAML report, and enforces 100% coverage in CI with no allowlist.

**Architecture:** Extends the existing CLI with a `coverage` subcommand that parses `stdlib.snapshot.json` for feature inventory, scans `golden/starlark/*.star` and `golden/speccade/specs/**/*.json` for usages, generates `docs/coverage/feature-coverage.yaml`, and provides CI enforcement via tests in `speccade-tests`.

**Tech Stack:** Rust CLI (clap for commands, serde_yaml for output, regex for scanning), existing `speccade-tests` crate for CI enforcement.

**Reference Design:** `docs/plans/2026-01-29-feature-coverage-system.md`

---

### Task 1: Add `Coverage` subcommand enum to CLI

**Files:**
- Modify: `crates/speccade-cli/src/main.rs` (add CoverageCommands enum and Coverage variant)

**Step 1: Write failing test for CLI parsing**

Add to the `#[cfg(test)] mod tests` block at the bottom of `main.rs`:

```rust
#[test]
fn test_cli_parses_coverage_generate() {
    let cli = Cli::try_parse_from([
        "speccade",
        "coverage",
        "generate",
    ])
    .unwrap();
    match cli.command {
        Commands::Coverage { command } => match command {
            CoverageCommands::Generate { strict, output } => {
                assert!(!strict);
                assert!(output.is_none());
            }
            _ => panic!("expected generate subcommand"),
        },
        _ => panic!("expected coverage command"),
    }
}

#[test]
fn test_cli_parses_coverage_generate_strict() {
    let cli = Cli::try_parse_from([
        "speccade",
        "coverage",
        "generate",
        "--strict",
        "--output",
        "custom.yaml",
    ])
    .unwrap();
    match cli.command {
        Commands::Coverage { command } => match command {
            CoverageCommands::Generate { strict, output } => {
                assert!(strict);
                assert_eq!(output.as_deref(), Some("custom.yaml"));
            }
            _ => panic!("expected generate subcommand"),
        },
        _ => panic!("expected coverage command"),
    }
}

#[test]
fn test_cli_parses_coverage_report() {
    let cli = Cli::try_parse_from([
        "speccade",
        "coverage",
        "report",
    ])
    .unwrap();
    match cli.command {
        Commands::Coverage { command } => match command {
            CoverageCommands::Report => {}
            _ => panic!("expected report subcommand"),
        },
        _ => panic!("expected coverage command"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-cli --bin speccade test_cli_parses_coverage -- --nocapture`
Expected: FAIL - CoverageCommands and Coverage variant not defined

**Step 3: Add CoverageCommands enum after CacheCommands**

In `main.rs`, find `CacheCommands` enum (around line 434) and add after it:

```rust
/// Subcommands for feature coverage tracking
#[derive(Subcommand, Debug)]
pub enum CoverageCommands {
    /// Generate coverage report (writes YAML)
    Generate {
        /// Fail if coverage < 100% (CI mode)
        #[arg(long)]
        strict: bool,

        /// Output YAML path (default: docs/coverage/feature-coverage.yaml)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Print coverage summary to stdout
    Report,
}
```

**Step 4: Add Coverage variant to Commands enum**

Find the `Commands` enum and add after the `Cache` variant:

```rust
    /// Feature coverage tracking and enforcement
    Coverage {
        #[command(subcommand)]
        command: CoverageCommands,
    },
```

**Step 5: Run tests to verify they pass**

Run: `cargo test -p speccade-cli --bin speccade test_cli_parses_coverage -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/speccade-cli/src/main.rs
git commit -m "feat(cli): add coverage command enum and parsing"
```

---

### Task 2: Create coverage module with stub implementation

**Files:**
- Create: `crates/speccade-cli/src/commands/coverage.rs`
- Modify: `crates/speccade-cli/src/commands/mod.rs`

**Step 1: Add module declaration to mod.rs**

In `crates/speccade-cli/src/commands/mod.rs`, add after the last `pub mod` line:

```rust
pub mod coverage;
```

Also add to the test block (if one exists):

```rust
let _ = coverage::run_generate;
let _ = coverage::run_report;
```

**Step 2: Create coverage.rs with stubs**

Create `crates/speccade-cli/src/commands/coverage.rs`:

```rust
//! Feature coverage tracking command implementation
//!
//! Generates coverage reports showing which stdlib features have golden examples.

use anyhow::Result;
use std::process::ExitCode;

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
```

**Step 3: Add dispatch in main.rs**

Find the match block for `Commands` and add after `Commands::Cache`:

```rust
        Commands::Coverage { command } => match command {
            CoverageCommands::Generate { strict, output } => {
                commands::coverage::run_generate(strict, output.as_deref())
            }
            CoverageCommands::Report => commands::coverage::run_report(),
        },
```

**Step 4: Verify compilation**

Run: `cargo check -p speccade-cli`
Expected: Compiles without errors

**Step 5: Test CLI help**

Run: `cargo run -p speccade-cli -- coverage --help`
Expected: Shows coverage subcommands (generate, report)

Run: `cargo run -p speccade-cli -- coverage generate --help`
Expected: Shows generate options (--strict, --output)

**Step 6: Commit**

```bash
git add crates/speccade-cli/src/commands/coverage.rs crates/speccade-cli/src/commands/mod.rs crates/speccade-cli/src/main.rs
git commit -m "feat(cli): add coverage command stub implementation"
```

---

### Task 3: Implement feature inventory loading from stdlib.snapshot.json

**Files:**
- Modify: `crates/speccade-cli/src/commands/coverage.rs`

**Step 1: Write failing test for feature inventory**

Add to the bottom of `coverage.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_feature_inventory() {
        let inventory = load_feature_inventory().unwrap();

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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-cli --lib load_feature_inventory -- --nocapture`
Expected: FAIL - function not defined

**Step 3: Add types and implement load_feature_inventory**

Add above the stubs in `coverage.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
pub fn load_feature_inventory() -> Result<FeatureInventory> {
    let snapshot_path = Path::new("golden/stdlib/stdlib.snapshot.json");

    let content = fs::read_to_string(snapshot_path)
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
```

**Step 4: Add anyhow import**

Add at the top of the file:

```rust
use anyhow::{Context, Result};
```

**Step 5: Run tests to verify they pass**

Run: `cargo test -p speccade-cli --lib load_feature_inventory -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/speccade-cli/src/commands/coverage.rs
git commit -m "feat(coverage): implement feature inventory loading from stdlib snapshot"
```

---

### Task 4: Implement Starlark file scanning for function usages

**Files:**
- Modify: `crates/speccade-cli/src/commands/coverage.rs`

**Step 1: Write failing test for Starlark scanning**

Add to the tests module:

```rust
#[test]
fn test_scan_starlark_usages() {
    let usages = scan_starlark_usages().unwrap();

    // Should find usages in golden/starlark files
    assert!(!usages.is_empty(), "expected some usages");

    // Should find oscillator usage (common in audio tests)
    let osc_usages = usages.get("oscillator");
    assert!(osc_usages.is_some(), "expected oscillator usage");
    assert!(!osc_usages.unwrap().is_empty(), "expected oscillator examples");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-cli --lib scan_starlark_usages -- --nocapture`
Expected: FAIL - function not defined

**Step 3: Implement scan_starlark_usages**

Add after `load_feature_inventory`:

```rust
use glob::glob;
use regex::Regex;

/// A location where a feature is used
#[derive(Debug, Clone, Serialize)]
pub struct UsageLocation {
    pub file: String,
    pub line: Option<u32>,
}

/// Scan golden/starlark/*.star files for function usages
pub fn scan_starlark_usages() -> Result<HashMap<String, Vec<UsageLocation>>> {
    let mut usages: HashMap<String, Vec<UsageLocation>> = HashMap::new();

    let pattern = "golden/starlark/**/*.star";
    let entries: Vec<_> = glob(pattern)
        .with_context(|| format!("Invalid glob pattern: {}", pattern))?
        .filter_map(|e| e.ok())
        .collect();

    for path in entries {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let file_str = path.to_string_lossy().replace('\\', "/");

        for (line_num, line) in content.lines().enumerate() {
            // Skip comment lines
            if line.trim().starts_with('#') {
                continue;
            }

            // Find function calls: word boundary + function name + optional whitespace + (
            // Use a regex that captures the function name
            let call_re = Regex::new(r"\b([a-z_][a-z0-9_]*)\s*\(")
                .expect("valid regex");

            for cap in call_re.captures_iter(line) {
                let func_name = cap.get(1).unwrap().as_str().to_string();

                let location = UsageLocation {
                    file: file_str.clone(),
                    line: Some((line_num + 1) as u32),
                };

                usages.entry(func_name).or_default().push(location);
            }
        }
    }

    Ok(usages)
}
```

**Step 4: Add glob and regex to Cargo.toml**

In `crates/speccade-cli/Cargo.toml`, add to `[dependencies]`:

```toml
glob = "0.3"
regex = "1"
```

**Step 5: Run tests to verify they pass**

Run: `cargo test -p speccade-cli --lib scan_starlark_usages -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/speccade-cli/src/commands/coverage.rs crates/speccade-cli/Cargo.toml
git commit -m "feat(coverage): implement Starlark file scanning for function usages"
```

---

### Task 5: Implement JSON spec scanning for recipe features

**Files:**
- Modify: `crates/speccade-cli/src/commands/coverage.rs`

**Step 1: Write failing test for JSON spec scanning**

Add to the tests module:

```rust
#[test]
fn test_scan_json_spec_usages() {
    let usages = scan_json_spec_usages().unwrap();

    // Should find usages in golden/speccade/specs files
    assert!(!usages.function_usages.is_empty() || !usages.recipe_usages.is_empty(),
        "expected some usages from JSON specs");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-cli --lib scan_json_spec_usages -- --nocapture`
Expected: FAIL - function not defined

**Step 3: Implement scan_json_spec_usages**

Add after `scan_starlark_usages`:

```rust
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

/// Scan golden/speccade/specs/**/*.json for recipe features
pub fn scan_json_spec_usages() -> Result<JsonSpecUsages> {
    let mut result = JsonSpecUsages::default();

    let pattern = "golden/speccade/specs/**/*.json";
    let entries: Vec<_> = glob(pattern)
        .with_context(|| format!("Invalid glob pattern: {}", pattern))?
        .filter_map(|e| e.ok())
        .collect();

    for path in entries {
        // Skip report files and non-spec files
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if filename.contains("report") || filename.contains("hash") {
            continue;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let file_str = path.to_string_lossy().replace('\\', "/");

        // Parse as JSON
        let spec: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue, // Skip invalid JSON
        };

        let location = UsageLocation {
            file: file_str.clone(),
            line: None,
        };

        // Extract recipe.kind if present
        if let Some(recipe) = spec.get("recipe") {
            if let Some(kind) = recipe.get("kind").and_then(|k| k.as_str()) {
                result.recipe_usages
                    .entry(kind.to_string())
                    .or_default()
                    .push(location.clone());

                // Scan recipe.params for enum values
                if let Some(params) = recipe.get("params").and_then(|p| p.as_object()) {
                    scan_params_for_enums(params, &location, &mut result.enum_usages);
                }
            }
        }
    }

    Ok(result)
}

/// Recursively scan params object for string values that might be enum values
fn scan_params_for_enums(
    params: &serde_json::Map<String, serde_json::Value>,
    location: &UsageLocation,
    enum_usages: &mut HashMap<String, HashMap<String, Vec<UsageLocation>>>,
) {
    for (key, value) in params {
        match value {
            serde_json::Value::String(s) => {
                // Record string values keyed by parameter name
                enum_usages
                    .entry(key.clone())
                    .or_default()
                    .entry(s.clone())
                    .or_default()
                    .push(location.clone());
            }
            serde_json::Value::Object(nested) => {
                scan_params_for_enums(nested, location, enum_usages);
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    if let serde_json::Value::Object(nested) = item {
                        scan_params_for_enums(nested, location, enum_usages);
                    }
                }
            }
            _ => {}
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p speccade-cli --lib scan_json_spec_usages -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/speccade-cli/src/commands/coverage.rs
git commit -m "feat(coverage): implement JSON spec scanning for recipe features"
```

---

### Task 6: Implement coverage report generation

**Files:**
- Modify: `crates/speccade-cli/src/commands/coverage.rs`

**Step 1: Write failing test for coverage report generation**

Add to the tests module:

```rust
#[test]
fn test_generate_coverage_report() {
    let report = generate_coverage_report().unwrap();

    // Should have summary
    assert!(report.summary.total_features > 0, "expected some features");

    // Should have stdlib section
    assert!(!report.stdlib.is_empty(), "expected stdlib coverage");

    // Coverage should be a valid percentage
    assert!(report.summary.coverage_percent >= 0.0);
    assert!(report.summary.coverage_percent <= 100.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-cli --lib generate_coverage_report -- --nocapture`
Expected: FAIL - function not defined

**Step 3: Define coverage report types**

Add after the scanning functions:

```rust
use chrono::Utc;

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
    #[serde(rename = "# AUTO-GENERATED - Do not edit manually")]
    _comment: String,
    pub schema_version: u32,
    pub generated_at: String,
    pub summary: CoverageSummary,
    /// Map of category -> functions
    pub stdlib: HashMap<String, Vec<FunctionCoverage>>,
    /// Features with no coverage
    pub uncovered_features: Vec<String>,
}
```

**Step 4: Implement generate_coverage_report**

Add after the types:

```rust
/// Generate the full coverage report
pub fn generate_coverage_report() -> Result<CoverageReport> {
    // Load inventory and scan usages
    let inventory = load_feature_inventory()?;
    let starlark_usages = scan_starlark_usages()?;
    let json_usages = scan_json_spec_usages()?;

    let mut stdlib: HashMap<String, Vec<FunctionCoverage>> = HashMap::new();
    let mut uncovered_features: Vec<String> = Vec::new();
    let mut total_features = 0u32;
    let mut covered = 0u32;

    // Process each function
    for func in &inventory.functions {
        total_features += 1;

        // Find examples from both Starlark and JSON
        let mut examples: Vec<String> = Vec::new();

        if let Some(usages) = starlark_usages.get(&func.name) {
            for usage in usages.iter().take(3) { // Limit to 3 examples
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

                    // Check if this enum value is used
                    let example = find_enum_value_example(
                        &param.name,
                        value,
                        &starlark_usages,
                        &json_usages,
                    );

                    let value_is_covered = example.is_some();
                    if value_is_covered {
                        covered += 1;
                    } else {
                        uncovered_features.push(format!("enum:{}::{}", param.name, value));
                    }

                    value_coverage.insert(value.clone(), EnumValueCoverage {
                        covered: value_is_covered,
                        example,
                    });
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
        _comment: "Regenerate with: speccade coverage generate".to_string(),
        schema_version: 1,
        generated_at: Utc::now().to_rfc3339(),
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

/// Find an example of an enum value being used
fn find_enum_value_example(
    param_name: &str,
    value: &str,
    starlark_usages: &HashMap<String, Vec<UsageLocation>>,
    json_usages: &JsonSpecUsages,
) -> Option<String> {
    // Check JSON spec usages first (more reliable)
    if let Some(param_values) = json_usages.enum_usages.get(param_name) {
        if let Some(locations) = param_values.get(value) {
            if let Some(loc) = locations.first() {
                return Some(loc.file.clone());
            }
        }
    }

    // Check Starlark for the value string (less reliable but catches direct usage)
    // This is a heuristic - we look for the value as a string literal
    for (_, usages) in starlark_usages {
        for usage in usages {
            // Crude check: if file contains this value, consider it covered
            // This could be improved with proper Starlark parsing
            if let Ok(content) = fs::read_to_string(&usage.file.replace('/', std::path::MAIN_SEPARATOR_STR)) {
                if content.contains(&format!("\"{}\"", value)) || content.contains(&format!("'{}'", value)) {
                    return Some(format!("{}:{}", usage.file, usage.line.unwrap_or(0)));
                }
            }
        }
    }

    None
}
```

**Step 5: Add chrono to Cargo.toml**

In `crates/speccade-cli/Cargo.toml`, add to `[dependencies]`:

```toml
chrono = { version = "0.4", features = ["serde"] }
```

**Step 6: Run tests to verify they pass**

Run: `cargo test -p speccade-cli --lib generate_coverage_report -- --nocapture`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/speccade-cli/src/commands/coverage.rs crates/speccade-cli/Cargo.toml
git commit -m "feat(coverage): implement coverage report generation"
```

---

### Task 7: Wire up run_generate and run_report

**Files:**
- Modify: `crates/speccade-cli/src/commands/coverage.rs`

**Step 1: Implement run_generate**

Replace the stub `run_generate` with:

```rust
use colored::Colorize;

/// Run the coverage generate subcommand
pub fn run_generate(strict: bool, output: Option<&str>) -> Result<ExitCode> {
    let output_path = output.unwrap_or("docs/coverage/feature-coverage.yaml");

    println!("{}", "Generating feature coverage report...".cyan().bold());

    let report = generate_coverage_report()?;

    // Create output directory if needed
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Write YAML
    let yaml = serde_yaml::to_string(&report)?;
    fs::write(output_path, &yaml)?;

    println!("{} Coverage report written to: {}", "OK".green().bold(), output_path);
    println!();
    print_summary(&report.summary);

    if !report.uncovered_features.is_empty() {
        println!();
        println!("{}", "Uncovered features:".red().bold());
        for feature in &report.uncovered_features {
            println!("  - {}", feature);
        }
    }

    if strict && report.summary.uncovered > 0 {
        println!();
        println!(
            "{} {} features have no golden examples.",
            "BLOCKING:".red().bold(),
            report.summary.uncovered
        );
        println!("Create examples in golden/starlark/ or golden/speccade/specs/.");
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}

fn print_summary(summary: &CoverageSummary) {
    println!("{}", "Coverage Summary".bold());
    println!("  Total features: {}", summary.total_features);
    println!("  Covered: {} ({:.1}%)", summary.covered, summary.coverage_percent);
    println!("  Uncovered: {}", summary.uncovered);
}
```

**Step 2: Implement run_report**

Replace the stub `run_report` with:

```rust
/// Run the coverage report subcommand
pub fn run_report() -> Result<ExitCode> {
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
            category,
            cat_covered,
            cat_total,
            cat_percent,
            status
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
```

**Step 3: Add serde_yaml to Cargo.toml**

In `crates/speccade-cli/Cargo.toml`, add to `[dependencies]`:

```toml
serde_yaml = "0.9"
```

**Step 4: Verify compilation**

Run: `cargo check -p speccade-cli`
Expected: Compiles without errors

**Step 5: Test the commands manually**

Run: `cargo run -p speccade-cli -- coverage report`
Expected: Prints coverage summary by category

Run: `cargo run -p speccade-cli -- coverage generate`
Expected: Creates docs/coverage/feature-coverage.yaml

**Step 6: Commit**

```bash
git add crates/speccade-cli/src/commands/coverage.rs crates/speccade-cli/Cargo.toml
git commit -m "feat(coverage): implement generate and report commands"
```

---

### Task 8: Create CI enforcement test

**Files:**
- Create: `crates/speccade-tests/tests/feature_coverage.rs`

**Step 1: Create the test file**

Create `crates/speccade-tests/tests/feature_coverage.rs`:

```rust
//! Feature coverage enforcement tests
//!
//! Policy: 100% coverage required. No allowlist. No exceptions.
//! If a feature exists, it MUST have a golden example.

use std::process::Command;

/// Run `speccade coverage generate --strict` and expect success
#[test]
fn coverage_is_complete() {
    let output = Command::new("cargo")
        .args(["run", "-p", "speccade-cli", "--", "coverage", "generate", "--strict"])
        .output()
        .expect("Failed to run coverage command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        panic!(
            "Coverage check failed!\n\nstdout:\n{}\n\nstderr:\n{}\n\n\
             Run `cargo run -p speccade-cli -- coverage report` to see uncovered features.\n\
             Create examples in golden/starlark/ or golden/speccade/specs/.",
            stdout,
            stderr
        );
    }
}

/// Verify the coverage YAML file exists and is valid
#[test]
fn coverage_yaml_exists() {
    let yaml_path = "docs/coverage/feature-coverage.yaml";

    assert!(
        std::path::Path::new(yaml_path).exists(),
        "Coverage YAML not found at {}. Run `speccade coverage generate` first.",
        yaml_path
    );

    let content = std::fs::read_to_string(yaml_path)
        .expect("Failed to read coverage YAML");

    // Verify it's valid YAML
    let _: serde_yaml::Value = serde_yaml::from_str(&content)
        .expect("Coverage YAML is not valid YAML");

    // Verify it has required fields
    assert!(content.contains("schema_version:"), "Missing schema_version");
    assert!(content.contains("summary:"), "Missing summary");
    assert!(content.contains("total_features:"), "Missing total_features");
}
```

**Step 2: Add serde_yaml dev dependency to speccade-tests**

In `crates/speccade-tests/Cargo.toml`, add to `[dev-dependencies]`:

```toml
serde_yaml = "0.9"
```

**Step 3: Run the test (expected to fail initially)**

Run: `cargo test -p speccade-tests --test feature_coverage -- --nocapture`
Expected: May fail if coverage is not 100% yet - that's OK, the test is working

**Step 4: Commit**

```bash
git add crates/speccade-tests/tests/feature_coverage.rs crates/speccade-tests/Cargo.toml
git commit -m "test(coverage): add CI enforcement tests for feature coverage"
```

---

### Task 9: Remove ALLOWLIST from stdlib_coverage.rs

**Files:**
- Modify: `crates/speccade-tests/tests/stdlib_coverage.rs`

**Step 1: Read current file to understand structure**

Read the file to locate the ALLOWLIST and understand the test structure.

**Step 2: Remove the ALLOWLIST constant**

Find and remove:

```rust
const ALLOWLIST: &[&str] = &[
    // The mesh_primitive function is a lower-level building block; mesh_recipe
    // is preferred in golden tests as it provides a more complete API.
    "mesh_primitive",
];
```

**Step 3: Update all_stdlib_functions_are_covered test**

Replace the allowlist filtering with direct assertion. Find the part that filters by allowlist and remove it:

```rust
// BEFORE:
let uncovered: Vec<_> = all_functions
    .iter()
    .filter(|f| !called_functions.contains(f.as_str()))
    .filter(|f| !ALLOWLIST.contains(&f.as_str()))
    .collect();

// AFTER:
let uncovered: Vec<_> = all_functions
    .iter()
    .filter(|f| !called_functions.contains(f.as_str()))
    .collect();
```

**Step 4: Remove the allowlist_functions_exist test**

Delete the entire test since there's no more allowlist to validate.

**Step 5: Update test documentation**

Update the module doc comment to reflect the no-exceptions policy:

```rust
//! Stdlib coverage tests
//!
//! Policy: 100% coverage required. No allowlist. No exceptions.
//! Every stdlib function must have at least one golden example.
```

**Step 6: Run the test to see what's uncovered**

Run: `cargo test -p speccade-tests --test stdlib_coverage -- --nocapture`
Expected: Will show what functions need examples (including mesh_primitive)

**Step 7: Commit**

```bash
git add crates/speccade-tests/tests/stdlib_coverage.rs
git commit -m "refactor(tests): remove ALLOWLIST from stdlib_coverage - enforce 100%"
```

---

### Task 10: Create golden example for mesh_primitive

**Files:**
- Create: `golden/starlark/mesh_primitives.star`

**Step 1: Create the example file**

Create `golden/starlark/mesh_primitives.star`:

```python
# Mesh primitives example - demonstrates all mesh_primitive variants
#
# This file ensures 100% coverage of the mesh_primitive function
# and its supported primitive types.

# Cube primitive
cube = mesh_primitive(
    primitive = "cube",
    size = [1.0, 1.0, 1.0],
)

# Sphere primitive
sphere = mesh_primitive(
    primitive = "sphere",
    radius = 0.5,
    segments = 32,
)

# Cylinder primitive
cylinder = mesh_primitive(
    primitive = "cylinder",
    radius = 0.3,
    height = 1.0,
    segments = 24,
)

# Cone primitive
cone = mesh_primitive(
    primitive = "cone",
    radius = 0.4,
    height = 0.8,
    segments = 24,
)

# Torus primitive
torus = mesh_primitive(
    primitive = "torus",
    major_radius = 0.5,
    minor_radius = 0.15,
    major_segments = 48,
    minor_segments = 12,
)

# Plane primitive
plane = mesh_primitive(
    primitive = "plane",
    size = [2.0, 2.0],
)

# Ico sphere primitive
ico_sphere = mesh_primitive(
    primitive = "ico_sphere",
    radius = 0.5,
    subdivisions = 2,
)
```

**Step 2: Verify the file parses correctly**

Run: `cargo run -p speccade-cli -- eval --spec golden/starlark/mesh_primitives.star --pretty | head -30`
Expected: Valid JSON output (or error if mesh_primitive isn't a real function - adjust accordingly)

**Step 3: Run stdlib coverage to verify coverage**

Run: `cargo test -p speccade-tests --test stdlib_coverage all_stdlib_functions_are_covered -- --nocapture`
Expected: mesh_primitive should now be covered

**Step 4: Commit**

```bash
git add golden/starlark/mesh_primitives.star
git commit -m "test(golden): add mesh_primitives.star for 100% stdlib coverage"
```

---

### Task 11: Update CI workflow

**Files:**
- Modify: `.github/workflows/ci.yml`

**Step 1: Add coverage step to the test job**

Find the `test` job in `.github/workflows/ci.yml` and add after the existing test step:

```yaml
      - name: Generate coverage report
        run: cargo run -p speccade-cli -- coverage generate

      - name: Feature coverage check
        run: cargo test -p speccade-tests --test feature_coverage -- --nocapture
```

**Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add feature coverage check to CI workflow"
```

---

### Task 12: Create docs/coverage directory and initial YAML

**Files:**
- Create: `docs/coverage/.gitkeep` (placeholder for directory)
- Generate: `docs/coverage/feature-coverage.yaml`

**Step 1: Create the directory**

Run: `mkdir -p docs/coverage`

**Step 2: Generate the initial coverage YAML**

Run: `cargo run -p speccade-cli -- coverage generate`
Expected: Creates docs/coverage/feature-coverage.yaml

**Step 3: Verify the YAML content**

Run: `head -30 docs/coverage/feature-coverage.yaml`
Expected: Shows valid YAML with schema_version, summary, etc.

**Step 4: Commit**

```bash
git add docs/coverage/feature-coverage.yaml
git commit -m "docs: add generated feature coverage report"
```

---

### Task 13: Final integration verification

**Files:** None (verification only)

**Step 1: Run all coverage-related tests**

Run: `cargo test -p speccade-cli --lib -- coverage --nocapture`
Expected: All unit tests pass

Run: `cargo test -p speccade-tests --test stdlib_coverage -- --nocapture`
Expected: All functions covered (no uncovered)

Run: `cargo test -p speccade-tests --test feature_coverage -- --nocapture`
Expected: Coverage is complete

**Step 2: Test strict mode with deliberate failure**

Create a temporary test by adding a fake function to the inventory, then verify the test fails.

(Skip this step if you want to avoid touching production code)

**Step 3: Verify CLI commands work**

Run: `cargo run -p speccade-cli -- coverage --help`
Expected: Shows subcommands

Run: `cargo run -p speccade-cli -- coverage report`
Expected: Shows coverage report with categories

Run: `cargo run -p speccade-cli -- coverage generate --strict`
Expected: Exits 0 if 100% coverage, 1 otherwise

**Step 4: Final commit (if any remaining changes)**

```bash
git status
# If there are unstaged changes from verification, commit them
```

---

## Success Criteria

After completing all tasks:

- [ ] `speccade coverage generate` produces valid YAML at `docs/coverage/feature-coverage.yaml`
- [ ] `speccade coverage generate --strict` exits 0 when 100% coverage, 1 otherwise
- [ ] `speccade coverage report` shows human-readable output by category
- [ ] `cargo test -p speccade-tests --test feature_coverage` passes
- [ ] `cargo test -p speccade-tests --test stdlib_coverage` passes with no ALLOWLIST
- [ ] CI workflow includes coverage check step
- [ ] All stdlib functions have golden examples (including mesh_primitive)
- [ ] Coverage YAML is committed and CI verifies it matches fresh generation
