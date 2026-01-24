//! Recipe-specific output validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::{OutputFormat, OutputKind};
use crate::recipe::Recipe;
use crate::spec::Spec;
use crate::validation::BudgetProfile;

use super::recipe_outputs_audio::validate_audio_outputs_with_budget;
use super::recipe_outputs_music::{
    validate_music_compose_outputs_with_budget, validate_music_outputs_with_budget,
};
use super::recipe_outputs_texture::validate_texture_procedural_outputs_with_budget;

/// Validates outputs for a recipe using the default budget profile.
pub(super) fn validate_outputs_for_recipe(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    result: &mut ValidationResult,
) {
    validate_outputs_for_recipe_with_budget(spec, recipe, &BudgetProfile::default(), result)
}

/// Validates outputs for a recipe with a specific budget profile.
pub(super) fn validate_outputs_for_recipe_with_budget(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    budget: &BudgetProfile,
    result: &mut ValidationResult,
) {
    match recipe.kind.as_str() {
        "audio_v1" => validate_audio_outputs_with_budget(spec, recipe, budget, result),
        "music.tracker_song_v1" => validate_music_outputs_with_budget(spec, recipe, budget, result),
        "music.tracker_song_compose_v1" => {
            validate_music_compose_outputs_with_budget(spec, recipe, budget, result)
        }
        "texture.procedural_v1" => {
            validate_texture_procedural_outputs_with_budget(spec, recipe, budget, result)
        }
        "texture.trimsheet_v1" => validate_texture_trimsheet_outputs(spec, recipe, result),
        "texture.decal_v1" => validate_texture_decal_outputs(spec, recipe, result),
        "texture.splat_set_v1" => validate_texture_splat_set_outputs(spec, recipe, result),
        "texture.matcap_v1" => validate_texture_matcap_outputs(spec, recipe, result),
        "texture.material_preset_v1" => {
            validate_texture_material_preset_outputs(spec, recipe, result)
        }
        "static_mesh.blender_primitives_v1" => {
            validate_static_mesh_blender_primitives(recipe, result);
            validate_single_primary_output_format(spec, OutputFormat::Glb, result);
        }
        "skeletal_mesh.blender_rigged_mesh_v1" => {
            validate_skeletal_mesh_blender_rigged(recipe, result);
            validate_single_primary_output_format(spec, OutputFormat::Glb, result);
        }
        "skeletal_animation.blender_clip_v1" => {
            validate_skeletal_animation_blender_clip(recipe, result);
            validate_single_primary_output_format(spec, OutputFormat::Glb, result);
        }
        "skeletal_animation.blender_rigged_v1" => {
            validate_skeletal_animation_blender_rigged(recipe, result);
            validate_single_primary_output_format(spec, OutputFormat::Glb, result);
        }
        "sprite.sheet_v1" => validate_sprite_sheet_outputs(spec, recipe, result),
        "sprite.animation_v1" => validate_sprite_animation_outputs(spec, recipe, result),
        "vfx.flipbook_v1" => validate_vfx_flipbook_outputs(spec, recipe, result),
        "ui.nine_slice_v1" => validate_ui_nine_slice_outputs(spec, recipe, result),
        "ui.icon_set_v1" => validate_ui_icon_set_outputs(spec, recipe, result),
        "font.bitmap_v1" => validate_font_bitmap_outputs(spec, recipe, result),
        _ if recipe.kind.starts_with("texture.") => {
            result.add_error(ValidationError::with_path(
                ErrorCode::UnsupportedRecipeKind,
                format!(
                    "unsupported texture recipe kind '{}'; use 'texture.procedural_v1', 'texture.trimsheet_v1', 'texture.decal_v1', 'texture.splat_set_v1', 'texture.matcap_v1', or 'texture.material_preset_v1'",
                    recipe.kind
                ),
                "recipe.kind",
            ));
            validate_primary_output_present(spec, result);
        }
        _ => validate_primary_output_present(spec, result),
    }
}

pub(crate) fn validate_primary_output_present(spec: &Spec, result: &mut ValidationResult) {
    let has_primary = spec.outputs.iter().any(|o| o.kind == OutputKind::Primary);
    if !has_primary {
        result.add_error(ValidationError::with_path(
            ErrorCode::NoPrimaryOutput,
            "at least one output must have kind 'primary'",
            "outputs",
        ));
    }
}

