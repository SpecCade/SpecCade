//! Spec validation logic.

pub mod common;
mod path_safety;
mod recipe_outputs;
mod recipe_outputs_audio;
mod recipe_outputs_music;
mod recipe_outputs_texture;

#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;

use crate::error::{ErrorCode, ValidationError, ValidationResult, ValidationWarning, WarningCode};
use crate::output::OutputKind;
use crate::spec::{Spec, SPEC_VERSION};

// Re-export common validation utilities for convenience
pub use common::{
    validate_non_negative, validate_positive, validate_range, validate_resolution,
    validate_unit_interval, CommonValidationError,
};

// Re-export path safety functions
pub use path_safety::is_safe_output_path;

// Internal imports
use path_safety::validate_output_path;
use recipe_outputs::validate_outputs_for_recipe;

/// Regex pattern for valid asset_id.
/// Format: starts with lowercase letter, followed by 2-63 lowercase letters, digits, underscores, or hyphens.
const ASSET_ID_PATTERN: &str = r"^[a-z][a-z0-9_-]{2,63}$";

/// Threshold for warning about seed near overflow boundary.
const SEED_OVERFLOW_WARNING_THRESHOLD: u32 = u32::MAX - 1000;

static ASSET_ID_REGEX: OnceLock<Regex> = OnceLock::new();

fn asset_id_regex() -> &'static Regex {
    ASSET_ID_REGEX.get_or_init(|| Regex::new(ASSET_ID_PATTERN).expect("invalid regex pattern"))
}

/// Validates a spec and returns a validation result.
///
/// # Arguments
/// * `spec` - The spec to validate
///
/// # Returns
/// * `ValidationResult` with `ok=true` if validation passed, with any warnings.
/// * `ValidationResult` with `ok=false` and errors if validation failed.
///
/// # Example
/// ```
/// use speccade_spec::{Spec, AssetType, OutputSpec, OutputFormat};
/// use speccade_spec::validation::validate_spec;
///
/// let spec = Spec::builder("test-asset-01", AssetType::Audio)
///     .license("CC0-1.0")
///     .seed(42)
///     .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
///     .build();
///
/// let result = validate_spec(&spec);
/// assert!(result.is_ok());
/// ```
pub fn validate_spec(spec: &Spec) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Contract validation
    validate_spec_version(spec, &mut result);
    validate_asset_id(spec, &mut result);
    validate_seed(spec, &mut result);
    validate_outputs(spec, &mut result);

    // Recipe validation (if present)
    if let Some(ref recipe) = spec.recipe {
        validate_recipe_compatibility(spec, recipe, &mut result);
        validate_outputs_for_recipe(spec, recipe, &mut result);
    }

    // Warnings
    check_warnings(spec, &mut result);

    result
}

/// Validates the spec version.
fn validate_spec_version(spec: &Spec, result: &mut ValidationResult) {
    if spec.spec_version != SPEC_VERSION {
        result.add_error(ValidationError::with_path(
            ErrorCode::UnsupportedSpecVersion,
            format!(
                "spec_version must be {}, got {}",
                SPEC_VERSION, spec.spec_version
            ),
            "spec_version",
        ));
    }
}

/// Validates the asset_id format.
fn validate_asset_id(spec: &Spec, result: &mut ValidationResult) {
    if !asset_id_regex().is_match(&spec.asset_id) {
        result.add_error(ValidationError::with_path(
            ErrorCode::InvalidAssetId,
            format!(
                "asset_id must match pattern '{}', got '{}'",
                ASSET_ID_PATTERN, spec.asset_id
            ),
            "asset_id",
        ));
    }
}

/// Validates the seed value.
fn validate_seed(spec: &Spec, result: &mut ValidationResult) {
    // Seed is already constrained to u32 range by the type system.
    // We only need to check for the warning threshold.
    if spec.seed >= SEED_OVERFLOW_WARNING_THRESHOLD {
        result.add_warning(ValidationWarning::with_path(
            WarningCode::SeedNearOverflow,
            format!(
                "seed {} is close to the maximum value ({})",
                spec.seed,
                u32::MAX
            ),
            "seed",
        ));
    }
}

