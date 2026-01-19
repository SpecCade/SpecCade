//! Baking settings creation functions.

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::validate_enum;
use super::{hashed_key, new_dict};

/// Valid bake types.
const BAKE_TYPES: &[&str] = &["normal", "ao", "curvature", "combined"];

/// Registers baking functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_baking_functions(builder);
}

#[starlark_module]
fn register_baking_functions(builder: &mut GlobalsBuilder) {
    /// Creates baking settings for texture map generation.
    ///
    /// Bakes normal maps, AO, curvature, etc. from a mesh (or high-poly source)
    /// onto the UVs of the target mesh.
    ///
    /// # Arguments
    /// * `bake_types` - List of map types to bake: "normal", "ao", "curvature", "combined"
    /// * `ray_distance` - Ray casting distance for high-to-low baking (default: 0.1)
    /// * `margin` - Dilation in pixels for mip-safe edges (default: 16)
    /// * `resolution` - [width, height] of baked textures (default: [1024, 1024])
    /// * `high_poly_source` - Optional path to high-poly mesh for baking
    ///
    /// # Returns
    /// A dict matching the BakingSettings structure.
    ///
    /// # Example
    /// ```starlark
    /// baking_settings(["normal", "ao"])
    /// baking_settings(["normal"], ray_distance=0.2, margin=32, resolution=[2048, 2048])
    /// baking_settings(["normal"], high_poly_source="meshes/high_detail.glb")
    /// ```
    fn baking_settings<'v>(
        bake_types: UnpackList<&str>,
        #[starlark(default = 0.1)] ray_distance: f64,
        #[starlark(default = 16)] margin: i32,
        #[starlark(default = NoneType)] resolution: Value<'v>,
        #[starlark(default = NoneType)] high_poly_source: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate bake types
        if bake_types.items.is_empty() {
            return Err(anyhow::anyhow!(
                "S101: baking_settings(): 'bake_types' must not be empty"
            ));
        }
        for bake_type in &bake_types.items {
            validate_enum(bake_type, BAKE_TYPES, "baking_settings", "bake_types")
                .map_err(|e| anyhow::anyhow!(e))?;
        }

        // Validate ray_distance
        if ray_distance <= 0.0 {
            return Err(anyhow::anyhow!(
                "S103: baking_settings(): 'ray_distance' must be positive, got {}",
                ray_distance
            ));
        }

        // Validate margin
        if margin < 0 {
            return Err(anyhow::anyhow!(
                "S103: baking_settings(): 'margin' must be non-negative, got {}",
                margin
            ));
        }

        let mut dict = new_dict(heap);

        // bake_types as list
        let types_list: Vec<Value<'v>> = bake_types
            .items
            .iter()
            .map(|t| heap.alloc_str(t).to_value())
            .collect();
        dict.insert_hashed(
            hashed_key(heap, "bake_types"),
            heap.alloc(AllocList(types_list)),
        );

        dict.insert_hashed(
            hashed_key(heap, "ray_distance"),
            heap.alloc(ray_distance).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "margin"), heap.alloc(margin).to_value());

        // resolution - default to [1024, 1024] if not specified
        if resolution.is_none() {
            let default_res = heap.alloc(AllocList(vec![
                heap.alloc(1024i32).to_value(),
                heap.alloc(1024i32).to_value(),
            ]));
            dict.insert_hashed(hashed_key(heap, "resolution"), default_res);
        } else {
            // Validate resolution is a list of 2 integers
            let list = resolution.iterate(heap).map_err(|_| {
                anyhow::anyhow!(
                    "S102: baking_settings(): 'resolution' expected list [width, height], got {}",
                    resolution.get_type()
                )
            })?;
            let items: Vec<Value<'v>> = list.collect();
            if items.len() != 2 {
                return Err(anyhow::anyhow!(
                    "S101: baking_settings(): 'resolution' must be [width, height], got {} values",
                    items.len()
                ));
            }
            dict.insert_hashed(hashed_key(heap, "resolution"), resolution);
        }

        // high_poly_source - optional
        if !high_poly_source.is_none() {
            let source_str = high_poly_source.unpack_str().ok_or_else(|| {
                anyhow::anyhow!(
                    "S102: baking_settings(): 'high_poly_source' expected string, got {}",
                    high_poly_source.get_type()
                )
            })?;
            dict.insert_hashed(
                hashed_key(heap, "high_poly_source"),
                heap.alloc_str(source_str).to_value(),
            );
        }

        Ok(dict)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::eval_to_json;

    // ========================================================================
    // baking_settings() tests
    // ========================================================================

    #[test]
    fn test_baking_settings_normal_only() {
        let result = eval_to_json("baking_settings([\"normal\"])").unwrap();
        assert!(result["bake_types"].is_array());
        let types = result["bake_types"].as_array().unwrap();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0], "normal");
        assert_eq!(result["ray_distance"], 0.1);
        assert_eq!(result["margin"], 16);
        assert!(result["resolution"].is_array());
        let res = result["resolution"].as_array().unwrap();
        assert_eq!(res[0], 1024);
        assert_eq!(res[1], 1024);
    }

    #[test]
    fn test_baking_settings_multiple_types() {
        let result = eval_to_json("baking_settings([\"normal\", \"ao\"])").unwrap();
        let types = result["bake_types"].as_array().unwrap();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0], "normal");
        assert_eq!(types[1], "ao");
    }

    #[test]
    fn test_baking_settings_all_types() {
        let result =
            eval_to_json("baking_settings([\"normal\", \"ao\", \"curvature\", \"combined\"])")
                .unwrap();
        let types = result["bake_types"].as_array().unwrap();
        assert_eq!(types.len(), 4);
    }

    #[test]
    fn test_baking_settings_custom_ray_distance() {
        let result = eval_to_json("baking_settings([\"normal\"], ray_distance=0.2)").unwrap();
        assert_eq!(result["ray_distance"], 0.2);
    }

    #[test]
    fn test_baking_settings_custom_margin() {
        let result = eval_to_json("baking_settings([\"normal\"], margin=32)").unwrap();
        assert_eq!(result["margin"], 32);
    }

    #[test]
    fn test_baking_settings_custom_resolution() {
        let result =
            eval_to_json("baking_settings([\"normal\"], resolution=[2048, 2048])").unwrap();
        let res = result["resolution"].as_array().unwrap();
        assert_eq!(res[0], 2048);
        assert_eq!(res[1], 2048);
    }

    #[test]
    fn test_baking_settings_with_high_poly_source() {
        let result = eval_to_json(
            "baking_settings([\"normal\"], high_poly_source=\"meshes/high_detail.glb\")",
        )
        .unwrap();
        assert_eq!(result["high_poly_source"], "meshes/high_detail.glb");
    }

    #[test]
    fn test_baking_settings_complete() {
        let result = eval_to_json(
            r#"
baking_settings(
    ["normal", "ao"],
    ray_distance=0.15,
    margin=24,
    resolution=[4096, 4096],
    high_poly_source="high.glb"
)
"#,
        )
        .unwrap();

        let types = result["bake_types"].as_array().unwrap();
        assert_eq!(types.len(), 2);
        assert_eq!(result["ray_distance"], 0.15);
        assert_eq!(result["margin"], 24);
        let res = result["resolution"].as_array().unwrap();
        assert_eq!(res[0], 4096);
        assert_eq!(res[1], 4096);
        assert_eq!(result["high_poly_source"], "high.glb");
    }

    #[test]
    fn test_baking_settings_empty_types_fails() {
        let result = eval_to_json("baking_settings([])");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S101"));
        assert!(err.contains("bake_types"));
    }

    #[test]
    fn test_baking_settings_invalid_type() {
        let result = eval_to_json("baking_settings([\"invalid\"])");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
    }

    #[test]
    fn test_baking_settings_negative_ray_distance() {
        let result = eval_to_json("baking_settings([\"normal\"], ray_distance=-0.1)");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
        assert!(err.contains("ray_distance"));
    }

    #[test]
    fn test_baking_settings_negative_margin() {
        let result = eval_to_json("baking_settings([\"normal\"], margin=-1)");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
        assert!(err.contains("margin"));
    }

    #[test]
    fn test_baking_settings_wrong_resolution_length() {
        let result = eval_to_json("baking_settings([\"normal\"], resolution=[1024])");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S101"));
        assert!(err.contains("resolution"));
    }
}
