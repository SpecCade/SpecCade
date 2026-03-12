//! Pipeline command implementation.
//!
//! Runs repeatable assets-as-code validation/generation workflows across a
//! corpus of specs using existing command modules directly.

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use walkdir::WalkDir;

use super::{contract, coverage, generate, preview_grid, validate, validate_asset};
use crate::dispatch::{get_backend_tier, is_backend_available};
use crate::input::load_spec;

const SPEC_EXTENSIONS: &[&str] = &["json", "star", "bzl"];

/// Run the pipeline command.
#[allow(clippy::too_many_arguments)]
pub fn run(
    profile_name: &str,
    spec_dir: Option<&str>,
    out_root: Option<&str>,
    cycles: u32,
    sleep_seconds: u64,
    include_blender: bool,
    max_parallel: Option<usize>,
    deep_gate: bool,
    checkpoint_staged: bool,
    checkpoint_prefix: Option<&str>,
    json_output: bool,
) -> Result<ExitCode> {
    run_with_options(
        profile_name,
        spec_dir,
        out_root,
        cycles,
        sleep_seconds,
        include_blender,
        max_parallel,
        deep_gate,
        checkpoint_staged,
        checkpoint_prefix,
        json_output,
    )
}

/// Run the pipeline command with optional deep-gate repo stages.
#[allow(clippy::too_many_arguments)]
pub fn run_with_options(
    profile_name: &str,
    spec_dir: Option<&str>,
    out_root: Option<&str>,
    cycles: u32,
    sleep_seconds: u64,
    include_blender: bool,
    max_parallel: Option<usize>,
    deep_gate: bool,
    checkpoint_staged: bool,
    checkpoint_prefix: Option<&str>,
    json_output: bool,
) -> Result<ExitCode> {
    if cycles == 0 {
        anyhow::bail!("cycles must be >= 1");
    }

    let profile = PipelineProfile::parse(profile_name)?;
    let spec_dir = spec_dir.unwrap_or("./specs");
    let out_root = out_root.unwrap_or("./test-outputs");
    let run_id = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let run_root = PathBuf::from("target")
        .join("pipeline")
        .join(profile.as_str())
        .join(&run_id);
    fs::create_dir_all(&run_root).with_context(|| {
        format!(
            "failed to create pipeline run directory {}",
            run_root.display()
        )
    })?;

    let checkpoint_prefix = checkpoint_prefix.unwrap_or("pipeline checkpoint");
    let started_at = Utc::now();
    let start = Instant::now();
    let policy = ExecutionPolicy::for_profile(profile, max_parallel, include_blender);
    let specs = discover_specs(Path::new(spec_dir), profile, include_blender)?;

    let mut cycle_reports = Vec::new();
    let mut had_failures = false;
    let mut checkpoint_commits = Vec::new();

    for cycle in 1..=cycles {
        let cycle_result = run_cycle(
            profile,
            cycle,
            &run_root,
            &specs,
            Path::new(out_root),
            &policy,
            checkpoint_staged,
            checkpoint_prefix,
            deep_gate,
        )?;
        had_failures |= !cycle_result.success;
        if let Some(commit) = cycle_result.checkpoint_commit.clone() {
            checkpoint_commits.push(commit);
        }
        cycle_reports.push(cycle_result);

        if cycle < cycles && sleep_seconds > 0 {
            thread::sleep(Duration::from_secs(sleep_seconds));
        }
    }

    let report = PipelineReport {
        schema_version: 1,
        run_id,
        profile: profile.as_str().to_string(),
        started_at: started_at.to_rfc3339(),
        duration_seconds: start.elapsed().as_secs_f64(),
        spec_dir: normalize_path(Path::new(spec_dir)),
        out_root: normalize_path(Path::new(out_root)),
        include_blender,
        deep_gate,
        requested_cycles: cycles,
        sleep_seconds,
        execution_policy: policy,
        discovered_specs: specs,
        cycles: cycle_reports,
        checkpoint_commits,
        success: !had_failures,
    };

    let report_path = run_root.join("pipeline-report.json");
    write_json_file(&report_path, &report)?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human_summary(&report, &report_path);
    }

    Ok(if report.success {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PipelineProfile {
    Tier1Fast,
    AllFast,
    AllFull,
    Tier2Validate,
    Music,
}

impl PipelineProfile {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "tier1-fast" => Ok(Self::Tier1Fast),
            "all-fast" => Ok(Self::AllFast),
            "all-full" => Ok(Self::AllFull),
            "tier2-validate" => Ok(Self::Tier2Validate),
            "music" => Ok(Self::Music),
            _ => anyhow::bail!(
                "unknown pipeline profile '{}' (expected tier1-fast, all-fast, all-full, tier2-validate, or music)",
                value
            ),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Tier1Fast => "tier1-fast",
            Self::AllFast => "all-fast",
            Self::AllFull => "all-full",
            Self::Tier2Validate => "tier2-validate",
            Self::Music => "music",
        }
    }

    fn all_stages(self, deep_gate: bool) -> Vec<PipelineStage> {
        match self {
            Self::Tier1Fast => vec![PipelineStage::Validate, PipelineStage::Generate],
            Self::AllFast => vec![
                PipelineStage::Validate,
                PipelineStage::Generate,
                PipelineStage::PreviewGrid,
            ],
            Self::AllFull => vec![
                PipelineStage::Validate,
                PipelineStage::Generate,
                PipelineStage::PreviewGrid,
                PipelineStage::ValidateAsset,
                PipelineStage::CoverageReport,
                PipelineStage::ContractVerify,
            ],
            Self::Tier2Validate => vec![
                PipelineStage::Validate,
                PipelineStage::Generate,
                PipelineStage::PreviewGrid,
                PipelineStage::ValidateAsset,
            ],
            Self::Music => {
                let mut stages = vec![
                    PipelineStage::Validate,
                    PipelineStage::Generate,
                    PipelineStage::MusicSpecValidationTests,
                    PipelineStage::MusicBackendTests,
                    PipelineStage::CliTests,
                    PipelineStage::MusicParityTests,
                    PipelineStage::MusicComposeTests,
                ];
                if deep_gate {
                    stages.push(PipelineStage::MusicExternalConformanceTests);
                    stages.push(PipelineStage::GoldenHashVerification);
                }
                stages
            }
        }
    }

    fn spec_stages(self, deep_gate: bool) -> Vec<PipelineStage> {
        self.all_stages(deep_gate)
            .into_iter()
            .filter(|stage| stage.is_spec_stage())
            .collect()
    }

    fn repo_stages(self, deep_gate: bool) -> Vec<PipelineStage> {
        self.all_stages(deep_gate)
            .into_iter()
            .filter(|stage| !stage.is_spec_stage())
            .collect()
    }

    fn includes_spec(self, spec: &DiscoveredSpec, include_blender: bool) -> bool {
        match self {
            Self::Tier1Fast => spec.backend_tier != Some(2),
            Self::AllFast | Self::AllFull => include_blender || spec.backend_tier != Some(2),
            Self::Tier2Validate => spec.backend_tier == Some(2) && include_blender,
            Self::Music => spec.asset_type.as_deref() == Some("music"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum PipelineStage {
    Validate,
    Generate,
    PreviewGrid,
    ValidateAsset,
    CoverageReport,
    ContractVerify,
    MusicSpecValidationTests,
    MusicBackendTests,
    CliTests,
    MusicParityTests,
    MusicComposeTests,
    MusicExternalConformanceTests,
    GoldenHashVerification,
}

impl PipelineStage {
    fn is_spec_stage(self) -> bool {
        !matches!(
            self,
            Self::CoverageReport
                | Self::ContractVerify
                | Self::MusicSpecValidationTests
                | Self::MusicBackendTests
                | Self::CliTests
                | Self::MusicParityTests
                | Self::MusicComposeTests
                | Self::MusicExternalConformanceTests
                | Self::GoldenHashVerification
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionPolicy {
    requested_max_parallel: Option<usize>,
    effective_tier1_parallel: usize,
    effective_tier2_parallel: usize,
    include_blender: bool,
    scheduling_mode: String,
}

impl ExecutionPolicy {
    fn for_profile(
        profile: PipelineProfile,
        requested_max_parallel: Option<usize>,
        include_blender: bool,
    ) -> Self {
        let available = std::thread::available_parallelism()
            .map(|value| value.get())
            .unwrap_or(1);
        let requested =
            requested_max_parallel.unwrap_or_else(|| available.saturating_sub(1).clamp(1, 8));
        let tier1 = match profile {
            PipelineProfile::Tier2Validate => 1,
            _ => requested.max(1),
        };

        Self {
            requested_max_parallel,
            effective_tier1_parallel: tier1,
            effective_tier2_parallel: if include_blender { 1 } else { 0 },
            include_blender,
            scheduling_mode: "deterministic_sequential_v1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredSpec {
    path: String,
    file_stem: String,
    asset_id: Option<String>,
    asset_type: Option<String>,
    recipe_kind: Option<String>,
    backend_tier: Option<u8>,
    backend_available: bool,
    skipped_reason: Option<String>,
    discovery_error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PipelineReport {
    schema_version: u32,
    run_id: String,
    profile: String,
    started_at: String,
    duration_seconds: f64,
    spec_dir: String,
    out_root: String,
    include_blender: bool,
    deep_gate: bool,
    requested_cycles: u32,
    sleep_seconds: u64,
    execution_policy: ExecutionPolicy,
    discovered_specs: Vec<DiscoveredSpec>,
    cycles: Vec<PipelineCycleResult>,
    checkpoint_commits: Vec<String>,
    success: bool,
}

#[derive(Debug, Serialize)]
pub struct PipelineCycleResult {
    cycle: u32,
    started_at: String,
    duration_seconds: f64,
    spec_results: Vec<PipelineSpecResult>,
    repo_stage_results: Vec<PipelineStageResult>,
    checkpoint_commit: Option<String>,
    success: bool,
}

#[derive(Debug, Serialize)]
pub struct PipelineSpecResult {
    spec_path: String,
    asset_id: Option<String>,
    asset_type: Option<String>,
    recipe_kind: Option<String>,
    backend_tier: Option<u8>,
    stages: Vec<PipelineStageResult>,
    success: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineStageResult {
    stage: PipelineStage,
    scope: String,
    success: bool,
    skipped: bool,
    exit_code: Option<u8>,
    duration_ms: u64,
    artifact_path: Option<String>,
    message: String,
}

fn run_cycle(
    profile: PipelineProfile,
    cycle: u32,
    run_root: &Path,
    specs: &[DiscoveredSpec],
    out_root: &Path,
    policy: &ExecutionPolicy,
    checkpoint_staged: bool,
    checkpoint_prefix: &str,
    deep_gate: bool,
) -> Result<PipelineCycleResult> {
    let cycle_root = run_root.join(format!("cycle-{cycle:02}"));
    fs::create_dir_all(&cycle_root)?;

    let cycle_started_at = Utc::now();
    let cycle_start = Instant::now();
    let mut log_lines = Vec::new();
    log_lines.push(format!("profile={}", profile.as_str()));
    log_lines.push(format!("deep_gate={deep_gate}"));
    log_lines.push(format!(
        "execution_policy=tier1:{} tier2:{} mode:{}",
        policy.effective_tier1_parallel, policy.effective_tier2_parallel, policy.scheduling_mode
    ));

    let mut spec_results = Vec::new();
    let mut all_stage_results = Vec::new();
    let mut success = true;

    for spec in specs {
        if spec.skipped_reason.is_some() {
            continue;
        }
        let spec_result = execute_spec(profile, spec, out_root, &cycle_root, deep_gate)?;
        success &= spec_result.success;
        log_lines.push(format!(
            "spec={} success={} stages={}",
            spec.path,
            spec_result.success,
            spec_result.stages.len()
        ));
        all_stage_results.extend(spec_result.stages.iter().cloned());
        spec_results.push(spec_result);
    }

    let mut repo_stage_results = Vec::new();
    for stage in profile.repo_stages(deep_gate) {
        let stage_result = execute_repo_stage(stage, &cycle_root)?;
        success &= stage_result.success || stage_result.skipped;
        log_lines.push(format!(
            "repo-stage={:?} success={} skipped={} message={}",
            stage, stage_result.success, stage_result.skipped, stage_result.message
        ));
        all_stage_results.push(stage_result.clone());
        repo_stage_results.push(stage_result);
    }

    let checkpoint_commit = if checkpoint_staged && success {
        let commit = maybe_checkpoint_commit(cycle, checkpoint_prefix)?;
        if let Some(ref commit_msg) = commit {
            log_lines.push(format!("checkpoint_commit={commit_msg}"));
        }
        commit
    } else {
        None
    };

    let log_path = cycle_root.join("cycle.log");
    fs::write(&log_path, log_lines.join("\n"))?;

    let results_path = cycle_root.join("results.jsonl");
    write_json_lines_file(&results_path, all_stage_results.iter())?;

    let result = PipelineCycleResult {
        cycle,
        started_at: cycle_started_at.to_rfc3339(),
        duration_seconds: cycle_start.elapsed().as_secs_f64(),
        spec_results,
        repo_stage_results,
        checkpoint_commit,
        success,
    };

    let cycle_report_path = cycle_root.join("cycle-report.json");
    write_json_file(&cycle_report_path, &result)?;

    Ok(result)
}

fn execute_spec(
    profile: PipelineProfile,
    spec: &DiscoveredSpec,
    out_root: &Path,
    cycle_root: &Path,
    deep_gate: bool,
) -> Result<PipelineSpecResult> {
    let mut stages = Vec::new();
    let mut success = spec.discovery_error.is_none();

    for stage in profile.spec_stages(deep_gate) {
        let result = execute_spec_stage(stage, spec, out_root, cycle_root)?;
        success &= result.success || result.skipped;
        if !result.success && !result.skipped {
            stages.push(result);
            break;
        }
        stages.push(result);
    }

    Ok(PipelineSpecResult {
        spec_path: spec.path.clone(),
        asset_id: spec.asset_id.clone(),
        asset_type: spec.asset_type.clone(),
        recipe_kind: spec.recipe_kind.clone(),
        backend_tier: spec.backend_tier,
        stages,
        success,
    })
}

fn execute_spec_stage(
    stage: PipelineStage,
    spec: &DiscoveredSpec,
    out_root: &Path,
    cycle_root: &Path,
) -> Result<PipelineStageResult> {
    let start = Instant::now();
    let spec_path = PathBuf::from(&spec.path);
    let spec_key = safe_name(spec.asset_id.as_deref().unwrap_or(&spec.file_stem));

    if let Some(reason) = spec.skipped_reason.as_ref() {
        return Ok(PipelineStageResult {
            stage,
            scope: spec.path.clone(),
            success: false,
            skipped: true,
            exit_code: None,
            duration_ms: 0,
            artifact_path: None,
            message: reason.clone(),
        });
    }

    if let Some(error) = spec.discovery_error.as_ref() {
        return Ok(PipelineStageResult {
            stage,
            scope: spec.path.clone(),
            success: false,
            skipped: false,
            exit_code: Some(1),
            duration_ms: 0,
            artifact_path: None,
            message: format!("discovery failed: {error}"),
        });
    }

    if spec.backend_tier == Some(2) && !spec.backend_available {
        return Ok(PipelineStageResult {
            stage,
            scope: spec.path.clone(),
            success: false,
            skipped: true,
            exit_code: None,
            duration_ms: 0,
            artifact_path: None,
            message: "tier 2 backend unavailable on this machine".to_string(),
        });
    }

    let (exit, artifact_path, message) = match stage {
        PipelineStage::Validate => (
            validate::run(&spec.path, false, None, false)?,
            None,
            "validate completed".to_string(),
        ),
        PipelineStage::Generate => (
            generate::run(
                &spec.path,
                Some(out_root.to_str().unwrap_or(".")),
                false,
                None,
                false,
                None,
                false,
                false,
                None,
                None,
                None,
                false,
            )?,
            None,
            "generate completed".to_string(),
        ),
        PipelineStage::PreviewGrid => {
            if spec.backend_tier != Some(2) {
                return Ok(PipelineStageResult {
                    stage,
                    scope: spec.path.clone(),
                    success: false,
                    skipped: true,
                    exit_code: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                    artifact_path: None,
                    message: "preview-grid only applies to tier 2 specs".to_string(),
                });
            }
            let grid_path = cycle_root
                .join("preview-grid")
                .join(format!("{spec_key}.png"));
            if let Some(parent) = grid_path.parent() {
                fs::create_dir_all(parent)?;
            }
            (
                preview_grid::run(
                    &spec.path,
                    Some(grid_path.to_str().unwrap_or_default()),
                    256,
                )?,
                Some(normalize_path(&grid_path)),
                "preview-grid completed".to_string(),
            )
        }
        PipelineStage::ValidateAsset => {
            if spec.backend_tier != Some(2) {
                return Ok(PipelineStageResult {
                    stage,
                    scope: spec.path.clone(),
                    success: false,
                    skipped: true,
                    exit_code: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                    artifact_path: None,
                    message: "validate-asset only applies to tier 2 specs".to_string(),
                });
            }
            let validation_root = cycle_root.join("validate-asset").join(&spec_key);
            fs::create_dir_all(&validation_root)?;
            (
                validate_asset::run(
                    &spec.path,
                    Some(validation_root.to_str().unwrap_or_default()),
                    true,
                )?,
                Some(normalize_path(&validation_root)),
                "validate-asset completed".to_string(),
            )
        }
        PipelineStage::CoverageReport
        | PipelineStage::ContractVerify
        | PipelineStage::MusicSpecValidationTests
        | PipelineStage::MusicBackendTests
        | PipelineStage::CliTests
        | PipelineStage::MusicParityTests
        | PipelineStage::MusicComposeTests
        | PipelineStage::MusicExternalConformanceTests
        | PipelineStage::GoldenHashVerification => {
            anyhow::bail!("execute_spec_stage called with repo-wide stage {stage:?}");
        }
    };

    let exit_code = Some(exit_code_value(exit));
    Ok(PipelineStageResult {
        stage,
        scope: normalize_path(&spec_path),
        success: exit == ExitCode::SUCCESS,
        skipped: false,
        exit_code,
        duration_ms: start.elapsed().as_millis() as u64,
        artifact_path,
        message,
    })
}

fn execute_repo_stage(stage: PipelineStage, cycle_root: &Path) -> Result<PipelineStageResult> {
    let start = Instant::now();
    match stage {
        PipelineStage::CoverageReport => {
            let artifact = cycle_root.join("coverage-report.yaml");
            let exit = coverage::run_generate(false, Some(artifact.to_str().unwrap_or_default()))?;
            Ok(PipelineStageResult {
                stage,
                scope: "repo".to_string(),
                success: exit == ExitCode::SUCCESS,
                skipped: false,
                exit_code: Some(exit_code_value(exit)),
                duration_ms: start.elapsed().as_millis() as u64,
                artifact_path: Some(normalize_path(&artifact)),
                message: "coverage report generated".to_string(),
            })
        }
        PipelineStage::ContractVerify => {
            let artifact = cycle_root.join("contract-report.json");
            let exit =
                contract::run_verify(None, Some(artifact.to_str().unwrap_or_default()), false)?;
            Ok(PipelineStageResult {
                stage,
                scope: "repo".to_string(),
                success: exit == ExitCode::SUCCESS,
                skipped: false,
                exit_code: Some(exit_code_value(exit)),
                duration_ms: start.elapsed().as_millis() as u64,
                artifact_path: Some(normalize_path(&artifact)),
                message: "contract verify completed".to_string(),
            })
        }
        PipelineStage::MusicSpecValidationTests
        | PipelineStage::MusicBackendTests
        | PipelineStage::CliTests
        | PipelineStage::MusicParityTests
        | PipelineStage::MusicComposeTests
        | PipelineStage::MusicExternalConformanceTests
        | PipelineStage::GoldenHashVerification => {
            execute_repo_test_stage(stage, cycle_root, start)
        }
        _ => anyhow::bail!("execute_repo_stage called with per-spec stage {stage:?}"),
    }
}

fn execute_repo_test_stage(
    stage: PipelineStage,
    cycle_root: &Path,
    start: Instant,
) -> Result<PipelineStageResult> {
    let planned = plan_repo_test_command(stage)?;
    let logs_root = cycle_root.join("repo-stage-logs");
    fs::create_dir_all(&logs_root)?;
    let log_path = logs_root.join(format!("{}.log", planned.log_stem));
    let log_file = File::create(&log_path)
        .with_context(|| format!("failed to create repo-stage log {}", log_path.display()))?;

    let mut command = Command::new(planned.program);
    command.args(planned.args.iter().copied());
    command.stdout(Stdio::from(log_file.try_clone()?));
    command.stderr(Stdio::from(log_file.try_clone()?));

    let status = command.status().with_context(|| {
        format!(
            "failed to run repo-stage command `{}` for {:?}",
            planned.program, stage
        )
    })?;
    drop(log_file);

    Ok(PipelineStageResult {
        stage,
        scope: "repo".to_string(),
        success: status.success(),
        skipped: false,
        exit_code: Some(if status.success() { 0 } else { 1 }),
        duration_ms: start.elapsed().as_millis() as u64,
        artifact_path: Some(normalize_path(&log_path)),
        message: planned.success_message.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RepoStageCommandPlan {
    program: &'static str,
    args: &'static [&'static str],
    log_stem: &'static str,
    success_message: &'static str,
}

fn plan_repo_test_command(stage: PipelineStage) -> Result<RepoStageCommandPlan> {
    let plan = match stage {
        PipelineStage::MusicSpecValidationTests => RepoStageCommandPlan {
            program: "cargo",
            args: &[
                "test",
                "-p",
                "speccade-spec",
                "validation::tests::music_tests",
            ],
            log_stem: "music-spec-validation-tests",
            success_message: "speccade-spec music validation tests completed",
        },
        PipelineStage::MusicBackendTests => RepoStageCommandPlan {
            program: "cargo",
            args: &["test", "-p", "speccade-backend-music"],
            log_stem: "music-backend-tests",
            success_message: "speccade-backend-music tests completed",
        },
        PipelineStage::CliTests => RepoStageCommandPlan {
            program: "cargo",
            args: &["test", "-p", "speccade-cli"],
            log_stem: "cli-tests",
            success_message: "speccade-cli tests completed",
        },
        PipelineStage::MusicParityTests => RepoStageCommandPlan {
            program: "cargo",
            args: &["test", "-p", "speccade-tests", "--test", "music_parity"],
            log_stem: "music-parity-tests",
            success_message: "speccade-tests music_parity completed",
        },
        PipelineStage::MusicComposeTests => RepoStageCommandPlan {
            program: "cargo",
            args: &["test", "-p", "speccade-tests", "--test", "compose"],
            log_stem: "music-compose-tests",
            success_message: "speccade-tests compose completed",
        },
        PipelineStage::MusicExternalConformanceTests => RepoStageCommandPlan {
            program: "cargo",
            args: &[
                "test",
                "-p",
                "speccade-tests",
                "--test",
                "music_external_conformance",
            ],
            log_stem: "music-external-conformance-tests",
            success_message: "speccade-tests music_external_conformance completed",
        },
        PipelineStage::GoldenHashVerification => RepoStageCommandPlan {
            program: "cargo",
            args: &[
                "test",
                "-p",
                "speccade-tests",
                "--test",
                "golden_hash_verification",
                "--",
                "--nocapture",
            ],
            log_stem: "golden-hash-verification",
            success_message: "speccade-tests golden_hash_verification completed",
        },
        _ => anyhow::bail!("no repo test command is defined for stage {stage:?}"),
    };
    Ok(plan)
}

fn discover_specs(
    spec_dir: &Path,
    profile: PipelineProfile,
    include_blender: bool,
) -> Result<Vec<DiscoveredSpec>> {
    if !spec_dir.exists() {
        anyhow::bail!("spec directory does not exist: {}", spec_dir.display());
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(spec_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        let path = entry.path();
        if !path.is_file() || is_report_file(path) || is_library_file(path) {
            continue;
        }
        let Some(ext) = path.extension().and_then(OsStr::to_str) else {
            continue;
        };
        let ext = ext.to_ascii_lowercase();
        if SPEC_EXTENSIONS.contains(&ext.as_str()) {
            files.push(path.to_path_buf());
        }
    }
    files.sort();

    let mut discovered = Vec::new();
    for path in files {
        let file_stem = path
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("spec")
            .to_string();
        let mut spec = DiscoveredSpec {
            path: normalize_path(&path),
            file_stem,
            asset_id: None,
            asset_type: None,
            recipe_kind: None,
            backend_tier: None,
            backend_available: false,
            skipped_reason: None,
            discovery_error: None,
        };

        match load_spec(&path) {
            Ok(load_result) => {
                let loaded = load_result.spec;
                spec.asset_id = Some(loaded.asset_id.clone());
                spec.asset_type = Some(loaded.asset_type.to_string());
                spec.recipe_kind = loaded.recipe.as_ref().map(|recipe| recipe.kind.clone());
                spec.backend_tier = spec.recipe_kind.as_deref().and_then(get_backend_tier);
                spec.backend_available = spec
                    .recipe_kind
                    .as_deref()
                    .map(is_backend_available)
                    .unwrap_or(false);
                if !profile.includes_spec(&spec, include_blender) {
                    spec.skipped_reason =
                        Some(format!("profile '{}' excludes this spec", profile.as_str()));
                } else if spec.backend_tier == Some(2) && !include_blender {
                    spec.skipped_reason = Some(
                        "tier 2 spec skipped because --include-blender is disabled".to_string(),
                    );
                }
            }
            Err(err) => {
                spec.discovery_error = Some(err.to_string());
                if !matches!(
                    profile,
                    PipelineProfile::AllFast
                        | PipelineProfile::AllFull
                        | PipelineProfile::Tier1Fast
                        | PipelineProfile::Music
                ) {
                    spec.skipped_reason =
                        Some("spec metadata unavailable during profile filtering".to_string());
                }
            }
        }

        discovered.push(spec);
    }

    Ok(discovered)
}

fn maybe_checkpoint_commit(cycle: u32, prefix: &str) -> Result<Option<String>> {
    let status = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to run `git diff --cached --quiet`")?;

    if status.success() {
        return Ok(None);
    }

    let message = format!(
        "{} (cycle {}, {})",
        prefix,
        cycle,
        Utc::now().format("%Y-%m-%d %H:%M:%S")
    );
    let commit_status = Command::new("git")
        .args(["commit", "-m", &message])
        .status()
        .context("failed to run `git commit` for pipeline checkpoint")?;

    if !commit_status.success() {
        anyhow::bail!("pipeline checkpoint commit failed");
    }

    Ok(Some(message))
}

fn write_json_file(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

fn write_json_lines_file<'a>(
    path: &Path,
    values: impl Iterator<Item = &'a PipelineStageResult>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut lines = Vec::new();
    for value in values {
        lines.push(serde_json::to_string(value)?);
    }
    fs::write(path, lines.join("\n"))?;
    Ok(())
}

fn safe_name(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn exit_code_value(code: ExitCode) -> u8 {
    if code == ExitCode::SUCCESS {
        0
    } else {
        1
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn is_report_file(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.contains(".report."))
}

fn is_library_file(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.starts_with('_'))
}

fn print_human_summary(report: &PipelineReport, report_path: &Path) {
    let cycle_count = report.cycles.len();
    let spec_count = report
        .discovered_specs
        .iter()
        .filter(|spec| spec.skipped_reason.is_none())
        .count();
    println!("Pipeline profile: {}", report.profile);
    println!("Run id: {}", report.run_id);
    println!("Cycles: {}", cycle_count);
    println!("Deep gate: {}", report.deep_gate);
    println!("Specs selected: {}", spec_count);
    println!(
        "Tier policy: tier1={} tier2={} ({})",
        report.execution_policy.effective_tier1_parallel,
        report.execution_policy.effective_tier2_parallel,
        report.execution_policy.scheduling_mode
    );
    println!("Report: {}", report_path.display());
    if report.success {
        println!("Pipeline completed successfully.");
    } else {
        println!("Pipeline completed with failures.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn profile_parsing_accepts_known_names() {
        assert_eq!(
            PipelineProfile::parse("tier1-fast").unwrap(),
            PipelineProfile::Tier1Fast
        );
        assert_eq!(
            PipelineProfile::parse("all-full").unwrap(),
            PipelineProfile::AllFull
        );
        assert!(PipelineProfile::parse("unknown").is_err());
    }

    #[test]
    fn execution_policy_keeps_tier2_serial() {
        let policy = ExecutionPolicy::for_profile(PipelineProfile::AllFast, Some(6), true);
        assert_eq!(policy.effective_tier1_parallel, 6);
        assert_eq!(policy.effective_tier2_parallel, 1);
        assert_eq!(policy.scheduling_mode, "deterministic_sequential_v1");
    }

    #[test]
    fn all_full_splits_spec_and_repo_stages() {
        assert_eq!(
            PipelineProfile::AllFull.spec_stages(false),
            vec![
                PipelineStage::Validate,
                PipelineStage::Generate,
                PipelineStage::PreviewGrid,
                PipelineStage::ValidateAsset,
            ]
        );
        assert_eq!(
            PipelineProfile::AllFull.repo_stages(false),
            vec![PipelineStage::CoverageReport, PipelineStage::ContractVerify]
        );
    }

    #[test]
    fn safe_name_rewrites_non_ascii_path_chars() {
        assert_eq!(safe_name("foo/bar baz"), "foo_bar_baz");
        assert_eq!(safe_name("boss:phase#1"), "boss_phase_1");
    }

    #[test]
    fn discover_specs_is_sorted_and_skips_reports() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        fs::write(root.join("b.json"), minimal_spec_json("b", "music")).unwrap();
        fs::write(root.join("a.json"), minimal_spec_json("a", "audio")).unwrap();
        fs::write(root.join("c.report.json"), "{}").unwrap();
        fs::write(
            root.join("_shared.json"),
            minimal_spec_json("shared", "music"),
        )
        .unwrap();

        let specs = discover_specs(root, PipelineProfile::AllFast, true).unwrap();
        let names: Vec<_> = specs.iter().map(|spec| spec.file_stem.as_str()).collect();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn music_profile_only_includes_music_specs() {
        let music = DiscoveredSpec {
            path: "music.star".to_string(),
            file_stem: "music".to_string(),
            asset_id: Some("song".to_string()),
            asset_type: Some("music".to_string()),
            recipe_kind: Some("music.tracker_song_v1".to_string()),
            backend_tier: Some(1),
            backend_available: true,
            skipped_reason: None,
            discovery_error: None,
        };
        let texture = DiscoveredSpec {
            asset_type: Some("texture".to_string()),
            ..music.clone()
        };
        assert!(PipelineProfile::Music.includes_spec(&music, true));
        assert!(!PipelineProfile::Music.includes_spec(&texture, true));
    }

    #[test]
    fn all_fast_excludes_tier2_without_blender_flag() {
        let spec = DiscoveredSpec {
            path: "mesh.star".to_string(),
            file_stem: "mesh".to_string(),
            asset_id: Some("mesh".to_string()),
            asset_type: Some("static_mesh".to_string()),
            recipe_kind: Some("static_mesh.blender_primitives_v1".to_string()),
            backend_tier: Some(2),
            backend_available: true,
            skipped_reason: None,
            discovery_error: None,
        };

        assert!(!PipelineProfile::AllFast.includes_spec(&spec, false));
        assert!(PipelineProfile::AllFast.includes_spec(&spec, true));
    }

    #[test]
    fn music_profile_includes_repo_test_stages() {
        let stages = PipelineProfile::Music.all_stages(false);
        assert!(stages.contains(&PipelineStage::MusicSpecValidationTests));
        assert!(stages.contains(&PipelineStage::MusicBackendTests));
        assert!(stages.contains(&PipelineStage::CliTests));
        assert!(stages.contains(&PipelineStage::MusicParityTests));
        assert!(stages.contains(&PipelineStage::MusicComposeTests));
        assert!(!stages.contains(&PipelineStage::MusicExternalConformanceTests));
        assert!(!stages.contains(&PipelineStage::GoldenHashVerification));
    }

    #[test]
    fn music_profile_includes_deep_gate_repo_test_stages_when_enabled() {
        let stages = PipelineProfile::Music.all_stages(true);
        assert!(stages.contains(&PipelineStage::MusicExternalConformanceTests));
        assert!(stages.contains(&PipelineStage::GoldenHashVerification));
    }

    #[test]
    fn repo_test_command_plan_matches_music_parity_gate() {
        let plan = plan_repo_test_command(PipelineStage::MusicParityTests).unwrap();
        assert_eq!(plan.program, "cargo");
        assert_eq!(
            plan.args,
            ["test", "-p", "speccade-tests", "--test", "music_parity"]
        );
        assert_eq!(plan.log_stem, "music-parity-tests");
    }

    #[test]
    fn repo_test_command_plan_matches_golden_hash_deep_gate() {
        let plan = plan_repo_test_command(PipelineStage::GoldenHashVerification).unwrap();
        assert_eq!(plan.program, "cargo");
        assert_eq!(
            plan.args,
            [
                "test",
                "-p",
                "speccade-tests",
                "--test",
                "golden_hash_verification",
                "--",
                "--nocapture",
            ]
        );
        assert_eq!(plan.log_stem, "golden-hash-verification");
    }

    #[test]
    fn repo_test_command_plan_rejects_non_test_stage() {
        assert!(plan_repo_test_command(PipelineStage::CoverageReport).is_err());
    }

    #[test]
    fn preview_grid_stage_skips_tier1_specs() {
        let temp = tempdir().unwrap();
        let spec = DiscoveredSpec {
            path: "specs/music/song.star".to_string(),
            file_stem: "song".to_string(),
            asset_id: Some("song".to_string()),
            asset_type: Some("music".to_string()),
            recipe_kind: Some("music.tracker_song_v1".to_string()),
            backend_tier: Some(1),
            backend_available: true,
            skipped_reason: None,
            discovery_error: None,
        };

        let result =
            execute_spec_stage(PipelineStage::PreviewGrid, &spec, temp.path(), temp.path())
                .unwrap();

        assert!(result.skipped);
        assert!(!result.success);
        assert_eq!(result.exit_code, None);
        assert_eq!(result.message, "preview-grid only applies to tier 2 specs");
    }

    #[test]
    fn validate_asset_stage_skips_tier1_specs() {
        let temp = tempdir().unwrap();
        let spec = DiscoveredSpec {
            path: "specs/texture/example.star".to_string(),
            file_stem: "example".to_string(),
            asset_id: Some("example".to_string()),
            asset_type: Some("texture".to_string()),
            recipe_kind: Some("texture.procedural_v1".to_string()),
            backend_tier: Some(1),
            backend_available: true,
            skipped_reason: None,
            discovery_error: None,
        };

        let result = execute_spec_stage(
            PipelineStage::ValidateAsset,
            &spec,
            temp.path(),
            temp.path(),
        )
        .unwrap();

        assert!(result.skipped);
        assert!(!result.success);
        assert_eq!(result.exit_code, None);
        assert_eq!(
            result.message,
            "validate-asset only applies to tier 2 specs"
        );
    }

    #[test]
    fn pipeline_report_serialization_keeps_repo_stage_and_policy_shape() {
        let report = PipelineReport {
            schema_version: 1,
            run_id: "run-123".to_string(),
            profile: "all-full".to_string(),
            started_at: "2026-03-13T00:00:00Z".to_string(),
            duration_seconds: 12.5,
            spec_dir: "specs".to_string(),
            out_root: "target/pipeline".to_string(),
            include_blender: true,
            deep_gate: true,
            requested_cycles: 2,
            sleep_seconds: 15,
            execution_policy: ExecutionPolicy {
                requested_max_parallel: Some(4),
                effective_tier1_parallel: 4,
                effective_tier2_parallel: 1,
                include_blender: true,
                scheduling_mode: "deterministic_sequential_v1".to_string(),
            },
            discovered_specs: vec![DiscoveredSpec {
                path: "specs/music/song.star".to_string(),
                file_stem: "song".to_string(),
                asset_id: Some("song".to_string()),
                asset_type: Some("music".to_string()),
                recipe_kind: Some("music.tracker_song_v1".to_string()),
                backend_tier: Some(1),
                backend_available: true,
                skipped_reason: None,
                discovery_error: None,
            }],
            cycles: vec![PipelineCycleResult {
                cycle: 1,
                started_at: "2026-03-13T00:00:01Z".to_string(),
                duration_seconds: 5.0,
                spec_results: vec![PipelineSpecResult {
                    spec_path: "specs/music/song.star".to_string(),
                    asset_id: Some("song".to_string()),
                    asset_type: Some("music".to_string()),
                    recipe_kind: Some("music.tracker_song_v1".to_string()),
                    backend_tier: Some(1),
                    stages: vec![PipelineStageResult {
                        stage: PipelineStage::Validate,
                        scope: "specs/music/song.star".to_string(),
                        success: true,
                        skipped: false,
                        exit_code: Some(0),
                        duration_ms: 3,
                        artifact_path: None,
                        message: "validate completed".to_string(),
                    }],
                    success: true,
                }],
                repo_stage_results: vec![PipelineStageResult {
                    stage: PipelineStage::CoverageReport,
                    scope: "repo".to_string(),
                    success: true,
                    skipped: false,
                    exit_code: Some(0),
                    duration_ms: 11,
                    artifact_path: Some(
                        "target/pipeline/all-full/run-123/cycle-01/coverage-report.yaml"
                            .to_string(),
                    ),
                    message: "coverage report generated".to_string(),
                }],
                checkpoint_commit: Some("pipeline checkpoint".to_string()),
                success: true,
            }],
            checkpoint_commits: vec!["pipeline checkpoint".to_string()],
            success: true,
        };

        let value = serde_json::to_value(&report).unwrap();
        assert_eq!(value["execution_policy"]["effective_tier2_parallel"], 1);
        assert_eq!(value["deep_gate"], true);
        assert_eq!(
            value["cycles"][0]["repo_stage_results"][0]["stage"],
            "coverage_report"
        );
        assert_eq!(
            value["cycles"][0]["spec_results"][0]["stages"][0]["stage"],
            "validate"
        );
        assert_eq!(value["checkpoint_commits"][0], "pipeline checkpoint");
    }

    fn minimal_spec_json(asset_id: &str, asset_type: &str) -> String {
        format!(
            r#"{{
  "spec_version": 1,
  "asset_id": "{asset_id}",
  "asset_type": "{asset_type}",
  "license": "CC0-1.0",
  "seed": 1
}}"#
        )
    }
}
