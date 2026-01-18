//! Eval command implementation
//!
//! Evaluates a spec file (JSON or Starlark) and prints the canonical IR JSON to stdout.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::process::ExitCode;

use super::json_output::{
    compile_warnings_to_json, input_error_to_json, EvalOutput, JsonError, JsonWarning,
};
use crate::input::{load_spec, InputError};

/// Run the eval command
///
/// # Arguments
/// * `spec_path` - Path to the spec file (JSON or Starlark)
/// * `pretty` - Whether to pretty-print the output JSON
/// * `json_output` - Whether to output machine-readable JSON diagnostics
///
/// # Returns
/// Exit code: 0 on success, 1 on error
pub fn run(spec_path: &str, pretty: bool, json_output: bool) -> Result<ExitCode> {
    if json_output {
        run_json(spec_path, pretty)
    } else {
        run_human(spec_path, pretty)
    }
}

/// Run eval with human-readable (colored) output
fn run_human(spec_path: &str, pretty: bool) -> Result<ExitCode> {
    let path = Path::new(spec_path);

    // Load the spec
    let result = load_spec(path).map_err(|e| {
        match &e {
            InputError::FileRead { path, .. } => {
                eprintln!(
                    "{} Failed to read file: {}",
                    "error:".red().bold(),
                    path.display()
                );
            }
            InputError::UnknownExtension { extension } => {
                eprintln!(
                    "{} Unknown file extension: {:?}",
                    "error:".red().bold(),
                    extension
                );
            }
            InputError::JsonParse { message } => {
                eprintln!("{} JSON parse error: {}", "error:".red().bold(), message);
            }
            #[cfg(feature = "starlark")]
            InputError::StarlarkCompile { message } => {
                eprintln!(
                    "{} Starlark compilation error: {}",
                    "error:".red().bold(),
                    message
                );
            }
            #[cfg(feature = "starlark")]
            InputError::Timeout { seconds } => {
                eprintln!(
                    "{} Starlark evaluation timed out after {}s",
                    "error:".red().bold(),
                    seconds
                );
            }
            #[cfg(not(feature = "starlark"))]
            InputError::StarlarkNotEnabled => {
                eprintln!(
                    "{} Starlark support is disabled. Rebuild with --features starlark",
                    "error:".red().bold()
                );
            }
            InputError::InvalidSpec { message } => {
                eprintln!("{} Invalid spec: {}", "error:".red().bold(), message);
            }
        }
        anyhow::anyhow!("{}", e)
    })?;

    // Print warnings to stderr
    for warning in &result.warnings {
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

    // Serialize to JSON
    let json = if pretty {
        result
            .spec
            .to_json_pretty()
            .context("Failed to serialize spec to JSON")?
    } else {
        result
            .spec
            .to_json()
            .context("Failed to serialize spec to JSON")?
    };

    // Print to stdout
    println!("{}", json);

    Ok(ExitCode::SUCCESS)
}

/// Run eval with machine-readable JSON output
fn run_json(spec_path: &str, _pretty: bool) -> Result<ExitCode> {
    let path = Path::new(spec_path);

    // Load the spec
    let load_result = load_spec(path);

    let output = match load_result {
        Ok(result) => {
            // Convert warnings
            let warnings: Vec<JsonWarning> = compile_warnings_to_json(&result.warnings);

            // Serialize spec to JSON value
            match serde_json::to_value(&result.spec) {
                Ok(spec_json) => EvalOutput::success(spec_json, result.source_hash, warnings),
                Err(e) => {
                    let error = JsonError::new(
                        super::json_output::error_codes::JSON_SERIALIZE,
                        format!("Failed to serialize spec: {}", e),
                    );
                    EvalOutput::failure(vec![error], warnings)
                }
            }
        }
        Err(e) => {
            let error = input_error_to_json(&e, Some(spec_path));
            EvalOutput::failure(vec![error], vec![])
        }
    };

    // Always pretty-print the JSON output for readability
    let json =
        serde_json::to_string_pretty(&output).expect("EvalOutput serialization should not fail");
    println!("{}", json);

    if output.success {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Spec};

    fn write_spec(dir: &tempfile::TempDir, filename: &str, spec: &Spec) -> std::path::PathBuf {
        let path = dir.path().join(filename);
        std::fs::write(&path, spec.to_json_pretty().unwrap()).unwrap();
        path
    }

    #[test]
    fn eval_json_spec_success() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("eval-test-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        let code = run(spec_path.to_str().unwrap(), false, false).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn eval_nonexistent_file_fails() {
        let result = run("/nonexistent/spec.json", false, false);
        assert!(result.is_err());
    }

    #[test]
    fn eval_invalid_json_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("invalid.json");
        std::fs::write(&spec_path, "{ invalid json }").unwrap();

        let result = run(spec_path.to_str().unwrap(), false, false);
        assert!(result.is_err());
    }

    #[cfg(feature = "starlark")]
    #[test]
    fn eval_starlark_spec_success() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.star");

        let starlark_source = r#"
{
    "spec_version": 1,
    "asset_id": "starlark-eval-test-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        {
            "kind": "primary",
            "format": "wav",
            "path": "sounds/test.wav"
        }
    ]
}
"#;
        std::fs::write(&spec_path, starlark_source).unwrap();

        let code = run(spec_path.to_str().unwrap(), false, false).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[cfg(feature = "starlark")]
    #[test]
    fn eval_starlark_with_variables_success() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.star");

        let starlark_source = r#"
asset_id = "starlark-var-test-01"
seed = 123

{
    "spec_version": 1,
    "asset_id": asset_id,
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": seed,
    "outputs": [
        {
            "kind": "primary",
            "format": "wav",
            "path": "sounds/test.wav"
        }
    ]
}
"#;
        std::fs::write(&spec_path, starlark_source).unwrap();

        let code = run(spec_path.to_str().unwrap(), false, false).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn eval_json_output_success() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("eval-json-test-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        // Run with json=true - should succeed
        let code = run(spec_path.to_str().unwrap(), false, true).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn eval_json_output_failure() {
        // Run with json=true on nonexistent file - should return exit code 1, not error
        let code = run("/nonexistent/spec.json", false, true).unwrap();
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn eval_json_output_parses_correctly() {
        let tmp = tempfile::tempdir().unwrap();

        let spec = Spec::builder("eval-parse-test-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "test.wav"))
            .build();

        let spec_path = write_spec(&tmp, "spec.json", &spec);

        // We can't easily capture stdout in a unit test, but we can at least
        // verify the run_json function produces valid JSON by testing the output type
        let path = Path::new(spec_path.to_str().unwrap());
        let load_result = load_spec(path).unwrap();
        let spec_json = serde_json::to_value(&load_result.spec).unwrap();
        let output = EvalOutput::success(spec_json, load_result.source_hash, vec![]);

        let json_str = serde_json::to_string(&output).unwrap();
        let parsed: EvalOutput = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.success);
        assert!(parsed.result.is_some());
    }
}
