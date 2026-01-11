//! Legacy spec file parsing
//!
//! Handles parsing of legacy .spec.py files via static analysis or Python execution.

use anyhow::{bail, Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

/// Legacy spec data extracted from .spec.py
#[derive(Debug)]
pub struct LegacySpec {
    pub dict_name: String,
    pub category: String,
    pub data: HashMap<String, serde_json::Value>,
}

/// Find all legacy .spec.py files in the specs directory
pub fn find_legacy_specs(specs_dir: &Path) -> Result<Vec<std::path::PathBuf>> {
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

/// Parse legacy spec using static analysis (default, safe mode)
pub fn parse_legacy_spec_static(spec_file: &Path) -> Result<LegacySpec> {
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
        let dict_match = caps.get(1).ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to capture {} dict contents in {}",
                dict_name,
                spec_file.display()
            )
        })?;
        let dict_str = dict_match.as_str();

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
pub fn parse_legacy_spec_exec(spec_file: &Path) -> Result<LegacySpec> {
    let category = determine_category(spec_file)?;
    let dict_name = category_to_dict_name(&category);

    // Create a temporary Python script to extract the dict
    let parent = spec_file
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Spec file has no parent dir: {}", spec_file.display()))?;
    let stem = spec_file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Spec file has invalid stem: {}", spec_file.display()))?;
    let script = format!(
        r#"
import sys
import json
sys.path.insert(0, str('{}'))
from {} import {}
print(json.dumps({}))
"#,
        parent.display(),
        stem,
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
    let data: HashMap<String, serde_json::Value> =
        serde_json::from_str(&stdout).with_context(|| "Failed to parse Python output as JSON")?;

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
pub fn parse_python_dict_literal(dict_str: &str) -> Result<HashMap<String, serde_json::Value>> {
    let mut json_str = dict_str.to_string();

    // Replace Python keywords with JSON equivalents
    // Note: These regexes are simple and known-valid patterns, unwrap is safe
    json_str = Regex::new(r"\bNone\b")
        .unwrap()
        .replace_all(&json_str, "null")
        .to_string();
    json_str = Regex::new(r"\bTrue\b")
        .unwrap()
        .replace_all(&json_str, "true")
        .to_string();
    json_str = Regex::new(r"\bFalse\b")
        .unwrap()
        .replace_all(&json_str, "false")
        .to_string();

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
pub fn determine_category(spec_file: &Path) -> Result<String> {
    const CATEGORIES: [&str; 8] = [
        "sounds",
        "instruments",
        "music",
        "textures",
        "normals",
        "meshes",
        "characters",
        "animations",
    ];

    let components: Vec<&str> = spec_file
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    if let Some(specs_idx) = components.iter().position(|c| *c == "specs") {
        if let Some(category) = components.get(specs_idx + 1) {
            if CATEGORIES.contains(category) {
                return Ok((*category).to_string());
            }
        }
    }

    for component in &components {
        if CATEGORIES.contains(component) {
            return Ok((*component).to_string());
        }
    }

    bail!(
        "Could not determine category from path: {}",
        spec_file.display()
    );
}

/// Map category to dict name
pub fn category_to_dict_name(category: &str) -> String {
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
    fn test_parse_python_dict_literal_simple() {
        let dict = r#"{'name': 'laser', 'enabled': True, 'value': None, 'nums': [1, 2, 3]}"#;
        let parsed = parse_python_dict_literal(dict).unwrap();

        assert_eq!(parsed.get("name").and_then(|v| v.as_str()), Some("laser"));
        assert_eq!(parsed.get("enabled").and_then(|v| v.as_bool()), Some(true));
        assert!(parsed.get("value").is_some_and(|v| v.is_null()));
        assert_eq!(parsed.get("nums").unwrap().as_array().unwrap().len(), 3);
    }
}
