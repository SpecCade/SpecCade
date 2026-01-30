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

/// Validates params for `static_mesh.modular_kit_v1` recipe.
///
/// This validates that the params match the expected schema and rejects
/// unknown fields. Validates kit-specific constraints (cutout counts,
/// segment counts, dimension ranges).
fn validate_static_mesh_modular_kit(recipe: &Recipe, result: &mut ValidationResult) {
    use crate::recipe::{ModularKitType, MAX_PIPE_SEGMENTS, MAX_WALL_CUTOUTS};

    match recipe.as_static_mesh_modular_kit() {
        Ok(params) => {
            match &params.kit_type {
                ModularKitType::Wall(wall) => {
                    // Validate positive dimensions
                    if wall.width <= 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("wall width must be positive, got {}", wall.width),
                            "recipe.params.kit_type.width",
                        ));
                    }
                    if wall.height <= 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("wall height must be positive, got {}", wall.height),
                            "recipe.params.kit_type.height",
                        ));
                    }
                    if wall.thickness <= 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("wall thickness must be positive, got {}", wall.thickness),
                            "recipe.params.kit_type.thickness",
                        ));
                    }

                    // Validate cutout count
                    if wall.cutouts.len() > MAX_WALL_CUTOUTS {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!(
                                "too many cutouts: {} exceeds maximum of {}",
                                wall.cutouts.len(),
                                MAX_WALL_CUTOUTS
                            ),
                            "recipe.params.kit_type.cutouts",
                        ));
                    }

                    // Validate each cutout
                    for (i, cutout) in wall.cutouts.iter().enumerate() {
                        if cutout.width <= 0.0 {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::InvalidRecipeParams,
                                format!(
                                    "cutout[{}] width must be positive, got {}",
                                    i, cutout.width
                                ),
                                format!("recipe.params.kit_type.cutouts[{}].width", i),
                            ));
                        }
                        if cutout.height <= 0.0 {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::InvalidRecipeParams,
                                format!(
                                    "cutout[{}] height must be positive, got {}",
                                    i, cutout.height
                                ),
                                format!("recipe.params.kit_type.cutouts[{}].height", i),
                            ));
                        }
                    }

                    // Validate bevel width is non-negative
                    if wall.bevel_width < 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("bevel_width must be non-negative, got {}", wall.bevel_width),
                            "recipe.params.kit_type.bevel_width",
                        ));
                    }
                }
                ModularKitType::Pipe(pipe) => {
                    // Validate positive diameter
                    if pipe.diameter <= 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("pipe diameter must be positive, got {}", pipe.diameter),
                            "recipe.params.kit_type.diameter",
                        ));
                    }

                    // Validate wall thickness
                    if pipe.wall_thickness <= 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!(
                                "pipe wall_thickness must be positive, got {}",
                                pipe.wall_thickness
                            ),
                            "recipe.params.kit_type.wall_thickness",
                        ));
                    }

                    // Validate wall thickness is less than radius
                    if pipe.wall_thickness >= pipe.diameter / 2.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!(
                                "pipe wall_thickness ({}) must be less than radius ({})",
                                pipe.wall_thickness,
                                pipe.diameter / 2.0
                            ),
                            "recipe.params.kit_type.wall_thickness",
                        ));
                    }

                    // Validate segment count
                    if pipe.segments.len() > MAX_PIPE_SEGMENTS {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!(
                                "too many pipe segments: {} exceeds maximum of {}",
                                pipe.segments.len(),
                                MAX_PIPE_SEGMENTS
                            ),
                            "recipe.params.kit_type.segments",
                        ));
                    }

                    if pipe.segments.is_empty() {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            "pipe must have at least one segment",
                            "recipe.params.kit_type.segments",
                        ));
                    }

                    // Validate vertices (minimum 3)
                    if pipe.vertices < 3 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("pipe vertices must be at least 3, got {}", pipe.vertices),
                            "recipe.params.kit_type.vertices",
                        ));
                    }

                    // Validate bevel width is non-negative
                    if pipe.bevel_width < 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("bevel_width must be non-negative, got {}", pipe.bevel_width),
                            "recipe.params.kit_type.bevel_width",
                        ));
                    }
                }
                ModularKitType::Door(door) => {
                    // Validate positive dimensions
                    if door.width <= 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("door width must be positive, got {}", door.width),
                            "recipe.params.kit_type.width",
                        ));
                    }
                    if door.height <= 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("door height must be positive, got {}", door.height),
                            "recipe.params.kit_type.height",
                        ));
                    }
                    if door.frame_thickness <= 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!(
                                "door frame_thickness must be positive, got {}",
                                door.frame_thickness
                            ),
                            "recipe.params.kit_type.frame_thickness",
                        ));
                    }
                    if door.frame_depth <= 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!(
                                "door frame_depth must be positive, got {}",
                                door.frame_depth
                            ),
                            "recipe.params.kit_type.frame_depth",
                        ));
                    }

                    // Validate open_angle is in valid range (0-90)
                    if door.is_open && !(0.0..=90.0).contains(&door.open_angle) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!(
                                "door open_angle must be in range [0, 90], got {}",
                                door.open_angle
                            ),
                            "recipe.params.kit_type.open_angle",
                        ));
                    }

                    // Validate bevel width is non-negative
                    if door.bevel_width < 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("bevel_width must be non-negative, got {}", door.bevel_width),
                            "recipe.params.kit_type.bevel_width",
                        ));
                    }
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

