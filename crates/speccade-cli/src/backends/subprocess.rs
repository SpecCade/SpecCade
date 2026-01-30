//! Subprocess-based extension runner.

use speccade_spec::extension::{
    validate_output_manifest, DeterminismLevel, ExtensionError, ExtensionInterface,
    ExtensionManifest, ExtensionOutputManifest,
};
use speccade_spec::{canonical_spec_hash, hash::blake3_hash, Spec};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Configuration for the subprocess runner.
#[derive(Debug, Clone)]
pub struct SubprocessConfig {
    /// Default timeout in seconds.
    pub default_timeout: u64,
    /// Whether to capture stderr.
    pub capture_stderr: bool,
    /// Whether to verify output hashes for Tier 1 extensions.
    pub verify_hashes: bool,
    /// Working directory for subprocess.
    pub working_dir: Option<PathBuf>,
}

impl Default for SubprocessConfig {
    fn default() -> Self {
        Self {
            default_timeout: 300, // 5 minutes
            capture_stderr: true,
            verify_hashes: true,
            working_dir: None,
        }
    }
}

/// Result of running a subprocess extension.
#[derive(Debug)]
pub struct SubprocessResult {
    /// The output manifest from the extension.
    pub manifest: ExtensionOutputManifest,
    /// Actual execution duration.
    pub duration: Duration,
    /// Captured stderr (if any).
    pub stderr: Option<String>,
}

/// Runner for subprocess-based extensions.
#[derive(Debug, Clone)]
pub struct SubprocessRunner {
    config: SubprocessConfig,
}

impl SubprocessRunner {
    /// Creates a new subprocess runner with default configuration.
    pub fn new() -> Self {
        Self {
            config: SubprocessConfig::default(),
        }
    }

    /// Creates a new subprocess runner with custom configuration.
    pub fn with_config(config: SubprocessConfig) -> Self {
        Self { config }
    }

    /// Returns a reference to the configuration.
    pub fn config(&self) -> &SubprocessConfig {
        &self.config
    }

    /// Runs an extension subprocess.
    ///
    /// # Arguments
    /// * `spec` - The spec to generate from
    /// * `extension` - The extension manifest
    /// * `out_dir` - Output directory for generated files
    ///
    /// # Returns
    /// The result of the subprocess execution
    pub fn run(
        &self,
        spec: &Spec,
        extension: &ExtensionManifest,
        out_dir: &Path,
    ) -> Result<SubprocessResult, ExtensionError> {
        // Extract subprocess interface details
        let (executable, args, env, timeout_seconds) = match &extension.interface {
            ExtensionInterface::Subprocess {
                executable,
                args,
                env,
                timeout_seconds,
            } => (executable, args, env, *timeout_seconds),
            ExtensionInterface::Wasm { .. } => {
                return Err(ExtensionError::SpawnFailed(
                    "WASM extensions not yet supported".to_string(),
                ));
            }
        };

        // Create output directory
        std::fs::create_dir_all(out_dir).map_err(|e| {
            ExtensionError::SpawnFailed(format!("Failed to create output directory: {}", e))
        })?;

        // Write spec to temp file
        let spec_path = out_dir.join("input.spec.json");
        let spec_json = spec
            .to_json_pretty()
            .map_err(|e| ExtensionError::SpawnFailed(format!("Failed to serialize spec: {}", e)))?;
        std::fs::write(&spec_path, &spec_json).map_err(|e| {
            ExtensionError::SpawnFailed(format!("Failed to write spec file: {}", e))
        })?;

        // Compute input hash for verification
        let input_hash = canonical_spec_hash(spec).map_err(|e| {
            ExtensionError::SpawnFailed(format!("Failed to compute spec hash: {}", e))
        })?;

        // Build command
        let mut cmd = Command::new(executable);

        // Add standard arguments
        cmd.arg("--spec").arg(&spec_path);
        cmd.arg("--out").arg(out_dir);
        cmd.arg("--seed").arg(spec.seed.to_string());

        // Substitute placeholders in custom args and add them
        for arg in args {
            let substituted = arg
                .replace("{spec_path}", spec_path.to_str().unwrap_or(""))
                .replace("{out_dir}", out_dir.to_str().unwrap_or(""))
                .replace("{seed}", &spec.seed.to_string());
            cmd.arg(substituted);
        }

        // Add environment variables
        for (key, value) in env {
            cmd.env(key, value);
        }

        // Configure I/O
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        if self.config.capture_stderr {
            cmd.stderr(Stdio::piped());
        } else {
            cmd.stderr(Stdio::inherit());
        }

        // Set working directory
        if let Some(ref cwd) = self.config.working_dir {
            cmd.current_dir(cwd);
        }

        // Spawn process
        let start = Instant::now();
        let child = cmd.spawn().map_err(|e| {
            ExtensionError::SpawnFailed(format!("Failed to spawn '{}': {}", executable, e))
        })?;

        // Wait with timeout
        let timeout = Duration::from_secs(timeout_seconds);
        let output = wait_with_timeout(child, timeout)?;
        let duration = start.elapsed();

        // Capture stderr
        let stderr = if self.config.capture_stderr {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        } else {
            None
        };

        // Check exit code
        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            return Err(ExtensionError::NonZeroExit {
                code,
                stderr: stderr.unwrap_or_default(),
            });
        }

