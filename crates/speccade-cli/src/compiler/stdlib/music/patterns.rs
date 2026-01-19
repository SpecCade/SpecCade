//! Pattern and arrangement functions for tracker module creation.

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{dict::Dict, none::NoneType, Heap, Value, ValueLike};

use crate::compiler::stdlib::validation::{validate_non_empty, validate_positive_int};

use super::util::{hashed_key, new_dict};

/// Registers pattern functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_pattern_functions(builder);
}

#[starlark_module]
fn register_pattern_functions(builder: &mut GlobalsBuilder) {
    /// Creates a pattern note event.
    ///
    /// # Arguments
    /// * `row` - Row number (0-indexed)
    /// * `note` - Note name (e.g., "C4", "A#3") or "---" for note off
    /// * `inst` - Instrument index (0-indexed)
    /// * `channel` - Channel number (0-indexed, for flat array format)
    /// * `vol` - Volume (0-64)
    /// * `effect` - Effect command number
    /// * `param` - Effect parameter
    /// * `effect_name` - Named effect (e.g., "arpeggio", "portamento")
    /// * `effect_xy` - Effect XY parameters as [X, Y]
    ///
    /// # Returns
    /// A dict matching the PatternNote IR structure.
    ///
    /// # Example
    /// ```starlark
    /// pattern_note(0, "C4", 0)
    /// pattern_note(4, "E4", 0, vol = 48)
    /// pattern_note(8, "G4", 0, effect_name = "arpeggio", effect_xy = [3, 7])
    /// ```
    fn pattern_note<'v>(
        row: i32,
        note: &str,
        inst: i32,
        #[starlark(default = NoneType)] channel: Value<'v>,
        #[starlark(default = NoneType)] vol: Value<'v>,
        #[starlark(default = NoneType)] effect: Value<'v>,
        #[starlark(default = NoneType)] param: Value<'v>,
        #[starlark(default = NoneType)] effect_name: Value<'v>,
        #[starlark(default = NoneType)] effect_xy: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if row < 0 {
            return Err(anyhow::anyhow!(
                "S103: pattern_note(): 'row' must be >= 0, got {}",
                row
            ));
        }
        if inst < 0 {
            return Err(anyhow::anyhow!(
                "S103: pattern_note(): 'inst' must be >= 0, got {}",
                inst
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "row"), heap.alloc(row).to_value());
        dict.insert_hashed(hashed_key(heap, "note"), heap.alloc_str(note).to_value());
        dict.insert_hashed(hashed_key(heap, "inst"), heap.alloc(inst).to_value());

        // Optional: channel
        if !channel.is_none() {
            if let Some(ch) = channel.unpack_i32() {
                dict.insert_hashed(hashed_key(heap, "channel"), heap.alloc(ch).to_value());
            }
        }

        // Optional: vol
        if !vol.is_none() {
            if let Some(v) = vol.unpack_i32() {
                if !(0..=64).contains(&v) {
                    return Err(anyhow::anyhow!(
                        "S103: pattern_note(): 'vol' must be 0-64, got {}",
                        v
                    ));
                }
                dict.insert_hashed(hashed_key(heap, "vol"), heap.alloc(v).to_value());
            }
        }

        // Optional: effect
        if !effect.is_none() {
            if let Some(e) = effect.unpack_i32() {
                dict.insert_hashed(hashed_key(heap, "effect"), heap.alloc(e).to_value());
            }
        }

        // Optional: param
        if !param.is_none() {
            if let Some(p) = param.unpack_i32() {
                dict.insert_hashed(hashed_key(heap, "param"), heap.alloc(p).to_value());
            }
        }

        // Optional: effect_name
        if !effect_name.is_none() {
            if let Some(name_str) = effect_name.unpack_str() {
                dict.insert_hashed(
                    hashed_key(heap, "effect_name"),
                    heap.alloc_str(name_str).to_value(),
                );
            }
        }

        // Optional: effect_xy
        if !effect_xy.is_none() {
            dict.insert_hashed(hashed_key(heap, "effect_xy"), effect_xy);
        }

        Ok(dict)
    }

    /// Creates a tracker pattern definition.
    ///
    /// # Arguments
    /// * `rows` - Number of rows in the pattern (typically 64)
    /// * `notes` - Dict of channel -> list of notes (channel-keyed format)
    /// * `data` - List of notes with channel field (flat array format)
    ///
    /// # Returns
    /// A dict matching the TrackerPattern IR structure.
    ///
    /// # Example
    /// ```starlark
    /// # Channel-keyed format (preferred)
    /// tracker_pattern(64, notes = {
    ///     "0": [pattern_note(0, "C4", 0), pattern_note(16, "E4", 0)],
    ///     "1": [pattern_note(0, "G4", 1)]
    /// })
    ///
    /// # Flat array format
    /// tracker_pattern(64, data = [
    ///     pattern_note(0, "C4", 0, channel = 0),
    ///     pattern_note(0, "G4", 1, channel = 1)
    /// ])
    /// ```
    fn tracker_pattern<'v>(
        rows: i32,
        #[starlark(default = NoneType)] notes: Value<'v>,
        #[starlark(default = NoneType)] data: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive_int(rows as i64, "tracker_pattern", "rows")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "rows"), heap.alloc(rows).to_value());

        // Optional: notes (channel-keyed format)
        if !notes.is_none() {
            dict.insert_hashed(hashed_key(heap, "notes"), notes);
        }

        // Optional: data (flat array format)
        if !data.is_none() {
            dict.insert_hashed(hashed_key(heap, "data"), data);
        }

        Ok(dict)
    }

    /// Creates an arrangement entry.
    ///
    /// # Arguments
    /// * `pattern` - Pattern name to play
    /// * `repeat` - Number of times to repeat (default: 1)
    ///
    /// # Returns
    /// A dict matching the ArrangementEntry IR structure.
    ///
    /// # Example
    /// ```starlark
    /// arrangement_entry("intro")
    /// arrangement_entry("verse", 4)
    /// arrangement_entry("chorus", repeat = 2)
    /// ```
    fn arrangement_entry<'v>(
        pattern: &str,
        #[starlark(default = 1)] repeat: i32,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(pattern, "arrangement_entry", "pattern")
            .map_err(|e| anyhow::anyhow!(e))?;

        if repeat < 1 {
            return Err(anyhow::anyhow!(
                "S103: arrangement_entry(): 'repeat' must be >= 1, got {}",
                repeat
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "pattern"),
            heap.alloc_str(pattern).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "repeat"), heap.alloc(repeat).to_value());

        Ok(dict)
    }

    /// Creates a volume fade automation entry.
    ///
    /// # Arguments
    /// * `pattern` - Target pattern name
    /// * `channel` - Target channel (0-indexed)
    /// * `start_row` - Start row
    /// * `end_row` - End row
    /// * `start_vol` - Start volume (0-64)
    /// * `end_vol` - End volume (0-64)
    ///
    /// # Returns
    /// A dict matching the AutomationEntry::VolumeFade IR structure.
    ///
    /// # Example
    /// ```starlark
    /// volume_fade("verse", 0, 0, 64, 64, 0)  # Fade out
    /// ```
    fn volume_fade<'v>(
        pattern: &str,
        channel: i32,
        start_row: i32,
        end_row: i32,
        start_vol: i32,
        end_vol: i32,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(pattern, "volume_fade", "pattern").map_err(|e| anyhow::anyhow!(e))?;

        if !(0..=64).contains(&start_vol) {
            return Err(anyhow::anyhow!(
                "S103: volume_fade(): 'start_vol' must be 0-64, got {}",
                start_vol
            ));
        }
        if !(0..=64).contains(&end_vol) {
            return Err(anyhow::anyhow!(
                "S103: volume_fade(): 'end_vol' must be 0-64, got {}",
                end_vol
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("volume_fade").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "pattern"),
            heap.alloc_str(pattern).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "channel"), heap.alloc(channel).to_value());
        dict.insert_hashed(
            hashed_key(heap, "start_row"),
            heap.alloc(start_row).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "end_row"), heap.alloc(end_row).to_value());
        dict.insert_hashed(
            hashed_key(heap, "start_vol"),
            heap.alloc(start_vol).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "end_vol"), heap.alloc(end_vol).to_value());

        Ok(dict)
    }

    /// Creates a tempo change automation entry.
    ///
    /// # Arguments
    /// * `pattern` - Target pattern name
    /// * `row` - Row for tempo change
    /// * `bpm` - New BPM (32-255)
    ///
    /// # Returns
    /// A dict matching the AutomationEntry::TempoChange IR structure.
    ///
    /// # Example
    /// ```starlark
    /// tempo_change("bridge", 32, 140)
    /// ```
    fn tempo_change<'v>(
        pattern: &str,
        row: i32,
        bpm: i32,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(pattern, "tempo_change", "pattern").map_err(|e| anyhow::anyhow!(e))?;

        if !(32..=255).contains(&bpm) {
            return Err(anyhow::anyhow!(
                "S103: tempo_change(): 'bpm' must be 32-255, got {}",
                bpm
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("tempo_change").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "pattern"),
            heap.alloc_str(pattern).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "row"), heap.alloc(row).to_value());
        dict.insert_hashed(hashed_key(heap, "bpm"), heap.alloc(bpm).to_value());

        Ok(dict)
    }

    /// Creates a humanize_vol transform operation.
    ///
    /// Applies per-cell volume variation for musical humanization.
    /// The volume is derived deterministically from (seed, salt, pattern_name, row, channel).
    ///
    /// # Arguments
    /// * `min_vol` - Minimum volume (0-64)
    /// * `max_vol` - Maximum volume (0-64)
    /// * `seed_salt` - Salt for deterministic randomization
    ///
    /// # Returns
    /// A dict matching the TransformOp::HumanizeVol IR structure.
    ///
    /// # Example
    /// ```starlark
    /// humanize_vol(40, 60, "velocity")
    /// ```
    fn humanize_vol<'v>(
        min_vol: i32,
        max_vol: i32,
        seed_salt: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(0..=64).contains(&min_vol) {
            return Err(anyhow::anyhow!(
                "S103: humanize_vol(): 'min_vol' must be 0-64, got {}",
                min_vol
            ));
        }
        if !(0..=64).contains(&max_vol) {
            return Err(anyhow::anyhow!(
                "S103: humanize_vol(): 'max_vol' must be 0-64, got {}",
                max_vol
            ));
        }
        if min_vol > max_vol {
            return Err(anyhow::anyhow!(
                "S103: humanize_vol(): 'min_vol' ({}) must be <= 'max_vol' ({})",
                min_vol,
                max_vol
            ));
        }
        validate_non_empty(seed_salt, "humanize_vol", "seed_salt")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "op"),
            heap.alloc_str("humanize_vol").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "min_vol"), heap.alloc(min_vol).to_value());
        dict.insert_hashed(hashed_key(heap, "max_vol"), heap.alloc(max_vol).to_value());
        dict.insert_hashed(
            hashed_key(heap, "seed_salt"),
            heap.alloc_str(seed_salt).to_value(),
        );

        Ok(dict)
    }

    /// Creates a swing transform operation.
    ///
    /// Applies offbeat timing offset (swing feel) via note delay effects.
    /// Delay is applied to offbeat positions (row % stride != 0).
    ///
    /// # Arguments
    /// * `amount_permille` - Delay amount in permille of a row (0-1000)
    /// * `stride` - Stride for determining offbeat positions (must be > 0)
    /// * `seed_salt` - Salt for deterministic randomization
    ///
    /// # Returns
    /// A dict matching the TransformOp::Swing IR structure.
    ///
    /// # Example
    /// ```starlark
    /// swing(500, 2, "groove")  # Apply 50% swing with stride 2
    /// ```
    fn swing<'v>(
        amount_permille: i32,
        stride: i32,
        seed_salt: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(0..=1000).contains(&amount_permille) {
            return Err(anyhow::anyhow!(
                "S103: swing(): 'amount_permille' must be 0-1000, got {}",
                amount_permille
            ));
        }
        if stride <= 0 {
            return Err(anyhow::anyhow!(
                "S103: swing(): 'stride' must be > 0, got {}",
                stride
            ));
        }
        validate_non_empty(seed_salt, "swing", "seed_salt").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "op"), heap.alloc_str("swing").to_value());
        dict.insert_hashed(
            hashed_key(heap, "amount_permille"),
            heap.alloc(amount_permille).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "stride"), heap.alloc(stride).to_value());
        dict.insert_hashed(
            hashed_key(heap, "seed_salt"),
            heap.alloc_str(seed_salt).to_value(),
        );

        Ok(dict)
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::stdlib::tests::eval_to_json;

    #[test]
    fn test_pattern_note_minimal() {
        let result = eval_to_json("pattern_note(0, \"C4\", 0)").unwrap();
        assert_eq!(result["row"], 0);
        assert_eq!(result["note"], "C4");
        assert_eq!(result["inst"], 0);
    }

    #[test]
    fn test_pattern_note_with_volume() {
        let result = eval_to_json("pattern_note(4, \"E4\", 0, vol = 48)").unwrap();
        assert_eq!(result["vol"], 48);
    }

    #[test]
    fn test_pattern_note_negative_row() {
        let result = eval_to_json("pattern_note(-1, \"C4\", 0)");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    #[test]
    fn test_tracker_pattern_basic() {
        let result = eval_to_json("tracker_pattern(64)").unwrap();
        assert_eq!(result["rows"], 64);
    }

    #[test]
    fn test_tracker_pattern_with_notes() {
        let result = eval_to_json(
            r#"
tracker_pattern(64, notes = {
    "0": [pattern_note(0, "C4", 0)]
})
"#,
        )
        .unwrap();
        assert_eq!(result["rows"], 64);
        assert!(result["notes"].is_object());
    }

    #[test]
    fn test_arrangement_entry_basic() {
        let result = eval_to_json("arrangement_entry(\"intro\")").unwrap();
        assert_eq!(result["pattern"], "intro");
        assert_eq!(result["repeat"], 1);
    }

    #[test]
    fn test_arrangement_entry_with_repeat() {
        let result = eval_to_json("arrangement_entry(\"verse\", 4)").unwrap();
        assert_eq!(result["repeat"], 4);
    }

    #[test]
    fn test_volume_fade() {
        let result = eval_to_json("volume_fade(\"verse\", 0, 0, 64, 64, 0)").unwrap();
        assert_eq!(result["type"], "volume_fade");
        assert_eq!(result["pattern"], "verse");
        assert_eq!(result["start_vol"], 64);
        assert_eq!(result["end_vol"], 0);
    }

    #[test]
    fn test_tempo_change() {
        let result = eval_to_json("tempo_change(\"bridge\", 32, 140)").unwrap();
        assert_eq!(result["type"], "tempo_change");
        assert_eq!(result["bpm"], 140);
    }

    #[test]
    fn test_humanize_vol_basic() {
        let result = eval_to_json("humanize_vol(40, 60, \"velocity\")").unwrap();
        assert_eq!(result["op"], "humanize_vol");
        assert_eq!(result["min_vol"], 40);
        assert_eq!(result["max_vol"], 60);
        assert_eq!(result["seed_salt"], "velocity");
    }

    #[test]
    fn test_humanize_vol_invalid_min() {
        let result = eval_to_json("humanize_vol(-1, 60, \"velocity\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    #[test]
    fn test_humanize_vol_invalid_max() {
        let result = eval_to_json("humanize_vol(40, 65, \"velocity\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    #[test]
    fn test_humanize_vol_invalid_range() {
        let result = eval_to_json("humanize_vol(60, 40, \"velocity\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    #[test]
    fn test_humanize_vol_empty_salt() {
        let result = eval_to_json("humanize_vol(40, 60, \"\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S101"));
    }

    #[test]
    fn test_swing_basic() {
        let result = eval_to_json("swing(500, 2, \"groove\")").unwrap();
        assert_eq!(result["op"], "swing");
        assert_eq!(result["amount_permille"], 500);
        assert_eq!(result["stride"], 2);
        assert_eq!(result["seed_salt"], "groove");
    }

    #[test]
    fn test_swing_invalid_amount() {
        let result = eval_to_json("swing(1001, 2, \"groove\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    #[test]
    fn test_swing_invalid_stride() {
        let result = eval_to_json("swing(500, 0, \"groove\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    #[test]
    fn test_swing_negative_stride() {
        let result = eval_to_json("swing(500, -1, \"groove\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    #[test]
    fn test_swing_empty_salt() {
        let result = eval_to_json("swing(500, 2, \"\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S101"));
    }
}
