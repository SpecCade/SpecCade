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

/// Migration report entry
#[derive(Debug)]
#[allow(dead_code)]
struct MigrationEntry {
    source_path: PathBuf,
    target_path: Option<PathBuf>,
    success: bool,
    warnings: Vec<String>,
    error: Option<String>,
}

/// Legacy spec data extracted from .spec.py
#[derive(Debug)]
#[allow(dead_code)]
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
        let entry = migrate_spec(spec_file, path, allow_exec_specs);

        // Print progress
        match &entry {
            Ok(e) if e.success => {
                print!("{} ", "✓".green());
            }
            Ok(e) if !e.warnings.is_empty() => {
                print!("{} ", "⚠".yellow());
            }
            Err(_) => {
                print!("{} ", "✗".red());
            }
            _ => {}
        }

        if let Ok(e) = entry {
            entries.push(e);
        }
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
fn parse_python_dict_literal(dict_str: &str) -> Result<HashMap<String, serde_json::Value>> {
    // This is a simplified parser for basic Python dict literals
    // It handles: strings, numbers, bools, None, lists, and nested dicts

    // First, make some simple replacements to convert to JSON
    let mut json_str = dict_str.to_string();

    // Replace None with null
    json_str = Regex::new(r"\bNone\b")?.replace_all(&json_str, "null").to_string();

    // Replace True with true
    json_str = Regex::new(r"\bTrue\b")?.replace_all(&json_str, "true").to_string();

    // Replace False with false
    json_str = Regex::new(r"\bFalse\b")?.replace_all(&json_str, "false").to_string();

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

/// Extract asset_id from filename
fn extract_asset_id(spec_file: &Path) -> Result<String> {
    let file_stem = spec_file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

    // Remove .spec suffix if present
    let asset_id = if file_stem.ends_with(".spec") {
        &file_stem[..file_stem.len() - 5]
    } else {
        file_stem
    };

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
    // In a full implementation, we would map each key according to PARITY_MATRIX.md

    // Remove the 'name' field as it's used for asset_id
    let mut params = legacy.data.clone();
    params.remove("name");

    // Add warning for manual review
    if !params.is_empty() {
        warnings.push(format!(
            "Legacy params passed through for {}. Manual review recommended.",
            recipe_kind
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
    let failed = total - success;

    println!("{}", "Migration Report".cyan().bold());
    println!("{}", "=".repeat(50).dimmed());
    println!();
    println!("{:20} {}", "Total files:", total);
    println!("{:20} {}", "Converted:", format!("{}", success).green());
    println!("{:20} {}", "With warnings:", format!("{}", with_warnings).yellow());
    println!("{:20} {}", "Failed:", format!("{}", failed).red());
    println!();

    if with_warnings > 0 {
        println!("{}", "Warnings:".yellow().bold());
        for entry in entries {
            if !entry.warnings.is_empty() {
                println!("  {} {}", "⚠".yellow(), entry.source_path.display().to_string().dimmed());
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
