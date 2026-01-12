//! Generate-all command implementation
//!
//! Generates assets from all spec files in a directory structure.
//! Collects outputs and produces a summary report.

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use speccade_spec::{canonical_spec_hash, validate_for_generate, Spec};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use walkdir::WalkDir;

use crate::dispatch::{dispatch_generate, get_backend_tier, is_backend_available};

/// Asset types that require Blender (Tier 2 backends)
const BLENDER_ASSET_TYPES: &[&str] = &["static_mesh", "skeletal_mesh", "skeletal_animation"];

/// Asset types that use pure Rust backends (Tier 1)
const RUST_ASSET_TYPES: &[&str] = &["audio", "music", "texture"];

/// Result of generating a single spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecResult {
    /// Path to the spec file
    pub spec_path: String,
    /// Asset ID
    pub asset_id: String,
    /// Asset type
    pub asset_type: String,
    /// Recipe kind
    pub recipe_kind: Option<String>,
    /// Whether generation succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// BLAKE3 hash of the spec
    pub spec_hash: Option<String>,
    /// Output hashes (for Tier 1 backends)
    pub output_hashes: Vec<String>,
    /// Generation time in milliseconds
    pub duration_ms: u64,
    /// Backend tier (1 = deterministic, 2 = metric validation)
    pub backend_tier: Option<u8>,
}

/// Summary report for all generations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationSummary {
    /// Timestamp of generation
    pub timestamp: String,
    /// Total specs processed
    pub total_specs: usize,
    /// Successful generations
    pub successful: usize,
    /// Failed generations
    pub failed: usize,
    /// Skipped specs (e.g., Blender assets when not included)
    pub skipped: usize,
    /// Total runtime in seconds
    pub runtime_seconds: f64,
    /// Whether Blender assets were included
    pub include_blender: bool,
    /// Results for each spec
    pub specs: Vec<SpecResult>,
    /// Summary by asset type
    pub by_asset_type: HashMap<String, AssetTypeSummary>,
}

/// Summary for a single asset type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTypeSummary {
    /// Total specs of this type
    pub total: usize,
    /// Successful generations
    pub successful: usize,
    /// Failed generations
    pub failed: usize,
    /// Skipped specs
    pub skipped: usize,
}

