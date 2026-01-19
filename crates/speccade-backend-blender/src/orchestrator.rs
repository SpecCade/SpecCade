//! Blender subprocess orchestrator.
//!
//! This module handles spawning Blender as a subprocess and managing
//! communication via JSON files.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use crate::error::{BlenderError, BlenderResult};
use crate::metrics::BlenderReport;

const EMBEDDED_ENTRYPOINT_PY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../blender/entrypoint.py"
));

/// Default timeout for Blender execution (5 minutes).
pub const DEFAULT_TIMEOUT_SECS: u64 = 300;

/// Generation mode for the Blender entrypoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationMode {
    /// Static mesh generation.
    StaticMesh,
    /// Skeletal mesh generation.
    SkeletalMesh,
    /// Animation generation (simple keyframes).
    Animation,
    /// Rigged animation generation (IK/rig-aware).
    RiggedAnimation,
}

impl GenerationMode {
    /// Returns the string identifier for this mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            GenerationMode::StaticMesh => "static_mesh",
            GenerationMode::SkeletalMesh => "skeletal_mesh",
            GenerationMode::Animation => "animation",
            GenerationMode::RiggedAnimation => "rigged_animation",
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

struct ResolvedEntrypoint {
    path: PathBuf,
    _tempfile: Option<tempfile::NamedTempFile>,
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

    fn resolve_entrypoint(&self) -> BlenderResult<ResolvedEntrypoint> {
        // Config override first.
        if self.config.entrypoint_path.exists() {
            return Ok(ResolvedEntrypoint {
                path: self.config.entrypoint_path.clone(),
                _tempfile: None,
            });
        }

        // Environment override (fallback).
        if let Ok(path) = std::env::var("SPECCADE_BLENDER_ENTRYPOINT") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(ResolvedEntrypoint {
                    path,
                    _tempfile: None,
                });
            }
            return Err(BlenderError::EntrypointNotFound { path });
        }

        // Last resort: write embedded entrypoint to a temp file.
        let mut file = tempfile::Builder::new()
            .prefix("speccade_blender_entrypoint_")
            .suffix(".py")
            .tempfile()
            .map_err(BlenderError::Io)?;
        file.write_all(EMBEDDED_ENTRYPOINT_PY.as_bytes())
            .map_err(BlenderError::Io)?;
        file.flush().map_err(BlenderError::Io)?;

        Ok(ResolvedEntrypoint {
            path: file.path().to_path_buf(),
            _tempfile: Some(file),
        })
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

        let entrypoint = self.resolve_entrypoint()?;

        // Build the command
        // blender --background --factory-startup --python entrypoint.py -- --mode <mode> --spec <path> --out-root <path> --report <path>
        let mut cmd = Command::new(&blender_path);
        cmd.arg("--background")
            .arg("--factory-startup")
            .arg("--python")
            .arg(&entrypoint.path)
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
            // Only stderr is surfaced today; keep stdout unpiped to reduce the risk of
            // subprocess deadlocks caused by a filled stdout pipe.
            cmd.stdout(Stdio::null()).stderr(Stdio::piped());
        }

        // Spawn the process
        let child = cmd.spawn().map_err(BlenderError::SpawnFailed)?;

        let (status, stderr) =
            wait_with_timeout(child, self.config.timeout, self.config.capture_output)?;

        // Check exit status
        if !status.success() {
            let exit_code = status.code().unwrap_or(-1);
            return Err(BlenderError::process_failed(exit_code, stderr));
        }

        // Read and parse the report
        let report_content =
            std::fs::read_to_string(report_path).map_err(|e| BlenderError::ReadReportFailed {
                path: report_path.to_path_buf(),
                source: e,
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

fn wait_with_timeout(
    mut child: Child,
    timeout: Duration,
    capture_output: bool,
) -> BlenderResult<(ExitStatus, String)> {
    let start = Instant::now();

    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(BlenderError::Timeout {
                        timeout_secs: timeout.as_secs(),
                    });
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(BlenderError::SpawnFailed(e)),
        }
    };

    let stderr = if capture_output {
        let mut buf = String::new();
        if let Some(mut err) = child.stderr.take() {
            let _ = err.read_to_string(&mut buf);
        }
        buf
    } else {
        String::new()
    };

    Ok((status, stderr))
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
        "skeletal_animation.blender_rigged_v1" => Ok(GenerationMode::RiggedAnimation),
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
        assert_eq!(GenerationMode::RiggedAnimation.as_str(), "rigged_animation");
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
        assert_eq!(
            mode_from_recipe_kind("skeletal_animation.blender_rigged_v1").unwrap(),
            GenerationMode::RiggedAnimation
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

    #[test]
    fn test_wait_with_timeout_captures_stderr() {
        let mut cmd = if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", "echo hello 1>&2"]);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", "echo hello 1>&2"]);
            cmd
        };

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let child = cmd.spawn().unwrap();

        let (status, stderr) = wait_with_timeout(child, Duration::from_secs(2), true).unwrap();
        assert!(status.success());
        assert!(stderr.to_lowercase().contains("hello"));
    }

    #[test]
    fn test_resolve_entrypoint_falls_back_to_embedded() {
        // If the user has configured an environment override, don't stomp it.
        if std::env::var_os("SPECCADE_BLENDER_ENTRYPOINT").is_some() {
            eprintln!("SPECCADE_BLENDER_ENTRYPOINT is set; skipping embedded entrypoint test");
            return;
        }

        let config = OrchestratorConfig::with_entrypoint("this/does/not/exist.py");
        let orchestrator = Orchestrator::with_config(config);

        let entrypoint = orchestrator.resolve_entrypoint().unwrap();
        assert!(entrypoint.path.exists());

        let content = std::fs::read_to_string(&entrypoint.path).unwrap();
        assert!(content.contains("SpecCade Blender Entrypoint"));
    }
}