/// Validates params for `static_mesh.organic_sculpt_v1` recipe.
///
/// This validates that the params match the expected schema and enforces
/// budget constraints for metaball count, remesh voxel size, and smooth iterations.
fn validate_static_mesh_organic_sculpt(recipe: &Recipe, result: &mut ValidationResult) {
    use crate::recipe::{
        MAX_METABALLS, MAX_REMESH_VOXEL_SIZE, MAX_SMOOTH_ITERATIONS, MIN_REMESH_VOXEL_SIZE,
    };

    match recipe.as_static_mesh_organic_sculpt() {
        Ok(params) => {
            // Validate metaball count
            if params.metaballs.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "metaballs array must contain at least one metaball",
                    "recipe.params.metaballs",
                ));
            }
            if params.metaballs.len() > MAX_METABALLS {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "too many metaballs: {} exceeds maximum of {}",
                        params.metaballs.len(),
                        MAX_METABALLS
                    ),
                    "recipe.params.metaballs",
                ));
            }

            // Validate each metaball
            for (i, metaball) in params.metaballs.iter().enumerate() {
                if metaball.radius <= 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "metaball[{}].radius must be positive, got {}",
                            i, metaball.radius
                        ),
                        format!("recipe.params.metaballs[{}].radius", i),
                    ));
                }
                if metaball.stiffness <= 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "metaball[{}].stiffness must be positive, got {}",
                            i, metaball.stiffness
                        ),
                        format!("recipe.params.metaballs[{}].stiffness", i),
                    ));
                }
            }

            // Validate remesh voxel size
            if params.remesh_voxel_size < MIN_REMESH_VOXEL_SIZE {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "remesh_voxel_size must be at least {}, got {}",
                        MIN_REMESH_VOXEL_SIZE, params.remesh_voxel_size
                    ),
                    "recipe.params.remesh_voxel_size",
                ));
            }
            if params.remesh_voxel_size > MAX_REMESH_VOXEL_SIZE {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "remesh_voxel_size must be at most {}, got {}",
                        MAX_REMESH_VOXEL_SIZE, params.remesh_voxel_size
                    ),
                    "recipe.params.remesh_voxel_size",
                ));
            }

            // Validate smooth iterations
            if params.smooth_iterations > MAX_SMOOTH_ITERATIONS {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "smooth_iterations must be at most {}, got {}",
                        MAX_SMOOTH_ITERATIONS, params.smooth_iterations
                    ),
                    "recipe.params.smooth_iterations",
                ));
            }

            // Validate displacement noise if present
            if let Some(ref displacement) = params.displacement {
                if displacement.strength <= 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "displacement.strength must be positive, got {}",
                            displacement.strength
                        ),
                        "recipe.params.displacement.strength",
                    ));
                }
                if displacement.scale <= 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "displacement.scale must be positive, got {}",
                            displacement.scale
                        ),
                        "recipe.params.displacement.scale",
                    ));
                }
                if displacement.octaves == 0 || displacement.octaves > 8 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "displacement.octaves must be between 1 and 8, got {}",
                            displacement.octaves
                        ),
                        "recipe.params.displacement.octaves",
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

