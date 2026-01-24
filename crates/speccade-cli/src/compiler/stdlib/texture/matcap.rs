//! Matcap texture Starlark helper functions.

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

/// Registers matcap Starlark functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_matcap_functions(builder);
}

#[starlark_module]
fn register_matcap_functions(builder: &mut GlobalsBuilder) {
    /// Creates a matcap texture spec with matcap_v1 recipe.
    ///
    /// # Arguments
    /// * `asset_id` - Kebab-case identifier for the asset
    /// * `seed` - Deterministic seed (0 to 2^32-1)
    /// * `output_path` - Output file path for the PNG
    /// * `resolution` - [width, height] in pixels (typically square, e.g., [256, 256])
    /// * `preset` - Matcap preset name (see MatcapPreset enum)
    /// * `base_color` - Optional RGB color override [r, g, b] (0.0-1.0)
    /// * `toon_steps` - Optional toon shading steps (2-16)
    /// * `outline_width` - Optional outline width in pixels (1-10)
    /// * `outline_color` - Optional outline color [r, g, b] (0.0-1.0)
    /// * `curvature_enabled` - Optional curvature mask enabled (default: False)
    /// * `curvature_strength` - Optional curvature mask strength (0.0-1.0, default: 0.5)
    /// * `cavity_enabled` - Optional cavity mask enabled (default: False)
    /// * `cavity_strength` - Optional cavity mask strength (0.0-1.0, default: 0.5)
    /// * `description` - Asset description (optional)
    /// * `tags` - Style tags (optional)
    /// * `license` - SPDX license identifier (default: "CC0-1.0")
    ///
    /// # Valid Presets
    /// - "toon_basic" - Basic toon shading
    /// - "toon_rim" - Toon shading with rim lighting
    /// - "metallic" - Metallic shading with strong specular
    /// - "ceramic" - Ceramic/porcelain shading
    /// - "clay" - Matte clay shading
    /// - "skin" - Skin/subsurface shading
    /// - "plastic" - Glossy plastic
    /// - "velvet" - Velvet/fabric
    ///
    /// # Returns
    /// A complete spec dict ready for serialization.
    ///
    /// # Example
    /// ```starlark
    /// matcap_v1(
    ///     asset_id = "toon-red-01",
    ///     seed = 42,
    ///     output_path = "matcaps/toon_red.png",
    ///     resolution = [512, 512],
    ///     preset = "toon_basic",
    ///     base_color = [0.8, 0.2, 0.2],
    ///     toon_steps = 4,
    ///     outline_width = 2,
    ///     outline_color = [0.0, 0.0, 0.0]
    /// )
    /// ```
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn matcap_v1<'v>(
        #[starlark(require = named)] asset_id: &str,
        #[starlark(require = named)] seed: i64,
        #[starlark(require = named)] output_path: &str,
        #[starlark(require = named)] resolution: UnpackList<i32>,
        #[starlark(require = named)] preset: &str,
        #[starlark(default = NoneType)] base_color: Value<'v>,
        #[starlark(default = NoneType)] toon_steps: Value<'v>,
        #[starlark(default = NoneType)] outline_width: Value<'v>,
        #[starlark(default = NoneType)] outline_color: Value<'v>,
        #[starlark(default = false)] curvature_enabled: bool,
        #[starlark(default = 0.5)] curvature_strength: f64,
        #[starlark(default = false)] cavity_enabled: bool,
        #[starlark(default = 0.5)] cavity_strength: f64,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate asset_id
        validate_non_empty(asset_id, "matcap_v1", "asset_id").map_err(|e| anyhow::anyhow!(e))?;

        // Validate seed
        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: matcap_v1(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        // Validate resolution
        if resolution.items.len() != 2 {
            return Err(anyhow::anyhow!(
                "S101: matcap_v1(): 'resolution' must be [width, height], got {} values",
                resolution.items.len()
            ));
        }
        let width = resolution.items[0];
        let height = resolution.items[1];
        if width <= 0 || height <= 0 {
            return Err(anyhow::anyhow!(
                "S103: matcap_v1(): resolution values must be positive, got [{}, {}]",
                width,
                height
            ));
        }

        // Validate preset
        let valid_presets = [
            "toon_basic",
            "toon_rim",
            "metallic",
            "ceramic",
            "clay",
            "skin",
            "plastic",
            "velvet",
        ];
        if !valid_presets.contains(&preset) {
            return Err(anyhow::anyhow!(
                "S103: matcap_v1(): 'preset' must be one of {:?}, got '{}'",
                valid_presets,
                preset
            ));
        }

        // Validate curvature/cavity strengths
        validate_unit_range(curvature_strength, "matcap_v1", "curvature_strength")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(cavity_strength, "matcap_v1", "cavity_strength")
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

        let outputs_list = heap.alloc(AllocList(vec![heap.alloc(primary_output).to_value()]));
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
            heap.alloc_str("texture.matcap_v1").to_value(),
        );

        // Create params
        let mut params = new_dict(heap);

        // resolution as list
        let res_list = heap.alloc(AllocList(vec![
            heap.alloc(width).to_value(),
            heap.alloc(height).to_value(),
        ]));
        params.insert_hashed(hashed_key(heap, "resolution"), res_list);

        // preset
        params.insert_hashed(
            hashed_key(heap, "preset"),
            heap.alloc_str(preset).to_value(),
        );

        // Optional base_color
        if !base_color.is_none() {
            params.insert_hashed(hashed_key(heap, "base_color"), base_color);
        }

        // Optional toon_steps
        if !toon_steps.is_none() {
            if let Some(steps) = toon_steps.unpack_i32() {
                if !(2..=16).contains(&steps) {
                    return Err(anyhow::anyhow!(
                        "S103: matcap_v1(): 'toon_steps' must be between 2 and 16, got {}",
                        steps
                    ));
                }
                params.insert_hashed(hashed_key(heap, "toon_steps"), heap.alloc(steps).to_value());
            }
        }

        // Optional outline
        if !outline_width.is_none() || !outline_color.is_none() {
            let mut outline = new_dict(heap);

            if let Some(width_val) = outline_width.unpack_i32() {
                if !(1..=10).contains(&width_val) {
                    return Err(anyhow::anyhow!(
                        "S103: matcap_v1(): 'outline_width' must be between 1 and 10, got {}",
                        width_val
                    ));
                }
                outline.insert_hashed(hashed_key(heap, "width"), heap.alloc(width_val).to_value());
            } else if !outline_color.is_none() {
                // If color is specified but not width, use default width
                outline.insert_hashed(hashed_key(heap, "width"), heap.alloc(2).to_value());
            }

            if !outline_color.is_none() {
                outline.insert_hashed(hashed_key(heap, "color"), outline_color);
            } else if !outline_width.is_none() {
                // If width is specified but not color, use default black
                let default_color = heap.alloc(AllocList(vec![
                    heap.alloc(0.0).to_value(),
                    heap.alloc(0.0).to_value(),
                    heap.alloc(0.0).to_value(),
                ]));
                outline.insert_hashed(hashed_key(heap, "color"), default_color);
            }

            if !outline.is_empty() {
                params.insert_hashed(hashed_key(heap, "outline"), heap.alloc(outline).to_value());
            }
        }

        // Optional curvature_mask
        if curvature_enabled {
            let mut curvature = new_dict(heap);
            curvature.insert_hashed(hashed_key(heap, "enabled"), heap.alloc(true).to_value());
            curvature.insert_hashed(
                hashed_key(heap, "strength"),
                heap.alloc(curvature_strength).to_value(),
            );
            params.insert_hashed(
                hashed_key(heap, "curvature_mask"),
                heap.alloc(curvature).to_value(),
            );
        }

        // Optional cavity_mask
        if cavity_enabled {
            let mut cavity = new_dict(heap);
            cavity.insert_hashed(hashed_key(heap, "enabled"), heap.alloc(true).to_value());
            cavity.insert_hashed(
                hashed_key(heap, "strength"),
                heap.alloc(cavity_strength).to_value(),
            );
            params.insert_hashed(
                hashed_key(heap, "cavity_mask"),
                heap.alloc(cavity).to_value(),
            );
        }

        recipe.insert_hashed(hashed_key(heap, "params"), heap.alloc(params).to_value());

        spec.insert_hashed(hashed_key(heap, "recipe"), heap.alloc(recipe).to_value());

        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::eval_to_json;

    #[test]
    fn test_matcap_v1_basic() {
        let result = eval_to_json(
            r#"
matcap_v1(
    asset_id = "test-matcap-01",
    seed = 42,
    output_path = "matcaps/test.png",
    resolution = [256, 256],
    preset = "toon_basic"
)
"#,
        )
        .unwrap();

        assert_eq!(result["spec_version"], 1);
        assert_eq!(result["asset_id"], "test-matcap-01");
        assert_eq!(result["asset_type"], "texture");
        assert_eq!(result["seed"], 42);
        assert_eq!(result["recipe"]["kind"], "texture.matcap_v1");
        assert_eq!(result["recipe"]["params"]["preset"], "toon_basic");

        let res = result["recipe"]["params"]["resolution"].as_array().unwrap();
        assert_eq!(res[0], 256);
        assert_eq!(res[1], 256);

        let outputs = result["outputs"].as_array().unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0]["kind"], "primary");
        assert_eq!(outputs[0]["format"], "png");
        assert_eq!(outputs[0]["path"], "matcaps/test.png");
    }

    #[test]
    fn test_matcap_v1_with_all_options() {
        let result = eval_to_json(
            r#"
matcap_v1(
    asset_id = "test-matcap-02",
    seed = 123,
    output_path = "matcaps/toon.png",
    resolution = [512, 512],
    preset = "metallic",
    base_color = [0.9, 0.9, 0.95],
    toon_steps = 3,
    outline_width = 2,
    outline_color = [0.1, 0.1, 0.1],
    curvature_enabled = True,
    curvature_strength = 0.6,
    cavity_enabled = True,
    cavity_strength = 0.4,
    description = "Test matcap",
    tags = ["stylized", "toon"]
)
"#,
        )
        .unwrap();

        assert_eq!(result["recipe"]["params"]["preset"], "metallic");
        assert_eq!(result["recipe"]["params"]["base_color"][0], 0.9);
        assert_eq!(result["recipe"]["params"]["base_color"][1], 0.9);
        assert_eq!(result["recipe"]["params"]["base_color"][2], 0.95);
        assert_eq!(result["recipe"]["params"]["toon_steps"], 3);

        assert_eq!(result["recipe"]["params"]["outline"]["width"], 2);
        assert_eq!(result["recipe"]["params"]["outline"]["color"][0], 0.1);

        assert_eq!(
            result["recipe"]["params"]["curvature_mask"]["enabled"],
            true
        );
        assert_eq!(
            result["recipe"]["params"]["curvature_mask"]["strength"],
            0.6
        );

        assert_eq!(result["recipe"]["params"]["cavity_mask"]["enabled"], true);
        assert_eq!(result["recipe"]["params"]["cavity_mask"]["strength"], 0.4);

        assert_eq!(result["description"], "Test matcap");
    }

    #[test]
    fn test_matcap_v1_invalid_preset() {
        let result = eval_to_json(
            r#"
matcap_v1(
    asset_id = "test",
    seed = 42,
    output_path = "test.png",
    resolution = [256, 256],
    preset = "invalid_preset"
)
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("preset"));
    }

    #[test]
    fn test_matcap_v1_invalid_toon_steps() {
        let result = eval_to_json(
            r#"
matcap_v1(
    asset_id = "test",
    seed = 42,
    output_path = "test.png",
    resolution = [256, 256],
    preset = "toon_basic",
    toon_steps = 20
)
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("toon_steps"));
    }

    #[test]
    fn test_matcap_v1_invalid_outline_width() {
        let result = eval_to_json(
            r#"
matcap_v1(
    asset_id = "test",
    seed = 42,
    output_path = "test.png",
    resolution = [256, 256],
    preset = "clay",
    outline_width = 15
)
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("outline_width"));
    }

    #[test]
    fn test_matcap_v1_invalid_resolution() {
        let result = eval_to_json(
            r#"
matcap_v1(
    asset_id = "test",
    seed = 42,
    output_path = "test.png",
    resolution = [256],
    preset = "toon_basic"
)
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("S101"));
    }

    #[test]
    fn test_matcap_v1_outline_width_only() {
        let result = eval_to_json(
            r#"
matcap_v1(
    asset_id = "test",
    seed = 42,
    output_path = "test.png",
    resolution = [256, 256],
    preset = "toon_basic",
    outline_width = 3
)
"#,
        )
        .unwrap();

        assert_eq!(result["recipe"]["params"]["outline"]["width"], 3);
        assert_eq!(result["recipe"]["params"]["outline"]["color"][0], 0.0);
        assert_eq!(result["recipe"]["params"]["outline"]["color"][1], 0.0);
        assert_eq!(result["recipe"]["params"]["outline"]["color"][2], 0.0);
    }

    #[test]
    fn test_matcap_v1_outline_color_only() {
        let result = eval_to_json(
            r#"
matcap_v1(
    asset_id = "test",
    seed = 42,
    output_path = "test.png",
    resolution = [256, 256],
    preset = "toon_basic",
    outline_color = [0.2, 0.2, 0.2]
)
"#,
        )
        .unwrap();

        assert_eq!(result["recipe"]["params"]["outline"]["width"], 2); // default
        assert_eq!(result["recipe"]["params"]["outline"]["color"][0], 0.2);
    }
}
