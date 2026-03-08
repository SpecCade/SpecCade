//! Contract verification command.
//!
//! Enforces the checked SpecCade contract surface:
//! - recipe-kind coverage in the canonical example manifest
//! - stdlib/example coverage remains at 100%
//! - docs only reference current recipe kinds
//! - canonical examples still generate, lint cleanly, and satisfy quality gates

use anyhow::{bail, Context, Result};
use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Serialize};
use speccade_lint::{LintReport, RuleRegistry};
use speccade_spec::{recipe::RecipeKind, OutputMetrics, Report};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use tempfile::tempdir;
use walkdir::WalkDir;

use crate::commands::coverage;
use crate::commands::json_output::GenerateOutput;
use crate::commands::lint::lint_report_to_data;
use crate::dispatch::is_backend_available;
use crate::input::load_spec;

#[derive(Debug, Deserialize)]
struct ContractManifest {
    schema_version: u32,
    #[serde(default)]
    docs: Vec<ContractDoc>,
    #[serde(default)]
    examples: Vec<ContractExample>,
}

#[derive(Debug, Deserialize)]
struct ContractDoc {
    path: String,
    #[serde(default)]
    required_recipe_kinds: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ContractExample {
    recipe_kind: String,
    spec_path: String,
    #[serde(default)]
    setup_specs: Vec<String>,
    #[serde(default)]
    copy_paths: Vec<String>,
    #[serde(default)]
    lint_include_paths: Vec<String>,
    #[serde(default)]
    allowed_warning_rule_ids: Vec<String>,
    #[serde(default)]
    allowed_info_rule_ids: Vec<String>,
    #[serde(default)]
    expectations: ExampleExpectations,
}

#[derive(Debug, Default, Deserialize)]
struct ExampleExpectations {
    #[serde(default)]
    min_outputs: Option<usize>,
    #[serde(default)]
    require_geometry: bool,
    #[serde(default)]
    require_manifold: bool,
    #[serde(default)]
    require_uvs: bool,
    #[serde(default)]
    min_material_slots: Option<u32>,
    #[serde(default)]
    min_bone_count: Option<u32>,
    #[serde(default)]
    min_animation_frames: Option<u32>,
    #[serde(default)]
    max_hinge_axis_violations: Option<u32>,
    #[serde(default)]
    max_range_violations: Option<u32>,
    #[serde(default)]
    max_velocity_spikes: Option<u32>,
    #[serde(default)]
    max_root_motion_magnitude: Option<f32>,
}

#[derive(Debug, Serialize)]
struct ContractVerifyReport {
    success: bool,
    manifest_path: String,
    recipe_kind_count: usize,
    coverage: CoverageCheck,
    backends: Vec<BackendCheck>,
    docs: Vec<DocCheck>,
    examples: Vec<ExampleCheck>,
    errors: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CoverageCheck {
    total_features: u32,
    covered: u32,
    uncovered: u32,
}

#[derive(Debug, Serialize)]
struct BackendCheck {
    recipe_kind: String,
    available: bool,
}

#[derive(Debug, Serialize)]
struct DocCheck {
    path: String,
    success: bool,
    missing_recipe_kinds: Vec<String>,
    unknown_recipe_tokens: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ExampleCheck {
    recipe_kind: String,
    spec_path: String,
    success: bool,
    determinism_checked: bool,
    output_count: usize,
    lint_warning_rule_ids: Vec<String>,
    lint_info_rule_ids: Vec<String>,
    metrics: Option<ExampleMetricSummary>,
    errors: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ExampleMetricSummary {
    vertex_count: Option<u32>,
    triangle_count: Option<u32>,
    manifold: Option<bool>,
    has_uv_map: Option<bool>,
    material_slot_count: Option<u32>,
    bone_count: Option<u32>,
    animation_frame_count: Option<u32>,
    hinge_axis_violations: Option<u32>,
    range_violations: Option<u32>,
    velocity_spikes: Option<u32>,
}

struct ExampleRun {
    generated: GenerateOutput,
    report: Report,
    output_bytes: BTreeMap<String, Vec<u8>>,
}

pub fn run_verify(
    manifest_path: Option<&str>,
    output_path: Option<&str>,
    json_output: bool,
) -> Result<ExitCode> {
    let root = project_root();
    let manifest_rel = manifest_path.unwrap_or("specs/contract_manifest.json");
    let manifest_abs = root.join(manifest_rel);
    let manifest = load_manifest(&manifest_abs)?;
    let recipe_kinds = recipe_kind_set();

    let coverage_report = coverage::generate_coverage_report_from(Some(&root))?;
    let mut errors = Vec::new();

    if manifest.schema_version != 1 {
        errors.push(format!(
            "Manifest schema_version {} is unsupported (expected 1)",
            manifest.schema_version
        ));
    }

    let backends = verify_backends(&recipe_kinds, &mut errors);
    verify_example_surface(&manifest.examples, &recipe_kinds, &mut errors);
    if coverage_report.summary.uncovered > 0 {
        errors.push(format!(
            "Feature coverage drift detected: {} uncovered stdlib features",
            coverage_report.summary.uncovered
        ));
    }

    let docs = verify_docs(&root, &manifest.docs, &recipe_kinds, &mut errors);
    let examples = verify_examples(&root, &manifest.examples, &mut errors);

    let success = errors.is_empty();
    let report = ContractVerifyReport {
        success,
        manifest_path: manifest_rel.to_string(),
        recipe_kind_count: recipe_kinds.len(),
        coverage: CoverageCheck {
            total_features: coverage_report.summary.total_features,
            covered: coverage_report.summary.covered,
            uncovered: coverage_report.summary.uncovered,
        },
        backends,
        docs,
        examples,
        errors,
    };

    if let Some(path) = output_path {
        let abs = root.join(path);
        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&abs, serde_json::to_string_pretty(&report)?)?;
    }

    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human_report(&report);
    }

