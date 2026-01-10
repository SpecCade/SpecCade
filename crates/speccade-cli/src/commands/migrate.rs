//! Migrate command implementation
//!
//! Migrates legacy .spec.py files to canonical JSON format.

use anyhow::{Context, Result, bail};
use colored::Colorize;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use walkdir::WalkDir;

use speccade_spec::{AssetType, OutputFormat, OutputKind, OutputSpec, Spec};

use crate::parity_data::{self, KeyStatus};

/// Migration key status for classification
/// Maps to parity_data::KeyStatus but adds Unknown for keys not in parity matrix
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MigrationKeyStatus {
    Implemented,
    Partial,
    NotImplemented,
    Deprecated,
    Unknown,
}

impl From<KeyStatus> for MigrationKeyStatus {
    fn from(status: KeyStatus) -> Self {
        match status {
            KeyStatus::Implemented => MigrationKeyStatus::Implemented,
            KeyStatus::Partial => MigrationKeyStatus::Partial,
            KeyStatus::NotImplemented => MigrationKeyStatus::NotImplemented,
            KeyStatus::Deprecated => MigrationKeyStatus::Deprecated,
        }
    }
}

impl std::fmt::Display for MigrationKeyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationKeyStatus::Implemented => write!(f, "Implemented"),
            MigrationKeyStatus::Partial => write!(f, "Partial"),
            MigrationKeyStatus::NotImplemented => write!(f, "NotImplemented"),
            MigrationKeyStatus::Deprecated => write!(f, "Deprecated"),
            MigrationKeyStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Key classification results for a migrated file
#[derive(Debug, Default, Clone)]
struct KeyClassification {
    implemented: usize,
    partial: usize,
    not_implemented: usize,
    deprecated: usize,
    unknown: usize,
    /// Details for each key: (key_name, status)
    key_details: Vec<(String, MigrationKeyStatus)>,
}

impl KeyClassification {
    /// Total number of keys used (excluding deprecated)
    fn total_used(&self) -> usize {
        self.implemented + self.partial + self.not_implemented + self.unknown
    }

    /// Compute gap score: (implemented + 0.5*partial) / (total_used)
    /// Returns None if total_used is 0
    fn gap_score(&self) -> Option<f64> {
        let total = self.total_used();
        if total == 0 {
            return None;
        }
        let score = (self.implemented as f64 + 0.5 * self.partial as f64) / total as f64;
        Some(score)
    }
}

/// Migration report entry
#[derive(Debug)]
struct MigrationEntry {
    source_path: PathBuf,
    target_path: Option<PathBuf>,
    success: bool,
    warnings: Vec<String>,
    error: Option<String>,
    /// Key classification for parity analysis
    key_classification: KeyClassification,
}

/// Legacy spec data extracted from .spec.py
#[derive(Debug)]
struct LegacySpec {
    dict_name: String,
    category: String,
    data: HashMap<String, serde_json::Value>,
}

