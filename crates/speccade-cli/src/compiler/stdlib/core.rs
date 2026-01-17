//! Core stdlib functions for spec scaffolding.
//!
//! Provides `spec()` and `output()` functions for creating complete spec structures.

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use super::validation::validate_non_empty;

/// Helper to create a hashed key for dict insertion.
/// String hashing cannot fail, so we use expect.
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

/// Registers core stdlib functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_core_functions(builder);
}

#[starlark_module]
fn register_core_functions(builder: &mut GlobalsBuilder) {
    /// Creates an output specification.
    ///
    /// # Arguments
    /// * `path` - Output file path (e.g., "sounds/laser.wav")
    /// * `format` - Output format (e.g., "wav", "png", "glb")
    /// * `kind` - Output kind: "primary" (default) or "secondary"
    /// * `source` - Optional source node ID (for texture procedural graphs)
    ///
    /// # Returns
    /// A dict matching the Output IR structure.
    ///
    /// # Example
    /// ```starlark
    /// output("sounds/laser.wav", "wav")
    /// output("textures/preview.png", "png", "secondary")
    /// output("textures/noise.png", "png", source = "mask")
    /// ```
    fn output<'v>(
        path: &str,
        format: &str,
        #[starlark(default = "primary")] kind: &str,
        #[starlark(default = NoneType)] source: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(path, "output", "path")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_non_empty(format, "output", "format")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate kind
        let valid_kinds = &["primary", "secondary"];
        if !valid_kinds.contains(&kind) {
            return Err(anyhow::anyhow!(
                "S104: output(): 'kind' must be one of: {}",
                valid_kinds.join(", ")
            ));
        }

        let mut dict = new_dict(heap);
        dict.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str(kind).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "format"),
            heap.alloc_str(format).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "path"),
            heap.alloc_str(path).to_value(),
        );

        // Optional: source
        if !source.is_none() {
            if let Some(source_str) = source.unpack_str() {
                validate_non_empty(source_str, "output", "source")
                    .map_err(|e| anyhow::anyhow!(e))?;
                dict.insert_hashed(
                    hashed_key(heap, "source"),
                    heap.alloc_str(source_str).to_value(),
                );
            } else {
                return Err(anyhow::anyhow!(
                    "S104: output(): 'source' must be a string"
                ));
            }
        }

        Ok(dict)
    }

    /// Creates a complete spec dictionary.
    ///
    /// # Arguments
    /// * `asset_id` - Kebab-case identifier for the asset
    /// * `asset_type` - Asset type: "audio", "texture", "static_mesh", etc.
    /// * `seed` - Deterministic seed (0 to 2^32-1)
    /// * `outputs` - List of output specifications from `output()`
    /// * `recipe` - Optional recipe specification dict
    /// * `description` - Optional asset description
    /// * `tags` - Optional list of style tags
    /// * `license` - Optional SPDX license identifier (default: "CC0-1.0")
    ///
    /// # Returns
    /// A dict matching the Spec IR structure with spec_version: 1.
    ///
    /// # Example
    /// ```starlark
    /// spec(
    ///     asset_id = "my-sound-01",
    ///     asset_type = "audio",
    ///     seed = 42,
    ///     outputs = [output("sounds/laser.wav", "wav")]
    /// )
    /// ```
    fn spec<'v>(
        asset_id: &str,
        asset_type: &str,
        seed: i64,
        outputs: UnpackList<Value<'v>>,
        #[starlark(default = NoneType)] recipe: Value<'v>,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate asset_id
        validate_non_empty(asset_id, "spec", "asset_id")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate asset_type
        let valid_types = &["audio", "texture", "static_mesh", "animation", "music", "character"];
        if !valid_types.contains(&asset_type) {
            return Err(anyhow::anyhow!(
                "S104: spec(): 'asset_type' must be one of: {}",
                valid_types.join(", ")
            ));
        }

        // Validate seed is in valid range (0 to 2^32-1)
        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: spec(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        // Validate outputs is non-empty
        if outputs.items.is_empty() {
            return Err(anyhow::anyhow!(
                "S101: spec(): 'outputs' must have at least one output"
            ));
        }

        let mut dict = new_dict(heap);

        // spec_version (always 1)
        dict.insert_hashed(
            hashed_key(heap, "spec_version"),
            heap.alloc(1).to_value(),
        );

        // asset_id
        dict.insert_hashed(
            hashed_key(heap, "asset_id"),
            heap.alloc_str(asset_id).to_value(),
        );

        // asset_type
        dict.insert_hashed(
            hashed_key(heap, "asset_type"),
            heap.alloc_str(asset_type).to_value(),
        );

        // license
        dict.insert_hashed(
            hashed_key(heap, "license"),
            heap.alloc_str(license).to_value(),
        );

        // seed
        dict.insert_hashed(
            hashed_key(heap, "seed"),
            heap.alloc(seed as i32).to_value(),
        );

        // outputs (convert from list)
        let outputs_list = heap.alloc(AllocList(outputs.items));
        dict.insert_hashed(
            hashed_key(heap, "outputs"),
            outputs_list,
        );

        // Optional: description
        if !description.is_none() {
            if let Some(desc) = description.unpack_str() {
                dict.insert_hashed(
                    hashed_key(heap, "description"),
                    heap.alloc_str(desc).to_value(),
                );
            }
        }

        // Optional: style_tags
        if !tags.is_none() {
            dict.insert_hashed(
                hashed_key(heap, "style_tags"),
                tags,
            );
        }

        // Optional: recipe
        if !recipe.is_none() {
            dict.insert_hashed(
                hashed_key(heap, "recipe"),
                recipe,
            );
        }

        Ok(dict)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::eval_to_json;

    #[test]
    fn test_output_minimal() {
        let result = eval_to_json(r#"output("sounds/test.wav", "wav")"#).unwrap();
        assert_eq!(result["path"], "sounds/test.wav");
        assert_eq!(result["format"], "wav");
        assert_eq!(result["kind"], "primary");
    }

    #[test]
    fn test_output_secondary() {
        let result = eval_to_json(r#"output("textures/preview.png", "png", "secondary")"#).unwrap();
        assert_eq!(result["kind"], "secondary");
    }

    #[test]
    fn test_output_invalid_kind() {
        let result = eval_to_json(r#"output("test.wav", "wav", "tertiary")"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
    }

    #[test]
    fn test_spec_minimal() {
        let result = eval_to_json(r#"
spec(
    asset_id = "test-asset-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("test.wav", "wav")]
)
"#).unwrap();

        assert_eq!(result["spec_version"], 1);
        assert_eq!(result["asset_id"], "test-asset-01");
        assert_eq!(result["asset_type"], "audio");
        assert_eq!(result["seed"], 42);
        assert_eq!(result["license"], "CC0-1.0");
        assert!(result["outputs"].is_array());
        assert_eq!(result["outputs"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_spec_with_all_options() {
        let result = eval_to_json(r#"
spec(
    asset_id = "full-asset-01",
    asset_type = "audio",
    seed = 12345,
    outputs = [output("test.wav", "wav")],
    description = "Test description",
    tags = ["retro", "sfx"],
    license = "MIT",
    recipe = {"kind": "audio_v1", "params": {}}
)
"#).unwrap();

        assert_eq!(result["description"], "Test description");
        assert_eq!(result["license"], "MIT");
        assert!(result["style_tags"].is_array());
        assert!(result["recipe"].is_object());
    }

    #[test]
    fn test_spec_invalid_asset_type() {
        let result = eval_to_json(r#"
spec(
    asset_id = "test",
    asset_type = "invalid_type",
    seed = 42,
    outputs = [output("test.wav", "wav")]
)
"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
    }

    #[test]
    fn test_spec_empty_outputs() {
        let result = eval_to_json(r#"
spec(
    asset_id = "test",
    asset_type = "audio",
    seed = 42,
    outputs = []
)
"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S101"));
    }

    #[test]
    fn test_spec_negative_seed() {
        let result = eval_to_json(r#"
spec(
    asset_id = "test",
    asset_type = "audio",
    seed = -1,
    outputs = [output("test.wav", "wav")]
)
"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }
}