pub(crate) fn validate_single_primary_output_format(
    spec: &Spec,
    expected_format: OutputFormat,
    result: &mut ValidationResult,
) {
    validate_primary_output_present(spec, result);

    let primary_outputs: Vec<(usize, &crate::output::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.len() != 1 {
        result.add_error(ValidationError::with_path(
            ErrorCode::OutputValidationFailed,
            format!(
                "expected exactly 1 primary output, got {}",
                primary_outputs.len()
            ),
            "outputs",
        ));
        return;
    }

    let (index, output) = primary_outputs[0];
    if output.format != expected_format {
        result.add_error(ValidationError::with_path(
            ErrorCode::OutputValidationFailed,
            format!(
                "primary output format must be '{}' for this recipe, got '{}'",
                expected_format, output.format
            ),
            format!("outputs[{}].format", index),
        ));
    }
}

// =============================================================================
// Tier-2 Recipe Params Validation
// =============================================================================

/// Validates params for `static_mesh.blender_primitives_v1` recipe.
///
/// This validates that the params match the expected schema and rejects
/// unknown fields.
fn validate_static_mesh_blender_primitives(recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_static_mesh_blender_primitives() {
        Ok(params) => {
            // Validate dimensions are positive
            for (i, &dim) in params.dimensions.iter().enumerate() {
                if dim <= 0.0 {
                    let axis = ["X", "Y", "Z"][i];
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("dimensions[{}] ({}) must be positive, got {}", i, axis, dim),
                        format!("recipe.params.dimensions[{}]", i),
                    ));
                }
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }
}

/// Validates params for `skeletal_mesh.blender_rigged_mesh_v1` recipe.
///
/// This validates that the params match the expected schema and rejects
/// unknown fields.
fn validate_skeletal_mesh_blender_rigged(recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_skeletal_mesh_blender_rigged_mesh() {
        Ok(params) => {
            // Validate that either skeleton_preset or skeleton is provided
            if params.skeleton_preset.is_none() && params.skeleton.is_empty() {
                // Only warn if both body_parts and parts are also empty
                // (legacy specs may rely on external armature)
                if params.body_parts.is_empty() && params.parts.is_empty() {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        "either 'skeleton_preset', 'skeleton', 'body_parts', or 'parts' must be provided",
                        "recipe.params",
                    ));
                }
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }
}

/// IK-specific field names that are only valid in `blender_rigged_v1`.
const IK_ONLY_FIELDS: &[&str] = &[
    "rig_setup",
    "poses",
    "phases",
    "ik_keyframes",
    "procedural_layers",
    "animator_rig",
    "input_armature",
    "character",
    "duration_frames",
    "ground_offset",
    "conventions",
];

/// Validates params for `skeletal_animation.blender_clip_v1` recipe.
///
/// This validates that the params match the expected schema and rejects
/// unknown fields. Additionally provides clear guidance if IK-specific
/// fields are incorrectly used.
fn validate_skeletal_animation_blender_clip(recipe: &Recipe, result: &mut ValidationResult) {
    // First check if any IK-only fields are present and provide helpful error
    if let Some(obj) = recipe.params.as_object() {
        for &ik_field in IK_ONLY_FIELDS {
            if obj.contains_key(ik_field) {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "'{}' is an IK/rigging feature not supported by 'skeletal_animation.blender_clip_v1'; \
                         use 'skeletal_animation.blender_rigged_v1' instead for IK targets, poses, phases, and procedural layers",
                        ik_field
                    ),
                    format!("recipe.params.{}", ik_field),
                ));
            }
        }
    }

    match recipe.as_skeletal_animation_blender_clip() {
        Ok(params) => {
            // Validate duration is positive
            if params.duration_seconds <= 0.0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "duration_seconds must be positive, got {}",
                        params.duration_seconds
                    ),
                    "recipe.params.duration_seconds",
                ));
            }

            // Validate fps is reasonable
            if params.fps == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "fps must be greater than 0",
                    "recipe.params.fps",
                ));
            }

            // Validate clip_name is not empty
            if params.clip_name.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "clip_name must not be empty",
                    "recipe.params.clip_name",
                ));
            }
        }
        Err(e) => {
            // Check if the error mentions any IK fields to provide better guidance
            let err_str = e.to_string();
            let is_ik_field_error = IK_ONLY_FIELDS.iter().any(|f| err_str.contains(f));

            if is_ik_field_error {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "invalid params for {}: {}. Note: IK features (rig_setup, poses, phases, \
                         ik_keyframes, procedural_layers) require 'skeletal_animation.blender_rigged_v1'",
                        recipe.kind, e
                    ),
                    "recipe.params",
                ));
            } else {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!("invalid params for {}: {}", recipe.kind, e),
                    "recipe.params",
                ));
            }
        }
    }
}