/// Run the generate-all command
///
/// # Arguments
/// * `spec_dir` - Directory containing spec files (default: ./golden/speccade/specs)
/// * `out_root` - Output root directory (default: ./test-outputs)
/// * `include_blender` - Whether to include Blender-based assets
/// * `verbose` - Whether to show verbose output
///
/// # Returns
/// Exit code: 0 success, 1 if any specs failed
pub fn run(
    spec_dir: Option<&str>,
    out_root: Option<&str>,
    include_blender: bool,
    verbose: bool,
) -> Result<ExitCode> {
    let start = Instant::now();

    // Resolve paths
    let spec_dir = spec_dir.unwrap_or("./golden/speccade/specs");
    let out_root = out_root.unwrap_or("./test-outputs");

    let spec_path = Path::new(spec_dir);
    let out_path = Path::new(out_root);

    // Validate spec directory exists
    if !spec_path.exists() {
        anyhow::bail!("Spec directory does not exist: {}", spec_dir);
    }

    println!("{}", "======================================".cyan());
    println!("{}", "  SpecCade Golden Spec Generator".cyan());
    println!("{}", "======================================".cyan());
    println!();
    println!("{} {}", "Spec directory:".blue().bold(), spec_dir);
    println!("{} {}", "Output directory:".blue().bold(), out_root);
    println!(
        "{} {}",
        "Include Blender:".blue().bold(),
        if include_blender { "yes" } else { "no" }
    );
    println!();

    // Create output directory
    fs::create_dir_all(out_path)
        .with_context(|| format!("Failed to create output directory: {}", out_root))?;

    // Collect all spec files
    let mut spec_files: Vec<PathBuf> = Vec::new();
    let mut skipped_blender: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(spec_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json")
            && !path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().contains(".report."))
        {
            // Check if this is a Blender asset type
            let is_blender = BLENDER_ASSET_TYPES.iter().any(|t| {
                path.components()
                    .any(|c| c.as_os_str().to_string_lossy() == *t)
            });

            if is_blender && !include_blender {
                skipped_blender.push(path.to_path_buf());
            } else {
                spec_files.push(path.to_path_buf());
            }
        }
    }

    // Sort for deterministic order
    spec_files.sort();

    println!(
        "{} Found {} spec files to process",
        "INFO".blue().bold(),
        spec_files.len()
    );
    if !skipped_blender.is_empty() {
        println!(
            "{} Skipping {} Blender assets (use --include-blender to include)",
            "INFO".yellow().bold(),
            skipped_blender.len()
        );
    }
    println!();

    // Statistics
    let mut results: Vec<SpecResult> = Vec::new();
    let mut success_count = 0;
    let mut failure_count = 0;

    // Process each spec
    for spec_file in &spec_files {
        let result = process_spec(spec_file, out_path, verbose);

        if result.success {
            success_count += 1;
            if verbose {
                println!(
                    "  {} {} ({}ms)",
                    "SUCCESS".green(),
                    result.asset_id,
                    result.duration_ms
                );
            } else {
                print!("{}", ".".green());
            }
        } else {
            failure_count += 1;
            if verbose {
                println!(
                    "  {} {} - {}",
                    "FAILED".red(),
                    result.asset_id,
                    result.error.as_deref().unwrap_or("unknown error")
                );
            } else {
                print!("{}", "x".red());
            }
        }

        results.push(result);
    }

    if !verbose {
        println!(); // Newline after progress dots
    }

    // Build summary
    let elapsed = start.elapsed().as_secs_f64();
    let mut by_asset_type: HashMap<String, AssetTypeSummary> = HashMap::new();

    for result in &results {
        let entry = by_asset_type
            .entry(result.asset_type.clone())
            .or_insert(AssetTypeSummary {
                total: 0,
                successful: 0,
                failed: 0,
                skipped: 0,
            });
        entry.total += 1;
        if result.success {
            entry.successful += 1;
        } else {
            entry.failed += 1;
        }
    }

    // Add skipped Blender assets to summary
    for path in &skipped_blender {
        if let Some(asset_type) = extract_asset_type(path) {
            let entry = by_asset_type.entry(asset_type).or_insert(AssetTypeSummary {
                total: 0,
                successful: 0,
                failed: 0,
                skipped: 0,
            });
            entry.total += 1;
            entry.skipped += 1;
        }
    }

    let summary = GenerationSummary {
        timestamp: chrono_lite_timestamp(),
        total_specs: spec_files.len() + skipped_blender.len(),
        successful: success_count,
        failed: failure_count,
        skipped: skipped_blender.len(),
        runtime_seconds: elapsed,
        include_blender,
        specs: results.clone(),
        by_asset_type,
    };

    // Print summary
    println!();
    println!("{}", "======================================".cyan());
    println!("{}", "  Generation Summary".cyan());
    println!("{}", "======================================".cyan());
    println!();
    println!(
        "{} {}",
        "Total specs processed:".blue().bold(),
        summary.total_specs
    );
    println!("{} {}", "Successful:".green().bold(), summary.successful);
    println!("{} {}", "Failed:".red().bold(), summary.failed);
    println!("{} {}", "Skipped:".yellow().bold(), summary.skipped);
    println!(
        "{} {:.2}s",
        "Total runtime:".blue().bold(),
        summary.runtime_seconds
    );
    println!();

    // Print successful specs with hashes
    let with_hashes: Vec<_> = results
        .iter()
        .filter(|r| r.success && r.spec_hash.is_some())
        .collect();

    if !with_hashes.is_empty() {
        println!("{}", "Generated specs with BLAKE3 hashes:".green().bold());
        for result in with_hashes {
            let tier_str = result
                .backend_tier
                .map(|t| format!(" [Tier {}]", t))
                .unwrap_or_default();
            println!(
                "  {}/{}: {}{}",
                result.asset_type,
                result.asset_id,
                result.spec_hash.as_deref().unwrap_or("unknown"),
                tier_str.dimmed()
            );
        }
        println!();
    }

    // Print failed specs
    let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();
    if !failed.is_empty() {
        println!("{}", "Failed specs:".red().bold());
        for result in failed {
            println!(
                "  - {}/{}: {}",
                result.asset_type,
                result.asset_id,
                result.error.as_deref().unwrap_or("unknown error")
            );
        }
        println!();
    }

    // Write summary report
    let summary_path = out_path.join("generation_summary.json");
    let summary_json =
        serde_json::to_string_pretty(&summary).context("Failed to serialize summary")?;
    fs::write(&summary_path, summary_json)
        .with_context(|| format!("Failed to write summary: {}", summary_path.display()))?;

    println!("{} {}", "Outputs saved to:".blue().bold(), out_root);
    println!(
        "{} {}",
        "Summary report:".blue().bold(),
        summary_path.display()
    );

    if failure_count > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