/// Validates params for `skeletal_mesh.armature_driven_v1` recipe.
fn validate_skeletal_mesh_armature_driven(recipe: &Recipe, result: &mut ValidationResult) {
    fn validate_profile(profile: &Option<String>) -> Result<(), String> {
        let Some(profile) = profile.as_ref() else {
            return Ok(());
        };

        let s = profile.trim();
        if s == "square" || s == "rectangle" {
            return Ok(());
        }

        let segments = if let Some(inner) =
            s.strip_prefix("circle(").and_then(|r| r.strip_suffix(')'))
        {
            Some(inner)
        } else if let Some(inner) = s.strip_prefix("hexagon(").and_then(|r| r.strip_suffix(')')) {
            Some(inner)
        } else {
            None
        };

        if let Some(inner) = segments {
            let n: u32 = inner
                .parse()
                .map_err(|_| format!("invalid profile segments: {inner:?}"))?;
            if n < 3 {
                return Err(format!("profile segments must be >= 3, got {n}"));
            }
            return Ok(());
        }

        Err(format!(
            "unknown profile string: {profile:?}; expected one of: None, 'square', 'rectangle', 'circle(N)', 'hexagon(N)'"
        ))
    }

    match recipe.as_skeletal_mesh_armature_driven_v1() {
        Ok(params) => {
            if params.skeleton_preset.is_none() && params.skeleton.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "either 'skeleton_preset' or 'skeleton' must be provided",
                    "recipe.params",
                ));
            }

            if params.bone_meshes.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "'bone_meshes' must not be empty",
                    "recipe.params.bone_meshes",
                ));
            }

            use std::collections::HashSet;

            let mut allowed_bones: HashSet<String> = HashSet::new();
            if let Some(preset) = params.skeleton_preset {
                for &bone in preset.bone_names() {
                    allowed_bones.insert(bone.to_string());
                }
            }
            for bone in &params.skeleton {
                allowed_bones.insert(bone.bone.clone());
            }

            // Validate bone_meshes keys are actual bones.
            for (bone_name, def) in &params.bone_meshes {
                if !allowed_bones.contains(bone_name) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("bone_meshes key '{bone_name}' refers to unknown bone"),
                        format!("recipe.params.bone_meshes.{bone_name}"),
                    ));
                }

                match def {
                    crate::recipe::character::ArmatureDrivenBoneMeshDef::Mirror(m) => {
                        if !params.bone_meshes.contains_key(&m.mirror) {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::InvalidRecipeParams,
                                format!(
                                    "bone_meshes['{bone_name}'] mirrors missing bone_meshes key '{target}'",
                                    target = m.mirror
                                ),
                                format!("recipe.params.bone_meshes.{bone_name}.mirror"),
                            ));
                        }
                    }
                    crate::recipe::character::ArmatureDrivenBoneMeshDef::Mesh(mesh) => {
                        // Profile strings must match Blender's _parse_profile() grammar.
                        if let Err(msg) = validate_profile(&mesh.profile) {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::InvalidRecipeParams,
                                msg,
                                format!("recipe.params.bone_meshes.{bone_name}.profile"),
                            ));
                        }

                        // Validate extrusion_steps
                        for (idx, step) in mesh.extrusion_steps.iter().enumerate() {
                            let extrude_val = match step {
                                crate::recipe::character::ExtrusionStep::Shorthand(d) => *d,
                                crate::recipe::character::ExtrusionStep::Full(def) => def.extrude,
                            };
                            if extrude_val <= 0.0 {
                                result.add_error(ValidationError::with_path(
                                    ErrorCode::InvalidRecipeParams,
                                    format!(
                                        "extrusion_steps[{idx}].extrude must be positive, got {}",
                                        extrude_val
                                    ),
                                    format!(
                                        "recipe.params.bone_meshes.{bone_name}.extrusion_steps[{idx}]"
                                    ),
                                ));
                            }
                        }

                        if let Some(material_index) = mesh.material_index {
                            if (material_index as usize) >= params.material_slots.len() {
                                result.add_error(ValidationError::with_path(
                                    ErrorCode::InvalidRecipeParams,
                                    format!(
                                        "material_index {} out of range for material_slots (len={})",
                                        material_index,
                                        params.material_slots.len()
                                    ),
                                    format!("recipe.params.bone_meshes.{bone_name}.material_index"),
                                ));
                            }
                        }

                        for (m_idx, modifier) in mesh.modifiers.iter().enumerate() {
                            if let crate::recipe::character::ArmatureDrivenModifier::Bool {
                                r#bool,
                            } = modifier
                            {
                                if !params.bool_shapes.contains_key(&r#bool.target) {
                                    result.add_error(ValidationError::with_path(
                                        ErrorCode::InvalidRecipeParams,
                                        format!(
                                            "bool modifier target '{target}' not found in bool_shapes",
                                            target = r#bool.target
                                        ),
                                        format!(
                                            "recipe.params.bone_meshes.{bone_name}.modifiers[{m_idx}].bool.target"
                                        ),
                                    ));
                                }
                            }
                        }

                        for (a_idx, attachment) in mesh.attachments.iter().enumerate() {
                            match attachment {
                                crate::recipe::character::ArmatureDrivenAttachment::Primitive(
                                    p,
                                ) => {
                                    if let Some(material_index) = p.material_index {
                                        if (material_index as usize) >= params.material_slots.len()
                                        {
                                            result.add_error(ValidationError::with_path(
                                                ErrorCode::InvalidRecipeParams,
                                                format!(
                                                    "material_index {} out of range for material_slots (len={})",
                                                    material_index,
                                                    params.material_slots.len()
                                                ),
                                                format!(
                                                    "recipe.params.bone_meshes.{bone_name}.attachments[{a_idx}].material_index"
                                                ),
                                            ));
                                        }
                                    }
                                }
                                crate::recipe::character::ArmatureDrivenAttachment::Extrude {
                                    extrude,
                                } => {
                                    if let Err(msg) = validate_profile(&extrude.profile) {
                                        result.add_error(ValidationError::with_path(
                                            ErrorCode::InvalidRecipeParams,
                                            msg,
                                            format!(
                                                "recipe.params.bone_meshes.{bone_name}.attachments[{a_idx}].extrude.profile"
                                            ),
                                        ));
                                    }
                                }
                                crate::recipe::character::ArmatureDrivenAttachment::Asset(_) => {}
                            }
                        }
                    }
                }
            }

            // Validate bool_shapes mirrors and bone references.
            for (shape_name, def) in &params.bool_shapes {
                match def {
                    crate::recipe::character::ArmatureDrivenBoolShapeDef::Mirror(m) => {
                        if !params.bool_shapes.contains_key(&m.mirror) {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::InvalidRecipeParams,
                                format!(
                                    "bool_shapes['{shape_name}'] mirrors missing bool_shapes key '{target}'",
                                    target = m.mirror
                                ),
                                format!("recipe.params.bool_shapes.{shape_name}.mirror"),
                            ));
                        }
                    }
                    crate::recipe::character::ArmatureDrivenBoolShapeDef::Shape(shape) => {
                        if let Some(bone) = &shape.bone {
                            if !allowed_bones.contains(bone) {
                                result.add_error(ValidationError::with_path(
                                    ErrorCode::InvalidRecipeParams,
                                    format!("bool_shapes['{shape_name}'].bone refers to unknown bone '{bone}'"),
                                    format!("recipe.params.bool_shapes.{shape_name}.bone"),
                                ));
                            }
                        }
                    }
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