/// Validates params for `skeletal_animation.blender_rigged_v1` recipe.
///
/// This validates that the params match the expected schema and rejects
/// unknown fields.
fn validate_skeletal_animation_blender_rigged(recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_skeletal_animation_blender_rigged() {
        Ok(params) => {
            // Validate duration_frames is positive
            if params.duration_frames == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "duration_frames must be greater than 0",
                    "recipe.params.duration_frames",
                ));
            }

            // Validate fps is reasonable
            if params.fps == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "fps must be greater than 0",
                    "recipe.params.fps",
                ));
            }

            // Validate clip_name is not empty
            if params.clip_name.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "clip_name must not be empty",
                    "recipe.params.clip_name",
                ));
            }

            // If duration_seconds is provided, it should be positive
            if let Some(duration_seconds) = params.duration_seconds {
                if duration_seconds <= 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "duration_seconds must be positive, got {}",
                            duration_seconds
                        ),
                        "recipe.params.duration_seconds",
                    ));
                }
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }
}

/// Validates outputs for `texture.trimsheet_v1` recipe.
///
/// Trimsheet specs require:
/// - At least one primary output with PNG format
/// - Optional metadata output(s) with JSON format
fn validate_texture_trimsheet_outputs(spec: &Spec, recipe: &Recipe, result: &mut ValidationResult) {
    // Validate params parse correctly
    match recipe.as_texture_trimsheet() {
        Ok(params) => {
            // Validate resolution is positive
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }

            // Validate tiles have unique ids
            let mut seen_ids = std::collections::HashSet::new();
            for (i, tile) in params.tiles.iter().enumerate() {
                if !seen_ids.insert(&tile.id) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("duplicate tile id: '{}'", tile.id),
                        format!("recipe.params.tiles[{}].id", i),
                    ));
                }

                // Validate tile dimensions
                if tile.width == 0 || tile.height == 0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "tile '{}' dimensions must be positive, got {}x{}",
                            tile.id, tile.width, tile.height
                        ),
                        format!("recipe.params.tiles[{}]", i),
                    ));
                }

                // Validate tile fits in atlas with padding
                let padded_width = tile.width + params.padding * 2;
                let padded_height = tile.height + params.padding * 2;
                if padded_width > params.resolution[0] || padded_height > params.resolution[1] {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "tile '{}' ({}x{}) with padding {} is too large for atlas ({}x{})",
                            tile.id,
                            tile.width,
                            tile.height,
                            params.padding,
                            params.resolution[0],
                            params.resolution[1]
                        ),
                        format!("recipe.params.tiles[{}]", i),
                    ));
                }
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    // Validate outputs
    validate_primary_output_present(spec, result);

    // Check primary outputs are PNG
    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "texture.trimsheet_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }

        // Check metadata outputs are JSON
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "texture.trimsheet_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `texture.decal_v1` recipe.
fn validate_texture_decal_outputs(spec: &Spec, recipe: &Recipe, result: &mut ValidationResult) {
    let params = match recipe.as_texture_decal() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
            params
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
            return;
        }
    };

    validate_primary_output_present(spec, result);

    let mut has_albedo_output = false;
    for (i, output) in spec.outputs.iter().enumerate() {
        match output.kind {
            OutputKind::Primary => {
                if output.format != OutputFormat::Png {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.decal_v1 primary outputs must have format 'png'",
                        format!("outputs[{}].format", i),
                    ));
                }

                let source = output.source.as_deref().unwrap_or("");
                match source {
                    "" | "albedo" => has_albedo_output = true,
                    "normal" => {
                        if params.normal_output.is_none() {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::OutputValidationFailed,
                                "texture.decal_v1 output requests normal map but recipe.params.normal_output is not set",
                                format!("outputs[{}].source", i),
                            ));
                        }
                    }
                    "roughness" => {
                        if params.roughness_output.is_none() {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::OutputValidationFailed,
                                "texture.decal_v1 output requests roughness map but recipe.params.roughness_output is not set",
                                format!("outputs[{}].source", i),
                            ));
                        }
                    }
                    other => {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::OutputValidationFailed,
                            format!(
                                "texture.decal_v1 output source '{}' is not supported (expected '', 'albedo', 'normal', or 'roughness')",
                                other
                            ),
                            format!("outputs[{}].source", i),
                        ));
                    }
                }
            }
            OutputKind::Metadata => {
                if output.format != OutputFormat::Json {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.decal_v1 metadata outputs must have format 'json'",
                        format!("outputs[{}].format", i),
                    ));
                }
            }
            OutputKind::Preview => {}
        }
    }

    if !has_albedo_output {
        result.add_error(ValidationError::with_path(
            ErrorCode::OutputValidationFailed,
            "texture.decal_v1 requires at least one albedo output (primary output with empty source or source 'albedo')",
            "outputs",
        ));
    }
}

