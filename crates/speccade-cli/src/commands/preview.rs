//! Preview command implementation
//!
//! Opens an asset preview (stub for Blender preview).

use anyhow::{Context, Result};
use colored::Colorize;
use speccade_spec::Spec;
use std::fs;
use std::process::ExitCode;

/// Run the preview command
///
/// # Arguments
/// * `spec_path` - Path to the spec JSON file
/// * `out_root` - Output root directory (default: current directory)
///
/// # Returns
/// Exit code: 0 success, 1 error
pub fn run(spec_path: &str, _out_root: Option<&str>) -> Result<ExitCode> {
    // TODO: Implement preview for generated assets (open viewers, or launch Blender for mesh/anim).
    println!("{} {}", "Preview:".cyan().bold(), spec_path);

    // Read and parse spec to get asset type
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path))?;

    let spec = Spec::from_json(&spec_content)
        .with_context(|| format!("Failed to parse spec file: {}", spec_path))?;

    // Preview is currently only planned for Blender-based assets
    println!(
        "\n{} Preview is not yet implemented for asset type '{}'",
        "INFO".yellow().bold(),
        spec.asset_type
    );
    println!(
        "{}",
        "Preview functionality will be available in a future release for mesh and animation assets."
            .dimmed()
    );

    // Return success since this is expected behavior for now
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_stub_returns_success() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.json");
        std::fs::write(
            &spec_path,
            r#"{
	  "spec_version": 1,
	  "asset_id": "test-asset-01",
	  "asset_type": "audio",
	  "license": "CC0-1.0",
	  "seed": 42,
	  "outputs": [{"kind": "primary", "format": "wav", "path": "sounds/test.wav"}]
	}"#,
        )
        .unwrap();

        let code = run(spec_path.to_str().unwrap(), None).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }
}
