//! Texture spec creation functions.

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::{validate_enum, validate_non_empty};
use super::{hashed_key, new_dict, TEXTURE_FORMATS};

/// Registers texture spec functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_texture_spec_functions(builder);
}

#[starlark_module]
fn register_texture_spec_functions(builder: &mut GlobalsBuilder) {
    /// Creates a complete texture spec with procedural_v1 recipe.
    ///
    /// # Arguments
    /// * `asset_id` - Kebab-case identifier for the asset
    /// * `seed` - Deterministic seed (0 to 2^32-1)
    /// * `output_path` - Output file path
    /// * `format` - Texture format: "png", "jpg", "exr", "tga"
    /// * `resolution` - [width, height] in pixels
    /// * `nodes` - List of texture nodes
    /// * `tileable` - Whether texture should tile seamlessly (default: true)
    /// * `description` - Asset description (optional)
    /// * `tags` - Style tags (optional)
    /// * `license` - SPDX license identifier (default: "CC0-1.0")
    ///
    /// # Returns
    /// A complete spec dict ready for serialization.
    ///
    /// # Example
    /// ```starlark
    /// texture_spec(
    ///     asset_id = "test-texture-01",
    ///     seed = 42,
    ///     output_path = "textures/test.png",
    ///     format = "png",
    ///     resolution = [512, 512],
    ///     nodes = [noise_node("height")]
    /// )
    /// ```
    fn texture_spec<'v>(
        #[starlark(require = named)] asset_id: &str,
        #[starlark(require = named)] seed: i64,
        #[starlark(require = named)] output_path: &str,
        #[starlark(require = named)] format: &str,
        #[starlark(require = named)] resolution: UnpackList<i32>,
        #[starlark(require = named)] nodes: UnpackList<Value<'v>>,
        #[starlark(default = true)] tileable: bool,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate asset_id
        validate_non_empty(asset_id, "texture_spec", "asset_id").map_err(|e| anyhow::anyhow!(e))?;

        // Validate format
        validate_enum(format, TEXTURE_FORMATS, "texture_spec", "format")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate seed
        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: texture_spec(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        // Validate resolution
        if resolution.items.len() != 2 {
            return Err(anyhow::anyhow!(
                "S101: texture_spec(): 'resolution' must be [width, height], got {} values",
                resolution.items.len()
            ));
        }
        let width = resolution.items[0];
        let height = resolution.items[1];
        if width <= 0 || height <= 0 {
            return Err(anyhow::anyhow!(
                "S103: texture_spec(): resolution values must be positive, got [{}, {}]",
                width,
                height
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
            heap.alloc_str("texture.procedural_v1").to_value(),
        );

        // Create params
        let mut params = new_dict(heap);

        // resolution as list
        let res_list = heap.alloc(AllocList(vec![
            heap.alloc(width).to_value(),
            heap.alloc(height).to_value(),
        ]));
        params.insert_hashed(hashed_key(heap, "resolution"), res_list);

        params.insert_hashed(
            hashed_key(heap, "tileable"),
            heap.alloc(tileable).to_value(),
        );

        // nodes as list
        let nodes_list = heap.alloc(AllocList(nodes.items));
        params.insert_hashed(hashed_key(heap, "nodes"), nodes_list);

        recipe.insert_hashed(hashed_key(heap, "params"), heap.alloc(params).to_value());

        spec.insert_hashed(hashed_key(heap, "recipe"), heap.alloc(recipe).to_value());

        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::eval_to_json;

    // ========================================================================
    // texture_spec() tests
    // ========================================================================

    #[test]
    fn test_texture_spec_basic() {
        let result = eval_to_json(
            r#"
texture_spec(
    asset_id = "test-texture-01",
    seed = 42,
    output_path = "textures/test.png",
    format = "png",
    resolution = [512, 512],
    nodes = [noise_node("height")]
)
"#,
        )
        .unwrap();
        assert_eq!(result["spec_version"], 1);
        assert_eq!(result["asset_id"], "test-texture-01");
        assert_eq!(result["asset_type"], "texture");
        assert_eq!(result["seed"], 42);
        assert_eq!(result["recipe"]["kind"], "texture.procedural_v1");
        assert!(result["recipe"]["params"]["resolution"].is_array());
        assert_eq!(result["recipe"]["params"]["tileable"], true);
        assert!(result["outputs"].is_array());
    }

    #[test]
    fn test_texture_spec_invalid_format() {
        let result = eval_to_json(
            r#"
texture_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.bmp",
    format = "bmp",
    resolution = [512, 512],
    nodes = []
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
    }

    #[test]
    fn test_texture_spec_invalid_seed() {
        let result = eval_to_json(
            r#"
texture_spec(
    asset_id = "test",
    seed = -1,
    output_path = "test.png",
    format = "png",
    resolution = [512, 512],
    nodes = []
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("seed"));
    }
}
