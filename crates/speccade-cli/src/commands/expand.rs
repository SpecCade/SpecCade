//! Expand command implementation
//!
//! Expands Pattern IR compose specs into canonical tracker params JSON.
//! Supports both JSON and Starlark input files via `load_spec()`.

use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use super::json_output::{
    compile_warnings_to_json, error_codes, input_error_to_json, ExpandOutput, JsonError,
    JsonWarning,
};
use crate::input::{load_spec, LoadResult};

/// Run the expand command.
///
/// # Arguments
/// * `spec_path` - Path to the spec file (JSON or Starlark)
/// * `output_path` - Optional output file path (default: stdout)
/// * `pretty` - Whether to pretty-print the output JSON
/// * `json_output` - Whether to output machine-readable JSON envelope
///
/// # Returns
/// Exit code: 0 on success, 1 on error
pub fn run(
    spec_path: &str,
    output_path: Option<&str>,
    pretty: bool,
    json_output: bool,
) -> Result<ExitCode> {
    if json_output {
        run_json(spec_path, output_path, pretty)
    } else {
        run_human(spec_path, output_path, pretty)
    }
}

/// Run expand with human-readable (colored) output
fn run_human(spec_path: &str, output_path: Option<&str>, pretty: bool) -> Result<ExitCode> {
    let path = Path::new(spec_path);

    // Load the spec (supports JSON and Starlark)
    let LoadResult {
        spec,
        source_kind,
        source_hash: _,
        warnings: load_warnings,
    } = load_spec(path).with_context(|| format!("Failed to load spec file: {}", spec_path))?;

    // Print any load warnings
    for warning in &load_warnings {
        let location = warning
            .location
            .as_ref()
            .map(|l| format!(" at {}", l))
            .unwrap_or_default();
        eprintln!(
            "{}{}: {}",
            "warning".yellow().bold(),
            location,
            warning.message
        );
    }

    // Get recipe
    let recipe = spec
        .recipe
        .as_ref()
        .with_context(|| "Spec is missing recipe".to_string())?;

    // Check recipe kind and expand
    match recipe.kind.as_str() {
        "music.tracker_song_compose_v1" => {
            let params = recipe
                .as_music_tracker_song_compose()
                .with_context(|| format!("Invalid compose params for {}", recipe.kind))?;
            let expanded = speccade_backend_music::expand_compose(&params, spec.seed)
                .with_context(|| "Compose expansion failed".to_string())?;

            // Serialize to JSON with stable key ordering (serde_json uses BTreeMap internally
            // for struct fields, and TrackerSongParams uses Vec for ordered collections)
            let json = if pretty {
                serde_json::to_string_pretty(&expanded)?
            } else {
                serde_json::to_string(&expanded)?
            };

            // Output to file or stdout
            match output_path {
                Some(out_path) => {
                    fs::write(out_path, &json)
                        .with_context(|| format!("Failed to write output file: {}", out_path))?;
                    eprintln!(
                        "{} Expanded {} spec to: {}",
                        "success:".green().bold(),
                        source_kind.as_str(),
                        out_path
                    );
                }
                None => {
                    println!("{}", json);
                }
            }

            Ok(ExitCode::SUCCESS)
        }
        _ => bail!(
            "expand is only supported for music.tracker_song_compose_v1 (got {})",
            recipe.kind
        ),
    }
}

