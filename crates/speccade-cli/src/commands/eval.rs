//! Eval command implementation
//!
//! Evaluates a spec file (JSON or Starlark) and prints the canonical IR JSON to stdout.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::process::ExitCode;

use crate::input::{load_spec, InputError};

/// Run the eval command
///
/// # Arguments
/// * `spec_path` - Path to the spec file (JSON or Starlark)
/// * `pretty` - Whether to pretty-print the output JSON
///
/// # Returns
/// Exit code: 0 on success, 1 on error
pub fn run(spec_path: &str, pretty: bool) -> Result<ExitCode> {
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

        let code = run(spec_path.to_str().unwrap(), false).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn eval_nonexistent_file_fails() {
        let result = run("/nonexistent/spec.json", false);
        assert!(result.is_err());
    }

    #[test]
    fn eval_invalid_json_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("invalid.json");
        std::fs::write(&spec_path, "{ invalid json }").unwrap();

        let result = run(spec_path.to_str().unwrap(), false);
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

        let code = run(spec_path.to_str().unwrap(), false).unwrap();
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

        let code = run(spec_path.to_str().unwrap(), false).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }
}