/// Validates outputs for `texture.splat_set_v1` recipe.
fn validate_texture_splat_set_outputs(spec: &Spec, recipe: &Recipe, result: &mut ValidationResult) {
    let params = match recipe.as_texture_splat_set() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
            if params.layers.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "layers must not be empty",
                    "recipe.params.layers",
                ));
            }
            params
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
            return;
        }
    };

    let mut layer_ids = std::collections::HashSet::new();
    for (i, layer) in params.layers.iter().enumerate() {
        if layer.id.is_empty() {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                "layer.id must not be empty",
                format!("recipe.params.layers[{}].id", i),
            ));
        }
        if !layer_ids.insert(layer.id.as_str()) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("duplicate layer id: '{}'", layer.id),
                format!("recipe.params.layers[{}].id", i),
            ));
        }
    }

    let mask_count = params.layers.len().div_ceil(4);

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        match output.kind {
            OutputKind::Primary => {
                if output.format != OutputFormat::Png {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.splat_set_v1 primary outputs must have format 'png'",
                        format!("outputs[{}].format", i),
                    ));
                }

                let source = output.source.as_deref().unwrap_or("").trim();
                if source.is_empty() {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.splat_set_v1 primary outputs must set 'source' (e.g. 'grass.albedo', 'mask0', 'macro')",
                        format!("outputs[{}].source", i),
                    ));
                    continue;
                }

                if source == "macro" {
                    if !params.macro_variation {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::OutputValidationFailed,
                            "texture.splat_set_v1 output requests macro texture but recipe.params.macro_variation is false",
                            format!("outputs[{}].source", i),
                        ));
                    }
                    continue;
                }

                if let Some(rest) = source.strip_prefix("mask") {
                    match rest.parse::<usize>() {
                        Ok(idx) if idx < mask_count => {}
                        Ok(_) => {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::OutputValidationFailed,
                                format!(
                                    "texture.splat_set_v1 output requests '{}' but mask index is out of range (0..{})",
                                    source,
                                    mask_count.saturating_sub(1)
                                ),
                                format!("outputs[{}].source", i),
                            ));
                        }
                        Err(_) => {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::OutputValidationFailed,
                                format!(
                                    "texture.splat_set_v1 output source '{}' is invalid (expected 'maskN')",
                                    source
                                ),
                                format!("outputs[{}].source", i),
                            ));
                        }
                    }
                    continue;
                }

                if let Some((layer_id, map_type)) = source.split_once('.') {
                    if !layer_ids.contains(layer_id) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::OutputValidationFailed,
                            format!(
                                "texture.splat_set_v1 output source '{}' references unknown layer id '{}'",
                                source, layer_id
                            ),
                            format!("outputs[{}].source", i),
                        ));
                        continue;
                    }

                    if !matches!(map_type, "albedo" | "normal" | "roughness") {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::OutputValidationFailed,
                            format!(
                                "texture.splat_set_v1 output source '{}' has invalid map type '{}' (expected 'albedo', 'normal', or 'roughness')",
                                source, map_type
                            ),
                            format!("outputs[{}].source", i),
                        ));
                    }
                    continue;
                }

                result.add_error(ValidationError::with_path(
                    ErrorCode::OutputValidationFailed,
                    format!(
                        "texture.splat_set_v1 output source '{}' is invalid (expected '<layer>.<map>', 'maskN', or 'macro')",
                        source
                    ),
                    format!("outputs[{}].source", i),
                ));
            }
            OutputKind::Metadata => {
                if output.format != OutputFormat::Json {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.splat_set_v1 metadata outputs must have format 'json'",
                        format!("outputs[{}].format", i),
                    ));
                }
            }
            OutputKind::Preview => {}
        }
    }
}

