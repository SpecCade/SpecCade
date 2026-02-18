//! Mesh recipe output validation (static_mesh, skeletal_mesh, skeletal_animation).

use crate::error::{ErrorCode, ValidationError, ValidationResult, ValidationWarning, WarningCode};
use crate::recipe::mesh::MeshPrimitive;
use crate::recipe::Recipe;

/// Validates params for `static_mesh.blender_primitives_v1` recipe.
///
/// This validates that the params match the expected schema and rejects
/// unknown fields.
pub(super) fn validate_static_mesh_blender_primitives(
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_static_mesh_blender_primitives() {
        Ok(params) => {
            // Validate dimensions are positive (plane allows Z=0)
            let is_plane = params.base_primitive == MeshPrimitive::Plane;
            for (i, &dim) in params.dimensions.iter().enumerate() {
                let allow_zero = is_plane && i == 2; // plane Z dimension is 0
                if dim < 0.0 || (!allow_zero && dim <= 0.0) {
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
pub(super) fn validate_static_mesh_modular_kit(recipe: &Recipe, result: &mut ValidationResult) {
    use crate::recipe::{ModularKitType, MAX_PIPE_SEGMENTS, MAX_WALL_CUTOUTS};

    match recipe.as_static_mesh_modular_kit() {
        Ok(params) => match &params.kit_type {
            ModularKitType::Wall(wall) => {
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
                for (i, cutout) in wall.cutouts.iter().enumerate() {
                    if cutout.width <= 0.0 {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("cutout[{}] width must be positive, got {}", i, cutout.width),
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
                if wall.bevel_width < 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("bevel_width must be non-negative, got {}", wall.bevel_width),
                        "recipe.params.kit_type.bevel_width",
                    ));
                }
            }
            ModularKitType::Pipe(pipe) => {
                if pipe.diameter <= 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("pipe diameter must be positive, got {}", pipe.diameter),
                        "recipe.params.kit_type.diameter",
                    ));
                }
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
                if pipe.vertices < 3 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("pipe vertices must be at least 3, got {}", pipe.vertices),
                        "recipe.params.kit_type.vertices",
                    ));
                }
                if pipe.bevel_width < 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("bevel_width must be non-negative, got {}", pipe.bevel_width),
                        "recipe.params.kit_type.bevel_width",
                    ));
                }
            }
            ModularKitType::Door(door) => {
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
                if door.bevel_width < 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("bevel_width must be non-negative, got {}", door.bevel_width),
                        "recipe.params.kit_type.bevel_width",
                    ));
                }
            }
        },
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
pub(super) fn validate_static_mesh_organic_sculpt(recipe: &Recipe, result: &mut ValidationResult) {
    use crate::recipe::{
        MAX_METABALLS, MAX_REMESH_VOXEL_SIZE, MAX_SMOOTH_ITERATIONS, MIN_REMESH_VOXEL_SIZE,
    };

    match recipe.as_static_mesh_organic_sculpt() {
        Ok(params) => {
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
pub(super) fn validate_skeletal_mesh_armature_driven(
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    fn add_invalid_params_error(result: &mut ValidationResult, message: String, path: String) {
        result.add_error(ValidationError::with_path(
            ErrorCode::InvalidRecipeParams,
            message,
            path,
        ));
    }

    fn validate_unit_interval(
        result: &mut ValidationResult,
        value: f64,
        field_name: &str,
        path: String,
    ) {
        if !(0.0..=1.0).contains(&value) {
            add_invalid_params_error(
                result,
                format!("{field_name} must be in range [0.0, 1.0], got {value}"),
                path,
            );
        }
    }

    fn validate_bone_part_shape(
        result: &mut ValidationResult,
        shape: &crate::recipe::character::BonePartShape,
        path_prefix: &str,
    ) {
        match shape {
            crate::recipe::character::BonePartShape::Primitive(primitive) => {
                for (idx, dim) in primitive.dimensions.iter().enumerate() {
                    if *dim <= 0.0 {
                        add_invalid_params_error(
                            result,
                            format!("dimensions[{idx}] must be positive, got {dim}"),
                            format!("{path_prefix}.dimensions[{idx}]"),
                        );
                    }
                }
            }
            crate::recipe::character::BonePartShape::Asset(asset) => {
                if asset.asset.trim().is_empty() {
                    add_invalid_params_error(
                        result,
                        "asset path must be a non-empty string".to_string(),
                        format!("{path_prefix}.asset"),
                    );
                }
                if let Some(scale) = asset.scale {
                    if !scale.is_finite() || scale <= 0.0 {
                        add_invalid_params_error(
                            result,
                            format!("asset.scale must be a finite, positive number, got {scale}"),
                            format!("{path_prefix}.scale"),
                        );
                    }
                }
            }
            crate::recipe::character::BonePartShape::AssetRef(asset_ref) => {
                let candidate = asset_ref.asset_ref.trim();
                if candidate.is_empty() {
                    add_invalid_params_error(
                        result,
                        "asset_ref must be a non-empty asset id".to_string(),
                        format!("{path_prefix}.asset_ref"),
                    );
                } else if !super::is_valid_asset_id(candidate) {
                    add_invalid_params_error(
                        result,
                        format!(
                            "asset_ref '{candidate}' must match the asset_id format (lowercase kebab/snake)"
                        ),
                        format!("{path_prefix}.asset_ref"),
                    );
                }
                if let Some(scale) = asset_ref.scale {
                    if !scale.is_finite() || scale <= 0.0 {
                        add_invalid_params_error(
                            result,
                            format!(
                                "asset_ref.scale must be a finite, positive number, got {scale}"
                            ),
                            format!("{path_prefix}.scale"),
                        );
                    }
                }
            }
        }
    }

    fn validate_bone_part_scale(
        result: &mut ValidationResult,
        scale: &Option<crate::recipe::character::BonePartScale>,
        path_prefix: &str,
    ) {
        let Some(scale) = scale.as_ref() else {
            return;
        };
        if let Some(axes) = &scale.axes {
            use std::collections::HashSet;
            let mut seen = HashSet::new();
            for axis in axes {
                if !seen.insert(*axis) {
                    add_invalid_params_error(
                        result,
                        format!("scale.axes contains duplicate axis '{axis:?}'"),
                        format!("{path_prefix}.axes"),
                    );
                }
            }
        }
        if let Some(amount) = &scale.amount_from_z {
            if let Some(v) = amount.x {
                validate_unit_interval(
                    result,
                    v,
                    "scale.amount_from_z.x",
                    format!("{path_prefix}.amount_from_z.x"),
                );
            }
            if let Some(v) = amount.y {
                validate_unit_interval(
                    result,
                    v,
                    "scale.amount_from_z.y",
                    format!("{path_prefix}.amount_from_z.y"),
                );
            }
            if let Some(v) = amount.z {
                validate_unit_interval(
                    result,
                    v,
                    "scale.amount_from_z.z",
                    format!("{path_prefix}.amount_from_z.z"),
                );
            }
        }
    }

    fn validate_profile(profile: &Option<String>) -> Result<(), String> {
        let Some(profile) = profile.as_ref() else {
            return Ok(());
        };
        let s = profile.trim();
        if s == "square" || s == "rectangle" {
            return Ok(());
        }
        let segments = s
            .strip_prefix("circle(")
            .and_then(|r| r.strip_suffix(')'))
            .or_else(|| s.strip_prefix("hexagon(").and_then(|r| r.strip_suffix(')')));
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
                        if let Some(part) = &mesh.part {
                            if !mesh.extrusion_steps.is_empty() {
                                add_invalid_params_error(
                                    result,
                                    "part and extrusion_steps are mutually exclusive; choose one"
                                        .to_string(),
                                    format!("recipe.params.bone_meshes.{bone_name}.part"),
                                );
                            }
                            let part_base_path =
                                format!("recipe.params.bone_meshes.{bone_name}.part.base");
                            validate_bone_part_shape(result, &part.base, &part_base_path);
                            for (op_idx, op) in part.operations.iter().enumerate() {
                                let op_path = format!(
                                    "recipe.params.bone_meshes.{bone_name}.part.operations[{op_idx}].target"
                                );
                                validate_bone_part_shape(result, &op.target, &op_path);
                            }
                            let scale_path =
                                format!("recipe.params.bone_meshes.{bone_name}.part.scale");
                            validate_bone_part_scale(result, &part.scale, &scale_path);
                            if matches!(
                                mesh.connect_start,
                                Some(crate::recipe::character::ConnectionMode::Bridge)
                            ) {
                                result.add_warning(ValidationWarning::with_path(
                                    WarningCode::UnusedRecipeParams,
                                    "connect_start='bridge' is ignored when part is set",
                                    format!("recipe.params.bone_meshes.{bone_name}.connect_start"),
                                ));
                            }
                            if matches!(
                                mesh.connect_end,
                                Some(crate::recipe::character::ConnectionMode::Bridge)
                            ) {
                                result.add_warning(ValidationWarning::with_path(
                                    WarningCode::UnusedRecipeParams,
                                    "connect_end='bridge' is ignored when part is set",
                                    format!("recipe.params.bone_meshes.{bone_name}.connect_end"),
                                ));
                            }
                        } else {
                            if let Err(msg) = validate_profile(&mesh.profile) {
                                result.add_error(ValidationError::with_path(
                                    ErrorCode::InvalidRecipeParams,
                                    msg,
                                    format!("recipe.params.bone_meshes.{bone_name}.profile"),
                                ));
                            }
                            for (idx, step) in mesh.extrusion_steps.iter().enumerate() {
                                let extrude_val = match step {
                                    crate::recipe::character::ExtrusionStep::Shorthand(d) => *d,
                                    crate::recipe::character::ExtrusionStep::Full(def) => {
                                        def.extrude
                                    }
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

/// Validates params for `skeletal_mesh.skinned_mesh_v1` recipe.
pub(super) fn validate_skeletal_mesh_skinned_mesh(recipe: &Recipe, result: &mut ValidationResult) {
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
pub(super) fn validate_skeletal_animation_blender_clip(
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
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
            if params.fps == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "fps must be greater than 0",
                    "recipe.params.fps",
                ));
            }
            if params.clip_name.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "clip_name must not be empty",
                    "recipe.params.clip_name",
                ));
            }
        }
        Err(e) => {
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
pub(super) fn validate_skeletal_animation_blender_rigged(
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_skeletal_animation_blender_rigged() {
        Ok(params) => {
            if params.duration_frames == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "duration_frames must be greater than 0",
                    "recipe.params.duration_frames",
                ));
            }
            if params.fps == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "fps must be greater than 0",
                    "recipe.params.fps",
                ));
            }
            if params.clip_name.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "clip_name must not be empty",
                    "recipe.params.clip_name",
                ));
            }
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
