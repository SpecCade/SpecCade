//! Blender subprocess orchestrator.
//!
//! This module handles spawning Blender as a subprocess and managing
//! communication via JSON files.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::error::{BlenderError, BlenderResult};
use crate::metrics::BlenderReport;

/// Default timeout for Blender execution (5 minutes).
pub const DEFAULT_TIMEOUT_SECS: u64 = 300;

/// Generation mode for the Blender entrypoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationMode {
    /// Static mesh generation.
    StaticMesh,
    /// Skeletal mesh generation.
    SkeletalMesh,
    /// Animation generation.
    Animation,
}

impl GenerationMode {
    /// Returns the string identifier for this mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            GenerationMode::StaticMesh => "static_mesh",
            GenerationMode::SkeletalMesh => "skeletal_mesh",
            GenerationMode::Animation => "animation",
        }
    }
}

/// Configuration for the Blender orchestrator.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Path to the Blender executable.
    pub blender_path: Option<PathBuf>,
    /// Path to the Python entrypoint script.
    pub entrypoint_path: PathBuf,
    /// Timeout for Blender execution.
    pub timeout: Duration,
    /// Whether to capture Blender's stdout/stderr.
    pub capture_output: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            blender_path: None,
            entrypoint_path: PathBuf::from("blender/entrypoint.py"),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            capture_output: true,
        }
    }
}

impl OrchestratorConfig {
    /// Creates a new config with the given entrypoint path.
    pub fn with_entrypoint(entrypoint_path: impl Into<PathBuf>) -> Self {
        Self {
            entrypoint_path: entrypoint_path.into(),
            ..Default::default()
        }
    }

    /// Sets the Blender executable path.
    pub fn blender_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.blender_path = Some(path.into());
        self
    }

    /// Sets the timeout duration.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the timeout in seconds.
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }
}

/// The Blender subprocess orchestrator.
pub struct Orchestrator {
    config: OrchestratorConfig,
}

impl Orchestrator {
    /// Creates a new orchestrator with default configuration.
    pub fn new() -> Self {
        Self {
            config: OrchestratorConfig::default(),
        }
    }

    /// Creates a new orchestrator with the given configuration.
    pub fn with_config(config: OrchestratorConfig) -> Self {
        Self { config }
    }

    /// Finds the Blender executable path.
    fn find_blender(&self) -> BlenderResult<PathBuf> {
        // Check config override first
        if let Some(ref path) = self.config.blender_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // Check BLENDER_PATH environment variable
        if let Ok(path) = std::env::var("BLENDER_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // Try to find Blender in PATH
        let blender_names = if cfg!(windows) {
            vec!["blender.exe", "blender"]
        } else {
            vec!["blender"]
        };

        for name in blender_names {
            if let Ok(path) = which::which(name) {
                return Ok(path);
            }
        }

        // Try common installation paths
        let common_paths = if cfg!(windows) {
            vec![
                "C:\\Program Files\\Blender Foundation\\Blender 4.0\\blender.exe",
                "C:\\Program Files\\Blender Foundation\\Blender 3.6\\blender.exe",
                "C:\\Program Files\\Blender Foundation\\Blender\\blender.exe",
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                "/Applications/Blender.app/Contents/MacOS/Blender",
                "/Applications/Blender.app/Contents/MacOS/blender",
            ]
        } else {
            vec![
                "/usr/bin/blender",
                "/usr/local/bin/blender",
                "/snap/bin/blender",
            ]
        };

        for path_str in common_paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Ok(path);
            }
        }