/// Run the migrate command
///
/// # Arguments
/// * `project_path` - Path to the project directory containing legacy specs
/// * `allow_exec_specs` - Whether to allow Python execution for parsing
///
/// # Returns
/// Exit code: 0 success, 1 error
pub fn run(project_path: &str, allow_exec_specs: bool) -> Result<ExitCode> {
    println!(
        "{} {}",
        "Migrate project:".cyan().bold(),
        project_path
    );

    if allow_exec_specs {
        println!(
            "{} {}",
            "WARNING:".yellow().bold(),
            "Python execution enabled. Only use with trusted files!".yellow()
        );
    }

    // Check if project path exists
    let path = Path::new(project_path);
    if !path.exists() {
        bail!("Project directory does not exist: {}", project_path);
    }

    if !path.is_dir() {
        bail!("Path is not a directory: {}", project_path);
    }

    // Find legacy specs
    let specs_dir = path.join(".studio").join("specs");
    if !specs_dir.exists() {
        bail!(
            "No .studio/specs directory found in project: {}",
            project_path
        );
    }

    println!(
        "\n{} {}",
        "Scanning:".cyan(),
        specs_dir.display()
    );

    let spec_files = find_legacy_specs(&specs_dir)?;

    if spec_files.is_empty() {
        println!("{} No legacy .spec.py files found.", "INFO".yellow().bold());
        return Ok(ExitCode::SUCCESS);
    }

    println!(
        "{} {} legacy spec files found\n",
        "Found:".cyan(),
        spec_files.len()
    );

    // Migrate each spec
    let mut entries = Vec::new();

    for spec_file in &spec_files {
        let entry = match migrate_spec(spec_file, path, allow_exec_specs) {
            Ok(entry) => entry,
            Err(e) => MigrationEntry {
                source_path: spec_file.to_path_buf(),
                target_path: None,
                success: false,
                warnings: Vec::new(),
                error: Some(e.to_string()),
                key_classification: KeyClassification::default(),
            },
        };

        // Print progress
        if entry.success && !entry.warnings.is_empty() {
            print!("{} ", "⚠".yellow());
        } else if entry.success {
            print!("{} ", "✓".green());
        } else {
            print!("{} ", "✗".red());
        }

        entries.push(entry);
    }

    println!("\n");

    // Generate report
    print_migration_report(&entries);

    // Return success if any files were converted
    let success_count = entries.iter().filter(|e| e.success).count();
    if success_count > 0 {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

/// Audit result for a single spec file
#[derive(Debug)]
struct AuditEntry {
    source_path: PathBuf,
    success: bool,
    error: Option<String>,
    key_classification: KeyClassification,
}

/// Run the audit command (--audit mode)
///
/// Scans specs, parses them, collects legacy keys, and reports aggregate completeness.
///
/// # Arguments
/// * `project_path` - Path to the project directory containing legacy specs
/// * `allow_exec_specs` - Whether to allow Python execution for parsing
/// * `threshold` - Minimum completeness threshold (0.0-1.0)
///
/// # Returns
/// Exit code: 0 if completeness >= threshold, 1 otherwise
pub fn run_audit(project_path: &str, allow_exec_specs: bool, threshold: f64) -> Result<ExitCode> {
    println!(
        "{} {}",
        "Audit project:".cyan().bold(),
        project_path
    );
    println!(
        "{} {:.0}%",
        "Threshold:".cyan(),
        threshold * 100.0
    );

    if allow_exec_specs {
        println!(
            "{} {}",
            "WARNING:".yellow().bold(),
            "Python execution enabled. Only use with trusted files!".yellow()
        );
    }

    // Check if project path exists
    let path = Path::new(project_path);
    if !path.exists() {
        bail!("Project directory does not exist: {}", project_path);
    }

    if !path.is_dir() {
        bail!("Path is not a directory: {}", project_path);
    }

    // Find legacy specs
    let specs_dir = path.join(".studio").join("specs");
    if !specs_dir.exists() {
        bail!(
            "No .studio/specs directory found in project: {}",
            project_path
        );
    }

    println!(
        "\n{} {}",
        "Scanning:".cyan(),
        specs_dir.display()
    );

    let spec_files = find_legacy_specs(&specs_dir)?;

    if spec_files.is_empty() {
        println!("{} No legacy .spec.py files found.", "INFO".yellow().bold());
        return Ok(ExitCode::SUCCESS);
    }

    println!(
        "{} {} legacy spec files found\n",
        "Found:".cyan(),
        spec_files.len()
    );

    // Audit each spec (parse and classify keys, but don't migrate)
    let mut entries = Vec::new();
    let mut parse_errors = 0;

    for spec_file in &spec_files {
        let entry = match audit_spec(spec_file, allow_exec_specs) {
            Ok(entry) => entry,
            Err(e) => {
                parse_errors += 1;
                AuditEntry {
                    source_path: spec_file.to_path_buf(),
                    success: false,
                    error: Some(e.to_string()),
                    key_classification: KeyClassification::default(),
                }
            }
        };

        // Print progress
        if entry.success {
            print!("{} ", ".".dimmed());
        } else {
            print!("{} ", "!".red());
        }

        entries.push(entry);
    }

    println!("\n");

    // Generate audit report
    let completeness = print_audit_report(&entries, threshold);

    // Return appropriate exit code
    if parse_errors > 0 {
        // Hard failure: I/O or parse errors
        Ok(ExitCode::from(1))
    } else if completeness >= threshold {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

/// Audit a single spec file (parse and classify keys without migrating)
fn audit_spec(spec_file: &Path, allow_exec: bool) -> Result<AuditEntry> {
    // Parse the legacy spec
    let legacy = if allow_exec {
        parse_legacy_spec_exec(spec_file)?
    } else {
        parse_legacy_spec_static(spec_file)?
    };

    // Classify legacy keys against parity matrix
    let key_classification = classify_legacy_keys(&legacy);

    Ok(AuditEntry {
        source_path: spec_file.to_path_buf(),
        success: true,
        error: None,
        key_classification,
    })
}

/// Print audit report and return overall completeness score
fn print_audit_report(entries: &[AuditEntry], threshold: f64) -> f64 {
    let total = entries.len();
    let success = entries.iter().filter(|e| e.success).count();
    let failed = entries.iter().filter(|e| !e.success).count();

    println!("{}", "Audit Report".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();
    println!("{:20} {}", "Total files:", total);
    println!("{:20} {}", "Parsed:", format!("{}", success).green());
    println!("{:20} {}", "Failed:", format!("{}", failed).red());
    println!();

    // Collect all keys across all specs for aggregate analysis
    let successful_entries: Vec<_> = entries.iter().filter(|e| e.success).collect();

    if successful_entries.is_empty() {
        println!("{}", "No specs parsed successfully.".red());
        return 0.0;
    }

    // Compute aggregate key classification
    let mut agg = KeyClassification::default();
    // Track missing keys with frequency: (section, key) -> count
    let mut missing_keys: HashMap<(String, String), usize> = HashMap::new();

    for entry in &successful_entries {
        let kc = &entry.key_classification;
        agg.implemented += kc.implemented;
        agg.partial += kc.partial;
        agg.not_implemented += kc.not_implemented;
        agg.deprecated += kc.deprecated;
        agg.unknown += kc.unknown;

        // Collect missing keys (not_implemented and unknown)
        for (key, status) in &kc.key_details {
            if matches!(status, MigrationKeyStatus::NotImplemented | MigrationKeyStatus::Unknown) {
                // Get section from the entry path
                let section = determine_category(&entry.source_path)
                    .map(|c| category_to_parity_section(&c).to_string())
                    .unwrap_or_else(|_| "unknown".to_string());

                *missing_keys.entry((section, key.clone())).or_insert(0) += 1;
            }
        }
    }

    // Print aggregate classification
    println!("{}", "Aggregate Key Classification:".cyan().bold());
    println!("{}", "-".repeat(60).dimmed());
    println!(
        "{:20} {} ({})",
        "Implemented:",
        format!("{}", agg.implemented).green(),
        "fully supported".green()
    );
    println!(
        "{:20} {} ({})",
        "Partial:",
        format!("{}", agg.partial).yellow(),
        "some features missing".yellow()
    );
    println!(
        "{:20} {} ({})",
        "Not Implemented:",
        format!("{}", agg.not_implemented).red(),
        "not yet supported".red()
    );
    println!(
        "{:20} {} ({})",
        "Unknown:",
        format!("{}", agg.unknown).dimmed(),
        "not in parity matrix".dimmed()
    );
    if agg.deprecated > 0 {
        println!(
            "{:20} {} ({})",
            "Deprecated:",
            format!("{}", agg.deprecated).dimmed(),
            "legacy keys, ignored".dimmed()
        );
    }
    println!();

    // Compute overall completeness score (gap score)
    let completeness = agg.gap_score().unwrap_or(0.0);
    let completeness_percent = completeness * 100.0;
    let threshold_percent = threshold * 100.0;

    let completeness_str = format!("{:.1}%", completeness_percent);
    let completeness_colored = if completeness >= threshold {
        completeness_str.green()
    } else if completeness >= threshold * 0.8 {
        completeness_str.yellow()
    } else {
        completeness_str.red()
    };

    println!(
        "{:20} {} (threshold: {:.0}%)",
        "Completeness:",
        completeness_colored,
        threshold_percent
    );
    println!(
        "{:20} {}",
        "",
        "(implemented + 0.5*partial) / total_used".dimmed()
    );
    println!();

    // Print top missing keys sorted by frequency
    if !missing_keys.is_empty() {
        println!("{}", "Top Missing Keys:".cyan().bold());
        println!("{}", "-".repeat(60).dimmed());

        // Sort by frequency (descending), then by key name
        let mut sorted_missing: Vec<_> = missing_keys.iter().collect();
        sorted_missing.sort_by(|a, b| {
            b.1.cmp(a.1).then_with(|| a.0.cmp(b.0))
        });

        // Show top 10 missing keys
        let display_count = sorted_missing.len().min(10);
        for ((section, key), count) in sorted_missing.iter().take(display_count) {
            let qualified = format!("{}::{}", section, key);
            println!(
                "  {:>4}x  {}",
                count,
                qualified.red()
            );
        }

        if sorted_missing.len() > display_count {
            println!(
                "  {} {} more missing keys...",
                "...".dimmed(),
                sorted_missing.len() - display_count
            );
        }
        println!();
    }

    // Print parse errors if any
    if failed > 0 {
        println!("{}", "Parse Errors:".red().bold());
        for entry in entries {
            if !entry.success {
                println!("  {} {}", "!".red(), entry.source_path.display().to_string().dimmed());
                if let Some(ref error) = entry.error {
                    println!("    {}", error);
                }
            }
        }
        println!();
    }

    // Print result summary
    println!("{}", "Result:".cyan().bold());
    if completeness >= threshold {
        println!(
            "  {} Completeness {:.1}% meets threshold {:.0}%",
            "PASS".green().bold(),
            completeness_percent,
            threshold_percent
        );
    } else {
        println!(
            "  {} Completeness {:.1}% below threshold {:.0}%",
            "FAIL".red().bold(),
            completeness_percent,
            threshold_percent
        );
    }

    completeness
}

/// Find all legacy .spec.py files in the specs directory
fn find_legacy_specs(specs_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut specs = Vec::new();

    for entry in WalkDir::new(specs_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().ends_with(".spec.py") {
                    specs.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(specs)
}

/// Migrate a single legacy spec file
fn migrate_spec(
    spec_file: &Path,
    project_root: &Path,
    allow_exec: bool,
) -> Result<MigrationEntry> {
    let mut warnings = Vec::new();

    // Parse the legacy spec
    let legacy = if allow_exec {
        parse_legacy_spec_exec(spec_file)?
    } else {
        parse_legacy_spec_static(spec_file)?
    };

    // Classify legacy keys against parity matrix
    let key_classification = classify_legacy_keys(&legacy);

    // Determine asset type and recipe kind from category
    let (asset_type, recipe_kind) = map_category_to_type(&legacy.category)?;

    // Extract asset_id from filename
    let asset_id = extract_asset_id(spec_file)?;

    // Generate seed from filename hash
    let seed = generate_seed_from_filename(spec_file);

    // Map legacy keys to canonical format
    let (params, mut key_warnings) = map_legacy_keys_to_params(&legacy, &recipe_kind)?;
    warnings.append(&mut key_warnings);

    // Determine output paths
    let outputs = generate_outputs(&asset_id, &asset_type, &legacy.category)?;

    // Build the canonical spec
    let mut spec = Spec::builder(asset_id.clone(), asset_type)
        .license("UNKNOWN")
        .seed(seed);

    // Add outputs
    for output in outputs {
        spec = spec.output(output);
    }

    // Add recipe with params
    let recipe_json = serde_json::json!({
        "kind": recipe_kind,
        "params": params
    });

    let mut spec_value = spec.build().to_value()?;
    if let Some(obj) = spec_value.as_object_mut() {
        obj.insert("recipe".to_string(), recipe_json);

        // Add migration notes
        let mut notes = vec![
            "Migrated from legacy .spec.py format".to_string(),
            "Please review and update the license field".to_string(),
        ];
        if !warnings.is_empty() {
            notes.push("See warnings for manual review items".to_string());
        }
        obj.insert(
            "migration_notes".to_string(),
            serde_json::json!(notes)
        );
    }

    // Parse back to ensure it's valid
    let spec: Spec = serde_json::from_value(spec_value)?;

    // Determine target path
    let target_path = project_root
        .join("specs")
        .join(asset_type.as_str())
        .join(format!("{}.json", asset_id));

    // Create directory if needed
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the spec
    let json = spec.to_json_pretty()?;
    fs::write(&target_path, json)?;

    Ok(MigrationEntry {
        source_path: spec_file.to_path_buf(),
        target_path: Some(target_path),
        success: true,
        warnings,
        error: None,
        key_classification,
    })
}

/// Parse legacy spec using static analysis (default, safe mode)
fn parse_legacy_spec_static(spec_file: &Path) -> Result<LegacySpec> {
    let content = fs::read_to_string(spec_file)
        .with_context(|| format!("Failed to read spec file: {}", spec_file.display()))?;

    // Determine category from path
    let category = determine_category(spec_file)?;

    // Determine expected dict name
    let dict_name = category_to_dict_name(&category);

    // Try to extract the dict using regex
    let pattern = format!(r"(?s){}[\s]*=[\s]*(\{{.*?\n\}})", regex::escape(&dict_name));
    let re = Regex::new(&pattern)?;

    if let Some(caps) = re.captures(&content) {
        let dict_str = caps.get(1).unwrap().as_str();

        // Try to parse as Python dict literal
        match parse_python_dict_literal(dict_str) {
            Ok(data) => {
                return Ok(LegacySpec {
                    dict_name,
                    category,
                    data,
                });
            }
            Err(e) => {
                bail!(
                    "Static analysis failed: {}. Use --allow-exec-specs to execute Python.",
                    e
                );
            }
        }
    }

    bail!(
        "Could not find {} dict in file. Use --allow-exec-specs to execute Python.",
        dict_name
    );
}

/// Parse legacy spec by executing Python (unsafe, requires opt-in)
fn parse_legacy_spec_exec(spec_file: &Path) -> Result<LegacySpec> {
    let category = determine_category(spec_file)?;
    let dict_name = category_to_dict_name(&category);

    // Create a temporary Python script to extract the dict
    let script = format!(
        r#"
import sys
import json
sys.path.insert(0, str('{}'))
from {} import {}
print(json.dumps({}))
"#,
        spec_file.parent().unwrap().display(),
        spec_file.file_stem().unwrap().to_string_lossy(),
        dict_name,
        dict_name
    );

    let output = Command::new("python3")
        .arg("-c")
        .arg(&script)
        .output()
        .with_context(|| "Failed to execute python3")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Python execution failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let data: HashMap<String, serde_json::Value> = serde_json::from_str(&stdout)
        .with_context(|| "Failed to parse Python output as JSON")?;

    Ok(LegacySpec {
        dict_name,
        category,
        data,
    })
}

/// Parse a Python dict literal to JSON (simple cases only)
///
/// This is a simplified parser that handles basic Python dict literals:
/// - Strings (single-quoted converted to double-quoted)
/// - Numbers (integers and floats)
/// - Booleans (True/False -> true/false)
/// - None -> null
/// - Lists and nested dicts
///
/// Note: This parser has limitations and may fail on complex cases like:
/// - Strings containing escaped quotes
/// - Multi-line strings
/// - Python expressions or function calls
fn parse_python_dict_literal(dict_str: &str) -> Result<HashMap<String, serde_json::Value>> {
    let mut json_str = dict_str.to_string();

    // Replace Python keywords with JSON equivalents
    // Note: These regexes are simple and known-valid patterns, unwrap is safe
    json_str = Regex::new(r"\bNone\b").unwrap().replace_all(&json_str, "null").to_string();
    json_str = Regex::new(r"\bTrue\b").unwrap().replace_all(&json_str, "true").to_string();
    json_str = Regex::new(r"\bFalse\b").unwrap().replace_all(&json_str, "false").to_string();

    // Replace single quotes with double quotes (simple approach)
    // This won't handle escaped quotes properly, but works for most cases
    json_str = json_str.replace("'", "\"");

    // Try to parse as JSON
    match serde_json::from_str(&json_str) {
        Ok(value) => Ok(value),
        Err(e) => bail!("Could not parse dict as JSON: {}", e),
    }
}

/// Determine category from file path
fn determine_category(spec_file: &Path) -> Result<String> {
    let path_str = spec_file.to_string_lossy();

    if path_str.contains("/sounds/") || path_str.contains("\\sounds\\") {
        Ok("sounds".to_string())
    } else if path_str.contains("/instruments/") || path_str.contains("\\instruments\\") {
        Ok("instruments".to_string())
    } else if path_str.contains("/music/") || path_str.contains("\\music\\") {
        Ok("music".to_string())
    } else if path_str.contains("/textures/") || path_str.contains("\\textures\\") {
        Ok("textures".to_string())
    } else if path_str.contains("/normals/") || path_str.contains("\\normals\\") {
        Ok("normals".to_string())
    } else if path_str.contains("/meshes/") || path_str.contains("\\meshes\\") {
        Ok("meshes".to_string())
    } else if path_str.contains("/characters/") || path_str.contains("\\characters\\") {
        Ok("characters".to_string())
    } else if path_str.contains("/animations/") || path_str.contains("\\animations\\") {
        Ok("animations".to_string())
    } else {
        bail!("Could not determine category from path: {}", path_str);
    }
}

/// Map category to dict name
fn category_to_dict_name(category: &str) -> String {
    match category {
        "sounds" => "SOUND".to_string(),
        "instruments" => "INSTRUMENT".to_string(),
        "music" => "SONG".to_string(),
        "textures" => "TEXTURE".to_string(),
        "normals" => "NORMAL".to_string(),
        "meshes" => "MESH".to_string(),
        "characters" => "SPEC".to_string(),
        "animations" => "ANIMATION".to_string(),
        _ => category.to_uppercase(),
    }
}

/// Map category to asset type and recipe kind
fn map_category_to_type(category: &str) -> Result<(AssetType, String)> {
    match category {
        "sounds" => Ok((AssetType::AudioSfx, "audio_sfx.layered_synth_v1".to_string())),
        "instruments" => Ok((AssetType::AudioInstrument, "audio_instrument.synth_patch_v1".to_string())),
        "music" => Ok((AssetType::Music, "music.tracker_song_v1".to_string())),
        "textures" => Ok((AssetType::Texture2d, "texture_2d.material_maps_v1".to_string())),
        "normals" => Ok((AssetType::Texture2d, "texture_2d.normal_map_v1".to_string())),
        "meshes" => Ok((AssetType::StaticMesh, "static_mesh.blender_primitives_v1".to_string())),
        "characters" => Ok((AssetType::SkeletalMesh, "skeletal_mesh.blender_rigged_mesh_v1".to_string())),
        "animations" => Ok((AssetType::SkeletalAnimation, "skeletal_animation.blender_clip_v1".to_string())),
        _ => bail!("Unknown category: {}", category),
    }
}

/// Map category to parity matrix section name
/// The parity matrix uses section names like "SOUND (audio_sfx)"
fn category_to_parity_section(category: &str) -> &'static str {
    match category {
        "sounds" => "SOUND (audio_sfx)",
        "instruments" => "INSTRUMENT (audio_instrument)",
        "music" => "SONG (music)",
        "textures" => "TEXTURE (texture_2d)",
        "normals" => "NORMAL (normal_map)",
        "meshes" => "MESH (static_mesh)",
        "characters" => "SPEC/CHARACTER (skeletal_mesh)",
        "animations" => "ANIMATION (skeletal_animation)",
        _ => "",
    }
}

/// Classify a single legacy key against the parity matrix
/// Returns the MigrationKeyStatus for the key
fn classify_key(section: &str, key: &str) -> MigrationKeyStatus {
    // Try "Top-Level Keys" table first (most common)
    if let Some(info) = parity_data::find(section, "Top-Level Keys", key) {
        return MigrationKeyStatus::from(info.status);
    }

    // Try other common tables based on section
    let tables_to_check: &[&str] = match section {
        "SOUND (audio_sfx)" => &["Layer Keys", "Envelope Keys (ADSR)", "Filter Keys"],
        "INSTRUMENT (audio_instrument)" => &["Synthesis Keys", "Oscillator Keys (Subtractive)", "Output Keys"],
        "SONG (music)" => &["Instrument Keys (inline or ref)", "Pattern Keys", "Note Keys (within pattern)", "Arrangement Entry Keys", "Automation Keys", "IT Options Keys"],
        "TEXTURE (texture_2d)" => &["Layer Keys", "Solid Layer", "Noise Layer", "Gradient Layer", "Checkerboard Layer", "Stripes Layer", "Wood Grain Layer", "Brick Layer"],
        "NORMAL (normal_map)" => &["Processing Keys", "Pattern Keys", "Bricks Pattern", "Tiles Pattern", "Hexagons Pattern", "Noise Pattern", "Scratches Pattern", "Rivets Pattern", "Weave Pattern"],
        "MESH (static_mesh)" => &["Cube", "Cylinder", "Sphere (UV)", "Icosphere", "Cone", "Torus", "Modifier Keys", "Bevel Modifier", "Decimate Modifier", "UV Keys", "Export Keys"],
        "SPEC/CHARACTER (skeletal_mesh)" => &["Skeleton Bone Keys", "Part Keys", "Step Keys", "Instance Keys", "Texturing Keys"],
        "ANIMATION (skeletal_animation)" => &["Pose Keys (per bone)", "Phase Keys", "IK Target Keyframe Keys", "Procedural Layer Keys", "Rig Setup Keys", "IK Chain Keys", "Constraint Keys", "Twist Bone Keys", "Bake Settings Keys", "Animator Rig Config Keys", "Conventions Keys"],
        _ => &[],
    };

    for table in tables_to_check {
        if let Some(info) = parity_data::find(section, table, key) {
            return MigrationKeyStatus::from(info.status);
        }
    }

    // Key not found in parity matrix
    MigrationKeyStatus::Unknown
}

/// Classify all top-level keys in a legacy spec dict
/// Returns a KeyClassification with counts and details
fn classify_legacy_keys(legacy: &LegacySpec) -> KeyClassification {
    let section = category_to_parity_section(&legacy.category);
    let mut classification = KeyClassification::default();

    for key in legacy.data.keys() {
        let status = classify_key(section, key);
        classification.key_details.push((key.clone(), status));

        match status {
            MigrationKeyStatus::Implemented => classification.implemented += 1,
            MigrationKeyStatus::Partial => classification.partial += 1,
            MigrationKeyStatus::NotImplemented => classification.not_implemented += 1,
            MigrationKeyStatus::Deprecated => classification.deprecated += 1,
            MigrationKeyStatus::Unknown => classification.unknown += 1,
        }
    }

    // Sort key_details by status (for consistent output)
    classification.key_details.sort_by(|a, b| {
        let status_order = |s: &MigrationKeyStatus| match s {
            MigrationKeyStatus::Implemented => 0,
            MigrationKeyStatus::Partial => 1,
            MigrationKeyStatus::NotImplemented => 2,
            MigrationKeyStatus::Unknown => 3,
            MigrationKeyStatus::Deprecated => 4,
        };
        status_order(&a.1).cmp(&status_order(&b.1)).then(a.0.cmp(&b.0))
    });

    classification
}

/// Extract asset_id from filename
fn extract_asset_id(spec_file: &Path) -> Result<String> {
    let file_stem = spec_file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

    // Remove .spec suffix if present
    let asset_id = file_stem
        .strip_suffix(".spec")
        .unwrap_or(file_stem);

    // Convert to lowercase and replace invalid characters
    let asset_id = asset_id.to_lowercase().replace("_", "-");

    // Validate format: [a-z][a-z0-9_-]{2,63}
    let re = Regex::new(r"^[a-z][a-z0-9_-]{2,63}$")?;
    if !re.is_match(&asset_id) {
        bail!("Invalid asset_id format: {}", asset_id);
    }

    Ok(asset_id)
}

/// Generate seed from filename hash
fn generate_seed_from_filename(spec_file: &Path) -> u32 {
    let filename = spec_file.file_name().unwrap().to_string_lossy();
    let hash = blake3::hash(filename.as_bytes());
    let bytes = hash.as_bytes();

    // Take first 4 bytes and convert to u32
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// Map legacy keys to canonical params
fn map_legacy_keys_to_params(
    legacy: &LegacySpec,
    recipe_kind: &str,
) -> Result<(serde_json::Value, Vec<String>)> {
    let mut warnings = Vec::new();

    // For now, just pass through the legacy data as params
    // TODO: Map legacy keys to canonical params using PARITY_MATRIX.md (SSOT for mapping rules).

    // Remove the 'name' field as it's used for asset_id
    let mut params = legacy.data.clone();
    params.remove("name");

    // Add warning for manual review
    if !params.is_empty() {
        warnings.push(format!(
            "Legacy params dict '{}' passed through for {}. Manual review recommended (TODO: key mapping per PARITY_MATRIX.md).",
            legacy.dict_name, recipe_kind
        ));
    }

    Ok((serde_json::json!(params), warnings))
}

/// Generate output specs based on asset type
fn generate_outputs(
    asset_id: &str,
    asset_type: &AssetType,
    category: &str,
) -> Result<Vec<OutputSpec>> {
    let outputs = match asset_type {
        AssetType::AudioSfx => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Wav,
                path: format!("sounds/{}.wav", asset_id),
            }]
        }
        AssetType::AudioInstrument => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Wav,
                path: format!("instruments/{}.wav", asset_id),
            }]
        }
        AssetType::Music => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Xm,
                path: format!("music/{}.xm", asset_id),
            }]
        }
        AssetType::Texture2d => {
            if category == "normals" {
                vec![OutputSpec {
                    kind: OutputKind::Primary,
                    format: OutputFormat::Png,
                    path: format!("textures/{}_normal.png", asset_id),
                }]
            } else {
                vec![OutputSpec {
                    kind: OutputKind::Primary,
                    format: OutputFormat::Png,
                    path: format!("textures/{}.png", asset_id),
                }]
            }
        }
        AssetType::StaticMesh => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Glb,
                path: format!("meshes/{}.glb", asset_id),
            }]
        }
        AssetType::SkeletalMesh => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Glb,
                path: format!("characters/{}.glb", asset_id),
            }]
        }
        AssetType::SkeletalAnimation => {
            vec![OutputSpec {
                kind: OutputKind::Primary,
                format: OutputFormat::Glb,
                path: format!("animations/{}.glb", asset_id),
            }]
        }
    };

    Ok(outputs)
}

