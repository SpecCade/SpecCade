//! Mesh spec creation functions.

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::{validate_enum, validate_non_empty};
use super::{hashed_key, new_dict, MESH_FORMATS, PRIMITIVES};

/// Registers mesh spec functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_mesh_spec_functions(builder);
}

#[starlark_module]
fn register_mesh_spec_functions(builder: &mut GlobalsBuilder) {
    /// Creates a complete mesh spec with blender_primitives_v1 recipe.
    ///
    /// # Arguments
    /// * `asset_id` - Kebab-case identifier for the asset
    /// * `seed` - Deterministic seed (0 to 2^32-1)
    /// * `output_path` - Output file path
    /// * `format` - Mesh format: "glb", "gltf", "obj", "fbx"
    /// * `primitive` - Primitive type
    /// * `dimensions` - [x, y, z] dimensions in Blender units
    /// * `modifiers` - Optional list of modifiers
    /// * `description` - Asset description (optional)
    /// * `tags` - Style tags (optional)
    /// * `license` - SPDX license identifier (default: "CC0-1.0")
    ///
    /// # Returns
    /// A complete spec dict ready for serialization.
    ///
    /// # Example
    /// ```starlark
    /// mesh_spec(
    ///     asset_id = "test-mesh-01",
    ///     seed = 42,
    ///     output_path = "meshes/test.glb",
    ///     format = "glb",
    ///     primitive = "cube",
    ///     dimensions = [1.0, 1.0, 1.0]
    /// )
    /// ```
    fn mesh_spec<'v>(
        #[starlark(require = named)] asset_id: &str,
        #[starlark(require = named)] seed: i64,
        #[starlark(require = named)] output_path: &str,
        #[starlark(require = named)] format: &str,
        #[starlark(require = named)] primitive: &str,
        #[starlark(require = named)] dimensions: UnpackList<f64>,
        #[starlark(default = NoneType)] modifiers: Value<'v>,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate asset_id
        validate_non_empty(asset_id, "mesh_spec", "asset_id").map_err(|e| anyhow::anyhow!(e))?;

        // Validate format
        validate_enum(format, MESH_FORMATS, "mesh_spec", "format")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate primitive
        validate_enum(primitive, PRIMITIVES, "mesh_spec", "primitive")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate seed
        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: mesh_spec(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        // Validate dimensions
        if dimensions.items.len() != 3 {
            return Err(anyhow::anyhow!(
                "S101: mesh_spec(): 'dimensions' must be [x, y, z], got {} values",
                dimensions.items.len()
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
            heap.alloc_str("static_mesh").to_value(),
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
            heap.alloc_str("static_mesh.blender_primitives_v1")
                .to_value(),
        );

        // Create params
        let mut params = new_dict(heap);

        params.insert_hashed(
            hashed_key(heap, "base_primitive"),
            heap.alloc_str(primitive).to_value(),
        );

        // dimensions as list
        let dim_list = heap.alloc(AllocList(vec![
            heap.alloc(dimensions.items[0]).to_value(),
            heap.alloc(dimensions.items[1]).to_value(),
            heap.alloc(dimensions.items[2]).to_value(),
        ]));
        params.insert_hashed(hashed_key(heap, "dimensions"), dim_list);

        // modifiers - add empty list if None
        if modifiers.is_none() {
            let empty_list: Vec<Value> = vec![];
            params.insert_hashed(
                hashed_key(heap, "modifiers"),
                heap.alloc(AllocList(empty_list)),
            );
        } else {
            params.insert_hashed(hashed_key(heap, "modifiers"), modifiers);
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
    // mesh_spec() tests
    // ========================================================================

    #[test]
    fn test_mesh_spec_basic() {
        let result = eval_to_json(
            r#"
mesh_spec(
    asset_id = "test-mesh-01",
    seed = 42,
    output_path = "meshes/test.glb",
    format = "glb",
    primitive = "cube",
    dimensions = [1.0, 1.0, 1.0]
)
"#,
        )
        .unwrap();
        assert_eq!(result["spec_version"], 1);
        assert_eq!(result["asset_id"], "test-mesh-01");
        assert_eq!(result["asset_type"], "static_mesh");
        assert_eq!(result["seed"], 42);
        assert_eq!(
            result["recipe"]["kind"],
            "static_mesh.blender_primitives_v1"
        );
        assert_eq!(result["recipe"]["params"]["base_primitive"], "cube");
        assert!(result["recipe"]["params"]["dimensions"].is_array());
        assert!(result["outputs"].is_array());
    }

    #[test]
    fn test_mesh_spec_invalid_format() {
        let result = eval_to_json(
            r#"
mesh_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.stl",
    format = "stl",
    primitive = "cube",
    dimensions = [1.0, 1.0, 1.0]
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
    }

    #[test]
    fn test_mesh_spec_invalid_seed() {
        let result = eval_to_json(
            r#"
mesh_spec(
    asset_id = "test",
    seed = -1,
    output_path = "test.glb",
    format = "glb",
    primitive = "cube",
    dimensions = [1.0, 1.0, 1.0]
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("seed"));
    }
}
