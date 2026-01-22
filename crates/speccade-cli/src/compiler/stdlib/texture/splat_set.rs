//! Splat set texture Starlark helper functions.

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::dict::DictRef;
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

/// Valid splat mask modes.
const SPLAT_MASK_MODES: &[&str] = &["noise", "height", "slope", "height_slope"];

/// Registers splat set Starlark functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_splat_set_functions(builder);
}

#[starlark_module]
fn register_splat_set_functions(builder: &mut GlobalsBuilder) {
    /// Creates a terrain splat layer definition.
    ///
    /// # Arguments
    /// * `id` - Unique layer identifier (e.g., "grass", "dirt", "rock")
    /// * `albedo_color` - Base color [r, g, b, a] (0.0-1.0)
    /// * `normal_strength` - Normal map strength (default: 1.0)
    /// * `roughness` - Roughness value (default: 0.8)
    /// * `detail_scale` - Detail noise scale (default: 0.2)
    /// * `detail_intensity` - Detail noise intensity (default: 0.3)
    ///
    /// # Returns
    /// A dict matching SplatLayer.
    ///
    /// # Example
    /// ```starlark
    /// splat_layer(id = "grass", albedo_color = [0.2, 0.5, 0.1, 1.0])
    /// splat_layer(id = "dirt", albedo_color = [0.4, 0.3, 0.2, 1.0], roughness = 0.9)
    /// ```
    fn splat_layer<'v>(
        #[starlark(require = named)] id: &str,
        #[starlark(require = named)] albedo_color: UnpackList<f64>,
        #[starlark(default = 1.0)] normal_strength: f64,
        #[starlark(default = 0.8)] roughness: f64,
        #[starlark(default = 0.2)] detail_scale: f64,
        #[starlark(default = 0.3)] detail_intensity: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate id
        validate_non_empty(id, "splat_layer", "id").map_err(|e| anyhow::anyhow!(e))?;

        // Validate albedo_color
        if albedo_color.items.len() != 4 {
            return Err(anyhow::anyhow!(
                "S101: splat_layer(): 'albedo_color' must be [r, g, b, a], got {} values",
                albedo_color.items.len()
            ));
        }

        // Validate roughness
        validate_unit_range(roughness, "splat_layer", "roughness")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate normal_strength (non-negative)
        if normal_strength < 0.0 {
            return Err(anyhow::anyhow!(
                "S103: splat_layer(): 'normal_strength' must be non-negative, got {}",
                normal_strength
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());

        let color_list = heap.alloc(AllocList(
            albedo_color
                .items
                .iter()
                .map(|&v| heap.alloc(v).to_value())
                .collect::<Vec<_>>(),
        ));
        dict.insert_hashed(hashed_key(heap, "albedo_color"), color_list);

        dict.insert_hashed(
            hashed_key(heap, "normal_strength"),
            heap.alloc(normal_strength).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "roughness"),
            heap.alloc(roughness).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "detail_scale"),
            heap.alloc(detail_scale).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "detail_intensity"),
            heap.alloc(detail_intensity).to_value(),
        );

        Ok(dict)
    }

    /// Creates a complete splat set spec with splat_set_v1 recipe.
    ///
    /// # Arguments
    /// * `asset_id` - Kebab-case identifier for the asset
    /// * `seed` - Deterministic seed (0 to 2^32-1)
    /// * `resolution` - [width, height] in pixels
    /// * `layers` - List of splat_layer() definitions (max 4 per mask)
    /// * `mask_mode` - Mask generation mode: "noise", "height", "slope", "height_slope" (default: "noise")
    /// * `noise_scale` - Noise scale for noise-based blending (default: 0.1)
    /// * `macro_variation` - Whether to generate macro variation texture (default: false)
    /// * `macro_scale` - Macro variation scale (default: 0.05)
    /// * `macro_intensity` - Macro variation intensity (default: 0.3)
    /// * `output_prefix` - Output path prefix for generated textures
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
    /// splat_set_spec(
    ///     asset_id = "terrain-basic-01",
    ///     seed = 42,
    ///     resolution = [512, 512],
    ///     layers = [
    ///         splat_layer(id = "grass", albedo_color = [0.2, 0.5, 0.1, 1.0], roughness = 0.8),
    ///         splat_layer(id = "dirt", albedo_color = [0.4, 0.3, 0.2, 1.0], roughness = 0.9),
    ///         splat_layer(id = "rock", albedo_color = [0.5, 0.5, 0.5, 1.0], roughness = 0.7)
    ///     ],
    ///     output_prefix = "terrain/basic"
    /// )
    /// ```
    fn splat_set_spec<'v>(
        #[starlark(require = named)] asset_id: &str,
        #[starlark(require = named)] seed: i64,
        #[starlark(require = named)] resolution: UnpackList<i32>,
        #[starlark(require = named)] layers: UnpackList<Value<'v>>,
        #[starlark(require = named)] output_prefix: &str,
        #[starlark(default = "noise")] mask_mode: &str,
        #[starlark(default = 0.1)] noise_scale: f64,
        #[starlark(default = false)] macro_variation: bool,
        #[starlark(default = 0.05)] macro_scale: f64,
        #[starlark(default = 0.3)] macro_intensity: f64,
        #[starlark(default = NoneType)] metadata_path: Value<'v>,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate asset_id
        validate_non_empty(asset_id, "splat_set_spec", "asset_id")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate seed
        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: splat_set_spec(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        // Validate resolution
        if resolution.items.len() != 2 {
            return Err(anyhow::anyhow!(
                "S101: splat_set_spec(): 'resolution' must be [width, height], got {} values",
                resolution.items.len()
            ));
        }
        let width = resolution.items[0];
        let height = resolution.items[1];
        if width <= 0 || height <= 0 {
            return Err(anyhow::anyhow!(
                "S103: splat_set_spec(): resolution values must be positive, got [{}, {}]",
                width,
                height
            ));
        }

        // Validate layers count
        if layers.items.is_empty() {
            return Err(anyhow::anyhow!(
                "S101: splat_set_spec(): 'layers' must contain at least one layer"
            ));
        }

        // Validate mask_mode
        if !SPLAT_MASK_MODES.contains(&mask_mode) {
            return Err(anyhow::anyhow!(
                "S104: splat_set_spec(): 'mask_mode' must be one of {:?}, got '{}'",
                SPLAT_MASK_MODES,
                mask_mode
            ));
        }

        // Validate macro_intensity
        validate_unit_range(macro_intensity, "splat_set_spec", "macro_intensity")
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

        // Build outputs list
        let mut outputs_vec = Vec::new();

        // Extract layer IDs for output generation
        let layer_ids: Vec<String> = layers
            .items
            .iter()
            .filter_map(|v| {
                if let Some(dict) = DictRef::from_value(*v) {
                    let id_key = heap.alloc_str("id").to_value();
                    if let Ok(Some(id_val)) = dict.get(id_key) {
                        return id_val.unpack_str().map(|s| s.to_string());
                    }
                }
                None
            })
            .collect();

        // Per-layer outputs (albedo, normal, roughness)
        for layer_id in &layer_ids {
            for map_type in &["albedo", "normal", "roughness"] {
                let mut output = new_dict(heap);
                output.insert_hashed(
                    hashed_key(heap, "kind"),
                    heap.alloc_str("primary").to_value(),
                );
                output.insert_hashed(hashed_key(heap, "format"), heap.alloc_str("png").to_value());
                let path = format!("{}_{}.{}.png", output_prefix, layer_id, map_type);
                output.insert_hashed(hashed_key(heap, "path"), heap.alloc_str(&path).to_value());
                let source = format!("{}.{}", layer_id, map_type);
                output.insert_hashed(
                    hashed_key(heap, "source"),
                    heap.alloc_str(&source).to_value(),
                );
                outputs_vec.push(heap.alloc(output).to_value());
            }
        }

        // Splat mask outputs (one per 4 layers)
        let num_masks = layer_ids.len().div_ceil(4);
        for mask_idx in 0..num_masks {
            let mut output = new_dict(heap);
            output.insert_hashed(
                hashed_key(heap, "kind"),
                heap.alloc_str("primary").to_value(),
            );
            output.insert_hashed(hashed_key(heap, "format"), heap.alloc_str("png").to_value());
            let path = format!("{}_mask{}.png", output_prefix, mask_idx);
            output.insert_hashed(hashed_key(heap, "path"), heap.alloc_str(&path).to_value());
            let source = format!("mask{}", mask_idx);
            output.insert_hashed(
                hashed_key(heap, "source"),
                heap.alloc_str(&source).to_value(),
            );
            outputs_vec.push(heap.alloc(output).to_value());
        }

        // Macro variation output (if enabled)
        if macro_variation {
            let mut output = new_dict(heap);
            output.insert_hashed(
                hashed_key(heap, "kind"),
                heap.alloc_str("primary").to_value(),
            );
            output.insert_hashed(hashed_key(heap, "format"), heap.alloc_str("png").to_value());
            let path = format!("{}_macro.png", output_prefix);
            output.insert_hashed(hashed_key(heap, "path"), heap.alloc_str(&path).to_value());
            output.insert_hashed(
                hashed_key(heap, "source"),
                heap.alloc_str("macro").to_value(),
            );
            outputs_vec.push(heap.alloc(output).to_value());
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
            heap.alloc_str("texture.splat_set_v1").to_value(),
        );

        // Create params
        let mut params = new_dict(heap);

        // resolution as list
        let res_list = heap.alloc(AllocList(vec![
            heap.alloc(width).to_value(),
            heap.alloc(height).to_value(),
        ]));
        params.insert_hashed(hashed_key(heap, "resolution"), res_list);

        // layers as list
        let layers_list = heap.alloc(AllocList(layers.items));
        params.insert_hashed(hashed_key(heap, "layers"), layers_list);

        // mask_mode
        params.insert_hashed(
            hashed_key(heap, "mask_mode"),
            heap.alloc_str(mask_mode).to_value(),
        );

        // noise_scale
        params.insert_hashed(
            hashed_key(heap, "noise_scale"),
            heap.alloc(noise_scale).to_value(),
        );

        // macro_variation
        params.insert_hashed(
            hashed_key(heap, "macro_variation"),
            heap.alloc(macro_variation).to_value(),
        );

        // macro_scale
        params.insert_hashed(
            hashed_key(heap, "macro_scale"),
            heap.alloc(macro_scale).to_value(),
        );

        // macro_intensity
        params.insert_hashed(
            hashed_key(heap, "macro_intensity"),
            heap.alloc(macro_intensity).to_value(),
        );

        recipe.insert_hashed(hashed_key(heap, "params"), heap.alloc(params).to_value());

        spec.insert_hashed(hashed_key(heap, "recipe"), heap.alloc(recipe).to_value());

        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::eval_to_json;

    #[test]
    fn test_splat_layer_basic() {
        let result =
            eval_to_json(r#"splat_layer(id = "grass", albedo_color = [0.2, 0.5, 0.1, 1.0])"#)
                .unwrap();

        assert_eq!(result["id"], "grass");
        assert_eq!(result["albedo_color"][0], 0.2);
        assert_eq!(result["albedo_color"][1], 0.5);
        assert_eq!(result["albedo_color"][2], 0.1);
        assert_eq!(result["albedo_color"][3], 1.0);
        assert_eq!(result["normal_strength"], 1.0);
        assert_eq!(result["roughness"], 0.8);
        assert_eq!(result["detail_scale"], 0.2);
        assert_eq!(result["detail_intensity"], 0.3);
    }

    #[test]
    fn test_splat_layer_custom_params() {
        let result = eval_to_json(
            r#"splat_layer(
                id = "rock",
                albedo_color = [0.5, 0.5, 0.5, 1.0],
                normal_strength = 0.8,
                roughness = 0.7,
                detail_scale = 0.15,
                detail_intensity = 0.4
            )"#,
        )
        .unwrap();

        assert_eq!(result["id"], "rock");
        assert_eq!(result["normal_strength"], 0.8);
        assert_eq!(result["roughness"], 0.7);
        assert_eq!(result["detail_scale"], 0.15);
        assert_eq!(result["detail_intensity"], 0.4);
    }

    #[test]
    fn test_splat_layer_invalid_color() {
        let result = eval_to_json(r#"splat_layer(id = "grass", albedo_color = [0.2, 0.5, 0.1])"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("S101"));
    }

    #[test]
    fn test_splat_layer_invalid_roughness() {
        let result = eval_to_json(
            r#"splat_layer(id = "grass", albedo_color = [0.2, 0.5, 0.1, 1.0], roughness = 1.5)"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("roughness"));
    }

    #[test]
    fn test_splat_set_spec_basic() {
        let result = eval_to_json(
            r#"
splat_set_spec(
    asset_id = "terrain-basic-01",
    seed = 42,
    resolution = [512, 512],
    layers = [
        splat_layer(id = "grass", albedo_color = [0.2, 0.5, 0.1, 1.0]),
        splat_layer(id = "dirt", albedo_color = [0.4, 0.3, 0.2, 1.0])
    ],
    output_prefix = "terrain/basic"
)
"#,
        )
        .unwrap();

        assert_eq!(result["spec_version"], 1);
        assert_eq!(result["asset_id"], "terrain-basic-01");
        assert_eq!(result["asset_type"], "texture");
        assert_eq!(result["seed"], 42);
        assert_eq!(result["recipe"]["kind"], "texture.splat_set_v1");
        assert_eq!(result["recipe"]["params"]["mask_mode"], "noise");
        assert!(!result["recipe"]["params"]["macro_variation"]
            .as_bool()
            .unwrap());

        let outputs = result["outputs"].as_array().unwrap();
        // 2 layers * 3 maps + 1 mask = 7 outputs
        assert_eq!(outputs.len(), 7);
    }

    #[test]
    fn test_splat_set_spec_with_macro() {
        let result = eval_to_json(
            r#"
splat_set_spec(
    asset_id = "terrain-macro-01",
    seed = 123,
    resolution = [256, 256],
    layers = [
        splat_layer(id = "grass", albedo_color = [0.2, 0.5, 0.1, 1.0])
    ],
    output_prefix = "terrain/macro",
    macro_variation = True,
    macro_intensity = 0.5
)
"#,
        )
        .unwrap();

        assert!(result["recipe"]["params"]["macro_variation"]
            .as_bool()
            .unwrap());
        assert_eq!(result["recipe"]["params"]["macro_intensity"], 0.5);

        let outputs = result["outputs"].as_array().unwrap();
        // 1 layer * 3 maps + 1 mask + 1 macro = 5 outputs
        assert_eq!(outputs.len(), 5);

        // Find macro output
        let macro_output = outputs.iter().find(|o| o["source"] == "macro");
        assert!(macro_output.is_some());
    }

    #[test]
    fn test_splat_set_spec_four_layers() {
        let result = eval_to_json(
            r#"
splat_set_spec(
    asset_id = "terrain-four-01",
    seed = 42,
    resolution = [256, 256],
    layers = [
        splat_layer(id = "grass", albedo_color = [0.2, 0.5, 0.1, 1.0]),
        splat_layer(id = "dirt", albedo_color = [0.4, 0.3, 0.2, 1.0]),
        splat_layer(id = "rock", albedo_color = [0.5, 0.5, 0.5, 1.0]),
        splat_layer(id = "sand", albedo_color = [0.8, 0.7, 0.5, 1.0])
    ],
    output_prefix = "terrain/four"
)
"#,
        )
        .unwrap();

        let outputs = result["outputs"].as_array().unwrap();
        // 4 layers * 3 maps + 1 mask = 13 outputs
        assert_eq!(outputs.len(), 13);

        // Should have exactly 1 mask for 4 layers
        let mask_outputs: Vec<_> = outputs
            .iter()
            .filter(|o| {
                o["source"]
                    .as_str()
                    .map(|s| s.starts_with("mask"))
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(mask_outputs.len(), 1);
    }

    #[test]
    fn test_splat_set_spec_with_metadata() {
        let result = eval_to_json(
            r#"
splat_set_spec(
    asset_id = "terrain-meta-01",
    seed = 42,
    resolution = [256, 256],
    layers = [
        splat_layer(id = "grass", albedo_color = [0.2, 0.5, 0.1, 1.0])
    ],
    output_prefix = "terrain/meta",
    metadata_path = "terrain/meta.splat.json"
)
"#,
        )
        .unwrap();

        let outputs = result["outputs"].as_array().unwrap();
        let metadata_output = outputs.iter().find(|o| o["kind"] == "metadata");
        assert!(metadata_output.is_some());
        assert_eq!(metadata_output.unwrap()["path"], "terrain/meta.splat.json");
    }

    #[test]
    fn test_splat_set_spec_mask_modes() {
        for mode in &["noise", "height", "slope", "height_slope"] {
            let code = format!(
                r#"
splat_set_spec(
    asset_id = "terrain-{}-01",
    seed = 42,
    resolution = [64, 64],
    layers = [splat_layer(id = "base", albedo_color = [0.5, 0.5, 0.5, 1.0])],
    output_prefix = "terrain/{}",
    mask_mode = "{}"
)
"#,
                mode, mode, mode
            );
            let result = eval_to_json(&code).unwrap();
            assert_eq!(result["recipe"]["params"]["mask_mode"], *mode);
        }
    }

    #[test]
    fn test_splat_set_spec_invalid_mask_mode() {
        let result = eval_to_json(
            r#"
splat_set_spec(
    asset_id = "terrain-01",
    seed = 42,
    resolution = [64, 64],
    layers = [splat_layer(id = "base", albedo_color = [0.5, 0.5, 0.5, 1.0])],
    output_prefix = "terrain/test",
    mask_mode = "invalid"
)
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("S104"));
    }

    #[test]
    fn test_splat_set_spec_no_layers() {
        let result = eval_to_json(
            r#"
splat_set_spec(
    asset_id = "terrain-01",
    seed = 42,
    resolution = [64, 64],
    layers = [],
    output_prefix = "terrain/test"
)
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("S101"));
    }

    #[test]
    fn test_splat_set_spec_invalid_resolution() {
        let result = eval_to_json(
            r#"
splat_set_spec(
    asset_id = "terrain-01",
    seed = 42,
    resolution = [64],
    layers = [splat_layer(id = "base", albedo_color = [0.5, 0.5, 0.5, 1.0])],
    output_prefix = "terrain/test"
)
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("S101"));
    }
}
