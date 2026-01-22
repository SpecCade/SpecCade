//! Trimsheet/atlas Starlark helper functions.

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::{validate_non_empty, validate_positive_int};

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

/// Registers trimsheet Starlark functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_trimsheet_functions(builder);
}

#[starlark_module]
fn register_trimsheet_functions(builder: &mut GlobalsBuilder) {
    /// Creates a trimsheet tile definition with a solid color.
    ///
    /// # Arguments
    /// * `id` - Unique tile identifier
    /// * `width` - Tile width in pixels
    /// * `height` - Tile height in pixels
    /// * `color` - RGBA color as [r, g, b, a] with values in 0.0-1.0 range
    ///
    /// # Returns
    /// A dict matching TrimsheetTile with color source.
    ///
    /// # Example
    /// ```starlark
    /// trimsheet_tile("grass", 128, 128, [0.2, 0.6, 0.2, 1.0])
    /// trimsheet_tile("stone", 64, 64, [0.5, 0.5, 0.5, 1.0])
    /// ```
    fn trimsheet_tile<'v>(
        #[starlark(require = named)] id: &str,
        #[starlark(require = named)] width: i32,
        #[starlark(require = named)] height: i32,
        #[starlark(require = named)] color: UnpackList<f64>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(id, "trimsheet_tile", "id").map_err(|e| anyhow::anyhow!(e))?;
        validate_positive_int(width as i64, "trimsheet_tile", "width")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive_int(height as i64, "trimsheet_tile", "height")
            .map_err(|e| anyhow::anyhow!(e))?;

        if color.items.len() != 4 {
            return Err(anyhow::anyhow!(
                "S101: trimsheet_tile(): 'color' must be [r, g, b, a], got {} values",
                color.items.len()
            ));
        }

        for (i, &c) in color.items.iter().enumerate() {
            if !(0.0..=1.0).contains(&c) {
                return Err(anyhow::anyhow!(
                    "S103: trimsheet_tile(): color[{}] must be in 0.0-1.0 range, got {}",
                    i,
                    c
                ));
            }
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "id"), heap.alloc_str(id).to_value());
        dict.insert_hashed(hashed_key(heap, "width"), heap.alloc(width).to_value());
        dict.insert_hashed(hashed_key(heap, "height"), heap.alloc(height).to_value());

        // Create color array
        let color_list = heap.alloc(AllocList(
            color
                .items
                .iter()
                .map(|&c| heap.alloc(c).to_value())
                .collect::<Vec<_>>(),
        ));
        dict.insert_hashed(hashed_key(heap, "color"), color_list);

        Ok(dict)
    }

    /// Creates a complete trimsheet spec with trimsheet_v1 recipe.
    ///
    /// # Arguments
    /// * `asset_id` - Kebab-case identifier for the asset
    /// * `seed` - Deterministic seed (0 to 2^32-1)
    /// * `output_path` - Output file path for the atlas PNG
    /// * `metadata_path` - Output file path for the metadata JSON (optional)
    /// * `resolution` - [width, height] in pixels
    /// * `tiles` - List of tile definitions from trimsheet_tile()
    /// * `padding` - Padding/gutter in pixels (default: 2)
    /// * `description` - Asset description (optional)
    /// * `tags` - Style tags (optional)
    /// * `license` - SPDX license identifier (default: "CC0-1.0")
    ///
    /// # Returns
    /// A complete spec dict ready for serialization.
    ///
    /// # Example
    /// ```starlark
    /// trimsheet_spec(
    ///     asset_id = "tileset-01",
    ///     seed = 42,
    ///     output_path = "atlas/tileset.png",
    ///     metadata_path = "atlas/tileset.json",
    ///     resolution = [1024, 1024],
    ///     tiles = [
    ///         trimsheet_tile(id = "grass", width = 128, height = 128, color = [0.2, 0.6, 0.2, 1.0]),
    ///         trimsheet_tile(id = "stone", width = 64, height = 64, color = [0.5, 0.5, 0.5, 1.0])
    ///     ]
    /// )
    /// ```
    fn trimsheet_spec<'v>(
        #[starlark(require = named)] asset_id: &str,
        #[starlark(require = named)] seed: i64,
        #[starlark(require = named)] output_path: &str,
        #[starlark(require = named)] resolution: UnpackList<i32>,
        #[starlark(require = named)] tiles: UnpackList<Value<'v>>,
        #[starlark(default = NoneType)] metadata_path: Value<'v>,
        #[starlark(default = 2)] padding: i32,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate asset_id
        validate_non_empty(asset_id, "trimsheet_spec", "asset_id")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate seed
        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: trimsheet_spec(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        // Validate resolution
        if resolution.items.len() != 2 {
            return Err(anyhow::anyhow!(
                "S101: trimsheet_spec(): 'resolution' must be [width, height], got {} values",
                resolution.items.len()
            ));
        }
        let width = resolution.items[0];
        let height = resolution.items[1];
        if width <= 0 || height <= 0 {
            return Err(anyhow::anyhow!(
                "S103: trimsheet_spec(): resolution values must be positive, got [{}, {}]",
                width,
                height
            ));
        }

        // Validate padding
        if padding < 0 {
            return Err(anyhow::anyhow!(
                "S103: trimsheet_spec(): 'padding' must be non-negative, got {}",
                padding
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
        let mut outputs_vec = Vec::new();

        // Primary output (PNG)
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
            heap.alloc_str("texture.trimsheet_v1").to_value(),
        );

        // Create params
        let mut params = new_dict(heap);

        // resolution as list
        let res_list = heap.alloc(AllocList(vec![
            heap.alloc(width).to_value(),
            heap.alloc(height).to_value(),
        ]));
        params.insert_hashed(hashed_key(heap, "resolution"), res_list);

        // padding
        params.insert_hashed(hashed_key(heap, "padding"), heap.alloc(padding).to_value());

        // tiles as list
        let tiles_list = heap.alloc(AllocList(tiles.items));
        params.insert_hashed(hashed_key(heap, "tiles"), tiles_list);

        recipe.insert_hashed(hashed_key(heap, "params"), heap.alloc(params).to_value());

        spec.insert_hashed(hashed_key(heap, "recipe"), heap.alloc(recipe).to_value());

        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::eval_to_json;

    #[test]
    fn test_trimsheet_tile_basic() {
        let result = eval_to_json(
            r#"
trimsheet_tile(id = "grass", width = 128, height = 128, color = [0.2, 0.6, 0.2, 1.0])
"#,
        )
        .unwrap();

        assert_eq!(result["id"], "grass");
        assert_eq!(result["width"], 128);
        assert_eq!(result["height"], 128);
        assert!(result["color"].is_array());
        let color = result["color"].as_array().unwrap();
        assert_eq!(color.len(), 4);
        assert!((color[0].as_f64().unwrap() - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_trimsheet_tile_invalid_color_length() {
        let result = eval_to_json(
            r#"
trimsheet_tile(id = "test", width = 64, height = 64, color = [1.0, 0.0, 0.0])
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("S101"));
    }

    #[test]
    fn test_trimsheet_tile_color_out_of_range() {
        let result = eval_to_json(
            r#"
trimsheet_tile(id = "test", width = 64, height = 64, color = [1.5, 0.0, 0.0, 1.0])
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("S103"));
    }

    #[test]
    fn test_trimsheet_spec_basic() {
        let result = eval_to_json(
            r#"
trimsheet_spec(
    asset_id = "test-tileset-01",
    seed = 42,
    output_path = "atlas/test.png",
    resolution = [512, 512],
    tiles = [
        trimsheet_tile(id = "grass", width = 128, height = 128, color = [0.2, 0.6, 0.2, 1.0])
    ]
)
"#,
        )
        .unwrap();

        assert_eq!(result["spec_version"], 1);
        assert_eq!(result["asset_id"], "test-tileset-01");
        assert_eq!(result["asset_type"], "texture");
        assert_eq!(result["seed"], 42);
        assert_eq!(result["recipe"]["kind"], "texture.trimsheet_v1");
        assert_eq!(result["recipe"]["params"]["padding"], 2);

        let res = result["recipe"]["params"]["resolution"].as_array().unwrap();
        assert_eq!(res[0], 512);
        assert_eq!(res[1], 512);

        let tiles = result["recipe"]["params"]["tiles"].as_array().unwrap();
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0]["id"], "grass");
    }

    #[test]
    fn test_trimsheet_spec_with_metadata() {
        let result = eval_to_json(
            r#"
trimsheet_spec(
    asset_id = "test-tileset-02",
    seed = 123,
    output_path = "atlas/test.png",
    metadata_path = "atlas/test.json",
    resolution = [256, 256],
    tiles = [],
    padding = 4
)
"#,
        )
        .unwrap();

        let outputs = result["outputs"].as_array().unwrap();
        assert_eq!(outputs.len(), 2);

        // Find primary and metadata outputs
        let primary = outputs.iter().find(|o| o["kind"] == "primary").unwrap();
        let metadata = outputs.iter().find(|o| o["kind"] == "metadata").unwrap();

        assert_eq!(primary["format"], "png");
        assert_eq!(primary["path"], "atlas/test.png");
        assert_eq!(metadata["format"], "json");
        assert_eq!(metadata["path"], "atlas/test.json");

        assert_eq!(result["recipe"]["params"]["padding"], 4);
    }

    #[test]
    fn test_trimsheet_spec_invalid_seed() {
        let result = eval_to_json(
            r#"
trimsheet_spec(
    asset_id = "test",
    seed = -1,
    output_path = "test.png",
    resolution = [256, 256],
    tiles = []
)
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("seed"));
    }

    #[test]
    fn test_trimsheet_spec_invalid_resolution() {
        let result = eval_to_json(
            r#"
trimsheet_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.png",
    resolution = [256],
    tiles = []
)
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("S101"));
    }

    #[test]
    fn test_trimsheet_spec_negative_padding() {
        let result = eval_to_json(
            r#"
trimsheet_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.png",
    resolution = [256, 256],
    tiles = [],
    padding = -1
)
"#,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("padding"));
    }
}
