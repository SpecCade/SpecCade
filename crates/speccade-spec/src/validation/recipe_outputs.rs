//! Recipe-specific output validation.

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::{OutputFormat, OutputKind};
use crate::spec::Spec;
use crate::validation::BudgetProfile;

use super::recipe_outputs_audio::validate_audio_outputs_with_budget;
use super::recipe_outputs_mesh::{
    validate_skeletal_animation_blender_clip, validate_skeletal_animation_blender_rigged,
    validate_skeletal_mesh_armature_driven, validate_skeletal_mesh_skinned_mesh,
    validate_static_mesh_blender_primitives, validate_static_mesh_modular_kit,
    validate_static_mesh_organic_sculpt,
};
use super::recipe_outputs_music::{
    validate_music_compose_outputs_with_budget, validate_music_outputs_with_budget,
};
use super::recipe_outputs_sprite::{
    validate_sprite_animation_outputs, validate_sprite_render_from_mesh_outputs,
    validate_sprite_sheet_outputs,
};
use super::recipe_outputs_texture::{
    validate_texture_decal_outputs, validate_texture_matcap_outputs,
    validate_texture_material_preset_outputs, validate_texture_procedural_outputs_with_budget,
    validate_texture_splat_set_outputs, validate_texture_trimsheet_outputs,
};
use super::recipe_outputs_ui::{
    validate_font_bitmap_outputs, validate_ui_damage_number_outputs, validate_ui_icon_set_outputs,
    validate_ui_item_card_outputs, validate_ui_nine_slice_outputs,
};
use super::recipe_outputs_vfx::{
    validate_vfx_flipbook_outputs, validate_vfx_particle_profile_outputs,
};

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
            validate_single_primary_output_format_one_of(
                spec,
                &[OutputFormat::Glb, OutputFormat::Gltf],
                result,
            );
        }
        "static_mesh.modular_kit_v1" => {
            validate_static_mesh_modular_kit(recipe, result);
            validate_single_primary_output_format_one_of(
                spec,
                &[OutputFormat::Glb, OutputFormat::Gltf],
                result,
            );
        }
        "static_mesh.organic_sculpt_v1" => {
            validate_static_mesh_organic_sculpt(recipe, result);
            validate_single_primary_output_format_one_of(
                spec,
                &[OutputFormat::Glb, OutputFormat::Gltf],
                result,
            );
        }
        "skeletal_mesh.armature_driven_v1" => {
            validate_skeletal_mesh_armature_driven(recipe, result);
            validate_single_primary_output_format(spec, OutputFormat::Glb, result);
        }
        "skeletal_mesh.skinned_mesh_v1" => {
            validate_skeletal_mesh_skinned_mesh(recipe, result);
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
        "sprite.render_from_mesh_v1" => {
            validate_sprite_render_from_mesh_outputs(spec, recipe, result)
        }
        "vfx.flipbook_v1" => validate_vfx_flipbook_outputs(spec, recipe, result),
        "vfx.particle_profile_v1" => validate_vfx_particle_profile_outputs(spec, recipe, result),
        "ui.nine_slice_v1" => validate_ui_nine_slice_outputs(spec, recipe, result),
        "ui.icon_set_v1" => validate_ui_icon_set_outputs(spec, recipe, result),
        "ui.item_card_v1" => validate_ui_item_card_outputs(spec, recipe, result),
        "ui.damage_number_v1" => validate_ui_damage_number_outputs(spec, recipe, result),
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

pub(crate) fn validate_single_primary_output_format_one_of(
    spec: &Spec,
    allowed_formats: &[OutputFormat],
    result: &mut ValidationResult,
) {
    validate_primary_output_present(spec, result);

    let primary_outputs: Vec<(usize, &crate::output::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    for (idx, output) in primary_outputs {
        if !allowed_formats.contains(&output.format) {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                format!(
                    "primary output format must be one of {:?} for this recipe, got '{:?}'",
                    allowed_formats, output.format
                ),
                format!("outputs[{idx}].format"),
            ));
        }
    }
}
