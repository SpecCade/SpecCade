//! Mesh primitive creation functions.

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::validate_enum;
use super::{hashed_key, new_dict, PRIMITIVES};

/// Registers mesh primitive functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_mesh_primitive_functions(builder);
}

#[starlark_module]
fn register_mesh_primitive_functions(builder: &mut GlobalsBuilder) {
    /// Creates a base mesh primitive specification.
    ///
    /// # Arguments
    /// * `primitive` - Primitive type: "cube", "sphere", "cylinder", "cone", "torus", "plane", "ico_sphere"
    /// * `dimensions` - [x, y, z] dimensions in Blender units
    ///
    /// # Returns
    /// A dict with base_primitive and dimensions.
    ///
    /// # Example
    /// ```starlark
    /// mesh_primitive("cube", [1.0, 1.0, 1.0])
    /// mesh_primitive("sphere", [2.0, 2.0, 2.0])
    /// ```
    fn mesh_primitive<'v>(
        primitive: &str,
        dimensions: UnpackList<f64>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_enum(primitive, PRIMITIVES, "mesh_primitive", "primitive")
            .map_err(|e| anyhow::anyhow!(e))?;

        if dimensions.items.len() != 3 {
            return Err(anyhow::anyhow!(
                "S101: mesh_primitive(): 'dimensions' must be [x, y, z], got {} values",
                dimensions.items.len()
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "base_primitive"),
            heap.alloc_str(primitive).to_value(),
        );

        // dimensions as list
        let dim_list = heap.alloc(AllocList(vec![
            heap.alloc(dimensions.items[0]).to_value(),
            heap.alloc(dimensions.items[1]).to_value(),
            heap.alloc(dimensions.items[2]).to_value(),
        ]));
        dict.insert_hashed(hashed_key(heap, "dimensions"), dim_list);

        Ok(dict)
    }

    /// Creates a complete static mesh recipe params.
    ///
    /// # Arguments
    /// * `primitive` - Primitive type
    /// * `dimensions` - [x, y, z] dimensions
    /// * `modifiers` - Optional list of modifiers
    ///
    /// # Returns
    /// A dict matching the StaticMeshBlenderPrimitivesV1Params structure.
    ///
    /// # Example
    /// ```starlark
    /// mesh_recipe("cube", [1.0, 1.0, 1.0])
    /// mesh_recipe(
    ///     "cube",
    ///     [1.0, 1.0, 1.0],
    ///     [bevel_modifier(0.02, 2), subdivision_modifier(2)]
    /// )
    /// ```
    fn mesh_recipe<'v>(
        primitive: &str,
        dimensions: UnpackList<f64>,
        #[starlark(default = NoneType)] modifiers: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_enum(primitive, PRIMITIVES, "mesh_recipe", "primitive")
            .map_err(|e| anyhow::anyhow!(e))?;

        if dimensions.items.len() != 3 {
            return Err(anyhow::anyhow!(
                "S101: mesh_recipe(): 'dimensions' must be [x, y, z], got {} values",
                dimensions.items.len()
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "base_primitive"),
            heap.alloc_str(primitive).to_value(),
        );

        // dimensions as list
        let dim_list = heap.alloc(AllocList(vec![
            heap.alloc(dimensions.items[0]).to_value(),
            heap.alloc(dimensions.items[1]).to_value(),
            heap.alloc(dimensions.items[2]).to_value(),
        ]));
        dict.insert_hashed(hashed_key(heap, "dimensions"), dim_list);

        // modifiers - add empty list if None
        if modifiers.is_none() {
            let empty_list: Vec<Value> = vec![];
            dict.insert_hashed(
                hashed_key(heap, "modifiers"),
                heap.alloc(AllocList(empty_list)),
            );
        } else {
            dict.insert_hashed(hashed_key(heap, "modifiers"), modifiers);
        }

        Ok(dict)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::eval_to_json;

    // ========================================================================
    // mesh_primitive() tests
    // ========================================================================

    #[test]
    fn test_mesh_primitive_cube() {
        let result = eval_to_json("mesh_primitive(\"cube\", [1.0, 1.0, 1.0])").unwrap();
        assert_eq!(result["base_primitive"], "cube");
        assert!(result["dimensions"].is_array());
        let dims = result["dimensions"].as_array().unwrap();
        assert_eq!(dims.len(), 3);
        assert_eq!(dims[0], 1.0);
        assert_eq!(dims[1], 1.0);
        assert_eq!(dims[2], 1.0);
    }

    #[test]
    fn test_mesh_primitive_sphere() {
        let result = eval_to_json("mesh_primitive(\"sphere\", [2.0, 2.0, 2.0])").unwrap();
        assert_eq!(result["base_primitive"], "sphere");
    }

    #[test]
    fn test_mesh_primitive_invalid() {
        let result = eval_to_json("mesh_primitive(\"box\", [1.0, 1.0, 1.0])");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
    }

    #[test]
    fn test_mesh_primitive_wrong_dimensions() {
        let result = eval_to_json("mesh_primitive(\"cube\", [1.0, 1.0])");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S101"));
    }

    // ========================================================================
    // mesh_recipe() tests
    // ========================================================================

    #[test]
    fn test_mesh_recipe_minimal() {
        let result = eval_to_json("mesh_recipe(\"cube\", [1.0, 1.0, 1.0])").unwrap();
        assert_eq!(result["base_primitive"], "cube");
        assert!(result["dimensions"].is_array());
        assert!(result["modifiers"].is_array());
        assert_eq!(result["modifiers"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_mesh_recipe_with_modifiers() {
        let result = eval_to_json(
            r#"
mesh_recipe(
    "cube",
    [1.0, 1.0, 1.0],
    [bevel_modifier(0.05, 3), subdivision_modifier(2)]
)
"#,
        )
        .unwrap();

        assert_eq!(result["base_primitive"], "cube");
        assert!(result["modifiers"].is_array());
        let mods = result["modifiers"].as_array().unwrap();
        assert_eq!(mods.len(), 2);
        assert_eq!(mods[0]["type"], "bevel");
        assert_eq!(mods[1]["type"], "subdivision");
    }
}
