//! Decal texture Starlark helper functions.

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::{validate_non_empty, validate_unit_range};

/// Helper to create a hashed key for dict insertion.
fn hashed_key<'v>(heap: &'v Heap, key: &str) -> starlark::collections::Hashed<Value<'v>> {
    heap.alloc_str(key)
        .to_value()
        .get_hashed()
        .expect("string hashing cannot fail")
}

/// Helper to create an empty dict on the heap.
fn new_dict<'v>(_heap: &'v Heap) -> Dict<'v> {
    let map: SmallMap<Value<'v>, Value<'v>> = SmallMap::new();
    Dict::new(map)
}

/// Registers decal Starlark functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_decal_functions(builder);
}

#[starlark_module]
fn register_decal_functions(builder: &mut GlobalsBuilder) {
    /// Creates decal placement metadata.
    ///
    /// # Arguments
    /// * `aspect_ratio` - Width/height ratio (default: 1.0)
    /// * `anchor` - Anchor point in normalized [0,1] coordinates (default: [0.5, 0.5])
    /// * `fade_distance` - Edge fade distance in normalized [0,1] range (default: 0.0)
    /// * `projection_size` - Optional world-space size [width, height] in meters
    /// * `depth_range` - Optional depth clipping [near, far] in meters
    ///
    /// # Returns
    /// A dict matching DecalMetadata.
    ///
    /// # Example
    /// ```starlark
    /// decal_metadata(aspect_ratio = 2.0, anchor = [0.5, 1.0], fade_distance = 0.1)
    /// decal_metadata(projection_size = [1.0, 0.5], depth_range = [0.0, 0.1])
    /// ```
    fn decal_metadata<'v>(
        #[starlark(default = 1.0)] aspect_ratio: f64,
        #[starlark(default = NoneType)] anchor: Value<'v>,
        #[starlark(default = 0.0)] fade_distance: f64,
        #[starlark(default = NoneType)] projection_size: Value<'v>,
        #[starlark(default = NoneType)] depth_range: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate aspect_ratio
        if aspect_ratio <= 0.0 {
            return Err(anyhow::anyhow!(
                "S103: decal_metadata(): 'aspect_ratio' must be positive, got {}",
                aspect_ratio
            ));
        }

        // Validate fade_distance
        validate_unit_range(fade_distance, "decal_metadata", "fade_distance")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "aspect_ratio"),
            heap.alloc(aspect_ratio).to_value(),
        );

        // Handle anchor
        if anchor.is_none() {
            let default_anchor = heap.alloc(AllocList(vec![
                heap.alloc(0.5).to_value(),
                heap.alloc(0.5).to_value(),
            ]));
            dict.insert_hashed(hashed_key(heap, "anchor"), default_anchor);
        } else {
            // Just pass through the anchor value - Starlark will validate the list at runtime
            dict.insert_hashed(hashed_key(heap, "anchor"), anchor);
        }

        dict.insert_hashed(
            hashed_key(heap, "fade_distance"),
            heap.alloc(fade_distance).to_value(),
        );

        // Handle projection_size (optional)
        if !projection_size.is_none() {
            dict.insert_hashed(hashed_key(heap, "projection_size"), projection_size);
        }

        // Handle depth_range (optional)
        if !depth_range.is_none() {
            dict.insert_hashed(hashed_key(heap, "depth_range"), depth_range);
        }

        Ok(dict)
    }

    /// Creates a complete decal spec with decal_v1 recipe.
    ///
    /// # Arguments
    /// * `asset_id` - Kebab-case identifier for the asset
    /// * `seed` - Deterministic seed (0 to 2^32-1)
    /// * `output_path` - Output file path for the albedo PNG
    /// * `resolution` - [width, height] in pixels
    /// * `nodes` - List of texture nodes
    /// * `albedo_output` - Node id for albedo/diffuse output
    /// * `alpha_output` - Node id for alpha output
    /// * `metadata` - Decal placement metadata from decal_metadata()
    /// * `normal_output` - Optional node id for normal map output
    /// * `roughness_output` - Optional node id for roughness output
    /// * `normal_path` - Optional output path for normal map PNG
    /// * `roughness_path` - Optional output path for roughness PNG
    /// * `metadata_path` - Optional output path for metadata JSON
    /// * `description` - Asset description (optional)
    /// * `tags` - Style tags (optional)
    /// * `license` - SPDX license identifier (default: "CC0-1.0")
    ///
    /// # Returns
    /// A complete spec dict ready for serialization.
    ///
    /// # Example
    /// ```starlark
    /// decal_spec(
    ///     asset_id = "bullet-hole-01",
    ///     seed = 42,
    ///     output_path = "decals/bullet_hole.png",
    ///     resolution = [512, 512],
    ///     nodes = [
    ///         noise_node("base", algorithm = "perlin", scale = 0.05),
    ///         threshold_node("alpha", "base", threshold = 0.3)
    ///     ],
    ///     albedo_output = "base",
    ///     alpha_output = "alpha",
    ///     metadata = decal_metadata(aspect_ratio = 1.0, fade_distance = 0.1)
    /// )
    /// ```
    fn decal_spec<'v>(
        #[starlark(require = named)] asset_id: &str,
        #[starlark(require = named)] seed: i64,
        #[starlark(require = named)] output_path: &str,
        #[starlark(require = named)] resolution: UnpackList<i32>,
        #[starlark(require = named)] nodes: UnpackList<Value<'v>>,
        #[starlark(require = named)] albedo_output: &str,
        #[starlark(require = named)] alpha_output: &str,
        #[starlark(require = named)] metadata: Value<'v>,
        #[starlark(default = NoneType)] normal_output: Value<'v>,
        #[starlark(default = NoneType)] roughness_output: Value<'v>,
        #[starlark(default = NoneType)] normal_path: Value<'v>,
        #[starlark(default = NoneType)] roughness_path: Value<'v>,
        #[starlark(default = NoneType)] metadata_path: Value<'v>,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate asset_id
        validate_non_empty(asset_id, "decal_spec", "asset_id").map_err(|e| anyhow::anyhow!(e))?;

        // Validate seed
        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: decal_spec(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        // Validate resolution
        if resolution.items.len() != 2 {
            return Err(anyhow::anyhow!(
                "S101: decal_spec(): 'resolution' must be [width, height], got {} values",
                resolution.items.len()
            ));
        }
        let width = resolution.items[0];
        let height = resolution.items[1];
        if width <= 0 || height <= 0 {
            return Err(anyhow::anyhow!(
                "S103: decal_spec(): resolution values must be positive, got [{}, {}]",
                width,
                height
            ));
        }

        // Validate output references
        validate_non_empty(albedo_output, "decal_spec", "albedo_output")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(alpha_output, "decal_spec", "alpha_output")
            .map_err(|e| anyhow::anyhow!(e))?;

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
            heap.alloc_str("texture").to_value(),
        );

        // license
        spec.insert_hashed(
            hashed_key(heap, "license"),
            heap.alloc_str(license).to_value(),
        );

        // seed
        spec.insert_hashed(hashed_key(heap, "seed"), heap.alloc(seed).to_value());

        // outputs
        let mut outputs_vec = Vec::new();

        // Primary output (albedo PNG)
        let mut primary_output = new_dict(heap);
        primary_output.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("primary").to_value(),
        );
        primary_output.insert_hashed(hashed_key(heap, "format"), heap.alloc_str("png").to_value());
        primary_output.insert_hashed(
            hashed_key(heap, "path"),
            heap.alloc_str(output_path).to_value(),
        );
        outputs_vec.push(heap.alloc(primary_output).to_value());

        // Optional normal output
        if !normal_path.is_none() {
            if let Some(path) = normal_path.unpack_str() {
                let mut normal_output_spec = new_dict(heap);
                normal_output_spec.insert_hashed(
                    hashed_key(heap, "kind"),
                    heap.alloc_str("primary").to_value(),
                );
                normal_output_spec
                    .insert_hashed(hashed_key(heap, "format"), heap.alloc_str("png").to_value());
                normal_output_spec
                    .insert_hashed(hashed_key(heap, "path"), heap.alloc_str(path).to_value());
                normal_output_spec.insert_hashed(
                    hashed_key(heap, "source"),
                    heap.alloc_str("normal").to_value(),
                );
                outputs_vec.push(heap.alloc(normal_output_spec).to_value());
            }
        }

        // Optional roughness output
        if !roughness_path.is_none() {
            if let Some(path) = roughness_path.unpack_str() {
                let mut roughness_output_spec = new_dict(heap);
                roughness_output_spec.insert_hashed(
                    hashed_key(heap, "kind"),
                    heap.alloc_str("primary").to_value(),
                );
                roughness_output_spec
                    .insert_hashed(hashed_key(heap, "format"), heap.alloc_str("png").to_value());
                roughness_output_spec
                    .insert_hashed(hashed_key(heap, "path"), heap.alloc_str(path).to_value());
                roughness_output_spec.insert_hashed(
                    hashed_key(heap, "source"),
                    heap.alloc_str("roughness").to_value(),
                );
                outputs_vec.push(heap.alloc(roughness_output_spec).to_value());
            }
        }

        // Metadata output (JSON) - optional
        if !metadata_path.is_none() {
            if let Some(path) = metadata_path.unpack_str() {
                let mut metadata_output = new_dict(heap);
                metadata_output.insert_hashed(
                    hashed_key(heap, "kind"),
                    heap.alloc_str("metadata").to_value(),
                );
                metadata_output.insert_hashed(
                    hashed_key(heap, "format"),
                    heap.alloc_str("json").to_value(),
                );
                metadata_output
                    .insert_hashed(hashed_key(heap, "path"), heap.alloc_str(path).to_value());
                outputs_vec.push(heap.alloc(metadata_output).to_value());
            }
        }

        let outputs_list = heap.alloc(AllocList(outputs_vec));
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
            heap.alloc_str("texture.decal_v1").to_value(),
        );

        // Create params
        let mut params = new_dict(heap);

        // resolution as list
        let res_list = heap.alloc(AllocList(vec![
            heap.alloc(width).to_value(),
            heap.alloc(height).to_value(),
        ]));
        params.insert_hashed(hashed_key(heap, "resolution"), res_list);

        // nodes as list
        let nodes_list = heap.alloc(AllocList(nodes.items));
        params.insert_hashed(hashed_key(heap, "nodes"), nodes_list);

        // outputs
        params.insert_hashed(
            hashed_key(heap, "albedo_output"),
            heap.alloc_str(albedo_output).to_value(),
        );
        params.insert_hashed(
            hashed_key(heap, "alpha_output"),
            heap.alloc_str(alpha_output).to_value(),
        );

        // Optional normal_output
        if !normal_output.is_none() {
            if let Some(node_id) = normal_output.unpack_str() {
                params.insert_hashed(
                    hashed_key(heap, "normal_output"),
                    heap.alloc_str(node_id).to_value(),
                );
            }
        }

        // Optional roughness_output
        if !roughness_output.is_none() {
            if let Some(node_id) = roughness_output.unpack_str() {
                params.insert_hashed(
                    hashed_key(heap, "roughness_output"),
                    heap.alloc_str(node_id).to_value(),
                );
            }
        }

        // metadata
        params.insert_hashed(hashed_key(heap, "metadata"), metadata);

        recipe.insert_hashed(hashed_key(heap, "params"), heap.alloc(params).to_value());

        spec.insert_hashed(hashed_key(heap, "recipe"), heap.alloc(recipe).to_value());

        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::eval_to_json;

    #[test]
    fn test_decal_metadata_defaults() {
        let result = eval_to_json(r#"decal_metadata()"#).unwrap();
        assert_eq!(result["aspect_ratio"], 1.0);
        assert_eq!(result["anchor"][0], 0.5);
        assert_eq!(result["anchor"][1], 0.5);
        assert_eq!(result["fade_distance"], 0.0);
    }

    #[test]
    fn test_decal_metadata_custom() {
        let result = eval_to_json(
            r#"decal_metadata(aspect_ratio = 2.0, anchor = [0.5, 1.0], fade_distance = 0.1)"#,
        )
        .unwrap();
        assert_eq!(result["aspect_ratio"], 2.0);
        assert_eq!(result["anchor"][0], 0.5);
        assert_eq!(result["anchor"][1], 1.0);
        assert_eq!(result["fade_distance"], 0.1);
    }

    #[test]
    fn test_decal_metadata_with_projection() {
        let result = eval_to_json(
            r#"decal_metadata(projection_size = [1.0, 0.5], depth_range = [0.0, 0.1])"#,
        )
        .unwrap();
        assert_eq!(result["projection_size"][0], 1.0);
        assert_eq!(result["projection_size"][1], 0.5);
        assert_eq!(result["depth_range"][0], 0.0);
        assert_eq!(result["depth_range"][1], 0.1);
    }

    #[test]
    fn test_decal_metadata_invalid_aspect_ratio() {
        let result = eval_to_json(r#"decal_metadata(aspect_ratio = 0.0)"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("aspect_ratio"));
    }

    #[test]
    fn test_decal_metadata_invalid_fade_distance() {
        let result = eval_to_json(r#"decal_metadata(fade_distance = 1.5)"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("fade_distance"));
    }

    #[test]
    fn test_decal_spec_basic() {
        let result = eval_to_json(
            r#"
decal_spec(
    asset_id = "test-decal-01",
    seed = 42,
    output_path = "decals/test.png",
    resolution = [512, 512],
    nodes = [constant_node("base", 0.5), constant_node("alpha", 1.0)],
    albedo_output = "base",
    alpha_output = "alpha",
    metadata = decal_metadata()
)
"#,
        )
        .unwrap();

        assert_eq!(result["spec_version"], 1);
        assert_eq!(result["asset_id"], "test-decal-01");
        assert_eq!(result["asset_type"], "texture");
        assert_eq!(result["seed"], 42);
        assert_eq!(result["recipe"]["kind"], "texture.decal_v1");
        assert_eq!(result["recipe"]["params"]["albedo_output"], "base");
        assert_eq!(result["recipe"]["params"]["alpha_output"], "alpha");

        let res = result["recipe"]["params"]["resolution"].as_array().unwrap();
        assert_eq!(res[0], 512);
        assert_eq!(res[1], 512);

        let outputs = result["outputs"].as_array().unwrap();
        assert_eq!(outputs.len(), 1); // only primary output
    }

    #[test]
    fn test_decal_spec_with_all_outputs() {
        let result = eval_to_json(
            r#"
decal_spec(
    asset_id = "test-decal-02",
    seed = 123,
    output_path = "decals/test.png",
    resolution = [256, 256],
    nodes = [
        constant_node("base", 0.5),
        constant_node("alpha", 1.0),
        constant_node("normal", 0.5),
        constant_node("rough", 0.7)
    ],
    albedo_output = "base",
    alpha_output = "alpha",
    normal_output = "normal",
    roughness_output = "rough",
    normal_path = "decals/test_normal.png",
    roughness_path = "decals/test_roughness.png",
    metadata_path = "decals/test.decal.json",
    metadata = decal_metadata(aspect_ratio = 2.0, fade_distance = 0.1)
)
"#,
        )
        .unwrap();

        assert_eq!(result["recipe"]["params"]["normal_output"], "normal");
        assert_eq!(result["recipe"]["params"]["roughness_output"], "rough");

        let outputs = result["outputs"].as_array().unwrap();
        assert_eq!(outputs.len(), 4); // primary, 2 secondary, metadata

        // Find outputs by kind
        let primary = outputs.iter().find(|o| o["kind"] == "primary").unwrap();
        let normal = outputs.iter().find(|o| o["source"] == "normal").unwrap();
        let roughness = outputs.iter().find(|o| o["source"] == "roughness").unwrap();
        let metadata = outputs.iter().find(|o| o["kind"] == "metadata").unwrap();

        assert_eq!(primary["format"], "png");
        assert_eq!(normal["format"], "png");
        assert_eq!(roughness["format"], "png");
        assert_eq!(metadata["format"], "json");
    }

    #[test]
    fn test_decal_spec_invalid_seed() {
        let result = eval_to_json(
            r#"
decal_spec(
    asset_id = "test",
    seed = -1,
    output_path = "test.png",
    resolution = [256, 256],
    nodes = [],
    albedo_output = "base",
    alpha_output = "alpha",
    metadata = decal_metadata()
)
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("seed"));
    }

    #[test]
    fn test_decal_spec_invalid_resolution() {
        let result = eval_to_json(
            r#"
decal_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.png",
    resolution = [256],
    nodes = [],
    albedo_output = "base",
    alpha_output = "alpha",
    metadata = decal_metadata()
)
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("S101"));
    }
}