/// Process a single spec file
fn process_spec(spec_path: &Path, out_root: &Path, _verbose: bool) -> SpecResult {
    let start = Instant::now();

    // Default result for errors
    let spec_path_str = spec_path.to_string_lossy().to_string();
    let asset_type = extract_asset_type(spec_path).unwrap_or_else(|| "unknown".to_string());

    let mut result = SpecResult {
        spec_path: spec_path_str.clone(),
        asset_id: spec_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        asset_type: asset_type.clone(),
        recipe_kind: None,
        success: false,
        error: None,
        spec_hash: None,
        output_hashes: Vec::new(),
        duration_ms: 0,
        backend_tier: None,
    };

    // Read and parse spec
    let spec_content = match fs::read_to_string(spec_path) {
        Ok(content) => content,
        Err(e) => {
            result.error = Some(format!("Failed to read spec file: {}", e));
            result.duration_ms = start.elapsed().as_millis() as u64;
            return result;
        }
    };

    let spec = match Spec::from_json(&spec_content) {
        Ok(s) => s,
        Err(e) => {
            result.error = Some(format!("Failed to parse spec: {}", e));
            result.duration_ms = start.elapsed().as_millis() as u64;
            return result;
        }
    };

    result.asset_id = spec.asset_id.clone();

    // Get recipe kind and backend tier
    if let Some(recipe) = &spec.recipe {
        result.recipe_kind = Some(recipe.kind.clone());
        result.backend_tier = get_backend_tier(&recipe.kind);

        // Check if backend is available
        if !is_backend_available(&recipe.kind) {
            result.error = Some(format!("Backend not available for: {}", recipe.kind));
            result.duration_ms = start.elapsed().as_millis() as u64;
            return result;
        }
    }

    // Compute spec hash
    result.spec_hash = canonical_spec_hash(&spec).ok();

    // Validate for generation
    let validation_result = validate_for_generate(&spec);
    if !validation_result.is_ok() {
        let errors: Vec<String> = validation_result
            .errors
            .iter()
            .map(|e| e.message.clone())
            .collect();
        result.error = Some(errors.join("; "));
        result.duration_ms = start.elapsed().as_millis() as u64;
        return result;
    }

    // Determine output directory structure based on number of outputs
    // Single output: {asset_type}/{spec_name}.{ext} (flat)
    // Multiple outputs: {asset_type}/{spec_name}/... (subdirectory)
    let output_count = spec.outputs.len();
    let spec_out_dir = if output_count == 1 {
        // Single output: write directly to asset type folder
        out_root.join(&asset_type)
    } else {
        // Multiple outputs: create subdirectory
        out_root.join(&asset_type).join(&result.asset_id)
    };

    if let Err(e) = fs::create_dir_all(&spec_out_dir) {
        result.error = Some(format!("Failed to create output directory: {}", e));
        result.duration_ms = start.elapsed().as_millis() as u64;
        return result;
    }

    // Dispatch generation
    match dispatch_generate(&spec, spec_out_dir.to_str().unwrap_or("."), spec_path) {
        Ok(outputs) => {
            result.success = true;
            result.output_hashes = outputs.iter().filter_map(|o| o.hash.clone()).collect();
        }
        Err(e) => {
            result.error = Some(format!("Generation failed: {}", e));
        }
    }

    result.duration_ms = start.elapsed().as_millis() as u64;
    result
}