/// Run expand with machine-readable JSON output
fn run_json(spec_path: &str, output_path: Option<&str>, pretty: bool) -> Result<ExitCode> {
    let path = Path::new(spec_path);

    // Load the spec (supports JSON and Starlark)
    let load_result = load_spec(path);

    let (spec, source_kind, source_hash, load_warnings) = match load_result {
        Ok(LoadResult {
            spec,
            source_kind,
            source_hash,
            warnings,
        }) => (spec, source_kind, source_hash, warnings),
        Err(e) => {
            let error = input_error_to_json(&e, Some(spec_path));
            let output = ExpandOutput::failure(vec![error], vec![]);
            print_json_output(&output, output_path, pretty)?;
            return Ok(ExitCode::from(1));
        }
    };

    // Convert load warnings
    let all_warnings: Vec<JsonWarning> = compile_warnings_to_json(&load_warnings);

    // Get recipe
    let recipe = match spec.recipe.as_ref() {
        Some(r) => r,
        None => {
            let error = JsonError::new(error_codes::INVALID_SPEC, "Spec is missing recipe");
            let output = ExpandOutput::failure(vec![error], all_warnings);
            print_json_output(&output, output_path, pretty)?;
            return Ok(ExitCode::from(1));
        }
    };

    // Check recipe kind and expand
    match recipe.kind.as_str() {
        "music.tracker_song_compose_v1" => {
            let params = match recipe.as_music_tracker_song_compose() {
                Ok(p) => p,
                Err(e) => {
                    let error = JsonError::new(
                        error_codes::INVALID_SPEC,
                        format!("Invalid compose params for {}: {}", recipe.kind, e),
                    );
                    let output = ExpandOutput::failure(vec![error], all_warnings);
                    print_json_output(&output, output_path, pretty)?;
                    return Ok(ExitCode::from(1));
                }
            };

            let expanded = match speccade_backend_music::expand_compose(&params, spec.seed) {
                Ok(e) => e,
                Err(e) => {
                    let error = JsonError::new(
                        error_codes::GENERATION_ERROR,
                        format!("Compose expansion failed: {}", e),
                    );
                    let output = ExpandOutput::failure(vec![error], all_warnings);
                    print_json_output(&output, output_path, pretty)?;
                    return Ok(ExitCode::from(1));
                }
            };

            // Convert expanded params to JSON value
            let expanded_json = match serde_json::to_value(&expanded) {
                Ok(v) => v,
                Err(e) => {
                    let error = JsonError::new(
                        error_codes::JSON_SERIALIZE,
                        format!("Failed to serialize expanded params: {}", e),
                    );
                    let output = ExpandOutput::failure(vec![error], all_warnings);
                    print_json_output(&output, output_path, pretty)?;
                    return Ok(ExitCode::from(1));
                }
            };

            let output = ExpandOutput::success(
                expanded_json,
                source_hash,
                source_kind.as_str().to_string(),
                all_warnings,
            );
            print_json_output(&output, output_path, pretty)?;
            Ok(ExitCode::SUCCESS)
        }
        _ => {
            let error = JsonError::new(
                error_codes::INVALID_SPEC,
                format!(
                    "expand is only supported for music.tracker_song_compose_v1 (got {})",
                    recipe.kind
                ),
            );
            let output = ExpandOutput::failure(vec![error], all_warnings);
            print_json_output(&output, output_path, pretty)?;
            Ok(ExitCode::from(1))
        }
    }
}

