//! Audio stdlib functions for synthesis and effects.
//!
//! Provides helper functions for creating audio synthesis layers, envelopes,
//! oscillators, filters, and effects.

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use super::validation::validate_non_empty;

mod effects;
mod filters;
mod layers;
mod modulation;
mod synthesis;

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

/// Valid audio output formats.
const AUDIO_FORMATS: &[&str] = &["wav", "ogg", "mp3", "flac"];

/// Registers audio stdlib functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    modulation::register(builder);
    synthesis::register(builder);
    filters::register(builder);
    effects::register(builder);
    layers::register(builder);
    register_audio_spec_functions(builder);
}

#[starlark_module]
fn register_audio_spec_functions(builder: &mut GlobalsBuilder) {
    /// Creates a complete audio spec with audio_v1 recipe.
    ///
    /// # Arguments
    /// * `asset_id` - Kebab-case identifier for the asset
    /// * `seed` - Deterministic seed (0 to 2^32-1)
    /// * `output_path` - Output file path
    /// * `format` - Audio format: "wav", "ogg", "mp3", "flac"
    /// * `duration_seconds` - Duration of the audio in seconds
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100, 48000)
    /// * `layers` - List of audio layers from `audio_layer()`
    /// * `effects` - Optional list of global effects
    /// * `description` - Asset description (optional)
    /// * `tags` - Style tags (optional)
    /// * `license` - SPDX license identifier (default: "CC0-1.0")
    ///
    /// # Returns
    /// A complete spec dict ready for serialization.
    ///
    /// # Example
    /// ```starlark
    /// audio_spec(
    ///     asset_id = "test-audio-01",
    ///     seed = 42,
    ///     output_path = "audio/test.wav",
    ///     format = "wav",
    ///     duration_seconds = 2.0,
    ///     sample_rate = 44100,
    ///     layers = [audio_layer(oscillator(440.0))]
    /// )
    /// ```
    fn audio_spec<'v>(
        #[starlark(require = named)] asset_id: &str,
        #[starlark(require = named)] seed: i64,
        #[starlark(require = named)] output_path: &str,
        #[starlark(require = named)] format: &str,
        #[starlark(require = named)] duration_seconds: f64,
        #[starlark(require = named)] sample_rate: i32,
        #[starlark(require = named)] layers: UnpackList<Value<'v>>,
        #[starlark(default = NoneType)] effects: Value<'v>,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        use super::validation::validate_enum;

        // Validate asset_id
        validate_non_empty(asset_id, "audio_spec", "asset_id").map_err(|e| anyhow::anyhow!(e))?;

        // Validate format
        validate_enum(format, AUDIO_FORMATS, "audio_spec", "format")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate seed
        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: audio_spec(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        // Validate duration
        if duration_seconds <= 0.0 {
            return Err(anyhow::anyhow!(
                "S103: audio_spec(): 'duration_seconds' must be positive, got {}",
                duration_seconds
            ));
        }

        // Validate sample rate
        if sample_rate <= 0 {
            return Err(anyhow::anyhow!(
                "S103: audio_spec(): 'sample_rate' must be positive, got {}",
                sample_rate
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
            heap.alloc_str("audio").to_value(),
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
            heap.alloc_str("audio_v1").to_value(),
        );

        // Create params
        let mut params = new_dict(heap);
        params.insert_hashed(
            hashed_key(heap, "duration_seconds"),
            heap.alloc(duration_seconds).to_value(),
        );
        params.insert_hashed(
            hashed_key(heap, "sample_rate"),
            heap.alloc(sample_rate).to_value(),
        );

        // layers
        let layers_list = heap.alloc(AllocList(layers.items));
        params.insert_hashed(hashed_key(heap, "layers"), layers_list);

        // Optional: effects
        if !effects.is_none() {
            params.insert_hashed(hashed_key(heap, "effects"), effects);
        }

        recipe.insert_hashed(hashed_key(heap, "params"), heap.alloc(params).to_value());

        spec.insert_hashed(hashed_key(heap, "recipe"), heap.alloc(recipe).to_value());

        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::stdlib::tests::eval_to_json;

    #[test]
    fn test_audio_spec_basic() {
        let result = eval_to_json(
            r#"
audio_spec(
    asset_id = "test-audio-01",
    seed = 42,
    output_path = "audio/test.wav",
    format = "wav",
    duration_seconds = 2.0,
    sample_rate = 44100,
    layers = [audio_layer(oscillator(440.0))]
)
"#,
        )
        .unwrap();
        assert_eq!(result["spec_version"], 1);
        assert_eq!(result["asset_id"], "test-audio-01");
        assert_eq!(result["asset_type"], "audio");
        assert_eq!(result["seed"], 42);
        assert_eq!(result["recipe"]["kind"], "audio_v1");
        assert_eq!(result["recipe"]["params"]["duration_seconds"], 2.0);
        assert_eq!(result["recipe"]["params"]["sample_rate"], 44100);
        assert!(result["outputs"].is_array());
    }

    #[test]
    fn test_audio_spec_invalid_format() {
        let result = eval_to_json(
            r#"
audio_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.aac",
    format = "aac",
    duration_seconds = 1.0,
    sample_rate = 44100,
    layers = []
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
    }

    #[test]
    fn test_audio_spec_invalid_seed() {
        let result = eval_to_json(
            r#"
audio_spec(
    asset_id = "test",
    seed = -1,
    output_path = "test.wav",
    format = "wav",
    duration_seconds = 1.0,
    sample_rate = 44100,
    layers = []
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("seed"));
    }

    #[test]
    fn test_notch_filter_basic() {
        let result = eval_to_json(r#"notch(1000.0, 2.0)"#).unwrap();
        assert_eq!(result["type"], "notch");
        assert_eq!(result["center"], 1000.0);
        assert_eq!(result["resonance"], 2.0);
        assert!(result.get("center_end").is_none());
    }

    #[test]
    fn test_notch_filter_with_sweep() {
        let result = eval_to_json(r#"notch(500.0, 1.5, 2000.0)"#).unwrap();
        assert_eq!(result["type"], "notch");
        assert_eq!(result["center"], 500.0);
        assert_eq!(result["resonance"], 1.5);
        assert_eq!(result["center_end"], 2000.0);
    }

    #[test]
    fn test_notch_filter_default_resonance() {
        let result = eval_to_json(r#"notch(1000.0)"#).unwrap();
        assert_eq!(result["type"], "notch");
        assert_eq!(result["center"], 1000.0);
        assert_eq!(result["resonance"], 1.0);
    }

    #[test]
    fn test_notch_filter_invalid_center() {
        let result = eval_to_json(r#"notch(-100.0)"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    #[test]
    fn test_allpass_filter_basic() {
        let result = eval_to_json(r#"allpass(1000.0, 2.0)"#).unwrap();
        assert_eq!(result["type"], "allpass");
        assert_eq!(result["frequency"], 1000.0);
        assert_eq!(result["resonance"], 2.0);
        assert!(result.get("frequency_end").is_none());
    }

    #[test]
    fn test_allpass_filter_with_sweep() {
        let result = eval_to_json(r#"allpass(500.0, 1.5, 2000.0)"#).unwrap();
        assert_eq!(result["type"], "allpass");
        assert_eq!(result["frequency"], 500.0);
        assert_eq!(result["resonance"], 1.5);
        assert_eq!(result["frequency_end"], 2000.0);
    }

    #[test]
    fn test_allpass_filter_default_resonance() {
        let result = eval_to_json(r#"allpass(1000.0)"#).unwrap();
        assert_eq!(result["type"], "allpass");
        assert_eq!(result["frequency"], 1000.0);
        assert_eq!(result["resonance"], 0.707);
    }

    #[test]
    fn test_allpass_filter_invalid_frequency() {
        let result = eval_to_json(r#"allpass(-100.0)"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    #[test]
    fn test_layer_with_notch_filter() {
        let result = eval_to_json(
            r#"audio_layer(oscillator(440.0), filter = notch(1000.0, 2.0))"#,
        )
        .unwrap();
        assert_eq!(result["filter"]["type"], "notch");
        assert_eq!(result["filter"]["center"], 1000.0);
    }

    #[test]
    fn test_layer_with_allpass_filter() {
        let result = eval_to_json(
            r#"audio_layer(oscillator(440.0), filter = allpass(1000.0, 2.0))"#,
        )
        .unwrap();
        assert_eq!(result["filter"]["type"], "allpass");
        assert_eq!(result["filter"]["frequency"], 1000.0);
    }
}
