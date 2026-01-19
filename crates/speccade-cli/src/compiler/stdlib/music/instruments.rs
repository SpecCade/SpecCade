//! Instrument-related functions for tracker module creation.

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{dict::Dict, none::NoneType, Heap, Value, ValueLike};

use crate::compiler::stdlib::validation::{validate_enum, validate_non_empty};

use super::util::{hashed_key, new_dict};

/// Valid synthesis types for legacy synthesis.
const SYNTHESIS_TYPES: &[&str] = &["pulse", "square", "triangle", "sawtooth", "sine", "noise"];

/// Valid loop modes.
const LOOP_MODES: &[&str] = &["auto", "none", "forward", "pingpong"];

/// Registers instrument functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_instrument_functions(builder);
}

#[starlark_module]
fn register_instrument_functions(builder: &mut GlobalsBuilder) {
    /// Creates a tracker instrument synthesis configuration.
    ///
    /// # Arguments
    /// * `synth_type` - Synthesis type: "pulse", "square", "triangle", "sawtooth", "sine", "noise"
    /// * `duty_cycle` - Duty cycle for pulse wave (0.0-1.0, default: 0.5)
    /// * `periodic` - For noise: use periodic noise for more tonal sound (default: false)
    ///
    /// # Returns
    /// A dict matching the InstrumentSynthesis IR structure.
    ///
    /// # Example
    /// ```starlark
    /// instrument_synthesis("pulse", 0.25)
    /// instrument_synthesis("square")
    /// instrument_synthesis("noise", periodic = True)
    /// ```
    fn instrument_synthesis<'v>(
        synth_type: &str,
        #[starlark(default = 0.5)] duty_cycle: f64,
        #[starlark(default = false)] periodic: bool,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_enum(
            synth_type,
            SYNTHESIS_TYPES,
            "instrument_synthesis",
            "synth_type",
        )
        .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str(synth_type).to_value(),
        );

        // Add type-specific fields
        match synth_type {
            "pulse" => {
                dict.insert_hashed(
                    hashed_key(heap, "duty_cycle"),
                    heap.alloc(duty_cycle).to_value(),
                );
            }
            "noise" => {
                dict.insert_hashed(
                    hashed_key(heap, "periodic"),
                    heap.alloc(periodic).to_value(),
                );
            }
            _ => {}
        }

        Ok(dict)
    }

    /// Creates a tracker instrument definition.
    ///
    /// # Arguments
    /// * `name` - Instrument name (required)
    /// * `synthesis` - Synthesis config from `instrument_synthesis()` (mutually exclusive with `wav`, `ref`)
    /// * `wav` - Path to WAV sample file (mutually exclusive with `synthesis`, `ref`)
    /// * `r#ref` - Reference to external spec file (mutually exclusive with `synthesis`, `wav`)
    /// * `base_note` - Base note for pitch correction (e.g., "C4", "A#3")
    /// * `sample_rate` - Sample rate for synthesized instruments (default: 22050)
    /// * `envelope` - ADSR envelope dict (from `envelope()`)
    /// * `loop_mode` - Sample loop mode: "auto", "none", "forward", "pingpong"
    /// * `default_volume` - Default volume (0-64)
    /// * `comment` - Optional comment for documentation
    ///
    /// # Returns
    /// A dict matching the TrackerInstrument IR structure.
    ///
    /// # Example
    /// ```starlark
    /// tracker_instrument(
    ///     name = "bass",
    ///     synthesis = instrument_synthesis("sawtooth"),
    ///     envelope = envelope(0.01, 0.1, 0.7, 0.2)
    /// )
    /// tracker_instrument(name = "sample", wav = "samples/kick.wav")
    /// ```
    fn tracker_instrument<'v>(
        #[starlark(require = named)] name: &str,
        #[starlark(default = NoneType)] synthesis: Value<'v>,
        #[starlark(default = NoneType)] wav: Value<'v>,
        #[starlark(default = NoneType)] r#ref: Value<'v>,
        #[starlark(default = NoneType)] base_note: Value<'v>,
        #[starlark(default = NoneType)] sample_rate: Value<'v>,
        #[starlark(default = NoneType)] envelope: Value<'v>,
        #[starlark(default = NoneType)] loop_mode: Value<'v>,
        #[starlark(default = NoneType)] default_volume: Value<'v>,
        #[starlark(default = NoneType)] comment: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_non_empty(name, "tracker_instrument", "name").map_err(|e| anyhow::anyhow!(e))?;

        // Validate mutual exclusivity
        let has_synthesis = !synthesis.is_none();
        let has_wav = !wav.is_none();
        let has_ref = !r#ref.is_none();
        let source_count = [has_synthesis, has_wav, has_ref]
            .iter()
            .filter(|&&x| x)
            .count();

        if source_count > 1 {
            return Err(anyhow::anyhow!(
                "S101: tracker_instrument(): 'synthesis', 'wav', and 'ref' are mutually exclusive"
            ));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "name"), heap.alloc_str(name).to_value());

        // Add source (synthesis, wav, or ref)
        if has_synthesis {
            dict.insert_hashed(hashed_key(heap, "synthesis"), synthesis);
        } else if has_wav {
            if let Some(wav_str) = wav.unpack_str() {
                dict.insert_hashed(hashed_key(heap, "wav"), heap.alloc_str(wav_str).to_value());
            }
        } else if has_ref {
            if let Some(ref_str) = r#ref.unpack_str() {
                dict.insert_hashed(hashed_key(heap, "ref"), heap.alloc_str(ref_str).to_value());
            }
        }

        // Optional: base_note
        if !base_note.is_none() {
            if let Some(note_str) = base_note.unpack_str() {
                dict.insert_hashed(
                    hashed_key(heap, "base_note"),
                    heap.alloc_str(note_str).to_value(),
                );
            }
        }

        // Optional: sample_rate
        if !sample_rate.is_none() {
            if let Some(rate) = sample_rate.unpack_i32() {
                dict.insert_hashed(hashed_key(heap, "sample_rate"), heap.alloc(rate).to_value());
            }
        }

        // Envelope - use default if not provided
        if envelope.is_none() {
            let mut env_dict = new_dict(heap);
            env_dict.insert_hashed(hashed_key(heap, "attack"), heap.alloc(0.01).to_value());
            env_dict.insert_hashed(hashed_key(heap, "decay"), heap.alloc(0.1).to_value());
            env_dict.insert_hashed(hashed_key(heap, "sustain"), heap.alloc(0.7).to_value());
            env_dict.insert_hashed(hashed_key(heap, "release"), heap.alloc(0.2).to_value());
            dict.insert_hashed(
                hashed_key(heap, "envelope"),
                heap.alloc(env_dict).to_value(),
            );
        } else {
            dict.insert_hashed(hashed_key(heap, "envelope"), envelope);
        }

        // Optional: loop_mode
        if !loop_mode.is_none() {
            if let Some(mode_str) = loop_mode.unpack_str() {
                validate_enum(mode_str, LOOP_MODES, "tracker_instrument", "loop_mode")
                    .map_err(|e| anyhow::anyhow!(e))?;
                dict.insert_hashed(
                    hashed_key(heap, "loop_mode"),
                    heap.alloc_str(mode_str).to_value(),
                );
            }
        }

        // Optional: default_volume
        if !default_volume.is_none() {
            if let Some(vol) = default_volume.unpack_i32() {
                if !(0..=64).contains(&vol) {
                    return Err(anyhow::anyhow!(
                        "S103: tracker_instrument(): 'default_volume' must be 0-64, got {}",
                        vol
                    ));
                }
                dict.insert_hashed(
                    hashed_key(heap, "default_volume"),
                    heap.alloc(vol).to_value(),
                );
            }
        }

        // Optional: comment
        if !comment.is_none() {
            if let Some(comment_str) = comment.unpack_str() {
                dict.insert_hashed(
                    hashed_key(heap, "comment"),
                    heap.alloc_str(comment_str).to_value(),
                );
            }
        }

        Ok(dict)
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::stdlib::tests::eval_to_json;

    #[test]
    fn test_instrument_synthesis_pulse() {
        let result = eval_to_json("instrument_synthesis(\"pulse\", 0.25)").unwrap();
        assert_eq!(result["type"], "pulse");
        assert_eq!(result["duty_cycle"], 0.25);
    }

    #[test]
    fn test_instrument_synthesis_square() {
        let result = eval_to_json("instrument_synthesis(\"square\")").unwrap();
        assert_eq!(result["type"], "square");
    }

    #[test]
    fn test_instrument_synthesis_noise() {
        let result = eval_to_json("instrument_synthesis(\"noise\", periodic = True)").unwrap();
        assert_eq!(result["type"], "noise");
        assert_eq!(result["periodic"], true);
    }

    #[test]
    fn test_instrument_synthesis_invalid_type() {
        let result = eval_to_json("instrument_synthesis(\"invalid\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
    }

    #[test]
    fn test_tracker_instrument_minimal() {
        let result = eval_to_json("tracker_instrument(name = \"bass\")").unwrap();
        assert_eq!(result["name"], "bass");
        assert!(result["envelope"].is_object());
    }

    #[test]
    fn test_tracker_instrument_with_synthesis() {
        let result = eval_to_json(
            r#"
tracker_instrument(
    name = "lead",
    synthesis = instrument_synthesis("sawtooth")
)
"#,
        )
        .unwrap();
        assert_eq!(result["name"], "lead");
        assert_eq!(result["synthesis"]["type"], "sawtooth");
    }

    #[test]
    fn test_tracker_instrument_with_wav() {
        let result = eval_to_json(
            r#"
tracker_instrument(name = "kick", wav = "samples/kick.wav")
"#,
        )
        .unwrap();
        assert_eq!(result["wav"], "samples/kick.wav");
    }

    #[test]
    fn test_tracker_instrument_mutual_exclusivity() {
        let result = eval_to_json(
            r#"
tracker_instrument(
    name = "bad",
    synthesis = instrument_synthesis("sine"),
    wav = "samples/test.wav"
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("mutually exclusive"));
    }
}