/// Print JSON output to file or stdout
fn print_json_output(output: &ExpandOutput, output_path: Option<&str>, pretty: bool) -> Result<()> {
    let json = if pretty {
        serde_json::to_string_pretty(output)?
    } else {
        serde_json::to_string(output)?
    };

    match output_path {
        Some(out_path) => {
            let mut file = fs::File::create(out_path)
                .with_context(|| format!("Failed to create output file: {}", out_path))?;
            writeln!(file, "{}", json)
                .with_context(|| format!("Failed to write output file: {}", out_path))?;
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Recipe, Spec};

    fn write_spec(dir: &tempfile::TempDir, filename: &str, spec: &Spec) -> std::path::PathBuf {
        let path = dir.path().join(filename);
        std::fs::write(&path, spec.to_json_pretty().unwrap()).unwrap();
        path
    }

    fn compose_recipe() -> Recipe {
        let params = serde_json::json!({
            "format": "xm",
            "bpm": 120,
            "speed": 6,
            "channels": 4,
            "instruments": [{
                "name": "lead",
                "base_note": "C4",
                "synthesis": { "type": "sine" }
            }],
            "defs": {},
            "patterns": {
                "intro": {
                    "rows": 16,
                    "program": {
                        "op": "emit",
                        "at": { "op": "range", "start": 0, "step": 4, "count": 4 },
                        "cell": { "channel": 0, "note": "C4", "inst": 0, "vol": 64 }
                    }
                }
            },
            "arrangement": [{ "pattern": "intro", "repeat": 1 }]
        });
        Recipe::new("music.tracker_song_compose_v1", params)
    }

    #[test]
    fn expand_json_spec_success() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("expand-test-01", AssetType::Music)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Xm, "test.xm"))
            .recipe(compose_recipe())
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        let code = run(spec_path.to_str().unwrap(), None, true, false).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn expand_to_output_file() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("expand-test-02", AssetType::Music)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Xm, "test.xm"))
            .recipe(compose_recipe())
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);
        let out_path = tmp.path().join("expanded.json");

        let code = run(
            spec_path.to_str().unwrap(),
            Some(out_path.to_str().unwrap()),
            true,
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        // Verify output file exists and is valid JSON
        let content = std::fs::read_to_string(&out_path).unwrap();
        let _: serde_json::Value = serde_json::from_str(&content).unwrap();
    }

    #[test]
    fn expand_compact_output() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("expand-test-03", AssetType::Music)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Xm, "test.xm"))
            .recipe(compose_recipe())
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);
        let out_path = tmp.path().join("expanded.json");

        let code = run(
            spec_path.to_str().unwrap(),
            Some(out_path.to_str().unwrap()),
            false, // compact
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        // Verify output is compact (single line)
        let content = std::fs::read_to_string(&out_path).unwrap();
        assert!(!content.contains('\n') || content.lines().count() <= 1);
    }

    #[test]
    fn expand_json_output_mode() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("expand-test-04", AssetType::Music)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Xm, "test.xm"))
            .recipe(compose_recipe())
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        let code = run(spec_path.to_str().unwrap(), None, true, true).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn expand_missing_recipe_fails() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("expand-test-05", AssetType::Music)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Xm, "test.xm"))
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        let result = run(spec_path.to_str().unwrap(), None, true, false);
        assert!(result.is_err());
    }

    #[test]
    fn expand_json_output_missing_recipe() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("expand-test-06", AssetType::Music)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Xm, "test.xm"))
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        // In JSON mode, errors don't propagate as Result::Err
        let code = run(spec_path.to_str().unwrap(), None, true, true).unwrap();
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn expand_wrong_recipe_kind_fails() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("expand-test-07", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .recipe(Recipe::new("audio_v1", serde_json::json!({})))
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        let result = run(spec_path.to_str().unwrap(), None, true, false);
        assert!(result.is_err());
    }

    #[test]
    fn expand_nonexistent_file_fails() {
        let result = run("/nonexistent/spec.json", None, true, false);
        assert!(result.is_err());
    }

    #[test]
    fn expand_json_output_nonexistent_file() {
        let code = run("/nonexistent/spec.json", None, true, true).unwrap();
        assert_eq!(code, ExitCode::from(1));
    }

    #[cfg(feature = "starlark")]
    #[test]
    fn expand_starlark_spec_success() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.star");

        // Create a minimal Starlark spec with compose recipe
        let starlark_source = r#"
{
    "spec_version": 1,
    "asset_id": "starlark-expand-test-01",
    "asset_type": "music",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        { "kind": "primary", "format": "xm", "path": "test.xm" }
    ],
    "recipe": {
        "kind": "music.tracker_song_compose_v1",
        "params": {
            "format": "xm",
            "bpm": 120,
            "speed": 6,
            "channels": 4,
            "instruments": [{
                "name": "lead",
                "base_note": "C4",
                "synthesis": { "type": "sine" }
            }],
            "defs": {},
            "patterns": {
                "intro": {
                    "rows": 16,
                    "program": {
                        "op": "emit",
                        "at": { "op": "range", "start": 0, "step": 4, "count": 4 },
                        "cell": { "channel": 0, "note": "C4", "inst": 0, "vol": 64 }
                    }
                }
            },
            "arrangement": [{ "pattern": "intro", "repeat": 1 }]
        }
    }
}
"#;
        std::fs::write(&spec_path, starlark_source).unwrap();

        let code = run(spec_path.to_str().unwrap(), None, true, false).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn expand_output_serialization() {
        use super::super::json_output::ExpandOutput;

        let expanded_json = serde_json::json!({"patterns": {}});
        let output = ExpandOutput::success(
            expanded_json,
            "abc123".to_string(),
            "json".to_string(),
            vec![],
        );

        let json_str = serde_json::to_string(&output).unwrap();
        let parsed: ExpandOutput = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.success);
        assert!(parsed.expanded.is_some());
        assert_eq!(parsed.source_hash, Some("abc123".to_string()));
    }

    #[test]
    fn expand_output_failure_serialization() {
        use super::super::json_output::{ExpandOutput, JsonError};

        let error = JsonError::new("CLI_006", "test error");
        let output = ExpandOutput::failure(vec![error], vec![]);

        let json_str = serde_json::to_string(&output).unwrap();
        let parsed: ExpandOutput = serde_json::from_str(&json_str).unwrap();
        assert!(!parsed.success);
        assert!(parsed.expanded.is_none());
        assert_eq!(parsed.errors.len(), 1);
    }
}
