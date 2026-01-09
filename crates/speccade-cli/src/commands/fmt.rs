//! Fmt command implementation
//!
//! Formats a spec file to canonical style (sorted keys, 2-space indent).

use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::Value;
use std::fs;
use std::process::ExitCode;

/// Run the fmt command
///
/// # Arguments
/// * `spec_path` - Path to the spec JSON file
/// * `output` - Output file path (default: overwrite input file)
///
/// # Returns
/// Exit code: 0 success, 1 error
pub fn run(spec_path: &str, output: Option<&str>) -> Result<ExitCode> {
    println!(
        "{} {}",
        "Formatting:".cyan().bold(),
        spec_path
    );

    // Read spec file
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path))?;

    // Parse as generic JSON Value to preserve all fields
    let value: Value = serde_json::from_str(&spec_content)
        .with_context(|| format!("Failed to parse JSON: {}", spec_path))?;

    // Sort keys recursively and format with 2-space indent
    let sorted = sort_json_keys(&value);
    let formatted = serde_json::to_string_pretty(&sorted)
        .context("Failed to format JSON")?;

    // Determine output path
    let output_path = output.unwrap_or(spec_path);

    // Write formatted JSON
    fs::write(output_path, &formatted)
        .with_context(|| format!("Failed to write to: {}", output_path))?;

    if output_path == spec_path {
        println!(
            "{} Formatted in place",
            "SUCCESS".green().bold()
        );
    } else {
        println!(
            "{} Formatted to: {}",
            "SUCCESS".green().bold(),
            output_path
        );
    }

    Ok(ExitCode::SUCCESS)
}

/// Recursively sort JSON object keys alphabetically
fn sort_json_keys(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            // Collect keys and sort them
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();

            // Create a new map with sorted keys
            let mut sorted_map = serde_json::Map::new();
            for key in keys {
                if let Some(v) = map.get(key) {
                    sorted_map.insert(key.clone(), sort_json_keys(v));
                }
            }
            Value::Object(sorted_map)
        }
        Value::Array(arr) => {
            // Recursively sort keys in array elements
            Value::Array(arr.iter().map(sort_json_keys).collect())
        }
        // Primitives are returned as-is
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_json_keys() {
        let input: Value = serde_json::from_str(r#"{
            "z": 1,
            "a": 2,
            "m": {
                "z": 3,
                "a": 4
            }
        }"#).unwrap();

        let sorted = sort_json_keys(&input);
        let output = serde_json::to_string(&sorted).unwrap();

        // Keys should be sorted: a, m, z for outer, a, z for inner
        assert!(output.contains(r#""a":2"#) || output.contains(r#""a": 2"#));

        // Verify order by checking positions
        let a_pos = output.find("\"a\"").unwrap();
        let m_pos = output.find("\"m\"").unwrap();
        let z_pos = output.find("\"z\"").unwrap();

        assert!(a_pos < m_pos);
        assert!(m_pos < z_pos);
    }
}
