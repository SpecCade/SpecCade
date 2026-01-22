//! Mesh modifier creation functions.

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::{extract_float, validate_positive_int};
use super::{hashed_key, new_dict};

/// Registers mesh modifier functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_mesh_modifier_functions(builder);
}

#[starlark_module]
fn register_mesh_modifier_functions(builder: &mut GlobalsBuilder) {
    /// Creates a bevel modifier.
    ///
    /// # Arguments
    /// * `width` - Bevel width (default: 0.02)
    /// * `segments` - Number of bevel segments (default: 2)
    /// * `angle_limit` - Angle limit in degrees, only bevel edges below this angle (optional)
    ///
    /// # Returns
    /// A dict matching the MeshModifier::Bevel IR structure.
    ///
    /// # Example
    /// ```starlark
    /// bevel_modifier()
    /// bevel_modifier(0.05, 3)
    /// bevel_modifier(0.02, 2, 30.0)  // Only bevel edges with angle < 30 degrees
    /// ```
    fn bevel_modifier<'v>(
        #[starlark(default = 0.02)] width: f64,
        #[starlark(default = 2)] segments: i32,
        #[starlark(default = NoneType)] angle_limit: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if width <= 0.0 {
            return Err(anyhow::anyhow!(
                "S103: bevel_modifier(): 'width' must be positive, got {}",
                width
            ));
        }
        validate_positive_int(segments as i64, "bevel_modifier", "segments")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "type"), heap.alloc_str("bevel").to_value());
        dict.insert_hashed(hashed_key(heap, "width"), heap.alloc(width).to_value());
        dict.insert_hashed(
            hashed_key(heap, "segments"),
            heap.alloc(segments).to_value(),
        );

        // Add optional angle_limit (convert degrees to radians for spec)
        if !angle_limit.is_none() {
            let angle_deg = extract_float(angle_limit, "bevel_modifier", "angle_limit")
                .map_err(|e| anyhow::anyhow!(e))?;
            if angle_deg <= 0.0 || angle_deg > 180.0 {
                return Err(anyhow::anyhow!(
                    "S103: bevel_modifier(): 'angle_limit' must be in range (0, 180] degrees, got {}",
                    angle_deg
                ));
            }
            // Convert degrees to radians for the spec
            let angle_rad = angle_deg * std::f64::consts::PI / 180.0;
            dict.insert_hashed(
                hashed_key(heap, "angle_limit"),
                heap.alloc(angle_rad).to_value(),
            );
        }

        Ok(dict)
    }

    /// Creates a subdivision surface modifier.
    ///
    /// # Arguments
    /// * `levels` - Subdivision levels (default: 2)
    /// * `render_levels` - Render levels (default: same as levels)
    ///
    /// # Returns
    /// A dict matching the MeshModifier::Subdivision IR structure.
    ///
    /// # Example
    /// ```starlark
    /// subdivision_modifier()
    /// subdivision_modifier(3)
    /// subdivision_modifier(2, 4)
    /// ```
    fn subdivision_modifier<'v>(
        #[starlark(default = 2)] levels: i32,
        #[starlark(default = NoneType)] render_levels: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive_int(levels as i64, "subdivision_modifier", "levels")
            .map_err(|e| anyhow::anyhow!(e))?;

        let render = if render_levels.is_none() {
            levels
        } else {
            render_levels.unpack_i32().ok_or_else(|| {
                anyhow::anyhow!(
                    "S102: subdivision_modifier(): 'render_levels' expected int, got {}",
                    render_levels.get_type()
                )
            })?
        };

        validate_positive_int(render as i64, "subdivision_modifier", "render_levels")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("subdivision").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "levels"), heap.alloc(levels).to_value());
        dict.insert_hashed(
            hashed_key(heap, "render_levels"),
            heap.alloc(render).to_value(),
        );

        Ok(dict)
    }

    /// Creates a decimate modifier.
    ///
    /// # Arguments
    /// * `ratio` - Decimation ratio 0.0-1.0 (default: 0.5)
    ///
    /// # Returns
    /// A dict matching the MeshModifier::Decimate IR structure.
    ///
    /// # Example
    /// ```starlark
    /// decimate_modifier()
    /// decimate_modifier(0.25)
    /// ```
    fn decimate_modifier<'v>(
        #[starlark(default = 0.5)] ratio: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        use super::super::validation::validate_unit_range;
        validate_unit_range(ratio, "decimate_modifier", "ratio").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("decimate").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "ratio"), heap.alloc(ratio).to_value());

        Ok(dict)
    }

    /// Creates an edge split modifier.
    ///
    /// # Arguments
    /// * `angle` - Split angle in degrees (edges sharper than this will be split)
    ///
    /// # Returns
    /// A dict matching the MeshModifier::EdgeSplit IR structure.
    ///
    /// # Example
    /// ```starlark
    /// edge_split_modifier(30.0)
    /// ```
    fn edge_split_modifier<'v>(angle: f64, heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        if angle <= 0.0 || angle > 180.0 {
            return Err(anyhow::anyhow!(
                "S103: edge_split_modifier(): 'angle' must be in range (0, 180] degrees, got {}",
                angle
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("edge_split").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "angle"), heap.alloc(angle).to_value());

        Ok(dict)
    }

    /// Creates a mirror modifier.
    ///
    /// # Arguments
    /// * `axis_x` - Mirror along X axis (default: true)
    /// * `axis_y` - Mirror along Y axis (default: false)
    /// * `axis_z` - Mirror along Z axis (default: false)
    ///
    /// # Returns
    /// A dict matching the MeshModifier::Mirror IR structure.
    ///
    /// # Example
    /// ```starlark
    /// mirror_modifier()  // Mirror on X axis only
    /// mirror_modifier(True, True, False)  // Mirror on X and Y axes
    /// ```
    fn mirror_modifier<'v>(
        #[starlark(default = true)] axis_x: bool,
        #[starlark(default = false)] axis_y: bool,
        #[starlark(default = false)] axis_z: bool,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("mirror").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "axis_x"), heap.alloc(axis_x).to_value());
        dict.insert_hashed(hashed_key(heap, "axis_y"), heap.alloc(axis_y).to_value());
        dict.insert_hashed(hashed_key(heap, "axis_z"), heap.alloc(axis_z).to_value());

        Ok(dict)
    }

    /// Creates an array modifier.
    ///
    /// # Arguments
    /// * `count` - Number of copies
    /// * `offset` - Offset between copies [x, y, z]
    ///
    /// # Returns
    /// A dict matching the MeshModifier::Array IR structure.
    ///
    /// # Example
    /// ```starlark
    /// array_modifier(5, [1.0, 0.0, 0.0])  // 5 copies along X axis
    /// array_modifier(10, [0.0, 2.0, 0.0])  // 10 copies along Y axis
    /// ```
    fn array_modifier<'v>(
        count: i32,
        offset: UnpackList<f64>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive_int(count as i64, "array_modifier", "count")
            .map_err(|e| anyhow::anyhow!(e))?;

        if offset.items.len() != 3 {
            return Err(anyhow::anyhow!(
                "S101: array_modifier(): 'offset' must be [x, y, z], got {} values",
                offset.items.len()
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "type"), heap.alloc_str("array").to_value());
        dict.insert_hashed(hashed_key(heap, "count"), heap.alloc(count).to_value());

        // offset as list
        let offset_list = heap.alloc(AllocList(vec![
            heap.alloc(offset.items[0]).to_value(),
            heap.alloc(offset.items[1]).to_value(),
            heap.alloc(offset.items[2]).to_value(),
        ]));
        dict.insert_hashed(hashed_key(heap, "offset"), offset_list);

        Ok(dict)
    }

    /// Creates a solidify modifier.
    ///
    /// # Arguments
    /// * `thickness` - Thickness to add
    /// * `offset` - Offset (-1.0 to 1.0, default: 0.0)
    ///
    /// # Returns
    /// A dict matching the MeshModifier::Solidify IR structure.
    ///
    /// # Example
    /// ```starlark
    /// solidify_modifier(0.1)  // Add 0.1 thickness centered
    /// solidify_modifier(0.1, -1.0)  // Add thickness inward
    /// solidify_modifier(0.1, 1.0)  // Add thickness outward
    /// ```
    fn solidify_modifier<'v>(
        thickness: f64,
        #[starlark(default = 0.0)] offset: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if thickness <= 0.0 {
            return Err(anyhow::anyhow!(
                "S103: solidify_modifier(): 'thickness' must be positive, got {}",
                thickness
            ));
        }
        if !(-1.0..=1.0).contains(&offset) {
            return Err(anyhow::anyhow!(
                "S103: solidify_modifier(): 'offset' must be in range -1.0 to 1.0, got {}",
                offset
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("solidify").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "thickness"),
            heap.alloc(thickness).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "offset"), heap.alloc(offset).to_value());

        Ok(dict)
    }

    /// Creates a triangulate modifier.
    ///
    /// # Arguments
    /// * `ngon_method` - How to triangulate n-gons (default: "beauty")
    ///   Options: "beauty", "clip", "fixed"
    /// * `quad_method` - How to triangulate quads (default: "shortest_diagonal")
    ///   Options: "beauty", "fixed", "shortest_diagonal", "longest_diagonal"
    ///
    /// # Returns
    /// A dict matching the MeshModifier::Triangulate IR structure.
    ///
    /// # Example
    /// ```starlark
    /// triangulate_modifier()  // Use defaults
    /// triangulate_modifier("beauty", "shortest_diagonal")  // Explicit methods
    /// triangulate_modifier("clip", "fixed")  // Alternative methods
    /// ```
    fn triangulate_modifier<'v>(
        #[starlark(default = NoneType)] ngon_method: Value<'v>,
        #[starlark(default = NoneType)] quad_method: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        const VALID_NGON_METHODS: &[&str] = &["beauty", "clip", "fixed"];
        const VALID_QUAD_METHODS: &[&str] =
            &["beauty", "fixed", "shortest_diagonal", "longest_diagonal"];

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("triangulate").to_value(),
        );

        // Validate and add ngon_method if provided
        if !ngon_method.is_none() {
            let method_str = ngon_method.unpack_str().ok_or_else(|| {
                anyhow::anyhow!(
                    "S102: triangulate_modifier(): 'ngon_method' expected string, got {}",
                    ngon_method.get_type()
                )
            })?;
            if !VALID_NGON_METHODS.contains(&method_str) {
                return Err(anyhow::anyhow!(
                    "S104: triangulate_modifier(): 'ngon_method' must be one of {:?}, got \"{}\"",
                    VALID_NGON_METHODS,
                    method_str
                ));
            }
            dict.insert_hashed(
                hashed_key(heap, "ngon_method"),
                heap.alloc_str(method_str).to_value(),
            );
        }

        // Validate and add quad_method if provided
        if !quad_method.is_none() {
            let method_str = quad_method.unpack_str().ok_or_else(|| {
                anyhow::anyhow!(
                    "S102: triangulate_modifier(): 'quad_method' expected string, got {}",
                    quad_method.get_type()
                )
            })?;
            if !VALID_QUAD_METHODS.contains(&method_str) {
                return Err(anyhow::anyhow!(
                    "S104: triangulate_modifier(): 'quad_method' must be one of {:?}, got \"{}\"",
                    VALID_QUAD_METHODS,
                    method_str
                ));
            }
            dict.insert_hashed(
                hashed_key(heap, "quad_method"),
                heap.alloc_str(method_str).to_value(),
            );
        }

        Ok(dict)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::eval_to_json;

    // ========================================================================
    // bevel_modifier() tests
    // ========================================================================

    #[test]
    fn test_bevel_modifier_defaults() {
        let result = eval_to_json("bevel_modifier()").unwrap();
        assert_eq!(result["type"], "bevel");
        assert_eq!(result["width"], 0.02);
        assert_eq!(result["segments"], 2);
    }

    #[test]
    fn test_bevel_modifier_custom() {
        let result = eval_to_json("bevel_modifier(0.05, 3)").unwrap();
        assert_eq!(result["width"], 0.05);
        assert_eq!(result["segments"], 3);
    }

    #[test]
    fn test_bevel_modifier_negative_width() {
        let result = eval_to_json("bevel_modifier(-0.02)");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    #[test]
    fn test_bevel_modifier_with_angle_limit() {
        let result = eval_to_json("bevel_modifier(0.02, 2, 45.0)").unwrap();
        assert_eq!(result["type"], "bevel");
        assert!(result["angle_limit"].is_number());
        // 45 degrees in radians is approximately 0.785
        let angle = result["angle_limit"].as_f64().unwrap();
        assert!((angle - std::f64::consts::FRAC_PI_4).abs() < 0.001);
    }

    // ========================================================================
    // subdivision_modifier() tests
    // ========================================================================

    #[test]
    fn test_subdivision_modifier_defaults() {
        let result = eval_to_json("subdivision_modifier()").unwrap();
        assert_eq!(result["type"], "subdivision");
        assert_eq!(result["levels"], 2);
        assert_eq!(result["render_levels"], 2);
    }

    #[test]
    fn test_subdivision_modifier_custom() {
        let result = eval_to_json("subdivision_modifier(3, 4)").unwrap();
        assert_eq!(result["levels"], 3);
        assert_eq!(result["render_levels"], 4);
    }

    // ========================================================================
    // decimate_modifier() tests
    // ========================================================================

    #[test]
    fn test_decimate_modifier_defaults() {
        let result = eval_to_json("decimate_modifier()").unwrap();
        assert_eq!(result["type"], "decimate");
        assert_eq!(result["ratio"], 0.5);
    }

    #[test]
    fn test_decimate_modifier_custom() {
        let result = eval_to_json("decimate_modifier(0.25)").unwrap();
        assert_eq!(result["ratio"], 0.25);
    }

    #[test]
    fn test_decimate_modifier_out_of_range() {
        let result = eval_to_json("decimate_modifier(1.5)");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    // ========================================================================
    // edge_split_modifier() tests
    // ========================================================================

    #[test]
    fn test_edge_split_modifier() {
        let result = eval_to_json("edge_split_modifier(30.0)").unwrap();
        assert_eq!(result["type"], "edge_split");
        assert_eq!(result["angle"], 30.0);
    }

    #[test]
    fn test_edge_split_modifier_invalid_angle() {
        let result = eval_to_json("edge_split_modifier(-10.0)");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    // ========================================================================
    // mirror_modifier() tests
    // ========================================================================

    #[test]
    fn test_mirror_modifier_defaults() {
        let result = eval_to_json("mirror_modifier()").unwrap();
        assert_eq!(result["type"], "mirror");
        assert_eq!(result["axis_x"], true);
        assert_eq!(result["axis_y"], false);
        assert_eq!(result["axis_z"], false);
    }

    #[test]
    fn test_mirror_modifier_custom() {
        let result = eval_to_json("mirror_modifier(False, True, True)").unwrap();
        assert_eq!(result["axis_x"], false);
        assert_eq!(result["axis_y"], true);
        assert_eq!(result["axis_z"], true);
    }

    // ========================================================================
    // array_modifier() tests
    // ========================================================================

    #[test]
    fn test_array_modifier() {
        let result = eval_to_json("array_modifier(5, [1.0, 0.0, 0.0])").unwrap();
        assert_eq!(result["type"], "array");
        assert_eq!(result["count"], 5);
        assert!(result["offset"].is_array());
        let offset = result["offset"].as_array().unwrap();
        assert_eq!(offset[0], 1.0);
        assert_eq!(offset[1], 0.0);
        assert_eq!(offset[2], 0.0);
    }

    #[test]
    fn test_array_modifier_invalid_offset() {
        let result = eval_to_json("array_modifier(5, [1.0, 0.0])");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S101"));
    }

    // ========================================================================
    // solidify_modifier() tests
    // ========================================================================

    #[test]
    fn test_solidify_modifier_defaults() {
        let result = eval_to_json("solidify_modifier(0.1)").unwrap();
        assert_eq!(result["type"], "solidify");
        assert_eq!(result["thickness"], 0.1);
        assert_eq!(result["offset"], 0.0);
    }

    #[test]
    fn test_solidify_modifier_custom() {
        let result = eval_to_json("solidify_modifier(0.2, -0.5)").unwrap();
        assert_eq!(result["thickness"], 0.2);
        assert_eq!(result["offset"], -0.5);
    }

    #[test]
    fn test_solidify_modifier_invalid_offset() {
        let result = eval_to_json("solidify_modifier(0.1, 1.5)");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    // ========================================================================
    // triangulate_modifier() tests
    // ========================================================================

    #[test]
    fn test_triangulate_modifier_defaults() {
        let result = eval_to_json("triangulate_modifier()").unwrap();
        assert_eq!(result["type"], "triangulate");
        // Optional fields should not be present when using defaults
        assert!(result.get("ngon_method").is_none());
        assert!(result.get("quad_method").is_none());
    }

    #[test]
    fn test_triangulate_modifier_with_ngon_method() {
        let result = eval_to_json("triangulate_modifier(\"beauty\")").unwrap();
        assert_eq!(result["type"], "triangulate");
        assert_eq!(result["ngon_method"], "beauty");
    }

    #[test]
    fn test_triangulate_modifier_with_quad_method() {
        let result = eval_to_json("triangulate_modifier(None, \"shortest_diagonal\")").unwrap();
        assert_eq!(result["type"], "triangulate");
        assert!(result.get("ngon_method").is_none());
        assert_eq!(result["quad_method"], "shortest_diagonal");
    }

    #[test]
    fn test_triangulate_modifier_with_both_methods() {
        let result = eval_to_json("triangulate_modifier(\"clip\", \"fixed\")").unwrap();
        assert_eq!(result["type"], "triangulate");
        assert_eq!(result["ngon_method"], "clip");
        assert_eq!(result["quad_method"], "fixed");
    }

    #[test]
    fn test_triangulate_modifier_all_ngon_methods() {
        // Test all valid ngon methods
        for method in &["beauty", "clip", "fixed"] {
            let expr = format!("triangulate_modifier(\"{}\")", method);
            let result = eval_to_json(&expr).unwrap();
            assert_eq!(result["ngon_method"], *method);
        }
    }

    #[test]
    fn test_triangulate_modifier_all_quad_methods() {
        // Test all valid quad methods
        for method in &["beauty", "fixed", "shortest_diagonal", "longest_diagonal"] {
            let expr = format!("triangulate_modifier(None, \"{}\")", method);
            let result = eval_to_json(&expr).unwrap();
            assert_eq!(result["quad_method"], *method);
        }
    }

    #[test]
    fn test_triangulate_modifier_invalid_ngon_method() {
        let result = eval_to_json("triangulate_modifier(\"invalid\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
        assert!(err.contains("ngon_method"));
    }

    #[test]
    fn test_triangulate_modifier_invalid_quad_method() {
        let result = eval_to_json("triangulate_modifier(None, \"invalid\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
        assert!(err.contains("quad_method"));
    }

    #[test]
    fn test_triangulate_modifier_wrong_type_ngon() {
        let result = eval_to_json("triangulate_modifier(123)");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S102"));
    }

    #[test]
    fn test_triangulate_modifier_wrong_type_quad() {
        let result = eval_to_json("triangulate_modifier(None, 456)");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S102"));
    }
}
