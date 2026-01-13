//! Test harness utilities for running CLI commands and validating outputs.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use tempfile::TempDir;

use speccade_spec::{OutputFormat, Spec};

use crate::format_validators::{self, GlbInfo, GltfInfo, ItInfo, PngInfo, WavInfo, XmInfo};

// Re-export FormatError for convenience
pub use crate::format_validators::FormatError;

/// Result of running the speccade CLI.
#[derive(Debug)]
pub struct CliResult {
    pub success: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl CliResult {
    /// Create a CliResult from a Command Output.
    pub fn from_output(output: Output) -> Self {
        Self {
            success: output.status.success(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }

    /// Assert that the command succeeded.
    pub fn assert_success(&self) {
        assert!(
            self.success,
            "Command failed with exit code {}.\nstdout: {}\nstderr: {}",
            self.exit_code, self.stdout, self.stderr
        );
    }

    /// Assert that the command failed.
    pub fn assert_failure(&self) {
        assert!(
            !self.success,
            "Expected command to fail, but it succeeded.\nstdout: {}",
            self.stdout
        );
    }
}

/// A test harness for running speccade CLI commands.
pub struct TestHarness {
    /// Working directory for test outputs.
    pub work_dir: TempDir,
}

impl TestHarness {
    /// Create a new test harness.
    pub fn new() -> Self {
        Self {
            work_dir: TempDir::new().expect("Failed to create work dir"),
        }
    }

    /// Get the working directory path.
    pub fn path(&self) -> &Path {
        self.work_dir.path()
    }

    /// Run the speccade CLI with the given arguments.
    ///
    /// Note: This runs the CLI as a library call, not as a subprocess,
    /// which is more reliable for testing.
    pub fn run_cli(&self, args: &[&str]) -> CliResult {
        let manifest_path = speccade_manifest_path();

        let output = Command::new("cargo")
            .args(["run", "--quiet", "--manifest-path"])
            .arg(&manifest_path)
            .args(["-p", "speccade-cli", "--"])
            .args(args)
            .current_dir(self.path())
            .output();

        match output {
            Ok(out) => CliResult::from_output(out),
            Err(e) => CliResult {
                success: false,
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("Failed to run CLI: {}", e),
            },
        }
    }

    /// Validate a spec file using the CLI.
    pub fn validate_spec(&self, spec_path: &Path) -> CliResult {
        self.run_cli(&["validate", "--spec", spec_path.to_str().unwrap()])
    }

    /// Generate assets from a spec file.
    pub fn generate_spec(&self, spec_path: &Path) -> CliResult {
        self.run_cli(&[
            "generate",
            "--spec",
            spec_path.to_str().unwrap(),
            "--out-root",
            self.path().to_str().unwrap(),
        ])
    }

    /// Run migration on a project directory.
    pub fn migrate_project(&self, project_path: &Path) -> CliResult {
        self.run_cli(&["migrate", "--project", project_path.to_str().unwrap()])
    }

    /// Run audit on a project directory.
    pub fn audit_project(&self, project_path: &Path, threshold: f64) -> CliResult {
        self.run_cli(&[
            "migrate",
            "--project",
            project_path.to_str().unwrap(),
            "--audit",
            "--audit-threshold",
            &format!("{:.2}", threshold),
        ])
    }

    /// Copy a spec file to the work directory and return the new path.
    pub fn copy_spec(&self, spec_path: &Path) -> PathBuf {
        let dest = self.path().join(spec_path.file_name().unwrap());
        fs::copy(spec_path, &dest).expect("Failed to copy spec file");
        dest
    }
}

fn speccade_manifest_path() -> PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let manifest_path = manifest_dir.join("..").join("..").join("Cargo.toml");
        manifest_path.canonicalize().unwrap_or(manifest_path)
    })
    .clone()
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if Blender is available in the environment.
pub fn is_blender_available() -> bool {
    Command::new("blender")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Blender tests should run based on environment variable.
pub fn should_run_blender_tests() -> bool {
    std::env::var("SPECCADE_RUN_BLENDER_TESTS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Validate that an output file exists and has the expected format.
pub fn validate_output_exists(out_root: &Path, rel_path: &str) -> bool {
    out_root.join(rel_path).exists()
}

/// Validate a WAV file is properly formed.
pub fn validate_wav_file(path: &Path) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    format_validators::validate_wav(&data)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Validate a WAV file and return detailed information.
pub fn validate_wav_file_info(path: &Path) -> Result<WavInfo, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    format_validators::validate_wav(&data).map_err(|e| e.to_string())
}

/// Validate a PNG file is properly formed.
pub fn validate_png_file(path: &Path) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    format_validators::validate_png(&data)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Validate a PNG file and return detailed information.
pub fn validate_png_file_info(path: &Path) -> Result<PngInfo, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    format_validators::validate_png(&data).map_err(|e| e.to_string())
}

/// Validate an XM tracker module file is properly formed.
pub fn validate_xm_file(path: &Path) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    format_validators::validate_xm(&data)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Validate an XM file and return detailed information.
pub fn validate_xm_file_info(path: &Path) -> Result<XmInfo, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    format_validators::validate_xm(&data).map_err(|e| e.to_string())
}

/// Validate an IT tracker module file is properly formed.
pub fn validate_it_file(path: &Path) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    format_validators::validate_it(&data)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Validate an IT file and return detailed information.
pub fn validate_it_file_info(path: &Path) -> Result<ItInfo, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    format_validators::validate_it(&data).map_err(|e| e.to_string())
}

/// Validate a GLB file is properly formed.
pub fn validate_glb_file(path: &Path) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    format_validators::validate_glb(&data)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Validate a GLB file and return detailed information.
pub fn validate_glb_file_info(path: &Path) -> Result<GlbInfo, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    format_validators::validate_glb(&data).map_err(|e| e.to_string())
}

/// Validate a glTF (JSON) file is properly formed.
pub fn validate_gltf_file(path: &Path) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    format_validators::validate_gltf(&data)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Validate a glTF file and return detailed information.
pub fn validate_gltf_file_info(path: &Path) -> Result<GltfInfo, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    format_validators::validate_gltf(&data).map_err(|e| e.to_string())
}

/// Validate an output file based on its format.
pub fn validate_output_format(path: &Path, format: OutputFormat) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Output file does not exist: {:?}", path));
    }

    match format {
        OutputFormat::Wav => validate_wav_file(path),
        OutputFormat::Png => validate_png_file(path),
        OutputFormat::Xm => validate_xm_file(path),
        OutputFormat::It => validate_it_file(path),
        OutputFormat::Glb => validate_glb_file(path),
        OutputFormat::Gltf => validate_gltf_file(path),
        OutputFormat::Json => Ok(()), // JSON is text, no binary validation needed
    }
}

/// Parse a spec file and return the Spec.
pub fn parse_spec_file(path: &Path) -> Result<Spec, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read spec file: {}", e))?;
    Spec::from_json(&content).map_err(|e| format!("Failed to parse spec: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_creation() {
        let harness = TestHarness::new();
        assert!(harness.path().exists());
    }

    #[test]
    fn test_blender_check() {
        // Just ensure the function doesn't panic
        let _ = is_blender_available();
    }

    #[test]
    fn test_blender_env_check() {
        // Just ensure the function doesn't panic
        let _ = should_run_blender_tests();
    }
}