    Ok(if report.success {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn load_manifest(path: &Path) -> Result<ContractManifest> {
    let json = fs::read_to_string(path)
        .with_context(|| format!("Failed to read contract manifest: {}", path.display()))?;
    serde_json::from_str(&json)
        .with_context(|| format!("Failed to parse contract manifest: {}", path.display()))
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn recipe_kind_set() -> BTreeSet<String> {
    RecipeKind::all()
        .iter()
        .map(|kind| kind.as_str().to_string())
        .collect()
}

fn verify_backends(recipe_kinds: &BTreeSet<String>, errors: &mut Vec<String>) -> Vec<BackendCheck> {
    recipe_kinds
        .iter()
        .map(|recipe_kind| {
            let available = is_backend_available(recipe_kind);
            if !available {
                errors.push(format!(
                    "Recipe kind {} is in the public surface but not reported as backend-available",
                    recipe_kind
                ));
            }
            BackendCheck {
                recipe_kind: recipe_kind.clone(),
                available,
            }
        })
        .collect()
}

fn verify_example_surface(
    examples: &[ContractExample],
    recipe_kinds: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let mut seen = BTreeSet::new();
    for example in examples {
        if !seen.insert(example.recipe_kind.clone()) {
            errors.push(format!(
                "Manifest contains duplicate example entry for {}",
                example.recipe_kind
            ));
        }
    }

    let example_kinds: BTreeSet<String> = examples
        .iter()
        .map(|example| example.recipe_kind.clone())
        .collect();

    for missing in recipe_kinds.difference(&example_kinds) {
        errors.push(format!(
            "Manifest is missing a canonical example for recipe kind {}",
            missing
        ));
    }
    for extra in example_kinds.difference(recipe_kinds) {
        errors.push(format!("Manifest includes unknown recipe kind {}", extra));
    }
}

fn verify_docs(
    root: &Path,
    docs: &[ContractDoc],
    recipe_kinds: &BTreeSet<String>,
    errors: &mut Vec<String>,
) -> Vec<DocCheck> {
    docs.iter()
        .map(|doc| {
            let mut missing_recipe_kinds = Vec::new();
            let mut unknown_recipe_tokens = Vec::new();
            let path = root.join(&doc.path);

            let success = match fs::read_to_string(&path) {
                Ok(text) => {
                    for required in &doc.required_recipe_kinds {
                        if !text.contains(required) {
                            missing_recipe_kinds.push(required.clone());
                        }
                    }

                    for token in extract_recipe_tokens(&text) {
                        if !recipe_kinds.contains(&token) {
                            unknown_recipe_tokens.push(token);
                        }
                    }

                    missing_recipe_kinds.is_empty() && unknown_recipe_tokens.is_empty()
                }
                Err(err) => {
                    missing_recipe_kinds.push(format!("read error: {}", err));
                    false
                }
            };

            if !success {
                errors.push(format!("Doc drift detected in {}", doc.path));
            }

            DocCheck {
                path: doc.path.clone(),
                success,
                missing_recipe_kinds,
                unknown_recipe_tokens,
            }
        })
        .collect()
}

fn verify_examples(
    root: &Path,
    examples: &[ContractExample],
    errors: &mut Vec<String>,
) -> Vec<ExampleCheck> {
    examples
        .iter()
        .map(|example| {
            let result = verify_example(root, example);
            if !result.success {
                errors.push(format!(
                    "Canonical example failed for {} ({})",
                    result.recipe_kind, result.spec_path
                ));
            }
            result
        })
        .collect()
}

fn verify_example(root: &Path, example: &ContractExample) -> ExampleCheck {
    let mut check = ExampleCheck {
        recipe_kind: example.recipe_kind.clone(),
        spec_path: example.spec_path.clone(),
        success: false,
        determinism_checked: false,
        output_count: 0,
        lint_warning_rule_ids: Vec::new(),
        lint_info_rule_ids: Vec::new(),
        metrics: None,
        errors: Vec::new(),
    };

    let spec_abs = root.join(&example.spec_path);
    let spec = match load_spec(&spec_abs) {
        Ok(load_result) => load_result.spec,
        Err(err) => {
            check.errors.push(format!("Failed to load spec: {}", err));
            return check;
        }
    };

    let Some(recipe) = spec.recipe.as_ref() else {
        check.errors.push("Spec has no recipe".to_string());
        return check;
    };

    if recipe.kind != example.recipe_kind {
        check.errors.push(format!(
            "Spec recipe kind {} does not match manifest entry {}",
            recipe.kind, example.recipe_kind
        ));
        return check;
    }

    let Some(recipe_kind) = RecipeKind::all()
        .iter()
        .find(|kind| kind.as_str() == example.recipe_kind)
        .cloned()
    else {
        check
            .errors
            .push(format!("Unknown recipe kind {}", example.recipe_kind));
        return check;
    };

    let workspace_dir = match tempdir() {
        Ok(dir) => dir,
        Err(err) => {
            check
                .errors
                .push(format!("Failed to create temp workspace: {}", err));
            return check;
        }
    };
    let workspace_root = workspace_dir.path().join("workspace");
    if let Err(err) = fs::create_dir_all(&workspace_root) {
        check
            .errors
            .push(format!("Failed to create workspace root: {}", err));
        return check;
    }

    if let Err(err) = copy_relative_path(root, &workspace_root, &example.spec_path) {
        check.errors.push(err.to_string());
        return check;
    }
    for setup in &example.setup_specs {
        if let Err(err) = copy_relative_path(root, &workspace_root, setup) {
            check.errors.push(err.to_string());
            return check;
        }
    }
    for extra in &example.copy_paths {
        if let Err(err) = copy_relative_path(root, &workspace_root, extra) {
            check.errors.push(err.to_string());
            return check;
        }
    }

    let copied_spec_path = workspace_root.join(&example.spec_path);
    let run_a = match run_example_once(&copied_spec_path, &workspace_root, example, "run_a") {
        Ok(run) => run,
        Err(err) => {
            check.errors.push(format_error_chain(&err));
            return check;
        }
    };

    if let Some(result) = run_a.generated.result.as_ref() {
        check.output_count = result.outputs.len();
    }

    let lint = match lint_selected_outputs(&run_a, &spec, example) {
        Ok(lint) => lint,
        Err(err) => {
            check.errors.push(format_error_chain(&err));
            return check;
        }
    };

    if let Some(lint) = lint.as_ref() {
        check.lint_warning_rule_ids = lint
            .warnings
            .iter()
            .map(|issue| issue.rule_id.clone())
            .collect();
        check.lint_info_rule_ids = lint
            .info
            .iter()
            .map(|issue| issue.rule_id.clone())
            .collect();
    }

    if let Some(metrics) = extract_report_metrics(&run_a.report) {
        check.metrics = Some(ExampleMetricSummary {
            vertex_count: metrics.vertex_count,
            triangle_count: metrics.triangle_count,
            manifold: metrics.manifold,
            has_uv_map: metrics
                .has_uv_map
                .or_else(|| metrics.uv_layer_count.map(|count| count > 0)),
            material_slot_count: metrics.material_slot_count,
            bone_count: metrics.bone_count,
            animation_frame_count: metrics.animation_frame_count,
            hinge_axis_violations: metrics.hinge_axis_violations,
            range_violations: metrics.range_violations,
            velocity_spikes: metrics.velocity_spikes,
        });
        validate_expectations(&example.expectations, &metrics, &mut check.errors);
    } else if !recipe_kind.is_tier1() {
        check
            .errors
            .push("Tier 2 example report did not contain output metrics".to_string());
    }

    validate_lint_allowlists(example, lint.as_ref(), &mut check.errors);
    validate_output_count(example, &spec, check.output_count, &mut check.errors);
    validate_report_basics(&run_a.report, example, &mut check.errors);

    if recipe_kind.is_tier1() {
        check.determinism_checked = true;
        match run_example_once(&copied_spec_path, &workspace_root, example, "run_b") {
            Ok(run_b) => {
                if run_a.output_bytes != run_b.output_bytes {
                    check.errors.push(format!(
                        "Tier 1 determinism failed for {}: output bytes differed across runs",
                        example.recipe_kind
                    ));
                }
            }
            Err(err) => check.errors.push(format_error_chain(&err)),
        }
    }

    check.success = check.errors.is_empty();
    check
}

fn run_example_once(
    copied_spec_path: &Path,
    workspace_root: &Path,
    example: &ContractExample,
    run_name: &str,
) -> Result<ExampleRun> {
    let out_root = workspace_root.join(run_name);
    fs::create_dir_all(&out_root)?;

    for setup_spec in &example.setup_specs {
        let setup_path = workspace_root.join(setup_spec);
        run_generate_command(&setup_path, &out_root).with_context(|| {
            format!(
                "Setup spec {} failed for canonical example {}",
                setup_spec, example.recipe_kind
            )
        })?;
    }

    run_generate_command(copied_spec_path, &out_root).with_context(|| {
        format!(
            "Example spec {} failed for recipe kind {}",
            example.spec_path, example.recipe_kind
        )
    })
}

fn run_generate_command(spec_path: &Path, out_root: &Path) -> Result<ExampleRun> {
    let exe = std::env::current_exe().context("Failed to resolve current executable path")?;
    let output = Command::new(exe)
        .arg("generate")
        .arg("--spec")
        .arg(spec_path)
        .arg("--out-root")
        .arg(out_root)
        .arg("--json")
        .arg("--no-cache")
        .output()
        .with_context(|| format!("Failed to spawn generate for {}", spec_path.display()))?;

    let stdout = String::from_utf8(output.stdout).context("Generate stdout was not valid UTF-8")?;
    let stderr = String::from_utf8(output.stderr).context("Generate stderr was not valid UTF-8")?;

    let generated: GenerateOutput = serde_json::from_str(stdout.trim()).with_context(|| {
        format!(
            "Generate JSON parsing failed for {}.\nstdout:\n{}\nstderr:\n{}",
            spec_path.display(),
            stdout,
            stderr
        )
    })?;

    if !output.status.success() || !generated.success {
        let mut details = generated
            .errors
            .iter()
            .map(|error| format!("{}: {}", error.code, error.message))
            .collect::<Vec<_>>();
        if !stderr.trim().is_empty() {
            details.push(format!("stderr: {}", stderr.trim()));
        }
        bail!(
            "Generate failed for {}: {}",
            spec_path.display(),
            details.join(" | ")
        );
    }

    let result = generated
        .result
        .as_ref()
        .context("Generate succeeded but returned no result payload")?;

    let report_path = resolve_generated_report_path(spec_path, &result.report_path);
    let report_json = fs::read_to_string(&report_path)
        .with_context(|| format!("Failed to read report {}", report_path.display()))?;
    let report = Report::from_json(&report_json)
        .with_context(|| format!("Failed to parse report {}", report_path.display()))?;

    let mut output_bytes = BTreeMap::new();
    for file in &result.outputs {
        let bytes = fs::read(out_root.join(&file.path)).with_context(|| {
            format!(
                "Failed to read generated output {} for {}",
                file.path,
                spec_path.display()
            )
        })?;
        output_bytes.insert(file.path.clone(), bytes);
    }

    Ok(ExampleRun {
        generated,
        report,
        output_bytes,
    })
}

fn lint_selected_outputs(
    run: &ExampleRun,
    spec: &speccade_spec::Spec,
    example: &ContractExample,
) -> Result<Option<speccade_spec::report::LintReportData>> {
    let result = run
        .generated
        .result
        .as_ref()
        .context("Generate succeeded but returned no result payload")?;
    let out_root = Path::new(&result.out_root);

    let lint_paths = if example.lint_include_paths.is_empty() {
        spec.outputs
            .iter()
            .filter(|output| {
                output.kind == speccade_spec::OutputKind::Primary
                    && output.format != speccade_spec::OutputFormat::Json
            })
            .map(|output| output.path.clone())
            .collect::<Vec<_>>()
    } else {
        example.lint_include_paths.clone()
    };

    if lint_paths.is_empty() {
        return Ok(None);
    }

    let registry = RuleRegistry::default_rules();
    let mut combined = LintReport::new();
    let mut linted_any = false;

    for rel_path in lint_paths {
        let path = out_root.join(&rel_path);
        if !path.exists() {
            bail!(
                "Configured lint path does not exist for {}: {}",
                example.recipe_kind,
                path.display()
            );
        }
        let report = registry
            .lint(&path, Some(spec))
            .with_context(|| format!("Failed to lint {}", path.display()))?;
        combined.merge(report);
        linted_any = true;
    }

    if !linted_any {
        return Ok(None);
    }

    Ok(Some(lint_report_to_data(&combined)))
}

fn resolve_generated_report_path(spec_path: &Path, report_path: &str) -> PathBuf {
    let path = PathBuf::from(report_path);
    if path.is_absolute() {
        path
    } else {
        spec_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path)
    }
}

fn validate_expectations(
    expectations: &ExampleExpectations,
    metrics: &OutputMetrics,
    errors: &mut Vec<String>,
) {
    if expectations.require_geometry {
        let has_geometry = metrics
            .vertex_count
            .zip(metrics.triangle_count)
            .map(|(vertices, triangles)| vertices > 0 && triangles > 0)
            .unwrap_or(false);
        if !has_geometry {
            errors.push("Expected geometry metrics with non-zero vertices and triangles".into());
        }
    }
    if expectations.require_manifold && metrics.manifold != Some(true) {
        errors.push("Expected manifold=true".into());
    }
    if expectations.require_uvs
        && metrics
            .has_uv_map
            .or_else(|| metrics.uv_layer_count.map(|count| count > 0))
            != Some(true)
    {
        errors.push("Expected UV data on canonical example".into());
    }
    if let Some(min_material_slots) = expectations.min_material_slots {
        if metrics.material_slot_count.unwrap_or(0) < min_material_slots {
            errors.push(format!(
                "Expected at least {} material slots",
                min_material_slots
            ));
        }
    }
    if let Some(min_bone_count) = expectations.min_bone_count {
        if metrics.bone_count.unwrap_or(0) < min_bone_count {
            errors.push(format!("Expected at least {} bones", min_bone_count));
        }
    }
    if let Some(min_animation_frames) = expectations.min_animation_frames {
        if metrics.animation_frame_count.unwrap_or(0) < min_animation_frames {
            errors.push(format!(
                "Expected at least {} animation frames",
                min_animation_frames
            ));
        }
    }
    if let Some(max) = expectations.max_hinge_axis_violations {
        if metrics.hinge_axis_violations.unwrap_or(u32::MAX) > max {
            errors.push(format!("Expected hinge_axis_violations <= {}", max));
        }
    }
    if let Some(max) = expectations.max_range_violations {
        if metrics.range_violations.unwrap_or(u32::MAX) > max {
            errors.push(format!("Expected range_violations <= {}", max));
        }
    }
    if let Some(max) = expectations.max_velocity_spikes {
        if metrics.velocity_spikes.unwrap_or(u32::MAX) > max {
            errors.push(format!("Expected velocity_spikes <= {}", max));
        }
    }
    if let Some(max) = expectations.max_root_motion_magnitude {
        let magnitude = metrics
            .root_motion_delta
            .map(root_motion_magnitude)
            .unwrap_or(f32::MAX);
        if magnitude > max {
            errors.push(format!("Expected root motion magnitude <= {}", max));
        }
    }
}

fn validate_lint_allowlists(
    example: &ContractExample,
    lint: Option<&speccade_spec::report::LintReportData>,
    errors: &mut Vec<String>,
) {
    let Some(lint) = lint else {
        return;
    };

    if !lint.errors.is_empty() {
        errors.push(format!(
            "Lint errors present: {}",
            lint.errors
                .iter()
                .map(|issue| issue.rule_id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let allowed_warnings: BTreeSet<&str> = example
        .allowed_warning_rule_ids
        .iter()
        .map(String::as_str)
        .collect();
    for issue in &lint.warnings {
        if !allowed_warnings.contains(issue.rule_id.as_str()) {
            errors.push(format!(
                "Unexpected lint warning {} on {}",
                issue.rule_id, example.recipe_kind
            ));
        }
    }

    let allowed_info: BTreeSet<&str> = example
        .allowed_info_rule_ids
        .iter()
        .map(String::as_str)
        .collect();
    for issue in &lint.info {
        if !allowed_info.contains(issue.rule_id.as_str()) {
            errors.push(format!(
                "Unexpected lint info {} on {}",
                issue.rule_id, example.recipe_kind
            ));
        }
    }
}

fn validate_output_count(
    example: &ContractExample,
    spec: &speccade_spec::Spec,
    output_count: usize,
    errors: &mut Vec<String>,
) {
    let min_outputs = example
        .expectations
        .min_outputs
        .unwrap_or(spec.outputs.len());
    if output_count < min_outputs {
        errors.push(format!(
            "Expected at least {} outputs but saw {}",
            min_outputs, output_count
        ));
    }
}

fn validate_report_basics(report: &Report, example: &ContractExample, errors: &mut Vec<String>) {
    if !report.ok {
        errors.push("Generation report was not ok=true".to_string());
    }
    if report.recipe_kind.as_deref() != Some(example.recipe_kind.as_str()) {
        errors.push(format!(
            "Report recipe_kind {:?} did not match {}",
            report.recipe_kind, example.recipe_kind
        ));
    }
}

fn extract_report_metrics(report: &Report) -> Option<OutputMetrics> {
    report
        .outputs
        .iter()
        .find_map(|output| output.metrics.clone())
}

fn root_motion_magnitude(delta: [f32; 3]) -> f32 {
    (delta[0].powi(2) + delta[1].powi(2) + delta[2].powi(2)).sqrt()
}

fn copy_relative_path(root: &Path, workspace_root: &Path, rel_path: &str) -> Result<()> {
    let source = root.join(rel_path);
    let dest = workspace_root.join(rel_path);

    if !source.exists() {
        bail!(
            "Required manifest path does not exist: {}",
            source.display()
        );
    }

    if source.is_dir() {
        for entry in WalkDir::new(&source) {
            let entry = entry?;
            let relative = entry.path().strip_prefix(root)?;
            let dest_path = workspace_root.join(relative);
            if entry.file_type().is_dir() {
                fs::create_dir_all(&dest_path)?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(entry.path(), &dest_path)?;
            }
        }
    } else {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, dest)?;
    }

    Ok(())
}

fn format_error_chain(err: &anyhow::Error) -> String {
    err.chain()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" | caused by: ")
}

fn extract_recipe_tokens(text: &str) -> BTreeSet<String> {
    let re = Regex::new(r"(audio_v1|[a-z_]+\.[a-z0-9_]+_v1)").expect("valid recipe regex");
    re.captures_iter(text)
        .map(|captures| captures[1].to_string())
        .collect()
}

fn print_human_report(report: &ContractVerifyReport) {
    println!("{}", "Contract Verify".cyan().bold());
    println!("Manifest: {}", report.manifest_path);
    println!(
        "Recipe kinds: {} | Coverage: {}/{}",
        report.recipe_kind_count, report.coverage.covered, report.coverage.total_features
    );

    let missing_backends = report
        .backends
        .iter()
        .filter(|backend| !backend.available)
        .count();
    let failing_docs = report.docs.iter().filter(|doc| !doc.success).count();
    let failing_examples = report
        .examples
        .iter()
        .filter(|example| !example.success)
        .count();

    println!(
        "Backends: {} | Docs: {} failing | Examples: {} failing",
        if missing_backends == 0 {
            "all available".green().to_string()
        } else {
            format!("{} missing", missing_backends).red().to_string()
        },
        failing_docs,
        failing_examples
    );

    if report.success {
        println!("{}", "PASS".green().bold());
        return;
    }

    println!("{}", "FAIL".red().bold());
    for error in &report.errors {
        println!("  - {}", error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_recipe_tokens() {
        let tokens = extract_recipe_tokens(
            "`audio_v1`, `static_mesh.boolean_kit_v1`, and `skeletal_animation.blender_clip_v1`",
        );

        assert!(tokens.contains("audio_v1"));
        assert!(tokens.contains("static_mesh.boolean_kit_v1"));
        assert!(tokens.contains("skeletal_animation.blender_clip_v1"));
    }

    #[test]
    fn test_manifest_covers_all_recipe_kinds() {
        let root = project_root();
        let manifest = load_manifest(&root.join("specs/contract_manifest.json")).unwrap();

        let example_kinds: BTreeSet<String> = manifest
            .examples
            .iter()
            .map(|example| example.recipe_kind.clone())
            .collect();

        assert_eq!(example_kinds, recipe_kind_set());
    }
}
