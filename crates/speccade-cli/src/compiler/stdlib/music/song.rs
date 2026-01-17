//! Song assembly and spec functions for tracker module creation.

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, list::UnpackList, none::NoneType, Heap, Value, ValueLike};

use crate::compiler::stdlib::validation::{validate_enum, validate_non_empty};

use super::util::{hashed_key, new_dict};

/// Valid tracker formats.
const TRACKER_FORMATS: &[&str] = &["xm", "it"];

/// Registers song functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_song_functions(builder);
}

#[starlark_module]
fn register_song_functions(builder: &mut GlobalsBuilder) {
    /// Creates IT-specific module options.
    ///
    /// # Arguments
    /// * `stereo` - Enable stereo output (default: true)
    /// * `global_volume` - Global volume (0-128, default: 128)
    /// * `mix_volume` - Mix volume (0-128, default: 48)
    ///
    /// # Returns
    /// A dict matching the ItOptions IR structure.
    ///
    /// # Example
    /// ```starlark
    /// it_options()
    /// it_options(stereo = True, global_volume = 100)
    /// ```
    fn it_options<'v>(
        #[starlark(default = true)] stereo: bool,
        #[starlark(default = 128)] global_volume: i32,
        #[starlark(default = 48)] mix_volume: i32,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(0..=128).contains(&global_volume) {
            return Err(anyhow::anyhow!(
                "S103: it_options(): 'global_volume' must be 0-128, got {}",
                global_volume
            ));
        }
        if !(0..=128).contains(&mix_volume) {
            return Err(anyhow::anyhow!(
                "S103: it_options(): 'mix_volume' must be 0-128, got {}",
                mix_volume
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "stereo"),
            heap.alloc(stereo).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "global_volume"),
            heap.alloc(global_volume).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "mix_volume"),
            heap.alloc(mix_volume).to_value(),
        );

        Ok(dict)
    }

    /// Creates a complete tracker song recipe.
    ///
    /// # Arguments
    /// * `format` - Tracker format: "xm" or "it"
    /// * `bpm` - Beats per minute (30-300)
    /// * `speed` - Tracker speed (ticks per row, 1-31)
    /// * `channels` - Number of channels (XM: 1-32, IT: 1-64)
    /// * `instruments` - List of instrument dicts from `tracker_instrument()`
    /// * `patterns` - Dict of pattern_name -> pattern dict from `tracker_pattern()`
    /// * `arrangement` - List of arrangement entries from `arrangement_entry()`
    /// * `name` - Song internal name (used in module)
    /// * `title` - Song display title (for metadata)
    /// * `loop` - Whether the song should loop (default: false)
    /// * `restart_position` - Order-table index to restart at when looping
    /// * `automation` - List of automation entries
    /// * `it_options` - IT-specific options dict from `it_options()`
    ///
    /// # Returns
    /// A dict matching the recipe structure for `music.tracker_song_v1`.
    ///
    /// # Example
    /// ```starlark
    /// tracker_song(
    ///     format = "xm",
    ///     bpm = 120,
    ///     speed = 6,
    ///     channels = 4,
    ///     instruments = [
    ///         tracker_instrument(name = "bass", synthesis = instrument_synthesis("sawtooth"))
    ///     ],
    ///     patterns = {
    ///         "intro": tracker_pattern(64, notes = {"0": [pattern_note(0, "C4", 0)]})
    ///     },
    ///     arrangement = [arrangement_entry("intro", 4)]
    /// )
    /// ```
    fn tracker_song<'v>(
        #[starlark(require = named)] format: &str,
        #[starlark(require = named)] bpm: i32,
        #[starlark(require = named)] speed: i32,
        #[starlark(require = named)] channels: i32,
        #[starlark(require = named)] instruments: UnpackList<Value<'v>>,
        #[starlark(require = named)] patterns: Value<'v>,
        #[starlark(require = named)] arrangement: UnpackList<Value<'v>>,
        #[starlark(default = NoneType)] name: Value<'v>,
        #[starlark(default = NoneType)] title: Value<'v>,
        #[starlark(default = false)] r#loop: bool,
        #[starlark(default = NoneType)] restart_position: Value<'v>,
        #[starlark(default = NoneType)] automation: Value<'v>,
        #[starlark(default = NoneType)] it_options: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate format
        validate_enum(format, TRACKER_FORMATS, "tracker_song", "format")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate BPM
        if !(30..=300).contains(&bpm) {
            return Err(anyhow::anyhow!(
                "S103: tracker_song(): 'bpm' must be 30-300, got {}",
                bpm
            ));
        }

        // Validate speed
        if !(1..=31).contains(&speed) {
            return Err(anyhow::anyhow!(
                "S103: tracker_song(): 'speed' must be 1-31, got {}",
                speed
            ));
        }

        // Validate channels based on format
        let max_channels = if format == "xm" { 32 } else { 64 };
        if !(1..=max_channels).contains(&channels) {
            return Err(anyhow::anyhow!(
                "S103: tracker_song(): 'channels' must be 1-{} for {} format, got {}",
                max_channels, format, channels
            ));
        }

        let mut dict = new_dict(heap);

        // Build the recipe structure
        dict.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("music.tracker_song_v1").to_value(),
        );

        // Create params dict
        let mut params = new_dict(heap);

        params.insert_hashed(
            hashed_key(heap, "format"),
            heap.alloc_str(format).to_value(),
        );
        params.insert_hashed(
            hashed_key(heap, "bpm"),
            heap.alloc(bpm).to_value(),
        );
        params.insert_hashed(
            hashed_key(heap, "speed"),
            heap.alloc(speed).to_value(),
        );
        params.insert_hashed(
            hashed_key(heap, "channels"),
            heap.alloc(channels).to_value(),
        );
        params.insert_hashed(
            hashed_key(heap, "loop"),
            heap.alloc(r#loop).to_value(),
        );

        // Optional: name
        if !name.is_none() {
            if let Some(name_str) = name.unpack_str() {
                params.insert_hashed(
                    hashed_key(heap, "name"),
                    heap.alloc_str(name_str).to_value(),
                );
            }
        }

        // Optional: title
        if !title.is_none() {
            if let Some(title_str) = title.unpack_str() {
                params.insert_hashed(
                    hashed_key(heap, "title"),
                    heap.alloc_str(title_str).to_value(),
                );
            }
        }

        // Optional: restart_position
        if !restart_position.is_none() {
            if let Some(pos) = restart_position.unpack_i32() {
                params.insert_hashed(
                    hashed_key(heap, "restart_position"),
                    heap.alloc(pos).to_value(),
                );
            }
        }

        // instruments
        let instruments_list = heap.alloc(AllocList(instruments.items));
        params.insert_hashed(
            hashed_key(heap, "instruments"),
            instruments_list,
        );

        // patterns
        params.insert_hashed(
            hashed_key(heap, "patterns"),
            patterns,
        );

        // arrangement
        let arrangement_list = heap.alloc(AllocList(arrangement.items));
        params.insert_hashed(
            hashed_key(heap, "arrangement"),
            arrangement_list,
        );

        // Optional: automation
        if !automation.is_none() {
            params.insert_hashed(
                hashed_key(heap, "automation"),
                automation,
            );
        }

        // Optional: it_options (only for IT format)
        if !it_options.is_none()
            && format == "it" {
                params.insert_hashed(
                    hashed_key(heap, "it_options"),
                    it_options,
                );
            }

        dict.insert_hashed(
            hashed_key(heap, "params"),
            heap.alloc(params).to_value(),
        );

        Ok(dict)
    }

    /// Creates a music spec with a tracker song recipe.
    ///
    /// This is a convenience wrapper that combines `spec()` with a tracker song recipe.
    ///
    /// # Arguments
    /// * `asset_id` - Kebab-case identifier for the asset
    /// * `seed` - Deterministic seed (0 to 2^32-1)
    /// * `output_path` - Output file path
    /// * `format` - Tracker format: "xm" or "it"
    /// * `bpm` - Beats per minute (30-300)
    /// * `speed` - Tracker speed (ticks per row, 1-31)
    /// * `channels` - Number of channels (XM: 1-32, IT: 1-64)
    /// * `instruments` - List of instrument dicts from `tracker_instrument()`
    /// * `patterns` - Dict of pattern_name -> pattern dict from `tracker_pattern()`
    /// * `arrangement` - List of arrangement entries from `arrangement_entry()`
    /// * `name` - Song internal name
    /// * `title` - Song display title
    /// * `loop` - Whether the song should loop
    /// * `description` - Asset description
    /// * `tags` - Style tags
    /// * `license` - SPDX license identifier (default: "CC0-1.0")
    ///
    /// # Returns
    /// A complete spec dict ready for serialization.
    ///
    /// # Example
    /// ```starlark
    /// music_spec(
    ///     asset_id = "test-song-01",
    ///     seed = 42,
    ///     output_path = "music/test.xm",
    ///     format = "xm",
    ///     bpm = 120,
    ///     speed = 6,
    ///     channels = 4,
    ///     instruments = [...],
    ///     patterns = {...},
    ///     arrangement = [...]
    /// )
    /// ```
    fn music_spec<'v>(
        #[starlark(require = named)] asset_id: &str,
        #[starlark(require = named)] seed: i64,
        #[starlark(require = named)] output_path: &str,
        #[starlark(require = named)] format: &str,
        #[starlark(require = named)] bpm: i32,
        #[starlark(require = named)] speed: i32,
        #[starlark(require = named)] channels: i32,
        #[starlark(require = named)] instruments: UnpackList<Value<'v>>,
        #[starlark(require = named)] patterns: Value<'v>,
        #[starlark(require = named)] arrangement: UnpackList<Value<'v>>,
        #[starlark(default = NoneType)] name: Value<'v>,
        #[starlark(default = NoneType)] title: Value<'v>,
        #[starlark(default = false)] r#loop: bool,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate asset_id
        validate_non_empty(asset_id, "music_spec", "asset_id")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate format
        validate_enum(format, TRACKER_FORMATS, "music_spec", "format")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate seed
        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: music_spec(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        // Validate BPM
        if !(30..=300).contains(&bpm) {
            return Err(anyhow::anyhow!(
                "S103: music_spec(): 'bpm' must be 30-300, got {}",
                bpm
            ));
        }

        // Validate speed
        if !(1..=31).contains(&speed) {
            return Err(anyhow::anyhow!(
                "S103: music_spec(): 'speed' must be 1-31, got {}",
                speed
            ));
        }

        // Validate channels
        let max_channels = if format == "xm" { 32 } else { 64 };
        if !(1..=max_channels).contains(&channels) {
            return Err(anyhow::anyhow!(
                "S103: music_spec(): 'channels' must be 1-{} for {} format, got {}",
                max_channels, format, channels
            ));
        }

        let mut spec = new_dict(heap);

        // spec_version
        spec.insert_hashed(
            hashed_key(heap, "spec_version"),
            heap.alloc(1).to_value(),
        );

        // asset_id
        spec.insert_hashed(
            hashed_key(heap, "asset_id"),
            heap.alloc_str(asset_id).to_value(),
        );

        // asset_type
        spec.insert_hashed(
            hashed_key(heap, "asset_type"),
            heap.alloc_str("music").to_value(),
        );

        // license
        spec.insert_hashed(
            hashed_key(heap, "license"),
            heap.alloc_str(license).to_value(),
        );

        // seed
        spec.insert_hashed(
            hashed_key(heap, "seed"),
            heap.alloc(seed as i32).to_value(),
        );

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
        spec.insert_hashed(
            hashed_key(heap, "outputs"),
            outputs_list,
        );

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
            spec.insert_hashed(
                hashed_key(heap, "style_tags"),
                tags,
            );
        }

        // Build recipe
        let mut recipe = new_dict(heap);
        recipe.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("music.tracker_song_v1").to_value(),
        );

        // Create params
        let mut params = new_dict(heap);
        params.insert_hashed(
            hashed_key(heap, "format"),
            heap.alloc_str(format).to_value(),
        );
        params.insert_hashed(
            hashed_key(heap, "bpm"),
            heap.alloc(bpm).to_value(),
        );
        params.insert_hashed(
            hashed_key(heap, "speed"),
            heap.alloc(speed).to_value(),
        );
        params.insert_hashed(
            hashed_key(heap, "channels"),
            heap.alloc(channels).to_value(),
        );
        params.insert_hashed(
            hashed_key(heap, "loop"),
            heap.alloc(r#loop).to_value(),
        );

        // Optional name/title
        if !name.is_none() {
            if let Some(name_str) = name.unpack_str() {
                params.insert_hashed(
                    hashed_key(heap, "name"),
                    heap.alloc_str(name_str).to_value(),
                );
            }
        }
        if !title.is_none() {
            if let Some(title_str) = title.unpack_str() {
                params.insert_hashed(
                    hashed_key(heap, "title"),
                    heap.alloc_str(title_str).to_value(),
                );
            }
        }

        // instruments
        let instruments_list = heap.alloc(AllocList(instruments.items));
        params.insert_hashed(
            hashed_key(heap, "instruments"),
            instruments_list,
        );

        // patterns
        params.insert_hashed(
            hashed_key(heap, "patterns"),
            patterns,
        );

        // arrangement
        let arrangement_list = heap.alloc(AllocList(arrangement.items));
        params.insert_hashed(
            hashed_key(heap, "arrangement"),
            arrangement_list,
        );

        recipe.insert_hashed(
            hashed_key(heap, "params"),
            heap.alloc(params).to_value(),
        );

        spec.insert_hashed(
            hashed_key(heap, "recipe"),
            heap.alloc(recipe).to_value(),
        );

        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::stdlib::tests::eval_to_json;

    #[test]
    fn test_it_options_defaults() {
        let result = eval_to_json("it_options()").unwrap();
        assert_eq!(result["stereo"], true);
        assert_eq!(result["global_volume"], 128);
        assert_eq!(result["mix_volume"], 48);
    }

    #[test]
    fn test_tracker_song_basic() {
        let result = eval_to_json(r#"
tracker_song(
    format = "xm",
    bpm = 120,
    speed = 6,
    channels = 4,
    instruments = [tracker_instrument(name = "bass")],
    patterns = {"intro": tracker_pattern(64)},
    arrangement = [arrangement_entry("intro")]
)
"#).unwrap();
        assert_eq!(result["kind"], "music.tracker_song_v1");
        assert_eq!(result["params"]["format"], "xm");
        assert_eq!(result["params"]["bpm"], 120);
        assert_eq!(result["params"]["channels"], 4);
    }

    #[test]
    fn test_tracker_song_invalid_format() {
        let result = eval_to_json(r#"
tracker_song(
    format = "mod",
    bpm = 120,
    speed = 6,
    channels = 4,
    instruments = [],
    patterns = {},
    arrangement = []
)
"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
    }

    #[test]
    fn test_tracker_song_invalid_bpm() {
        let result = eval_to_json(r#"
tracker_song(
    format = "xm",
    bpm = 500,
    speed = 6,
    channels = 4,
    instruments = [],
    patterns = {},
    arrangement = []
)
"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("bpm"));
    }

    #[test]
    fn test_music_spec_basic() {
        let result = eval_to_json(r#"
music_spec(
    asset_id = "test-song-01",
    seed = 42,
    output_path = "music/test.xm",
    format = "xm",
    bpm = 120,
    speed = 6,
    channels = 4,
    instruments = [tracker_instrument(name = "bass")],
    patterns = {"intro": tracker_pattern(64)},
    arrangement = [arrangement_entry("intro")]
)
"#).unwrap();
        assert_eq!(result["spec_version"], 1);
        assert_eq!(result["asset_id"], "test-song-01");
        assert_eq!(result["asset_type"], "music");
        assert_eq!(result["seed"], 42);
        assert_eq!(result["recipe"]["kind"], "music.tracker_song_v1");
        assert!(result["outputs"].is_array());
    }
}