/// Validates the outputs array.
fn validate_outputs(spec: &Spec, result: &mut ValidationResult) {
    // Check for empty outputs
    if spec.outputs.is_empty() {
        result.add_error(ValidationError::with_path(
            ErrorCode::NoOutputs,
            "outputs array must have at least one entry",
            "outputs",
        ));
        return;
    }

    // Require at least one primary output.
    let has_primary_output = spec.outputs.iter().any(|o| o.kind == OutputKind::Primary);
    if !has_primary_output {
        result.add_error(ValidationError::with_path(
            ErrorCode::NoPrimaryOutput,
            "No primary output declared.",
            "outputs",
        ));
    }

    // Check for unique paths
    let mut seen_paths: HashSet<&str> = HashSet::new();
    for (i, output) in spec.outputs.iter().enumerate() {
        // NOTE: `metadata` / `preview` are defined in the enum for forward-compat,
        // but they are not produced by any current generators. The structured output
        // for generation/validation is the `${asset_id}.report.json` sibling file.
        if matches!(output.kind, OutputKind::Metadata | OutputKind::Preview) {
            let hint = if output.kind == OutputKind::Metadata {
                format!(
                    "output kind '{}' is reserved; use '{}' instead",
                    output.kind,
                    crate::report::Report::filename(&spec.asset_id)
                )
            } else {
                format!("output kind '{}' is reserved (not generated)", output.kind)
            };

            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                hint,
                format!("outputs[{}].kind", i),
            ));
        }

        if !seen_paths.insert(&output.path) {
            result.add_error(ValidationError::with_path(
                ErrorCode::DuplicateOutputPath,
                format!("duplicate output path: '{}'", output.path),
                format!("outputs[{}].path", i),
            ));
        }

        // Validate path safety
        validate_output_path(output, i, result);
    }
}

/// Validates recipe compatibility with asset type.
fn validate_recipe_compatibility(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    result: &mut ValidationResult,
) {
    // Check that recipe.kind is compatible with asset_type.
    //
    // Compatibility is based on the recipe kind prefix (e.g. `texture.*` for `asset_type: texture`,
    // or `audio_v1` for `asset_type: audio`).
    if !spec.asset_type.is_compatible_recipe(&recipe.kind) {
        result.add_error(ValidationError::with_path(
            ErrorCode::RecipeAssetTypeMismatch,
            format!(
                "recipe kind '{}' is not compatible with asset_type '{}'",
                recipe.kind, spec.asset_type
            ),
            "recipe.kind",
        ));
    }
}

/// Checks for warnings.
fn check_warnings(spec: &Spec, result: &mut ValidationResult) {
    // W001: Missing license
    if spec.license.is_empty() {
        result.add_warning(ValidationWarning::with_path(
            WarningCode::MissingLicense,
            "license field is empty",
            "license",
        ));
    }

    // W002: Missing description
    if spec.description.is_none()
        || spec
            .description
            .as_ref()
            .map(|d| d.is_empty())
            .unwrap_or(true)
    {
        result.add_warning(ValidationWarning::with_path(
            WarningCode::MissingDescription,
            "description field is missing or empty",
            "description",
        ));
    }
}

/// Validates that a spec is suitable for the `generate` command.
///
/// This performs standard validation plus checks that a recipe is present.
pub fn validate_for_generate(spec: &Spec) -> ValidationResult {
    let mut result = validate_spec(spec);

    // E010: Recipe required for generate
    if spec.recipe.is_none() {
        result.add_error(ValidationError::with_path(
            ErrorCode::MissingRecipe,
            "recipe is required for generate command",
            "recipe",
        ));
    }

    // Reject recipe kinds that this generator version doesn't implement.
    //
    // `validate_spec()` intentionally doesn't enforce this so specs can be parsed/validated
    // in other contexts (e.g. editors) without needing a backend.
    if let Some(recipe) = &spec.recipe {
        const SUPPORTED: &[&str] = &[
            "audio_v1",
            "music.tracker_song_v1",
            "music.tracker_song_compose_v1",
            "texture.procedural_v1",
            "static_mesh.blender_primitives_v1",
            "skeletal_mesh.blender_rigged_mesh_v1",
            "skeletal_animation.blender_clip_v1",
        ];

        if !SUPPORTED.contains(&recipe.kind.as_str()) {
            result.add_error(ValidationError::with_path(
                ErrorCode::UnsupportedRecipeKind,
                format!(
                    "recipe kind '{}' is not supported by this generator version",
                    recipe.kind
                ),
                "recipe.kind",
            ));
        }
    }

    result
}

/// Checks if an asset_id is valid.
///
/// # Arguments
/// * `asset_id` - The asset_id to validate
///
/// # Returns
/// * `true` if the asset_id is valid, `false` otherwise.
pub fn is_valid_asset_id(asset_id: &str) -> bool {
    asset_id_regex().is_match(asset_id)
}