        Err(BlenderError::BlenderNotFound)
    }

    /// Runs Blender to generate an asset.
    ///
    /// # Arguments
    ///
    /// * `mode` - The generation mode (static_mesh, skeletal_mesh, animation)
    /// * `spec_path` - Path to the JSON spec file
    /// * `out_root` - Root directory for output files
    /// * `report_path` - Path where Blender should write its report JSON
    pub fn run(
        &self,
        mode: GenerationMode,
        spec_path: &Path,
        out_root: &Path,
        report_path: &Path,
    ) -> BlenderResult<BlenderReport> {
        let blender_path = self.find_blender()?;

        // Verify entrypoint exists
        if !self.config.entrypoint_path.exists() {
            return Err(BlenderError::EntrypointNotFound {
                path: self.config.entrypoint_path.clone(),
            });
        }

        let start = Instant::now();

        // Build the command
        // blender --background --factory-startup --python entrypoint.py -- --mode <mode> --spec <path> --out-root <path> --report <path>
        let mut cmd = Command::new(&blender_path);
        cmd.arg("--background")
            .arg("--factory-startup")
            .arg("--python")
            .arg(&self.config.entrypoint_path)
            .arg("--")
            .arg("--mode")
            .arg(mode.as_str())
            .arg("--spec")
            .arg(spec_path)
            .arg("--out-root")
            .arg(out_root)
            .arg("--report")
            .arg(report_path);

        if self.config.capture_output {
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        }

        // Spawn the process
        let mut child = cmd.spawn().map_err(BlenderError::SpawnFailed)?;

        // Wait with timeout
        let timeout = self.config.timeout;
        let result = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => {
                    if start.elapsed() > timeout {
                        // Kill the process
                        let _ = child.kill();
                        break Err(BlenderError::Timeout {
                            timeout_secs: timeout.as_secs(),
                        });
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => break Err(BlenderError::SpawnFailed(e)),
            }
        };

        let status = result?;

        // Capture output if needed
        let stderr = if self.config.capture_output {
            let output = child.wait_with_output().map_err(BlenderError::SpawnFailed)?;
            String::from_utf8_lossy(&output.stderr).to_string()
        } else {
            String::new()
        };

        // Check exit status
        if !status.success() {
            let exit_code = status.code().unwrap_or(-1);
            return Err(BlenderError::process_failed(exit_code, stderr));
        }

        // Read and parse the report
        let report_content = std::fs::read_to_string(report_path).map_err(|e| {
            BlenderError::ReadReportFailed {
                path: report_path.to_path_buf(),
                source: e,
            }
        })?;

        let report: BlenderReport =
            serde_json::from_str(&report_content).map_err(BlenderError::ParseReportFailed)?;

        // Check if Blender reported an error
        if !report.ok {
            return Err(BlenderError::generation_failed(
                report.error.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        Ok(report)
    }

    /// Runs Blender with a spec provided as a JSON string.
    ///
    /// This creates temporary files for the spec and report, then invokes Blender.
    pub fn run_with_spec_json(
        &self,
        mode: GenerationMode,
        spec_json: &str,
        out_root: &Path,
    ) -> BlenderResult<BlenderReport> {
        // Create temp directory for spec and report
        let temp_dir = tempfile::tempdir().map_err(BlenderError::Io)?;
        let spec_path = temp_dir.path().join("spec.json");
        let report_path = temp_dir.path().join("report.json");

        // Write spec to temp file
        std::fs::write(&spec_path, spec_json).map_err(BlenderError::WriteSpecFailed)?;

        // Run Blender
        self.run(mode, &spec_path, out_root, &report_path)
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to get the recipe kind's generation mode.
pub fn mode_from_recipe_kind(kind: &str) -> BlenderResult<GenerationMode> {
    match kind {
        "static_mesh.blender_primitives_v1" => Ok(GenerationMode::StaticMesh),
        "skeletal_mesh.blender_rigged_mesh_v1" => Ok(GenerationMode::SkeletalMesh),
        "skeletal_animation.blender_clip_v1" => Ok(GenerationMode::Animation),
        _ => Err(BlenderError::InvalidRecipeKind {
            kind: kind.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_mode_as_str() {
        assert_eq!(GenerationMode::StaticMesh.as_str(), "static_mesh");
        assert_eq!(GenerationMode::SkeletalMesh.as_str(), "skeletal_mesh");
        assert_eq!(GenerationMode::Animation.as_str(), "animation");
    }

    #[test]
    fn test_mode_from_recipe_kind() {
        assert_eq!(
            mode_from_recipe_kind("static_mesh.blender_primitives_v1").unwrap(),
            GenerationMode::StaticMesh
        );
        assert_eq!(
            mode_from_recipe_kind("skeletal_mesh.blender_rigged_mesh_v1").unwrap(),
            GenerationMode::SkeletalMesh
        );
        assert_eq!(
            mode_from_recipe_kind("skeletal_animation.blender_clip_v1").unwrap(),
            GenerationMode::Animation
        );

        assert!(mode_from_recipe_kind("invalid.kind").is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = OrchestratorConfig::with_entrypoint("custom/path.py")
            .blender_path("/usr/bin/blender")
            .timeout_secs(600);

        assert_eq!(config.entrypoint_path, PathBuf::from("custom/path.py"));
        assert_eq!(config.blender_path, Some(PathBuf::from("/usr/bin/blender")));
        assert_eq!(config.timeout, Duration::from_secs(600));
    }
}