/// Extract asset type from path
fn extract_asset_type(path: &Path) -> Option<String> {
    // Look for known asset types in path components
    let all_types: Vec<&str> = RUST_ASSET_TYPES
        .iter()
        .chain(BLENDER_ASSET_TYPES.iter())
        .copied()
        .collect();

    for component in path.components() {
        let name = component.as_os_str().to_string_lossy();
        if all_types.contains(&name.as_ref()) {
            return Some(name.to_string());
        }
    }
    None
}

/// Generate a UTC RFC3339 timestamp without external dependencies.
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seconds_since_epoch = now.as_secs() as i64;

    const SECS_PER_DAY: i64 = 86_400;

    // Split into date and time-of-day.
    let days = seconds_since_epoch.div_euclid(SECS_PER_DAY);
    let secs_of_day = seconds_since_epoch.rem_euclid(SECS_PER_DAY);
    let hours = secs_of_day / 3600;
    let minutes = (secs_of_day % 3600) / 60;
    let seconds = secs_of_day % 60;

    // Convert days since 1970-01-01 to YYYY-MM-DD using the proleptic Gregorian calendar.
    //
    // Based on Howard Hinnant's "civil_from_days" algorithm.
    fn civil_from_days(days: i64) -> (i32, u32, u32) {
        let z = days + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
        let doe = z - era * 146_097; // [0, 146096]
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096).div_euclid(365); // [0, 399]
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
        let mp = (5 * doy + 2).div_euclid(153); // [0, 11]
        let day = doy - (153 * mp + 2).div_euclid(5) + 1; // [1, 31]
        let month = mp + if mp < 10 { 3 } else { -9 }; // [1, 12]
        let year = y + if month <= 2 { 1 } else { 0 };
        (year as i32, month as u32, day as u32)
    }

    let (year, month, day) = civil_from_days(days);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_asset_type() {
        assert_eq!(
            extract_asset_type(Path::new("golden/specs/audio/beep.json")),
            Some("audio".to_string())
        );
        assert_eq!(
            extract_asset_type(Path::new("specs/texture/brick.json")),
            Some("texture".to_string())
        );
        assert_eq!(
            extract_asset_type(Path::new("specs/static_mesh/cube.json")),
            Some("static_mesh".to_string())
        );
        assert_eq!(extract_asset_type(Path::new("random/path/file.json")), None);
    }

    #[test]
    fn test_chrono_lite_timestamp() {
        let ts = chrono_lite_timestamp();
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
        assert_eq!(ts.len(), 20); // YYYY-MM-DDTHH:MM:SSZ

        // Quick sanity check on the date portion.
        let (date, _) = ts.split_once('T').unwrap();
        let parts: Vec<_> = date.split('-').collect();
        assert_eq!(parts.len(), 3);
        let month: u32 = parts[1].parse().unwrap();
        let day: u32 = parts[2].parse().unwrap();
        assert!((1..=12).contains(&month));
        assert!((1..=31).contains(&day));
    }

    #[test]
    fn test_blender_asset_types() {
        assert!(BLENDER_ASSET_TYPES.contains(&"static_mesh"));
        assert!(BLENDER_ASSET_TYPES.contains(&"skeletal_mesh"));
        assert!(BLENDER_ASSET_TYPES.contains(&"skeletal_animation"));
        assert!(!BLENDER_ASSET_TYPES.contains(&"audio"));
    }

    #[test]
    fn test_rust_asset_types() {
        assert!(RUST_ASSET_TYPES.contains(&"audio"));
        assert!(RUST_ASSET_TYPES.contains(&"texture"));
        assert!(!RUST_ASSET_TYPES.contains(&"static_mesh"));
    }
}