/// Validates params for `skeletal_mesh.skinned_mesh_v1` recipe.
fn validate_skeletal_mesh_skinned_mesh(recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_skeletal_mesh_skinned_mesh_v1() {
        Ok(params) => {
            if params.mesh_file.is_none() && params.mesh_asset.is_none() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "either 'mesh_file' or 'mesh_asset' must be provided",
                    "recipe.params",
                ));
            }

            if params.skeleton_preset.is_none() && params.skeleton.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "either 'skeleton_preset' or 'skeleton' must be provided",
                    "recipe.params",
                ));
            }

            // Bounds check for max_bone_influences (shared contract with legacy SkinningSettings)
            if !(1..=8).contains(&params.binding.max_bone_influences) {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "binding.max_bone_influences must be in range [1, 8], got {}",
                        params.binding.max_bone_influences
                    ),
                    "recipe.params.binding.max_bone_influences",
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

/// Validates outputs for `vfx.particle_profile_v1` recipe.
///
/// Particle profile specs are metadata-only:
/// - At least one primary output with JSON format
fn validate_vfx_particle_profile_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_vfx_particle_profile() {
        Ok(params) => {
            // Validate intensity if provided (must be non-negative)
            if let Some(intensity) = params.intensity {
                if intensity < 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("intensity must be non-negative, got {}", intensity),
                        "recipe.params.intensity",
                    ));
                }
            }

            // Validate distortion_strength if provided (must be in [0.0, 1.0])
            if let Some(strength) = params.distortion_strength {
                if !(0.0..=1.0).contains(&strength) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "distortion_strength must be in [0.0, 1.0], got {}",
                            strength
                        ),
                        "recipe.params.distortion_strength",
                    ));
                }
            }

            // Validate color_tint if provided (each component in [0.0, 1.0])
            if let Some(tint) = params.color_tint {
                for (i, &c) in tint.iter().enumerate() {
                    if !(0.0..=1.0).contains(&c) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("color_tint[{}] must be in [0.0, 1.0], got {}", i, c),
                            format!("recipe.params.color_tint[{}]", i),
                        ));
                    }
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

    validate_primary_output_present(spec, result);

    // Particle profile outputs are JSON (metadata-only)
    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "vfx.particle_profile_v1 primary outputs must have format 'json'",
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

