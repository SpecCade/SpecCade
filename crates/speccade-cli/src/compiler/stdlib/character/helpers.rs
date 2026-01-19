//! Helper functions for skeletal mesh construction.

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::{validate_enum, validate_non_empty, validate_positive_int};
use super::{hashed_key, new_dict, PRIMITIVES, UV_MODES};

/// Registers character helper functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_character_helpers(builder);
}

#[starlark_module]
fn register_character_helpers(builder: &mut GlobalsBuilder) {
    /// Creates a body part definition attached to a bone.
    ///
    /// # Arguments
    /// * `bone` - Name of the bone to attach to
    /// * `primitive` - Mesh primitive type: "cube", "sphere", "cylinder", etc.
    /// * `dimensions` - [X, Y, Z] dimensions
    /// * `segments` - Optional segment count for curved primitives
    /// * `offset` - Optional [X, Y, Z] position offset from bone
    /// * `rotation` - Optional [X, Y, Z] euler rotation in degrees
    /// * `material_index` - Optional material slot index
    ///
    /// # Example
    /// ```starlark
    /// body_part(
    ///     bone = "chest",
    ///     primitive = "cylinder",
    ///     dimensions = [0.3, 0.3, 0.28],
    ///     segments = 8,
    ///     offset = [0, 0, 0.6]
    /// )
    /// ```
    fn body_part<'v>(
        #[starlark(require = named)] bone: &str,
        #[starlark(require = named)] primitive: &str,
        #[starlark(require = named)] dimensions: UnpackList<f64>,
        #[starlark(default = NoneType)] segments: Value<'v>,
        #[starlark(default = NoneType)] offset: Value<'v>,
        #[starlark(default = NoneType)] rotation: Value<'v>,
        #[starlark(default = NoneType)] material_index: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate bone name
        validate_non_empty(bone, "body_part", "bone").map_err(|e| anyhow::anyhow!(e))?;

        // Validate primitive
        validate_enum(primitive, PRIMITIVES, "body_part", "primitive")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate dimensions
        if dimensions.items.len() != 3 {
            return Err(anyhow::anyhow!(
                "S101: body_part(): 'dimensions' must be [x, y, z], got {} values",
                dimensions.items.len()
            ));
        }

        let mut result = new_dict(heap);

        // bone
        result.insert_hashed(hashed_key(heap, "bone"), heap.alloc_str(bone).to_value());

        // Build mesh sub-dict
        let mut mesh = new_dict(heap);
        mesh.insert_hashed(
            hashed_key(heap, "primitive"),
            heap.alloc_str(primitive).to_value(),
        );

        // dimensions
        let dim_list = heap.alloc(AllocList(vec![
            heap.alloc(dimensions.items[0]).to_value(),
            heap.alloc(dimensions.items[1]).to_value(),
            heap.alloc(dimensions.items[2]).to_value(),
        ]));
        mesh.insert_hashed(hashed_key(heap, "dimensions"), dim_list);

        // Optional: segments
        if !segments.is_none() {
            if let Some(seg) = segments.unpack_i32() {
                mesh.insert_hashed(hashed_key(heap, "segments"), heap.alloc(seg).to_value());
            }
        }

        // Optional: offset
        if !offset.is_none() {
            mesh.insert_hashed(hashed_key(heap, "offset"), offset);
        }

        // Optional: rotation
        if !rotation.is_none() {
            mesh.insert_hashed(hashed_key(heap, "rotation"), rotation);
        }

        result.insert_hashed(hashed_key(heap, "mesh"), heap.alloc(mesh).to_value());

        // Optional: material_index
        if !material_index.is_none() {
            if let Some(idx) = material_index.unpack_i32() {
                result.insert_hashed(
                    hashed_key(heap, "material_index"),
                    heap.alloc(idx).to_value(),
                );
            }
        }

        Ok(result)
    }

    /// Creates a material slot definition.
    ///
    /// # Arguments
    /// * `name` - Material name
    /// * `base_color` - Optional [R, G, B, A] color (0.0 to 1.0)
    /// * `metallic` - Optional metallic value (0.0 to 1.0)
    /// * `roughness` - Optional roughness value (0.0 to 1.0)
    /// * `emissive` - Optional [R, G, B] emissive color
    ///
    /// # Example
    /// ```starlark
    /// material_slot(
    ///     name = "skin",
    ///     base_color = [0.8, 0.6, 0.5, 1.0]
    /// )
    /// ```
    fn material_slot<'v>(
        #[starlark(require = named)] name: &str,
        #[starlark(default = NoneType)] base_color: Value<'v>,
        #[starlark(default = NoneType)] metallic: Value<'v>,
        #[starlark(default = NoneType)] roughness: Value<'v>,
        #[starlark(default = NoneType)] emissive: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(name, "material_slot", "name").map_err(|e| anyhow::anyhow!(e))?;

        let mut result = new_dict(heap);

        result.insert_hashed(hashed_key(heap, "name"), heap.alloc_str(name).to_value());

        if !base_color.is_none() {
            result.insert_hashed(hashed_key(heap, "base_color"), base_color);
        }

        if !metallic.is_none() {
            result.insert_hashed(hashed_key(heap, "metallic"), metallic);
        }

        if !roughness.is_none() {
            result.insert_hashed(hashed_key(heap, "roughness"), roughness);
        }

        if !emissive.is_none() {
            result.insert_hashed(hashed_key(heap, "emissive"), emissive);
        }

        Ok(result)
    }

    /// Creates skinning configuration.
    ///
    /// # Arguments
    /// * `max_bone_influences` - Max bone influences per vertex (1-8, default 4)
    /// * `auto_weights` - Use automatic weight painting (default true)
    ///
    /// # Example
    /// ```starlark
    /// skinning_config(max_bone_influences = 4, auto_weights = True)
    /// ```
    fn skinning_config<'v>(
        #[starlark(default = 4)] max_bone_influences: i64,
        #[starlark(default = true)] auto_weights: bool,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate max_bone_influences range (1-8)
        if max_bone_influences < 1 || max_bone_influences > 8 {
            return Err(anyhow::anyhow!(
                "S103: skinning_config(): 'max_bone_influences' must be in range 1 to 8, got {}",
                max_bone_influences
            ));
        }

        let mut result = new_dict(heap);

        result.insert_hashed(
            hashed_key(heap, "max_bone_influences"),
            heap.alloc(max_bone_influences).to_value(),
        );

        result.insert_hashed(
            hashed_key(heap, "auto_weights"),
            heap.alloc(auto_weights).to_value(),
        );

        Ok(result)
    }

    /// Creates a custom bone definition for custom skeletons.
    ///
    /// # Arguments
    /// * `bone` - Unique bone name
    /// * `head` - Optional head position [X, Y, Z]
    /// * `tail` - Optional tail position [X, Y, Z]
    /// * `parent` - Optional parent bone name
    /// * `mirror` - Optional bone to mirror from (L->R reflection)
    ///
    /// # Example
    /// ```starlark
    /// custom_bone(
    ///     bone = "spine",
    ///     head = [0, 0, 0.1],
    ///     tail = [0, 0, 0.3],
    ///     parent = "root"
    /// )
    /// ```
    fn custom_bone<'v>(
        #[starlark(require = named)] bone: &str,
        #[starlark(default = NoneType)] head: Value<'v>,
        #[starlark(default = NoneType)] tail: Value<'v>,
        #[starlark(default = NoneType)] parent: Value<'v>,
        #[starlark(default = NoneType)] mirror: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(bone, "custom_bone", "bone").map_err(|e| anyhow::anyhow!(e))?;

        let mut result = new_dict(heap);

        result.insert_hashed(hashed_key(heap, "bone"), heap.alloc_str(bone).to_value());

        if !head.is_none() {
            result.insert_hashed(hashed_key(heap, "head"), head);
        }

        if !tail.is_none() {
            result.insert_hashed(hashed_key(heap, "tail"), tail);
        }

        if !parent.is_none() {
            if let Some(p) = parent.unpack_str() {
                result.insert_hashed(hashed_key(heap, "parent"), heap.alloc_str(p).to_value());
            }
        }

        if !mirror.is_none() {
            if let Some(m) = mirror.unpack_str() {
                result.insert_hashed(hashed_key(heap, "mirror"), heap.alloc_str(m).to_value());
            }
        }

        Ok(result)
    }

    /// Creates skeletal mesh export settings.
    ///
    /// # Arguments
    /// * `include_armature` - Include armature in export (default true)
    /// * `include_normals` - Include vertex normals (default true)
    /// * `include_uvs` - Include UV coordinates (default true)
    /// * `triangulate` - Triangulate mesh (default true)
    /// * `include_skin_weights` - Include skin weights (default true)
    /// * `save_blend` - Save .blend file alongside GLB (default false)
    ///
    /// # Example
    /// ```starlark
    /// skeletal_export_settings(triangulate = True, save_blend = False)
    /// ```
    fn skeletal_export_settings<'v>(
        #[starlark(default = true)] include_armature: bool,
        #[starlark(default = true)] include_normals: bool,
        #[starlark(default = true)] include_uvs: bool,
        #[starlark(default = true)] triangulate: bool,
        #[starlark(default = true)] include_skin_weights: bool,
        #[starlark(default = false)] save_blend: bool,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        let mut result = new_dict(heap);

        result.insert_hashed(
            hashed_key(heap, "include_armature"),
            heap.alloc(include_armature).to_value(),
        );
        result.insert_hashed(
            hashed_key(heap, "include_normals"),
            heap.alloc(include_normals).to_value(),
        );
        result.insert_hashed(
            hashed_key(heap, "include_uvs"),
            heap.alloc(include_uvs).to_value(),
        );
        result.insert_hashed(
            hashed_key(heap, "triangulate"),
            heap.alloc(triangulate).to_value(),
        );
        result.insert_hashed(
            hashed_key(heap, "include_skin_weights"),
            heap.alloc(include_skin_weights).to_value(),
        );
        result.insert_hashed(
            hashed_key(heap, "save_blend"),
            heap.alloc(save_blend).to_value(),
        );

        Ok(result)
    }

    /// Creates skeletal mesh constraints for validation.
    ///
    /// # Arguments
    /// * `max_triangles` - Maximum triangle count (optional)
    /// * `max_bones` - Maximum bone count (optional)
    /// * `max_materials` - Maximum material count (optional)
    ///
    /// # Example
    /// ```starlark
    /// skeletal_constraints(max_triangles = 5000, max_bones = 64)
    /// ```
    fn skeletal_constraints<'v>(
        #[starlark(default = NoneType)] max_triangles: Value<'v>,
        #[starlark(default = NoneType)] max_bones: Value<'v>,
        #[starlark(default = NoneType)] max_materials: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        let mut result = new_dict(heap);

        if !max_triangles.is_none() {
            if let Some(v) = max_triangles.unpack_i32() {
                validate_positive_int(v as i64, "skeletal_constraints", "max_triangles")
                    .map_err(|e| anyhow::anyhow!(e))?;
                result.insert_hashed(hashed_key(heap, "max_triangles"), heap.alloc(v).to_value());
            }
        }

        if !max_bones.is_none() {
            if let Some(v) = max_bones.unpack_i32() {
                validate_positive_int(v as i64, "skeletal_constraints", "max_bones")
                    .map_err(|e| anyhow::anyhow!(e))?;
                result.insert_hashed(hashed_key(heap, "max_bones"), heap.alloc(v).to_value());
            }
        }

        if !max_materials.is_none() {
            if let Some(v) = max_materials.unpack_i32() {
                validate_positive_int(v as i64, "skeletal_constraints", "max_materials")
                    .map_err(|e| anyhow::anyhow!(e))?;
                result.insert_hashed(hashed_key(heap, "max_materials"), heap.alloc(v).to_value());
            }
        }

        Ok(result)
    }

    /// Creates texturing configuration for skeletal meshes.
    ///
    /// # Arguments
    /// * `uv_mode` - UV unwrapping mode: "cylinder_project", "box_project", "sphere_project", "smart"
    ///
    /// # Example
    /// ```starlark
    /// skeletal_texturing(uv_mode = "cylinder_project")
    /// ```
    fn skeletal_texturing<'v>(
        #[starlark(default = "cylinder_project")] uv_mode: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_enum(uv_mode, UV_MODES, "skeletal_texturing", "uv_mode")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut result = new_dict(heap);

        result.insert_hashed(
            hashed_key(heap, "uv_mode"),
            heap.alloc_str(uv_mode).to_value(),
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::eval_to_json;

    // ========================================================================
    // body_part() tests
    // ========================================================================

    #[test]
    fn test_body_part_basic() {
        let result = eval_to_json(
            r#"
body_part(
    bone = "chest",
    primitive = "cylinder",
    dimensions = [0.3, 0.3, 0.28]
)
"#,
        )
        .unwrap();
        assert_eq!(result["bone"], "chest");
        assert_eq!(result["mesh"]["primitive"], "cylinder");
        assert!(result["mesh"]["dimensions"].is_array());
    }

    #[test]
    fn test_body_part_with_options() {
        let result = eval_to_json(
            r#"
body_part(
    bone = "head",
    primitive = "sphere",
    dimensions = [0.15, 0.18, 0.15],
    segments = 12,
    offset = [0, 0, 0.95],
    material_index = 1
)
"#,
        )
        .unwrap();
        assert_eq!(result["bone"], "head");
        assert_eq!(result["mesh"]["segments"], 12);
        assert!(result["mesh"]["offset"].is_array());
        assert_eq!(result["material_index"], 1);
    }

    #[test]
    fn test_body_part_invalid_primitive() {
        let result = eval_to_json(
            r#"
body_part(
    bone = "chest",
    primitive = "pyramid",
    dimensions = [1.0, 1.0, 1.0]
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
    }

    // ========================================================================
    // material_slot() tests
    // ========================================================================

    #[test]
    fn test_material_slot_basic() {
        let result = eval_to_json(
            r#"
material_slot(name = "skin")
"#,
        )
        .unwrap();
        assert_eq!(result["name"], "skin");
    }

    #[test]
    fn test_material_slot_with_color() {
        let result = eval_to_json(
            r#"
material_slot(
    name = "metal",
    base_color = [0.8, 0.8, 0.9, 1.0],
    metallic = 1.0,
    roughness = 0.2
)
"#,
        )
        .unwrap();
        assert_eq!(result["name"], "metal");
        assert!(result["base_color"].is_array());
        assert_eq!(result["metallic"], 1.0);
        assert_eq!(result["roughness"], 0.2);
    }

    // ========================================================================
    // skinning_config() tests
    // ========================================================================

    #[test]
    fn test_skinning_config_default() {
        let result = eval_to_json("skinning_config()").unwrap();
        assert_eq!(result["max_bone_influences"], 4);
        assert_eq!(result["auto_weights"], true);
    }

    #[test]
    fn test_skinning_config_custom() {
        let result = eval_to_json("skinning_config(max_bone_influences = 2, auto_weights = False)")
            .unwrap();
        assert_eq!(result["max_bone_influences"], 2);
        assert_eq!(result["auto_weights"], false);
    }

    #[test]
    fn test_skinning_config_invalid_range() {
        let result = eval_to_json("skinning_config(max_bone_influences = 10)");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("range 1 to 8"));
    }

    // ========================================================================
    // custom_bone() tests
    // ========================================================================

    #[test]
    fn test_custom_bone_basic() {
        let result = eval_to_json(r#"custom_bone(bone = "root")"#).unwrap();
        assert_eq!(result["bone"], "root");
    }

    #[test]
    fn test_custom_bone_with_positions() {
        let result = eval_to_json(
            r#"
custom_bone(
    bone = "spine",
    head = [0, 0, 0.1],
    tail = [0, 0, 0.3],
    parent = "root"
)
"#,
        )
        .unwrap();
        assert_eq!(result["bone"], "spine");
        assert!(result["head"].is_array());
        assert!(result["tail"].is_array());
        assert_eq!(result["parent"], "root");
    }

    #[test]
    fn test_custom_bone_with_mirror() {
        let result = eval_to_json(
            r#"
custom_bone(
    bone = "arm_r",
    mirror = "arm_l",
    parent = "spine"
)
"#,
        )
        .unwrap();
        assert_eq!(result["mirror"], "arm_l");
    }

    // ========================================================================
    // skeletal_export_settings() tests
    // ========================================================================

    #[test]
    fn test_skeletal_export_settings_default() {
        let result = eval_to_json("skeletal_export_settings()").unwrap();
        assert_eq!(result["include_armature"], true);
        assert_eq!(result["triangulate"], true);
        assert_eq!(result["save_blend"], false);
    }

    #[test]
    fn test_skeletal_export_settings_custom() {
        let result =
            eval_to_json("skeletal_export_settings(save_blend = True, triangulate = False)")
                .unwrap();
        assert_eq!(result["save_blend"], true);
        assert_eq!(result["triangulate"], false);
    }

    // ========================================================================
    // skeletal_constraints() tests
    // ========================================================================

    #[test]
    fn test_skeletal_constraints_basic() {
        let result =
            eval_to_json("skeletal_constraints(max_triangles = 5000, max_bones = 64)").unwrap();
        assert_eq!(result["max_triangles"], 5000);
        assert_eq!(result["max_bones"], 64);
    }

    #[test]
    fn test_skeletal_constraints_invalid() {
        let result = eval_to_json("skeletal_constraints(max_triangles = -1)");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("positive"));
    }

    // ========================================================================
    // skeletal_texturing() tests
    // ========================================================================

    #[test]
    fn test_skeletal_texturing_default() {
        let result = eval_to_json("skeletal_texturing()").unwrap();
        assert_eq!(result["uv_mode"], "cylinder_project");
    }

    #[test]
    fn test_skeletal_texturing_custom() {
        let result = eval_to_json(r#"skeletal_texturing(uv_mode = "smart")"#).unwrap();
        assert_eq!(result["uv_mode"], "smart");
    }

    #[test]
    fn test_skeletal_texturing_invalid() {
        let result = eval_to_json(r#"skeletal_texturing(uv_mode = "invalid_mode")"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
    }
}