/// Validates outputs for `sprite.sheet_v1` recipe.
fn validate_sprite_sheet_outputs(spec: &Spec, recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_sprite_sheet() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "sprite.sheet_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "sprite.sheet_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `sprite.animation_v1` recipe.
fn validate_sprite_animation_outputs(spec: &Spec, recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_sprite_animation() {
        Ok(_params) => {}
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "sprite.animation_v1 primary outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `vfx.flipbook_v1` recipe.
fn validate_vfx_flipbook_outputs(spec: &Spec, recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_vfx_flipbook() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "vfx.flipbook_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "vfx.flipbook_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `ui.nine_slice_v1` recipe.
fn validate_ui_nine_slice_outputs(spec: &Spec, recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_ui_nine_slice() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.nine_slice_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.nine_slice_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `ui.icon_set_v1` recipe.
fn validate_ui_icon_set_outputs(spec: &Spec, recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_ui_icon_set() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.icon_set_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.icon_set_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `font.bitmap_v1` recipe.
fn validate_font_bitmap_outputs(spec: &Spec, recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_font_bitmap() {
        Ok(params) => {
            if params.charset[0] > params.charset[1] {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "charset start must be <= end, got [{}, {}]",
                        params.charset[0], params.charset[1]
                    ),
                    "recipe.params.charset",
                ));
            }

            match (params.glyph_size[0], params.glyph_size[1]) {
                (5, 7) | (8, 8) | (6, 9) => {}
                (w, h) => {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "unsupported glyph_size [{}, {}]; supported sizes: [5,7], [8,8], [6,9]",
                            w, h
                        ),
                        "recipe.params.glyph_size",
                    ));
                }
            }

            for (idx, c) in params.color.iter().enumerate() {
                if !(0.0..=1.0).contains(c) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("color[{}] must be in [0,1], got {}", idx, c),
                        format!("recipe.params.color[{}]", idx),
                    ));
                }
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
            return;
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "font.bitmap_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "font.bitmap_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

fn validate_texture_matcap_outputs(spec: &Spec, recipe: &Recipe, result: &mut ValidationResult) {
    let _params = match recipe.as_texture_matcap() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
            if let Some(steps) = params.toon_steps {
                if !(2..=16).contains(&steps) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("toon_steps must be between 2 and 16, got {}", steps),
                        "recipe.params.toon_steps",
                    ));
                }
            }
            if let Some(ref outline) = params.outline {
                if !(1..=10).contains(&outline.width) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "outline width must be between 1 and 10, got {}",
                            outline.width
                        ),
                        "recipe.params.outline.width",
                    ));
                }
            }
            params
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
            return;
        }
    };

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "texture.matcap_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

fn validate_texture_material_preset_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    let params = match recipe.as_texture_material_preset() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
            // Validate base_color if provided
            if let Some(ref color) = params.base_color {
                for (i, &c) in color.iter().enumerate() {
                    if !(0.0..=1.0).contains(&c) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("base_color[{}] must be in range [0, 1], got {}", i, c),
                            format!("recipe.params.base_color[{}]", i),
                        ));
                    }
                }
            }
            // Validate roughness_range if provided
            if let Some(ref range) = params.roughness_range {
                for (i, &r) in range.iter().enumerate() {
                    if !(0.0..=1.0).contains(&r) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("roughness_range[{}] must be in range [0, 1], got {}", i, r),
                            format!("recipe.params.roughness_range[{}]", i),
                        ));
                    }
                }
            }
            // Validate metallic if provided
            if let Some(m) = params.metallic {
                if !(0.0..=1.0).contains(&m) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("metallic must be in range [0, 1], got {}", m),
                        "recipe.params.metallic",
                    ));
                }
            }
            // Validate noise_scale if provided
            if let Some(ns) = params.noise_scale {
                if ns <= 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("noise_scale must be positive, got {}", ns),
                        "recipe.params.noise_scale",
                    ));
                }
            }
            // Validate pattern_scale if provided
            if let Some(ps) = params.pattern_scale {
                if ps <= 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("pattern_scale must be positive, got {}", ps),
                        "recipe.params.pattern_scale",
                    ));
                }
            }
            params
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
            return;
        }
    };

    validate_primary_output_present(spec, result);

    // Material presets generate 4 primary outputs (albedo, roughness, metallic, normal)
    // and optionally a metadata output
    let valid_sources = ["albedo", "roughness", "metallic", "normal"];

    for (i, output) in spec.outputs.iter().enumerate() {
        match output.kind {
            OutputKind::Primary => {
                if output.format != OutputFormat::Png {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.material_preset_v1 primary outputs must have format 'png'",
                        format!("outputs[{}].format", i),
                    ));
                }

                let source = output.source.as_deref().unwrap_or("");
                if !valid_sources.contains(&source) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        format!(
                            "texture.material_preset_v1 output source '{}' is not valid; expected 'albedo', 'roughness', 'metallic', or 'normal'",
                            source
                        ),
                        format!("outputs[{}].source", i),
                    ));
                }
            }
            OutputKind::Metadata => {
                if output.format != OutputFormat::Json {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.material_preset_v1 metadata outputs must have format 'json'",
                        format!("outputs[{}].format", i),
                    ));
                }
            }
            OutputKind::Preview => {}
        }
    }

    // Suppress unused variable warning
    let _ = params;
}