        // Read and parse output manifest
        let manifest_path = out_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(ExtensionError::ManifestMissing);
        }

        let manifest_content = std::fs::read_to_string(&manifest_path).map_err(|e| {
            ExtensionError::ManifestInvalid(format!("Failed to read manifest: {}", e))
        })?;

        let manifest: ExtensionOutputManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| ExtensionError::ManifestInvalid(e.to_string()))?;

        // Validate manifest structure
        if let Err(errors) = validate_output_manifest(&manifest) {
            let msg = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(ExtensionError::ManifestValidation(msg));
        }

        // Check for extension-reported errors
        if !manifest.success {
            return Err(ExtensionError::ExtensionReportedError(
                manifest.errors.clone(),
            ));
        }

        // Verify input hash matches
        if manifest.determinism_report.input_hash != input_hash {
            return Err(ExtensionError::InputHashMismatch {
                expected: input_hash,
                actual: manifest.determinism_report.input_hash.clone(),
            });
        }

        // Verify tier matches extension declaration
        if manifest.determinism_report.tier != extension.tier {
            return Err(ExtensionError::TierMismatch {
                declared: manifest.determinism_report.tier,
                expected: extension.tier,
            });
        }

        // Verify output files exist and hashes match (for Tier 1)
        if self.config.verify_hashes && extension.determinism == DeterminismLevel::ByteIdentical {
            self.verify_output_hashes(&manifest, out_dir)?;
        }

        // Clean up spec file
        let _ = std::fs::remove_file(&spec_path);

        Ok(SubprocessResult {
            manifest,
            duration,
            stderr,
        })
    }

    /// Verifies that output files exist and their hashes match.
    fn verify_output_hashes(
        &self,
        manifest: &ExtensionOutputManifest,
        out_dir: &Path,
    ) -> Result<(), ExtensionError> {
        for file in &manifest.output_files {
            let file_path = out_dir.join(&file.path);

            // Check file exists
            if !file_path.exists() {
                return Err(ExtensionError::OutputFileMissing(file_path));
            }

            // Compute actual hash
            let content = std::fs::read(&file_path).map_err(|e| {
                ExtensionError::ManifestInvalid(format!(
                    "Failed to read output file '{}': {}",
                    file.path, e
                ))
            })?;
            let actual_hash = blake3_hash(&content);

            // Compare hashes
            if actual_hash != file.hash {
                return Err(ExtensionError::OutputHashMismatch {
                    path: file_path,
                    expected: file.hash.clone(),
                    actual: actual_hash,
                });
            }
        }

        Ok(())
    }
}

impl Default for SubprocessRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Waits for a child process with timeout.
///
/// On Unix, this uses SIGKILL after timeout.
/// On Windows, this uses TerminateProcess after timeout.
fn wait_with_timeout(
    mut child: std::process::Child,
    timeout: Duration,
) -> Result<std::process::Output, ExtensionError> {
    let start = Instant::now();

    // Poll for completion
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process has exited
                let stdout = child
                    .stdout
                    .take()
                    .map(|mut s| {
                        let mut buf = Vec::new();
                        std::io::Read::read_to_end(&mut s, &mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();

                let stderr = child
                    .stderr
                    .take()
                    .map(|mut s| {
                        let mut buf = Vec::new();
                        std::io::Read::read_to_end(&mut s, &mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();

                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                // Still running
                if start.elapsed() > timeout {
                    // Kill the process
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ExtensionError::Timeout {
                        timeout_seconds: timeout.as_secs(),
                    });
                }
                // Sleep briefly before polling again
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return Err(ExtensionError::SpawnFailed(format!(
                    "Failed to wait for process: {}",
                    e
                )));
            }
        }
    }
}

/// Combines multiple output file hashes into a single hash.
///
/// This is used for the combined `output_hash` in determinism reports.
#[allow(dead_code)] // Public API for extensions
pub fn combine_output_hashes(hashes: &[String]) -> String {
    // Sort hashes for determinism, then concatenate and hash
    let mut sorted = hashes.to_vec();
    sorted.sort();
    let combined = sorted.join("");
    blake3_hash(combined.as_bytes())
}
