//! Material preset stdlib function for PBR texture generation.
//!
//! Provides `material_preset_v1()` for creating material preset texture specs
//! with predefined styles and optional parameter overrides.

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::validate_non_empty;

/// Valid material preset types.
const MATERIAL_PRESETS: &[&str] = &[
    "toon_metal",
    "stylized_wood",
    "neon_glow",
    "ceramic_glaze",
    "sci_fi_panel",
    "clean_plastic",
    "rough_stone",
    "brushed_metal",
];

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

/// Registers material preset stdlib functions.
pub fn register(builder: &mut GlobalsBuilder) {
    register_material_preset_functions(builder);
}

#[starlark_module]
fn register_material_preset_functions(builder: &mut GlobalsBuilder) {
    /// Creates a complete material preset spec with texture.material_preset_v1 recipe.
    ///
    /// Material presets generate multiple PBR texture outputs (albedo, roughness,
    /// metallic, normal) from predefined style presets with optional overrides.
    ///
    /// # Arguments
    /// * `asset_id` - Kebab-case asset identifier
    /// * `seed` - Deterministic seed (0 to 2^32-1)
    /// * `output_prefix` - Output path prefix for generated textures
    /// * `resolution` - [width, height] in pixels
    /// * `preset` - Material preset type (see below)
    /// * `tileable` - Whether textures tile seamlessly (default: true)
    /// * `base_color` - Optional RGB color override [r, g, b] (0.0-1.0)
    /// * `roughness_range` - Optional roughness range [min, max] (0.0-1.0)
    /// * `metallic` - Optional metallic value (0.0-1.0)
    /// * `noise_scale` - Optional noise scale for detail patterns
    /// * `pattern_scale` - Optional pattern scale for macro features
    /// * `description` - Optional asset description
    /// * `style_tags` - Optional style style_tags
    /// * `license` - SPDX license identifier (default: "CC0-1.0")
    ///
    /// # Valid Presets
    /// - `"toon_metal"` - Flat albedo with rim highlights, stepped roughness
    /// - `"stylized_wood"` - Wood grain pattern with warm tones
    /// - `"neon_glow"` - Dark base with bright emissive-style highlights
    /// - `"ceramic_glaze"` - Smooth, high-gloss ceramic look
    /// - `"sci_fi_panel"` - Geometric patterns with panel lines
    /// - `"clean_plastic"` - Uniform albedo with medium roughness
    /// - `"rough_stone"` - Rocky noise patterns, high roughness
    /// - `"brushed_metal"` - Directional anisotropic streaks
    ///
    /// # Returns
    /// Complete spec dict with multiple outputs (albedo, roughness, metallic, normal).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn material_preset_v1<'v>(
        #[starlark(require = named)] asset_id: &str,
        #[starlark(require = named)] seed: i64,
        #[starlark(require = named)] output_prefix: &str,
        #[starlark(require = named)] resolution: UnpackList<i32>,
        #[starlark(require = named)] preset: &str,
        #[starlark(default = true)] tileable: bool,
        #[starlark(default = NoneType)] base_color: Value<'v>,
        #[starlark(default = NoneType)] roughness_range: Value<'v>,
        #[starlark(default = NoneType)] metallic: Value<'v>,
        #[starlark(default = NoneType)] noise_scale: Value<'v>,
        #[starlark(default = NoneType)] pattern_scale: Value<'v>,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] style_tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate asset_id
        validate_non_empty(asset_id, "material_preset_v1", "asset_id")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate seed
        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: material_preset_v1(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        // Validate resolution
        if resolution.items.len() != 2 {
            return Err(anyhow::anyhow!(
                "S102: material_preset_v1(): 'resolution' must be [width, height], got {} values",
                resolution.items.len()
            ));
        }
        let width = resolution.items[0];
        let height = resolution.items[1];
        if width <= 0 || height <= 0 {
            return Err(anyhow::anyhow!(
                "S103: material_preset_v1(): resolution values must be positive, got [{}, {}]",
                width,
                height
            ));
        }

        // Validate preset
        if !MATERIAL_PRESETS.contains(&preset) {
            return Err(anyhow::anyhow!(
                "S104: material_preset_v1(): invalid preset '{}'; expected one of: {}",
                preset,
                MATERIAL_PRESETS.join(", ")
            ));
        }

        // Validate output_prefix
        validate_non_empty(output_prefix, "material_preset_v1", "output_prefix")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Note: Deep validation of list/float values is deferred to the backend.
        // The Starlark layer accepts the values and the spec validation layer
        // will catch any issues before generation.

        // Build params dict
        let mut params = new_dict(heap);
        params.insert_hashed(
            hashed_key(heap, "preset"),
            heap.alloc_str(preset).to_value(),
        );

        let res_list = heap.alloc(AllocList(vec![
            heap.alloc(width).to_value(),
            heap.alloc(height).to_value(),
        ]));
        params.insert_hashed(hashed_key(heap, "resolution"), res_list);
        params.insert_hashed(
            hashed_key(heap, "tileable"),
            heap.alloc(tileable).to_value(),
        );

        if !base_color.is_none() {
            params.insert_hashed(hashed_key(heap, "base_color"), base_color);
        }
        if !roughness_range.is_none() {
            params.insert_hashed(hashed_key(heap, "roughness_range"), roughness_range);
        }
        if !metallic.is_none() {
            params.insert_hashed(hashed_key(heap, "metallic"), metallic);
        }
        if !noise_scale.is_none() {
            params.insert_hashed(hashed_key(heap, "noise_scale"), noise_scale);
        }
        if !pattern_scale.is_none() {
            params.insert_hashed(hashed_key(heap, "pattern_scale"), pattern_scale);
        }

        // Build recipe dict
        let mut recipe = new_dict(heap);
        recipe.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("texture.material_preset_v1").to_value(),
        );
        recipe.insert_hashed(hashed_key(heap, "params"), heap.alloc(params).to_value());

        // Build outputs list (4 primary PNG outputs + metadata JSON)
        let mut outputs_vec = Vec::with_capacity(5);

        // Albedo output
        let mut albedo_out = new_dict(heap);
        albedo_out.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("primary").to_value(),
        );
        albedo_out.insert_hashed(hashed_key(heap, "format"), heap.alloc_str("png").to_value());
        albedo_out.insert_hashed(
            hashed_key(heap, "path"),
            heap.alloc_str(&format!("{}_albedo.png", output_prefix))
                .to_value(),
        );
        albedo_out.insert_hashed(
            hashed_key(heap, "source"),
            heap.alloc_str("albedo").to_value(),
        );
        outputs_vec.push(heap.alloc(albedo_out).to_value());

        // Roughness output
        let mut roughness_out = new_dict(heap);
        roughness_out.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("primary").to_value(),
        );
        roughness_out.insert_hashed(hashed_key(heap, "format"), heap.alloc_str("png").to_value());
        roughness_out.insert_hashed(
            hashed_key(heap, "path"),
            heap.alloc_str(&format!("{}_roughness.png", output_prefix))
                .to_value(),
        );
        roughness_out.insert_hashed(
            hashed_key(heap, "source"),
            heap.alloc_str("roughness").to_value(),
        );
        outputs_vec.push(heap.alloc(roughness_out).to_value());

        // Metallic output
        let mut metallic_out = new_dict(heap);
        metallic_out.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("primary").to_value(),
        );
        metallic_out.insert_hashed(hashed_key(heap, "format"), heap.alloc_str("png").to_value());
        metallic_out.insert_hashed(
            hashed_key(heap, "path"),
            heap.alloc_str(&format!("{}_metallic.png", output_prefix))
                .to_value(),
        );
        metallic_out.insert_hashed(
            hashed_key(heap, "source"),
            heap.alloc_str("metallic").to_value(),
        );
        outputs_vec.push(heap.alloc(metallic_out).to_value());

        // Normal output
        let mut normal_out = new_dict(heap);
        normal_out.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("primary").to_value(),
        );
        normal_out.insert_hashed(hashed_key(heap, "format"), heap.alloc_str("png").to_value());
        normal_out.insert_hashed(
            hashed_key(heap, "path"),
            heap.alloc_str(&format!("{}_normal.png", output_prefix))
                .to_value(),
        );
        normal_out.insert_hashed(
            hashed_key(heap, "source"),
            heap.alloc_str("normal").to_value(),
        );
        outputs_vec.push(heap.alloc(normal_out).to_value());

        // Metadata output (JSON)
        let mut metadata_out = new_dict(heap);
        metadata_out.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("metadata").to_value(),
        );
        metadata_out.insert_hashed(
            hashed_key(heap, "format"),
            heap.alloc_str("json").to_value(),
        );
        metadata_out.insert_hashed(
            hashed_key(heap, "path"),
            heap.alloc_str(&format!("{}.material.json", output_prefix))
                .to_value(),
        );
        outputs_vec.push(heap.alloc(metadata_out).to_value());

        let outputs_list = heap.alloc(AllocList(outputs_vec));

        // Build spec dict
        let mut spec = new_dict(heap);
        spec.insert_hashed(hashed_key(heap, "spec_version"), heap.alloc(1).to_value());
        spec.insert_hashed(
            hashed_key(heap, "asset_id"),
            heap.alloc_str(asset_id).to_value(),
        );
        spec.insert_hashed(
            hashed_key(heap, "asset_type"),
            heap.alloc_str("texture").to_value(),
        );
        spec.insert_hashed(
            hashed_key(heap, "license"),
            heap.alloc_str(license).to_value(),
        );
        spec.insert_hashed(hashed_key(heap, "seed"), heap.alloc(seed).to_value());
        spec.insert_hashed(hashed_key(heap, "outputs"), outputs_list);
        spec.insert_hashed(hashed_key(heap, "recipe"), heap.alloc(recipe).to_value());

        if !description.is_none() {
            if let Some(desc) = description.unpack_str() {
                spec.insert_hashed(
                    hashed_key(heap, "description"),
                    heap.alloc_str(desc).to_value(),
                );
            }
        }

        if !style_tags.is_none() {
            spec.insert_hashed(hashed_key(heap, "style_tags"), style_tags);
        }

        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::eval_to_json;

    #[test]
    fn test_material_preset_basic() {
        let result = eval_to_json(
            r#"
material_preset_v1(
    asset_id = "metal-preset-01",
    seed = 42,
    output_prefix = "materials/metal",
    resolution = [256, 256],
    preset = "toon_metal"
)
"#,
        )
        .unwrap();

        assert_eq!(result["asset_id"], "metal-preset-01");
        assert_eq!(result["asset_type"], "texture");
        assert_eq!(result["recipe"]["kind"], "texture.material_preset_v1");
        assert_eq!(result["recipe"]["params"]["preset"], "toon_metal");
        assert_eq!(result["recipe"]["params"]["tileable"], true);

        let res = result["recipe"]["params"]["resolution"].as_array().unwrap();
        assert_eq!(res[0], 256);
        assert_eq!(res[1], 256);

        let outputs = result["outputs"].as_array().unwrap();
        assert_eq!(outputs.len(), 5);
    }

    #[test]
    fn test_material_preset_with_overrides() {
        let result = eval_to_json(
            r#"
material_preset_v1(
    asset_id = "wood-preset-01",
    seed = 123,
    output_prefix = "materials/wood",
    resolution = [512, 512],
    preset = "stylized_wood",
    tileable = False,
    base_color = [0.7, 0.5, 0.3],
    roughness_range = [0.5, 0.9],
    metallic = 0.0,
    noise_scale = 0.06,
    pattern_scale = 0.25,
    description = "Custom wood material",
    style_tags = ["wood", "stylized"],
    license = "CC-BY-4.0"
)
"#,
        )
        .unwrap();

        assert_eq!(result["recipe"]["params"]["preset"], "stylized_wood");
        assert_eq!(result["recipe"]["params"]["tileable"], false);
        assert_eq!(result["recipe"]["params"]["base_color"][0], 0.7);
        assert_eq!(result["recipe"]["params"]["roughness_range"][0], 0.5);
        assert_eq!(result["recipe"]["params"]["metallic"], 0.0);
        assert_eq!(result["description"], "Custom wood material");
        assert_eq!(result["license"], "CC-BY-4.0");
    }

    #[test]
    fn test_material_preset_outputs_structure() {
        let result = eval_to_json(
            r#"
material_preset_v1(
    asset_id = "test-preset",
    seed = 42,
    output_prefix = "tex/test",
    resolution = [64, 64],
    preset = "clean_plastic"
)
"#,
        )
        .unwrap();

        let outputs = result["outputs"].as_array().unwrap();

        // Check output paths and sources
        let albedo = &outputs[0];
        assert_eq!(albedo["path"], "tex/test_albedo.png");
        assert_eq!(albedo["source"], "albedo");
        assert_eq!(albedo["kind"], "primary");

        let roughness = &outputs[1];
        assert_eq!(roughness["path"], "tex/test_roughness.png");
        assert_eq!(roughness["source"], "roughness");

        let metallic = &outputs[2];
        assert_eq!(metallic["path"], "tex/test_metallic.png");
        assert_eq!(metallic["source"], "metallic");

        let normal = &outputs[3];
        assert_eq!(normal["path"], "tex/test_normal.png");
        assert_eq!(normal["source"], "normal");

        let metadata = &outputs[4];
        assert_eq!(metadata["path"], "tex/test.material.json");
        assert_eq!(metadata["kind"], "metadata");
        assert_eq!(metadata["format"], "json");
    }

    #[test]
    fn test_material_preset_invalid_preset() {
        let result = eval_to_json(
            r#"
material_preset_v1(
    asset_id = "test",
    seed = 42,
    output_prefix = "tex/test",
    resolution = [64, 64],
    preset = "invalid_preset"
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
        assert!(err.contains("invalid_preset"));
    }

    #[test]
    fn test_material_preset_invalid_resolution() {
        let result = eval_to_json(
            r#"
material_preset_v1(
    asset_id = "test",
    seed = 42,
    output_prefix = "tex/test",
    resolution = [64],
    preset = "toon_metal"
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S102"));
    }

    #[test]
    fn test_all_presets() {
        let presets = [
            "toon_metal",
            "stylized_wood",
            "neon_glow",
            "ceramic_glaze",
            "sci_fi_panel",
            "clean_plastic",
            "rough_stone",
            "brushed_metal",
        ];

        for preset in presets {
            let code = format!(
                r#"
material_preset_v1(
    asset_id = "test-{}",
    seed = 42,
    output_prefix = "tex/test",
    resolution = [64, 64],
    preset = "{}"
)
"#,
                preset.replace('_', "-"),
                preset
            );
            let result = eval_to_json(&code);
            assert!(result.is_ok(), "Failed for preset: {}", preset);
        }
    }
}