/// Validates outputs for `ui.item_card_v1` recipe.
fn validate_ui_item_card_outputs(spec: &Spec, recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_ui_item_card() {
        Ok(params) => {
            // Validate resolution bounds (min 32x32, max 4096x4096)
            if params.resolution[0] < 32 || params.resolution[1] < 32 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be at least 32x32, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }

            if params.resolution[0] > 4096 || params.resolution[1] > 4096 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be at most 4096x4096, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }

            // Validate at least one rarity preset is defined
            if params.rarity_presets.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "at least one rarity_preset must be defined",
                    "recipe.params.rarity_presets",
                ));
            }

            // Validate colors and check for duplicate tiers
            let mut seen_tiers = std::collections::HashSet::new();
            for (i, preset) in params.rarity_presets.iter().enumerate() {
                if !seen_tiers.insert(preset.tier) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("duplicate rarity tier: '{}'", preset.tier),
                        format!("recipe.params.rarity_presets[{}].tier", i),
                    ));
                }

                // Validate border color
                for (j, &c) in preset.border_color.iter().enumerate() {
                    if !(0.0..=1.0).contains(&c) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("border_color[{}] must be in [0, 1], got {}", j, c),
                            format!("recipe.params.rarity_presets[{}].border_color[{}]", i, j),
                        ));
                    }
                }

                // Validate background color
                for (j, &c) in preset.background_color.iter().enumerate() {
                    if !(0.0..=1.0).contains(&c) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("background_color[{}] must be in [0, 1], got {}", j, c),
                            format!(
                                "recipe.params.rarity_presets[{}].background_color[{}]",
                                i, j
                            ),
                        ));
                    }
                }

                // Validate glow color if present
                if let Some(ref glow) = preset.glow_color {
                    for (j, &c) in glow.iter().enumerate() {
                        if !(0.0..=1.0).contains(&c) {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::InvalidRecipeParams,
                                format!("glow_color[{}] must be in [0, 1], got {}", j, c),
                                format!("recipe.params.rarity_presets[{}].glow_color[{}]", i, j),
                            ));
                        }
                    }
                }
            }

            // Validate slot regions are within card bounds
            let card_w = params.resolution[0];
            let card_h = params.resolution[1];

            let mut validate_slot = |name: &str, region: &[u32; 4], path: &str| {
                let end_x = region[0] + region[2];
                let end_y = region[1] + region[3];
                if end_x > card_w || end_y > card_h {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "{} slot region extends beyond card bounds: ends at ({}, {}) but card is ({}x{})",
                            name, end_x, end_y, card_w, card_h
                        ),
                        path,
                    ));
                }
            };

            validate_slot(
                "icon",
                &params.slots.icon_region,
                "recipe.params.slots.icon_region",
            );
            validate_slot(
                "rarity_indicator",
                &params.slots.rarity_indicator_region,
                "recipe.params.slots.rarity_indicator_region",
            );
            validate_slot(
                "background",
                &params.slots.background_region,
                "recipe.params.slots.background_region",
            );
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
                "ui.item_card_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.item_card_v1 metadata outputs must have format 'json'",
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

