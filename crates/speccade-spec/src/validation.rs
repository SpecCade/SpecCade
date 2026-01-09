//! Spec validation logic.

use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;

use crate::error::{ErrorCode, ValidationError, ValidationResult, ValidationWarning, WarningCode};
use crate::output::OutputKind;
use crate::spec::{Spec, SPEC_VERSION};

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
/// let spec = Spec::builder("test-asset-01", AssetType::AudioSfx)
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

    // Check for at least one primary output
    let has_primary = spec
        .outputs
        .iter()
        .any(|o| o.kind == OutputKind::Primary);
    if !has_primary {
        result.add_error(ValidationError::with_path(
            ErrorCode::NoPrimaryOutput,
            "at least one output must have kind 'primary'",
            "outputs",
        ));
    }

    // Check for unique paths
    let mut seen_paths: HashSet<&str> = HashSet::new();
    for (i, output) in spec.outputs.iter().enumerate() {
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

/// Validates an output path for safety.
fn validate_output_path(
    output: &crate::output::OutputSpec,
    index: usize,
    result: &mut ValidationResult,
) {
    let path = &output.path;
    let path_field = format!("outputs[{}].path", index);

    for message in output_path_safety_errors(path) {
        result.add_error(ValidationError::with_path(
            ErrorCode::UnsafeOutputPath,
            message,
            &path_field,
        ));
    }

    // Check that extension matches format
    if !output.extension_matches() {
        result.add_error(ValidationError::with_path(
            ErrorCode::PathFormatMismatch,
            format!(
                "output path extension does not match format '{}': '{}'",
                output.format, path
            ),
            &path_field,
        ));
    }
}

/// Validates recipe compatibility with asset type.
fn validate_recipe_compatibility(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    result: &mut ValidationResult,
) {
    // Check that recipe.kind prefix matches asset_type
    if let Some(prefix) = recipe.asset_type_prefix() {
        if prefix != spec.asset_type.as_str() {
            result.add_error(ValidationError::with_path(
                ErrorCode::RecipeAssetTypeMismatch,
                format!(
                    "recipe kind '{}' is not compatible with asset_type '{}'",
                    recipe.kind, spec.asset_type
                ),
                "recipe.kind",
            ));
        }
    } else {
        result.add_error(ValidationError::with_path(
            ErrorCode::RecipeAssetTypeMismatch,
            format!(
                "recipe kind '{}' has invalid format (expected 'asset_type.recipe_name')",
                recipe.kind
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
    if spec.description.is_none() || spec.description.as_ref().map(|d| d.is_empty()).unwrap_or(true)
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

/// Checks if an output path is safe.
///
/// # Arguments
/// * `path` - The output path to validate
///
/// # Returns
/// * `true` if the path is safe, `false` otherwise.
pub fn is_safe_output_path(path: &str) -> bool {
    output_path_safety_errors(path).is_empty()
}

fn output_path_safety_errors(path: &str) -> Vec<String> {
    let mut errors = Vec::new();

    // Check for empty path
    if path.is_empty() {
        errors.push("output path cannot be empty".to_string());
        return errors;
    }

    // Check for absolute paths (leading slash or drive letter)
    if path.starts_with('/') || path.starts_with('\\') {
        errors.push(format!("output path must be relative, not absolute: '{}'", path));
    }

    // Check for Windows drive letter
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        errors.push(format!("output path must not contain drive letter: '{}'", path));
    }

    // Check for backslashes
    if path.contains('\\') {
        errors.push(format!("output path must use forward slashes only: '{}'", path));
    }

    // Check for path traversal (..)
    for segment in path.split('/') {
        if segment == ".." {
            errors.push(format!("output path must not contain '..': '{}'", path));
            break;
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{OutputFormat, OutputSpec};
    use crate::spec::AssetType;

    fn make_valid_spec() -> Spec {
        Spec::builder("test-asset-01", AssetType::AudioSfx)
            .license("CC0-1.0")
            .seed(42)
            .description("Test asset")
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build()
    }

    #[test]
    fn test_valid_spec() {
        let spec = make_valid_spec();
        let result = validate_spec(&spec);
        assert!(result.is_ok(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_invalid_spec_version() {
        let mut spec = make_valid_spec();
        spec.spec_version = 2;
        let result = validate_spec(&spec);
        assert!(!result.is_ok());
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::UnsupportedSpecVersion));
    }

    #[test]
    fn test_invalid_asset_id() {
        let test_cases = vec![
            ("1invalid", "starts with number"),
            ("ab", "too short"),
            ("UPPERCASE", "uppercase"),
            ("has spaces", "spaces"),
            ("a", "single char"),
        ];

        for (asset_id, desc) in test_cases {
            let mut spec = make_valid_spec();
            spec.asset_id = asset_id.to_string();
            let result = validate_spec(&spec);
            assert!(
                !result.is_ok(),
                "expected invalid for {}: {}",
                desc, asset_id
            );
            assert!(
                result
                    .errors
                    .iter()
                    .any(|e| e.code == ErrorCode::InvalidAssetId),
                "expected InvalidAssetId for {}: {}",
                desc,
                asset_id
            );
        }
    }

    #[test]
    fn test_valid_asset_ids() {
        let valid_ids = vec![
            "abc",
            "test-asset-01",
            "laser_blast_01",
            "a23",
            "my-cool-asset-name-with-dashes",
        ];

        for asset_id in valid_ids {
            assert!(is_valid_asset_id(asset_id), "expected valid: {}", asset_id);
        }
    }

    #[test]
    fn test_no_outputs() {
        let mut spec = make_valid_spec();
        spec.outputs.clear();
        let result = validate_spec(&spec);
        assert!(!result.is_ok());
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::NoOutputs));
    }

    #[test]
    fn test_no_primary_output() {
        let mut spec = make_valid_spec();
        spec.outputs = vec![OutputSpec::metadata("meta.json")];
        let result = validate_spec(&spec);
        assert!(!result.is_ok());
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::NoPrimaryOutput));
    }

    #[test]
    fn test_duplicate_output_path() {
        let mut spec = make_valid_spec();
        spec.outputs.push(OutputSpec::primary(
            OutputFormat::Wav,
            "sounds/test.wav",
        ));
        let result = validate_spec(&spec);
        assert!(!result.is_ok());
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::DuplicateOutputPath));
    }

    #[test]
    fn test_unsafe_output_paths() {
        let unsafe_paths = vec![
            ("/absolute/path.wav", "absolute path"),
            ("C:/windows/path.wav", "drive letter"),
            ("path\\with\\backslash.wav", "backslash"),
            ("../parent/path.wav", "parent traversal"),
            ("sounds/../../../etc/passwd", "deep traversal"),
        ];

        for (path, desc) in unsafe_paths {
            let mut spec = make_valid_spec();
            spec.outputs = vec![OutputSpec::primary(OutputFormat::Wav, path)];
            let result = validate_spec(&spec);
            assert!(
                !result.is_ok(),
                "expected unsafe for {}: {}",
                desc, path
            );
            assert!(
                result
                    .errors
                    .iter()
                    .any(|e| e.code == ErrorCode::UnsafeOutputPath),
                "expected UnsafeOutputPath for {}: {}",
                desc,
                path
            );
        }
    }

    #[test]
    fn test_path_format_mismatch() {
        let mut spec = make_valid_spec();
        spec.outputs = vec![OutputSpec::primary(OutputFormat::Wav, "sounds/test.png")];
        let result = validate_spec(&spec);
        assert!(!result.is_ok());
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::PathFormatMismatch));
    }

    #[test]
    fn test_recipe_asset_type_mismatch() {
        let mut spec = make_valid_spec();
        spec.recipe = Some(crate::recipe::Recipe::new(
            "music.tracker_song_v1",
            serde_json::json!({}),
        ));
        let result = validate_spec(&spec);
        assert!(!result.is_ok());
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::RecipeAssetTypeMismatch));
    }

    #[test]
    fn test_missing_recipe_for_generate() {
        let spec = make_valid_spec();
        let result = validate_for_generate(&spec);
        assert!(!result.is_ok());
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::MissingRecipe));
    }

    #[test]
    fn test_warnings() {
        let spec = Spec::builder("test-01", AssetType::AudioSfx)
            .license("")
            .seed(42)
            .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
            .build();

        let result = validate_spec(&spec);
        // Should pass but with warnings
        assert!(result.is_ok());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.code == WarningCode::MissingLicense));
        assert!(result
            .warnings
            .iter()
            .any(|w| w.code == WarningCode::MissingDescription));
    }

    #[test]
    fn test_seed_near_overflow_warning() {
        let mut spec = make_valid_spec();
        spec.seed = u32::MAX;
        let result = validate_spec(&spec);
        assert!(result.is_ok());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.code == WarningCode::SeedNearOverflow));
    }

    #[test]
    fn test_is_safe_output_path() {
        assert!(is_safe_output_path("sounds/laser.wav"));
        assert!(is_safe_output_path("textures/albedo.png"));
        assert!(is_safe_output_path("meshes/crate.glb"));

        assert!(!is_safe_output_path(""));
        assert!(!is_safe_output_path("/absolute/path"));
        assert!(!is_safe_output_path("C:/windows/path"));
        assert!(!is_safe_output_path("path\\backslash"));
        assert!(!is_safe_output_path("../traversal"));
    }
}
