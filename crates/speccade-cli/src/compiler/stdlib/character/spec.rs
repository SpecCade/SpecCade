//! Skeletal mesh spec creation function.

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, dict::DictRef, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::{validate_enum, validate_non_empty};
use super::{hashed_key, new_dict, SKELETON_PRESETS};

/// Valid mesh output formats.
///
/// Skeletal meshes currently validate/export as GLB only.
const MESH_FORMATS: &[&str] = &["glb"];

/// Registers skeletal mesh spec functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_skeletal_mesh_spec_functions(builder);
}

#[starlark_module]
fn register_skeletal_mesh_spec_functions(builder: &mut GlobalsBuilder) {
    /// Creates a complete skeletal mesh spec with the `skeletal_mesh.armature_driven_v1` recipe.
    ///
    /// # Arguments
    /// * `asset_id` - Kebab-case identifier for the asset
    /// * `seed` - Deterministic seed (0 to 2^32-1)
    /// * `output_path` - Output file path
    /// * `format` - Mesh format: "glb" or "gltf"
    /// * `skeleton_preset` - Skeleton preset name (e.g., "humanoid_basic_v1")
    /// * `bone_meshes` - Dict of per-bone mesh definitions (required)
    /// * `skeleton` - Optional list of custom bones (from custom_bone())
    /// * `bool_shapes` - Optional dict of boolean (subtraction) shapes
    /// * `material_slots` - Optional list of materials (from material_slot())
    /// * `export` - Optional export settings (from skeletal_export_settings())
    /// * `constraints` - Optional constraints (from skeletal_constraints())
    /// * `description` - Asset description (optional)
    /// * `tags` - Style tags (optional)
    /// * `license` - SPDX license identifier (default: "CC0-1.0")
    ///
    /// # Returns
    /// A complete spec dict ready for serialization.
    ///
    /// # Example
    /// ```starlark
    /// skeletal_mesh_spec(
    ///     asset_id = "test-character",
    ///     seed = 42,
    ///     output_path = "characters/test.glb",
    ///     format = "glb",
    ///     skeleton_preset = "humanoid_basic_v1",
    ///     bone_meshes = {
    ///         "chest": { "profile": "circle(8)", "profile_radius": 0.15 }
    ///     }
    /// )
    /// ```
    fn skeletal_mesh_spec<'v>(
        #[starlark(require = named)] asset_id: &str,
        #[starlark(require = named)] seed: i64,
        #[starlark(require = named)] output_path: &str,
        #[starlark(require = named)] format: &str,
        #[starlark(default = NoneType)] skeleton_preset: Value<'v>,
        #[starlark(default = NoneType)] bone_meshes: Value<'v>,
        #[starlark(default = NoneType)] skeleton: Value<'v>,
        #[starlark(default = NoneType)] bool_shapes: Value<'v>,
        #[starlark(default = NoneType)] material_slots: Value<'v>,
        #[starlark(default = NoneType)] export: Value<'v>,
        #[starlark(default = NoneType)] constraints: Value<'v>,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate asset_id
        validate_non_empty(asset_id, "skeletal_mesh_spec", "asset_id")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate format
        validate_enum(format, MESH_FORMATS, "skeletal_mesh_spec", "format")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate seed
        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: skeletal_mesh_spec(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        // Validate skeleton_preset if provided
        if !skeleton_preset.is_none() {
            if let Some(preset) = skeleton_preset.unpack_str() {
                validate_enum(
                    preset,
                    SKELETON_PRESETS,
                    "skeletal_mesh_spec",
                    "skeleton_preset",
                )
                .map_err(|e| anyhow::anyhow!(e))?;
            }
        }

        // Validate bone_meshes
        let bone_meshes_dict = DictRef::from_value(bone_meshes).ok_or_else(|| {
            anyhow::anyhow!(
                "S102: skeletal_mesh_spec(): 'bone_meshes' expected dict, got {}",
                bone_meshes.get_type()
            )
        })?;
        if bone_meshes_dict.iter().next().is_none() {
            return Err(anyhow::anyhow!(
                "S103: skeletal_mesh_spec(): 'bone_meshes' must not be empty"
            ));
        }

        let mut spec = new_dict(heap);

        // spec_version
        spec.insert_hashed(hashed_key(heap, "spec_version"), heap.alloc(1).to_value());

        // asset_id
        spec.insert_hashed(
            hashed_key(heap, "asset_id"),
            heap.alloc_str(asset_id).to_value(),
        );

        // asset_type
        spec.insert_hashed(
            hashed_key(heap, "asset_type"),
            heap.alloc_str("skeletal_mesh").to_value(),
        );

        // license
        spec.insert_hashed(
            hashed_key(heap, "license"),
            heap.alloc_str(license).to_value(),
        );

        // seed
        spec.insert_hashed(hashed_key(heap, "seed"), heap.alloc(seed).to_value());

        // outputs
        let mut output = new_dict(heap);
        output.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("primary").to_value(),
        );
        output.insert_hashed(
            hashed_key(heap, "format"),
            heap.alloc_str(format).to_value(),
        );
        output.insert_hashed(
            hashed_key(heap, "path"),
            heap.alloc_str(output_path).to_value(),
        );
        let outputs_list = heap.alloc(AllocList(vec![heap.alloc(output).to_value()]));
        spec.insert_hashed(hashed_key(heap, "outputs"), outputs_list);

        // Optional: description
        if !description.is_none() {
            if let Some(desc) = description.unpack_str() {
                spec.insert_hashed(
                    hashed_key(heap, "description"),
                    heap.alloc_str(desc).to_value(),
                );
            }
        }

        // Optional: style_tags
        if !tags.is_none() {
            spec.insert_hashed(hashed_key(heap, "style_tags"), tags);
        }

        // Build recipe
        let mut recipe = new_dict(heap);
        recipe.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("skeletal_mesh.armature_driven_v1")
                .to_value(),
        );

        // Create params
        let mut params = new_dict(heap);

        // skeleton_preset (default if neither preset nor skeleton provided)
        if !skeleton_preset.is_none() {
            if let Some(preset) = skeleton_preset.unpack_str() {
                params.insert_hashed(
                    hashed_key(heap, "skeleton_preset"),
                    heap.alloc_str(preset).to_value(),
                );
            }
        } else if skeleton.is_none() {
            params.insert_hashed(
                hashed_key(heap, "skeleton_preset"),
                heap.alloc_str("humanoid_basic_v1").to_value(),
            );
        }

        // skeleton (custom bones)
        if !skeleton.is_none() {
            params.insert_hashed(hashed_key(heap, "skeleton"), skeleton);
        }

        // bone_meshes
        params.insert_hashed(hashed_key(heap, "bone_meshes"), bone_meshes);

        // bool_shapes
        if !bool_shapes.is_none() {
            params.insert_hashed(hashed_key(heap, "bool_shapes"), bool_shapes);
        }

        // material_slots
        if !material_slots.is_none() {
            params.insert_hashed(hashed_key(heap, "material_slots"), material_slots);
        }

        // export
        if !export.is_none() {
            params.insert_hashed(hashed_key(heap, "export"), export);
        }

        // constraints
        if !constraints.is_none() {
            params.insert_hashed(hashed_key(heap, "constraints"), constraints);
        }

        recipe.insert_hashed(hashed_key(heap, "params"), heap.alloc(params).to_value());

        spec.insert_hashed(hashed_key(heap, "recipe"), heap.alloc(recipe).to_value());

        Ok(spec)
    }

    /// Creates a complete skeletal mesh spec with the `skeletal_mesh.skinned_mesh_v1` recipe.
    ///
    /// This binds an existing mesh to a skeleton.
    fn skeletal_mesh_skinned_spec<'v>(
        #[starlark(require = named)] asset_id: &str,
        #[starlark(require = named)] seed: i64,
        #[starlark(require = named)] output_path: &str,
        #[starlark(require = named)] format: &str,
        #[starlark(default = NoneType)] mesh_file: Value<'v>,
        #[starlark(default = NoneType)] mesh_asset: Value<'v>,
        #[starlark(default = NoneType)] skeleton_preset: Value<'v>,
        #[starlark(default = NoneType)] skeleton: Value<'v>,
        #[starlark(default = NoneType)] binding: Value<'v>,
        #[starlark(default = NoneType)] material_slots: Value<'v>,
        #[starlark(default = NoneType)] export: Value<'v>,
        #[starlark(default = NoneType)] constraints: Value<'v>,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(asset_id, "skeletal_mesh_skinned_spec", "asset_id")
            .map_err(|e| anyhow::anyhow!(e))?;

        validate_enum(format, MESH_FORMATS, "skeletal_mesh_skinned_spec", "format")
            .map_err(|e| anyhow::anyhow!(e))?;

        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: skeletal_mesh_skinned_spec(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        if mesh_file.is_none() && mesh_asset.is_none() {
            return Err(anyhow::anyhow!(
                "S103: skeletal_mesh_skinned_spec(): either 'mesh_file' or 'mesh_asset' must be provided"
            ));
        }

        // Validate skeleton_preset if provided
        if !skeleton_preset.is_none() {
            if let Some(preset) = skeleton_preset.unpack_str() {
                validate_enum(
                    preset,
                    SKELETON_PRESETS,
                    "skeletal_mesh_skinned_spec",
                    "skeleton_preset",
                )
                .map_err(|e| anyhow::anyhow!(e))?;
            }
        }

        let mut spec = new_dict(heap);
        spec.insert_hashed(hashed_key(heap, "spec_version"), heap.alloc(1).to_value());
        spec.insert_hashed(
            hashed_key(heap, "asset_id"),
            heap.alloc_str(asset_id).to_value(),
        );
        spec.insert_hashed(
            hashed_key(heap, "asset_type"),
            heap.alloc_str("skeletal_mesh").to_value(),
        );
        spec.insert_hashed(
            hashed_key(heap, "license"),
            heap.alloc_str(license).to_value(),
        );
        spec.insert_hashed(hashed_key(heap, "seed"), heap.alloc(seed).to_value());

        // outputs
        let mut output = new_dict(heap);
        output.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("primary").to_value(),
        );
        output.insert_hashed(
            hashed_key(heap, "format"),
            heap.alloc_str(format).to_value(),
        );
        output.insert_hashed(
            hashed_key(heap, "path"),
            heap.alloc_str(output_path).to_value(),
        );
        let outputs_list = heap.alloc(AllocList(vec![heap.alloc(output).to_value()]));
        spec.insert_hashed(hashed_key(heap, "outputs"), outputs_list);

        // Optional: description
        if !description.is_none() {
            if let Some(desc) = description.unpack_str() {
                spec.insert_hashed(
                    hashed_key(heap, "description"),
                    heap.alloc_str(desc).to_value(),
                );
            }
        }

        // Optional: style_tags
        if !tags.is_none() {
            spec.insert_hashed(hashed_key(heap, "style_tags"), tags);
        }

        let mut recipe = new_dict(heap);
        recipe.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("skeletal_mesh.skinned_mesh_v1").to_value(),
        );

        let mut params = new_dict(heap);

        if !mesh_file.is_none() {
            params.insert_hashed(hashed_key(heap, "mesh_file"), mesh_file);
        }
        if !mesh_asset.is_none() {
            params.insert_hashed(hashed_key(heap, "mesh_asset"), mesh_asset);
        }

        if !skeleton_preset.is_none() {
            if let Some(preset) = skeleton_preset.unpack_str() {
                params.insert_hashed(
                    hashed_key(heap, "skeleton_preset"),
                    heap.alloc_str(preset).to_value(),
                );
            }
        } else if skeleton.is_none() {
            params.insert_hashed(
                hashed_key(heap, "skeleton_preset"),
                heap.alloc_str("humanoid_basic_v1").to_value(),
            );
        }

        if !skeleton.is_none() {
            params.insert_hashed(hashed_key(heap, "skeleton"), skeleton);
        }

        if !binding.is_none() {
            params.insert_hashed(hashed_key(heap, "binding"), binding);
        } else {
            let mut default_binding = new_dict(heap);
            default_binding.insert_hashed(
                hashed_key(heap, "mode"),
                heap.alloc_str("auto_weights").to_value(),
            );
            default_binding.insert_hashed(
                hashed_key(heap, "vertex_group_map"),
                heap.alloc(new_dict(heap)).to_value(),
            );
            default_binding.insert_hashed(
                hashed_key(heap, "max_bone_influences"),
                heap.alloc(4).to_value(),
            );
            params.insert_hashed(
                hashed_key(heap, "binding"),
                heap.alloc(default_binding).to_value(),
            );
        }

        if !material_slots.is_none() {
            params.insert_hashed(hashed_key(heap, "material_slots"), material_slots);
        }
        if !export.is_none() {
            params.insert_hashed(hashed_key(heap, "export"), export);
        }
        if !constraints.is_none() {
            params.insert_hashed(hashed_key(heap, "constraints"), constraints);
        }

        recipe.insert_hashed(hashed_key(heap, "params"), heap.alloc(params).to_value());
        spec.insert_hashed(hashed_key(heap, "recipe"), heap.alloc(recipe).to_value());

        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::eval_to_json;

    // ========================================================================
    // skeletal_mesh_spec() tests
    // ========================================================================

    #[test]
    fn test_skeletal_mesh_spec_basic() {
        let result = eval_to_json(
            r#"
skeletal_mesh_spec(
    asset_id = "test-character",
    seed = 42,
    output_path = "characters/test.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    bone_meshes = { "spine": { "profile": "circle(8)", "profile_radius": 0.15 } }
)
"#,
        )
        .unwrap();

        assert_eq!(result["spec_version"], 1);
        assert_eq!(result["asset_id"], "test-character");
        assert_eq!(result["asset_type"], "skeletal_mesh");
        assert_eq!(result["seed"], 42);
        assert_eq!(result["recipe"]["kind"], "skeletal_mesh.armature_driven_v1");
        assert_eq!(
            result["recipe"]["params"]["skeleton_preset"],
            "humanoid_basic_v1"
        );
        assert!(result["outputs"].is_array());
    }

    #[test]
    fn test_skeletal_mesh_spec_requires_bone_meshes() {
        let result = eval_to_json(
            r#"
skeletal_mesh_spec(
    asset_id = "missing-bone-meshes",
    seed = 42,
    output_path = "characters/test.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1"
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("bone_meshes"));
    }

    #[test]
    fn test_skeletal_mesh_spec_with_bone_meshes() {
        let result = eval_to_json(
            r#"
skeletal_mesh_spec(
    asset_id = "character-with-parts",
    seed = 100,
    output_path = "char.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    bone_meshes = {
        "chest": { "profile": "circle(8)", "profile_radius": 0.15 },
        "head": { "profile": "circle(8)", "profile_radius": 0.12 }
    }
)
"#,
        )
        .unwrap();

        assert!(result["recipe"]["params"]["bone_meshes"].is_object());
        let bone_meshes = result["recipe"]["params"]["bone_meshes"]
            .as_object()
            .unwrap();
        assert_eq!(bone_meshes["chest"]["profile"], "circle(8)");
        assert_eq!(bone_meshes["head"]["profile"], "circle(8)");
    }

    #[test]
    fn test_skeletal_mesh_spec_with_all_options() {
        let result = eval_to_json(
            r#"
skeletal_mesh_spec(
    asset_id = "full-character",
    seed = 42,
    output_path = "full.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    bone_meshes = {
        "chest": { "profile": "circle(8)", "profile_radius": 0.15 }
    },
    material_slots = [
        material_slot(name = "skin", base_color = [0.8, 0.6, 0.5, 1.0])
    ],
    export = skeletal_export_settings(triangulate = True),
    constraints = skeletal_constraints(max_triangles = 5000),
    description = "Full character test"
)
"#,
        )
        .unwrap();

        assert_eq!(result["description"], "Full character test");
        assert!(result["recipe"]["params"]["material_slots"].is_array());
        assert!(result["recipe"]["params"]["export"].is_object());
        assert!(result["recipe"]["params"]["constraints"].is_object());
    }

    #[test]
    fn test_skeletal_mesh_spec_invalid_format() {
        let result = eval_to_json(
            r#"
skeletal_mesh_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.fbx",
    format = "fbx",
    skeleton_preset = "humanoid_basic_v1"
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
        assert!(err.contains("glb"));
    }

    #[test]
    fn test_skeletal_mesh_spec_invalid_preset() {
        let result = eval_to_json(
            r#"
skeletal_mesh_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.glb",
    format = "glb",
    skeleton_preset = "invalid_skeleton"
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
        assert!(err.contains("humanoid_basic_v1"));
    }

    #[test]
    fn test_skeletal_mesh_spec_invalid_seed() {
        let result = eval_to_json(
            r#"
skeletal_mesh_spec(
    asset_id = "test",
    seed = -1,
    output_path = "test.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1"
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("seed"));
    }

    #[test]
    fn test_skeletal_mesh_spec_defaults_skeleton_preset() {
        let result = eval_to_json(
            r#"
skeletal_mesh_spec(
    asset_id = "default-skeleton-char",
    seed = 42,
    output_path = "default.glb",
    format = "glb",
    bone_meshes = { "spine": { "profile": "circle(8)", "profile_radius": 0.15 } }
)
"#,
        )
        .unwrap();

        assert_eq!(
            result["recipe"]["params"]["skeleton_preset"],
            "humanoid_basic_v1"
        );
    }

    #[test]
    fn test_skeletal_mesh_spec_with_custom_skeleton() {
        let result = eval_to_json(
            r#"
skeletal_mesh_spec(
    asset_id = "custom-skeleton-char",
    seed = 42,
    output_path = "custom.glb",
    format = "glb",
    skeleton = [
        custom_bone(bone = "root", head = [0, 0, 0], tail = [0, 0, 0.1]),
        custom_bone(bone = "spine", parent = "root", head = [0, 0, 0.1], tail = [0, 0, 0.3])
    ],
    bone_meshes = {
        "spine": { "profile": "circle(8)", "profile_radius": 0.15 }
    }
)
"#,
        )
        .unwrap();

        assert!(result["recipe"]["params"]["skeleton"].is_array());
        let skeleton = result["recipe"]["params"]["skeleton"].as_array().unwrap();
        assert_eq!(skeleton.len(), 2);
        assert_eq!(skeleton[0]["bone"], "root");
        assert_eq!(skeleton[1]["parent"], "root");
    }

    // ========================================================================
    // skeletal_mesh_skinned_spec() tests
    // ========================================================================

    #[test]
    fn test_skeletal_mesh_skinned_spec_basic() {
        let result = eval_to_json(
            r#"
skeletal_mesh_skinned_spec(
    asset_id = "skinned-character",
    seed = 42,
    output_path = "characters/skinned.glb",
    format = "glb",
    mesh_file = "meshes/character.glb",
    skeleton_preset = "humanoid_basic_v1"
)
"#,
        )
        .unwrap();

        assert_eq!(result["recipe"]["kind"], "skeletal_mesh.skinned_mesh_v1");
        assert_eq!(
            result["recipe"]["params"]["mesh_file"],
            "meshes/character.glb"
        );
        assert_eq!(
            result["recipe"]["params"]["skeleton_preset"],
            "humanoid_basic_v1"
        );
        assert!(result["recipe"]["params"]["binding"].is_object());
        assert_eq!(
            result["recipe"]["params"]["binding"]["mode"],
            "auto_weights"
        );
    }

    #[test]
    fn test_skeletal_mesh_skinned_spec_defaults_skeleton_preset() {
        let result = eval_to_json(
            r#"
skeletal_mesh_skinned_spec(
    asset_id = "skinned-default-skeleton-char",
    seed = 42,
    output_path = "characters/skinned_default.glb",
    format = "glb",
    mesh_file = "meshes/character.glb"
)
"#,
        )
        .unwrap();

        assert_eq!(
            result["recipe"]["params"]["skeleton_preset"],
            "humanoid_basic_v1"
        );
    }
}