/// Validates outputs for `ui.damage_number_v1` recipe.
fn validate_ui_damage_number_outputs(spec: &Spec, recipe: &Recipe, result: &mut ValidationResult) {
    match recipe.as_ui_damage_number() {
        Ok(params) => {
            // Validate glyph size bounds (min 8x8, max 128x128)
            if params.glyph_size[0] < 8 || params.glyph_size[1] < 8 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "glyph_size must be at least 8x8, got [{}, {}]",
                        params.glyph_size[0], params.glyph_size[1]
                    ),
                    "recipe.params.glyph_size",
                ));
            }

            if params.glyph_size[0] > 128 || params.glyph_size[1] > 128 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "glyph_size must be at most 128x128, got [{}, {}]",
                        params.glyph_size[0], params.glyph_size[1]
                    ),
                    "recipe.params.glyph_size",
                ));
            }

            // Validate outline width (1-8)
            if params.outline_width < 1 || params.outline_width > 8 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "outline_width must be between 1 and 8, got {}",
                        params.outline_width
                    ),
                    "recipe.params.outline_width",
                ));
            }

            // Validate at least one style defined
            if params.styles.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "at least one style must be defined",
                    "recipe.params.styles",
                ));
            }

            // Validate colors and check for duplicate style types
            let mut seen_types = std::collections::HashSet::new();
            for (i, style) in params.styles.iter().enumerate() {
                if !seen_types.insert(&style.style_type) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("duplicate style_type: '{}'", style.style_type),
                        format!("recipe.params.styles[{}].style_type", i),
                    ));
                }

                // Validate text_color
                for (j, &c) in style.text_color.iter().enumerate() {
                    if !(0.0..=1.0).contains(&c) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("text_color[{}] must be in [0, 1], got {}", j, c),
                            format!("recipe.params.styles[{}].text_color[{}]", i, j),
                        ));
                    }
                }

                // Validate outline_color
                for (j, &c) in style.outline_color.iter().enumerate() {
                    if !(0.0..=1.0).contains(&c) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("outline_color[{}] must be in [0, 1], got {}", j, c),
                            format!("recipe.params.styles[{}].outline_color[{}]", i, j),
                        ));
                    }
                }

                // Validate glow_color if present
                if let Some(ref glow) = style.glow_color {
                    for (j, &c) in glow.iter().enumerate() {
                        if !(0.0..=1.0).contains(&c) {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::InvalidRecipeParams,
                                format!("glow_color[{}] must be in [0, 1], got {}", j, c),
                                format!("recipe.params.styles[{}].glow_color[{}]", i, j),
                            ));
                        }
                    }
                }

                // Validate scale if present (0.5 to 2.0)
                if let Some(scale) = style.scale {
                    if !(0.5..=2.0).contains(&scale) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("scale must be between 0.5 and 2.0, got {}", scale),
                            format!("recipe.params.styles[{}].scale", i),
                        ));
                    }
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

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.damage_number_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "ui.damage_number_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `sprite.render_from_mesh_v1` recipe.
///
/// This is a Tier 2 recipe (Blender backend) that renders a mesh from
/// multiple rotation angles and packs the frames into a sprite atlas.
fn validate_sprite_render_from_mesh_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_sprite_render_from_mesh() {
        Ok(params) => {
            // Validate frame resolution (max 1024x1024 per frame)
            if params.frame_resolution[0] > 1024 || params.frame_resolution[1] > 1024 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "frame_resolution must be at most 1024x1024, got [{}, {}]",
                        params.frame_resolution[0], params.frame_resolution[1]
                    ),
                    "recipe.params.frame_resolution",
                ));
            }

            // Validate frame resolution is positive
            if params.frame_resolution[0] == 0 || params.frame_resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "frame_resolution must be positive, got [{}, {}]",
                        params.frame_resolution[0], params.frame_resolution[1]
                    ),
                    "recipe.params.frame_resolution",
                ));
            }

            // Validate rotation_angles count (max 16)
            if params.rotation_angles.len() > 16 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "rotation_angles count must be at most 16, got {}",
                        params.rotation_angles.len()
                    ),
                    "recipe.params.rotation_angles",
                ));
            }

            // Validate rotation_angles is not empty
            if params.rotation_angles.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "rotation_angles must not be empty",
                    "recipe.params.rotation_angles",
                ));
            }

            // Validate camera distance is positive
            if params.camera_distance <= 0.0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "camera_distance must be positive, got {}",
                        params.camera_distance
                    ),
                    "recipe.params.camera_distance",
                ));
            }

            // Validate camera elevation is in valid range (-90 to 90)
            if !(-90.0..=90.0).contains(&params.camera_elevation) {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "camera_elevation must be in range [-90, 90] degrees, got {}",
                        params.camera_elevation
                    ),
                    "recipe.params.camera_elevation",
                ));
            }

            // Validate background color components are in [0, 1]
            for (i, &c) in params.background_color.iter().enumerate() {
                if !(0.0..=1.0).contains(&c) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("background_color[{}] must be in [0, 1], got {}", i, c),
                        format!("recipe.params.background_color[{}]", i),
                    ));
                }
            }

            // Validate mesh params (dimensions are positive)
            for (i, &dim) in params.mesh.dimensions.iter().enumerate() {
                if dim <= 0.0 {
                    let axis = ["X", "Y", "Z"][i];
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "mesh.dimensions[{}] ({}) must be positive, got {}",
                            i, axis, dim
                        ),
                        format!("recipe.params.mesh.dimensions[{}]", i),
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

    validate_primary_output_present(spec, result);

    // Primary output must be PNG (the atlas)
    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "sprite.render_from_mesh_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "sprite.render_from_mesh_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}