/// Print migration report
fn print_migration_report(entries: &[MigrationEntry]) {
    let total = entries.len();
    let success = entries.iter().filter(|e| e.success).count();
    let with_warnings = entries.iter().filter(|e| !e.warnings.is_empty()).count();
    let failed = entries.iter().filter(|e| !e.success).count();

    println!("{}", "Migration Report".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();
    println!("{:20} {}", "Total files:", total);
    println!("{:20} {}", "Converted:", format!("{}", success).green());
    println!("{:20} {}", "With warnings:", format!("{}", with_warnings).yellow());
    println!("{:20} {}", "Failed:", format!("{}", failed).red());
    println!();

    // Print per-file key classification
    let successful_entries: Vec<_> = entries.iter().filter(|e| e.success).collect();
    if !successful_entries.is_empty() {
        println!("{}", "Key Classification (per file):".cyan().bold());
        println!("{}", "-".repeat(60).dimmed());
        println!(
            "{:<40} {:>4} {:>4} {:>4} {:>4} {:>7}",
            "File", "Impl", "Part", "Miss", "Unkn", "Gap"
        );
        println!("{}", "-".repeat(60).dimmed());

        for entry in &successful_entries {
            let kc = &entry.key_classification;
            let filename = entry
                .source_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "?".to_string());
            let truncated = if filename.len() > 38 {
                format!("{}...", &filename[..35])
            } else {
                filename
            };

            let gap_str = match kc.gap_score() {
                Some(score) => format!("{:.0}%", score * 100.0),
                None => "-".to_string(),
            };

            // Color the gap score based on value
            let gap_colored = match kc.gap_score() {
                Some(score) if score >= 0.8 => gap_str.green().to_string(),
                Some(score) if score >= 0.5 => gap_str.yellow().to_string(),
                Some(_) => gap_str.red().to_string(),
                None => gap_str.dimmed().to_string(),
            };

            println!(
                "{:<40} {:>4} {:>4} {:>4} {:>4} {:>7}",
                truncated.dimmed(),
                format!("{}", kc.implemented).green(),
                format!("{}", kc.partial).yellow(),
                format!("{}", kc.not_implemented).red(),
                format!("{}", kc.unknown).dimmed(),
                gap_colored
            );
        }
        println!();

        // Compute and print overall aggregated stats
        let mut agg = KeyClassification::default();
        for entry in &successful_entries {
            let kc = &entry.key_classification;
            agg.implemented += kc.implemented;
            agg.partial += kc.partial;
            agg.not_implemented += kc.not_implemented;
            agg.deprecated += kc.deprecated;
            agg.unknown += kc.unknown;
        }

        println!("{}", "Overall Key Classification:".cyan().bold());
        println!("{}", "-".repeat(60).dimmed());
        println!(
            "{:20} {} ({})",
            "Implemented:",
            format!("{}", agg.implemented).green(),
            "fully supported".green()
        );
        println!(
            "{:20} {} ({})",
            "Partial:",
            format!("{}", agg.partial).yellow(),
            "some features missing".yellow()
        );
        println!(
            "{:20} {} ({})",
            "Not Implemented:",
            format!("{}", agg.not_implemented).red(),
            "not yet supported".red()
        );
        println!(
            "{:20} {} ({})",
            "Unknown:",
            format!("{}", agg.unknown).dimmed(),
            "not in parity matrix".dimmed()
        );
        if agg.deprecated > 0 {
            println!(
                "{:20} {} ({})",
                "Deprecated:",
                format!("{}", agg.deprecated).dimmed(),
                "legacy keys, ignored".dimmed()
            );
        }
        println!();

        // Overall gap score
        if let Some(gap) = agg.gap_score() {
            let gap_percent = gap * 100.0;
            let gap_str = format!("{:.1}%", gap_percent);
            let gap_colored = if gap_percent >= 80.0 {
                gap_str.green()
            } else if gap_percent >= 50.0 {
                gap_str.yellow()
            } else {
                gap_str.red()
            };
            println!(
                "{:20} {} ({} keys used, {} deprecated)",
                "Overall Gap Score:",
                gap_colored,
                agg.total_used(),
                agg.deprecated
            );
            println!(
                "{:20} {}",
                "",
                "(implemented + 0.5*partial) / total_used".dimmed()
            );
        } else {
            println!("{:20} {}", "Overall Gap Score:", "-".dimmed());
        }
        println!();
    }

    if with_warnings > 0 {
        println!("{}", "Warnings:".yellow().bold());
        for entry in entries {
            if !entry.warnings.is_empty() {
                println!("  {} {}", "⚠".yellow(), entry.source_path.display().to_string().dimmed());
                if let Some(ref target) = entry.target_path {
                    println!("    -> {}", target.display().to_string().dimmed());
                }
                for warning in &entry.warnings {
                    println!("    - {}", warning);
                }
            }
        }
        println!();
    }

    if failed > 0 {
        println!("{}", "Errors:".red().bold());
        for entry in entries {
            if !entry.success {
                println!("  {} {}", "✗".red(), entry.source_path.display().to_string().dimmed());
                if let Some(ref target) = entry.target_path {
                    println!("    -> {}", target.display().to_string().dimmed());
                }
                if let Some(ref error) = entry.error {
                    println!("    {}", error);
                }
            }
        }
        println!();
    }

    println!("{}", "Next Steps:".cyan().bold());
    println!("  1. Review migrated specs in specs/ directory");
    println!("  2. Update license fields from 'UNKNOWN'");
    println!("  3. Review and address any warnings");
    println!("  4. Test with: speccade validate --spec specs/<type>/<asset>.json");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_category_from_path() {
        assert_eq!(
            determine_category(Path::new("project/.studio/specs/sounds/laser.spec.py")).unwrap(),
            "sounds"
        );
        assert_eq!(
            determine_category(Path::new("project/.studio/specs/textures/metal.spec.py")).unwrap(),
            "textures"
        );
        assert_eq!(
            determine_category(Path::new("project/.studio/specs/normals/wall.spec.py")).unwrap(),
            "normals"
        );
        assert_eq!(
            determine_category(Path::new("project/.studio/specs/meshes/crate.spec.py")).unwrap(),
            "meshes"
        );
    }

    #[test]
    fn test_category_to_dict_name() {
        assert_eq!(category_to_dict_name("sounds"), "SOUND");
        assert_eq!(category_to_dict_name("animations"), "ANIMATION");
        assert_eq!(category_to_dict_name("unknown"), "UNKNOWN");
    }

    #[test]
    fn test_map_category_to_type() {
        let (asset_type, kind) = map_category_to_type("sounds").unwrap();
        assert_eq!(asset_type, AssetType::AudioSfx);
        assert_eq!(kind, "audio_sfx.layered_synth_v1");
    }

    #[test]
    fn test_extract_asset_id() {
        let id = extract_asset_id(Path::new("laser_blast_01.spec.py")).unwrap();
        assert_eq!(id, "laser-blast-01");

        assert!(extract_asset_id(Path::new("AB.spec.py")).is_err(), "too short");
        assert!(extract_asset_id(Path::new("INVALID!.spec.py")).is_err(), "invalid chars");
    }

    #[test]
    fn test_generate_seed_from_filename_is_deterministic() {
        let seed1 = generate_seed_from_filename(Path::new("a.spec.py"));
        let seed2 = generate_seed_from_filename(Path::new("a.spec.py"));
        let seed3 = generate_seed_from_filename(Path::new("b.spec.py"));

        assert_eq!(seed1, seed2);
        assert_ne!(seed1, seed3);
    }

    #[test]
    fn test_parse_python_dict_literal_simple() {
        let dict = r#"{'name': 'laser', 'enabled': True, 'value': None, 'nums': [1, 2, 3]}"#;
        let parsed = parse_python_dict_literal(dict).unwrap();

        assert_eq!(parsed.get("name").and_then(|v| v.as_str()), Some("laser"));
        assert_eq!(parsed.get("enabled").and_then(|v| v.as_bool()), Some(true));
        assert!(parsed.get("value").is_some_and(|v| v.is_null()));
        assert_eq!(parsed.get("nums").unwrap().as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_map_legacy_keys_to_params_removes_name_and_warns() {
        let legacy = LegacySpec {
            dict_name: "SOUND".to_string(),
            category: "sounds".to_string(),
            data: HashMap::from([
                ("name".to_string(), serde_json::json!("laser")),
                ("duration".to_string(), serde_json::json!(0.5)),
            ]),
        };

        let (params, warnings) = map_legacy_keys_to_params(&legacy, "audio_sfx.layered_synth_v1").unwrap();
        assert!(warnings.iter().any(|w| w.contains("PARITY_MATRIX.md")));
        assert!(params.get("name").is_none(), "name should be removed");
        assert!(params.get("duration").is_some());
    }

    #[test]
    fn test_generate_outputs_normals() {
        let outputs = generate_outputs("wall-01", &AssetType::Texture2d, "normals").unwrap();
        assert_eq!(outputs.len(), 1);
        assert!(outputs[0].path.ends_with("_normal.png"));
    }

    #[test]
    fn test_category_to_parity_section() {
        assert_eq!(category_to_parity_section("sounds"), "SOUND (audio_sfx)");
        assert_eq!(category_to_parity_section("instruments"), "INSTRUMENT (audio_instrument)");
        assert_eq!(category_to_parity_section("music"), "SONG (music)");
        assert_eq!(category_to_parity_section("textures"), "TEXTURE (texture_2d)");
        assert_eq!(category_to_parity_section("normals"), "NORMAL (normal_map)");
        assert_eq!(category_to_parity_section("meshes"), "MESH (static_mesh)");
        assert_eq!(category_to_parity_section("characters"), "SPEC/CHARACTER (skeletal_mesh)");
        assert_eq!(category_to_parity_section("animations"), "ANIMATION (skeletal_animation)");
        assert_eq!(category_to_parity_section("unknown"), "");
    }

    #[test]
    fn test_classify_key_known_implemented() {
        // "name" is a well-known implemented key in SOUND section
        let status = classify_key("SOUND (audio_sfx)", "name");
        assert_eq!(status, MigrationKeyStatus::Implemented);
    }

    #[test]
    fn test_classify_key_unknown() {
        // A completely unknown key should return Unknown
        let status = classify_key("SOUND (audio_sfx)", "totally_fake_key_that_doesnt_exist");
        assert_eq!(status, MigrationKeyStatus::Unknown);
    }

    #[test]
    fn test_classify_legacy_keys() {
        let legacy = LegacySpec {
            dict_name: "SOUND".to_string(),
            category: "sounds".to_string(),
            data: HashMap::from([
                ("name".to_string(), serde_json::json!("test")),
                ("duration".to_string(), serde_json::json!(1.0)),
                ("sample_rate".to_string(), serde_json::json!(22050)),
                ("fake_key".to_string(), serde_json::json!("unknown")),
            ]),
        };

        let classification = classify_legacy_keys(&legacy);

        // name, duration, sample_rate are all implemented in SOUND
        assert!(classification.implemented >= 3);
        // fake_key should be unknown
        assert!(classification.unknown >= 1);
        // Total should match
        assert_eq!(
            classification.implemented + classification.partial + classification.not_implemented + classification.deprecated + classification.unknown,
            4
        );
    }

    #[test]
    fn test_key_classification_gap_score() {
        // Test gap score calculation
        let mut kc = KeyClassification::default();
        kc.implemented = 8;
        kc.partial = 2;
        kc.not_implemented = 0;
        kc.unknown = 0;

        // (8 + 0.5*2) / 10 = 9/10 = 0.9
        let gap = kc.gap_score().unwrap();
        assert!((gap - 0.9).abs() < 0.001);

        // Test with mixed values
        let mut kc2 = KeyClassification::default();
        kc2.implemented = 4;
        kc2.partial = 2;
        kc2.not_implemented = 2;
        kc2.unknown = 2;

        // (4 + 0.5*2) / (4+2+2+2) = 5/10 = 0.5
        let gap2 = kc2.gap_score().unwrap();
        assert!((gap2 - 0.5).abs() < 0.001);

        // Deprecated keys are excluded from denominator
        kc2.deprecated = 5;
        // Still (4 + 0.5*2) / (4+2+2+2) = 5/10 = 0.5
        let gap3 = kc2.gap_score().unwrap();
        assert!((gap3 - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_key_classification_total_used() {
        let mut kc = KeyClassification::default();
        kc.implemented = 5;
        kc.partial = 3;
        kc.not_implemented = 2;
        kc.unknown = 1;
        kc.deprecated = 4;

        // total_used excludes deprecated
        assert_eq!(kc.total_used(), 5 + 3 + 2 + 1);
    }

    #[test]
    fn test_key_classification_gap_score_empty() {
        let kc = KeyClassification::default();
        assert!(kc.gap_score().is_none());
    }

    #[test]
    fn test_migration_key_status_from_key_status() {
        assert_eq!(MigrationKeyStatus::from(KeyStatus::Implemented), MigrationKeyStatus::Implemented);
        assert_eq!(MigrationKeyStatus::from(KeyStatus::Partial), MigrationKeyStatus::Partial);
        assert_eq!(MigrationKeyStatus::from(KeyStatus::NotImplemented), MigrationKeyStatus::NotImplemented);
        assert_eq!(MigrationKeyStatus::from(KeyStatus::Deprecated), MigrationKeyStatus::Deprecated);
    }

    #[test]
    fn test_migration_key_status_display() {
        assert_eq!(format!("{}", MigrationKeyStatus::Implemented), "Implemented");
        assert_eq!(format!("{}", MigrationKeyStatus::Partial), "Partial");
        assert_eq!(format!("{}", MigrationKeyStatus::NotImplemented), "NotImplemented");
        assert_eq!(format!("{}", MigrationKeyStatus::Deprecated), "Deprecated");
        assert_eq!(format!("{}", MigrationKeyStatus::Unknown), "Unknown");
    }

    #[test]
    fn test_audit_entry_success() {
        let entry = AuditEntry {
            source_path: PathBuf::from("test.spec.py"),
            success: true,
            error: None,
            key_classification: KeyClassification::default(),
        };
        assert!(entry.success);
        assert!(entry.error.is_none());
    }

    #[test]
    fn test_audit_entry_failure() {
        let entry = AuditEntry {
            source_path: PathBuf::from("test.spec.py"),
            success: false,
            error: Some("Parse error".to_string()),
            key_classification: KeyClassification::default(),
        };
        assert!(!entry.success);
        assert_eq!(entry.error.as_deref(), Some("Parse error"));
    }

    #[test]
    fn test_audit_completeness_threshold_pass() {
        // Create a key classification that passes 90% threshold
        let mut kc = KeyClassification::default();
        kc.implemented = 9;
        kc.partial = 0;
        kc.not_implemented = 1;
        kc.unknown = 0;

        // Gap score = (9 + 0*0.5) / (9+0+1+0) = 9/10 = 0.9
        let gap = kc.gap_score().unwrap();
        assert!((gap - 0.9).abs() < 0.001);
        assert!(gap >= 0.90); // Meets threshold
    }

    #[test]
    fn test_audit_completeness_threshold_fail() {
        // Create a key classification that fails 90% threshold
        let mut kc = KeyClassification::default();
        kc.implemented = 7;
        kc.partial = 2;
        kc.not_implemented = 1;
        kc.unknown = 0;

        // Gap score = (7 + 2*0.5) / (7+2+1+0) = 8/10 = 0.8
        let gap = kc.gap_score().unwrap();
        assert!((gap - 0.8).abs() < 0.001);
        assert!(gap < 0.90); // Fails threshold
    }

    #[test]
    fn test_missing_keys_frequency_counting() {
        // Simulate collecting missing keys from multiple specs
        let mut missing_keys: HashMap<(String, String), usize> = HashMap::new();

        // Same key appearing in multiple specs should increase frequency
        *missing_keys.entry(("SOUND (audio_sfx)".to_string(), "reverb".to_string())).or_insert(0) += 1;
        *missing_keys.entry(("SOUND (audio_sfx)".to_string(), "reverb".to_string())).or_insert(0) += 1;
        *missing_keys.entry(("SOUND (audio_sfx)".to_string(), "echo".to_string())).or_insert(0) += 1;

        assert_eq!(missing_keys.get(&("SOUND (audio_sfx)".to_string(), "reverb".to_string())), Some(&2));
        assert_eq!(missing_keys.get(&("SOUND (audio_sfx)".to_string(), "echo".to_string())), Some(&1));
    }

    #[test]
    fn test_missing_keys_sorted_by_frequency() {
        let mut missing_keys: HashMap<(String, String), usize> = HashMap::new();
        missing_keys.insert(("A".to_string(), "key1".to_string()), 5);
        missing_keys.insert(("A".to_string(), "key2".to_string()), 10);
        missing_keys.insert(("A".to_string(), "key3".to_string()), 1);

        let mut sorted: Vec<_> = missing_keys.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

        assert_eq!(sorted[0].0, &("A".to_string(), "key2".to_string()));
        assert_eq!(sorted[1].0, &("A".to_string(), "key1".to_string()));
        assert_eq!(sorted[2].0, &("A".to_string(), "key3".to_string()));
    }
}
