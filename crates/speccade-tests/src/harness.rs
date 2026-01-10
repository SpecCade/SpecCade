//! Test harness utilities for running CLI commands and validating outputs.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

use speccade_spec::{Spec, OutputFormat};

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
        // For now, we'll simulate CLI behavior by calling the library directly
        // In a real implementation, you might want to run the actual binary
        let output = Command::new("cargo")
            .args(["run", "-p", "speccade-cli", "--"])
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

    // Check RIFF header
    if data.len() < 44 {
        return Err("WAV file too short (< 44 bytes)".to_string());
    }

    if &data[0..4] != b"RIFF" {
        return Err("Missing RIFF header".to_string());
    }

    if &data[8..12] != b"WAVE" {
        return Err("Missing WAVE format".to_string());
    }

    Ok(())
}

/// Validate a PNG file is properly formed.
pub fn validate_png_file(path: &Path) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Check PNG signature
    let png_signature: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if data.len() < 8 {
        return Err("PNG file too short (< 8 bytes)".to_string());
    }

    if &data[0..8] != png_signature {
        return Err("Missing PNG signature".to_string());
    }

    Ok(())
}

/// Validate an XM tracker module file is properly formed.
pub fn validate_xm_file(path: &Path) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Check XM header
    if data.len() < 60 {
        return Err("XM file too short".to_string());
    }

    if &data[0..17] != b"Extended Module: " {
        return Err("Missing XM header".to_string());
    }

    Ok(())
}

/// Validate an IT tracker module file is properly formed.
pub fn validate_it_file(path: &Path) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Check IT header
    if data.len() < 64 {
        return Err("IT file too short".to_string());
    }

    if &data[0..4] != b"IMPM" {
        return Err("Missing IT header".to_string());
    }

    Ok(())
}

/// Validate a GLB file is properly formed.
pub fn validate_glb_file(path: &Path) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Check GLB magic number
    if data.len() < 12 {
        return Err("GLB file too short (< 12 bytes)".to_string());
    }

    // GLB magic: 0x46546C67 (little-endian "glTF")
    if &data[0..4] != b"glTF" {
        return Err("Missing GLB magic number".to_string());
    }

    // Check version (should be 2)
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if version != 2 {
        return Err(format!("Unexpected GLB version: {} (expected 2)", version));
    }

    Ok(())
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
        OutputFormat::Gltf => validate_glb_file(path), // Same validation as GLB
        OutputFormat::Ogg => Ok(()), // TODO: Implement OGG validation
        OutputFormat::Json => Ok(()), // JSON is text, no binary validation needed
        OutputFormat::Blend => Ok(()), // Blender files are opaque
    }
}

/// Parse a spec file and return the Spec.
pub fn parse_spec_file(path: &Path) -> Result<Spec, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read spec file: {}", e))?;
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
