//! Modulation functions (envelopes and LFOs)

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{dict::Dict, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::{validate_enum, validate_positive, validate_unit_range};

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

/// Helper function to extract a float from a Starlark Value.
fn extract_float(value: Value, function: &str, param: &str) -> anyhow::Result<f64> {
    if let Some(f) = value.unpack_i32() {
        return Ok(f as f64);
    }
    if value.get_type() == "float" {
        if let Ok(f) = value.to_str().parse::<f64>() {
            return Ok(f);
        }
    }
    Err(anyhow::anyhow!(
        "S102: {}(): '{}' expected float, got {}",
        function, param, value.get_type()
    ))
}

/// Valid waveform types.
const WAVEFORMS: &[&str] = &["sine", "square", "sawtooth", "triangle", "pulse"];

/// Registers modulation functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_modulation_functions(builder);
}

#[starlark_module]
fn register_modulation_functions(builder: &mut GlobalsBuilder) {
    fn envelope<'v>(
        #[starlark(default = 0.01)] attack: f64,
        #[starlark(default = 0.1)] decay: f64,
        #[starlark(default = 0.5)] sustain: f64,
        #[starlark(default = 0.2)] release: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate ranges
        validate_positive(attack, "envelope", "attack")
            .or_else(|_| if attack == 0.0 { Ok(()) } else { Err("".to_string()) })
            .map_err(|_| anyhow::anyhow!("S103: envelope(): 'attack' must be >= 0, got {}", attack))?;
        validate_positive(decay, "envelope", "decay")
            .or_else(|_| if decay == 0.0 { Ok(()) } else { Err("".to_string()) })
            .map_err(|_| anyhow::anyhow!("S103: envelope(): 'decay' must be >= 0, got {}", decay))?;
        validate_unit_range(sustain, "envelope", "sustain")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(release, "envelope", "release")
            .or_else(|_| if release == 0.0 { Ok(()) } else { Err("".to_string()) })
            .map_err(|_| anyhow::anyhow!("S103: envelope(): 'release' must be >= 0, got {}", release))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "attack"),
            heap.alloc(attack).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "decay"),
            heap.alloc(decay).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "sustain"),
            heap.alloc(sustain).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "release"),
            heap.alloc(release).to_value(),
        );

        Ok(dict)
    }

    /// Creates an oscillator synthesis block.
    ///
    /// # Arguments
    /// * `frequency` - Frequency in Hz (must be positive)
    /// * `waveform` - Waveform type: "sine", "square", "sawtooth", "triangle", "pulse"
    /// * `sweep_to` - Optional target frequency for sweep
    /// * `curve` - Sweep curve: "linear" or "exponential"
    /// * `detune` - Optional detune in cents (100 cents = 1 semitone)
    /// * `duty` - Optional duty cycle for pulse waves (0.0-1.0, default 0.5)
    ///
    /// # Returns
    /// A dict matching the Synthesis::Oscillator IR structure.
    ///
    /// # Example
    /// ```starlark
    /// oscillator(440)  # 440 Hz sine wave
    /// oscillator(880, "sawtooth")
    /// oscillator(440, "sine", 220, "exponential")  # Sweep from 440 to 220 Hz
    /// oscillator(440, "pulse", duty = 0.25)  # Pulse wave with 25% duty cycle
    /// oscillator(440, "sine", detune = 5.0)  # Slightly detuned
    /// ```
    fn lfo<'v>(
        waveform: &str,
        rate: f64,
        depth: f64,
        #[starlark(default = NoneType)] phase: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_enum(waveform, WAVEFORMS, "lfo", "waveform")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(rate, "lfo", "rate")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(depth, "lfo", "depth")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "waveform"),
            heap.alloc_str(waveform).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "rate"),
            heap.alloc(rate).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "depth"),
            heap.alloc(depth).to_value(),
        );

        // Add phase if provided
        if !phase.is_none() {
            let phase_val = extract_float(phase, "lfo", "phase")?;
            validate_unit_range(phase_val, "lfo", "phase")
                .map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(
                hashed_key(heap, "phase"),
                heap.alloc(phase_val).to_value(),
            );
        }

        Ok(dict)
    }

    /// Creates an LFO modulation with a target.
    ///
    /// # Arguments
    /// * `config` - LFO configuration from lfo()
    /// * `target` - Modulation target: "pitch", "volume", "filter_cutoff", "pan", etc.
    /// * `amount` - Target-specific modulation amount
    ///
    /// # Returns
    /// A dict matching the LfoModulation IR structure.
    ///
    /// # Example
    /// ```starlark
    /// lfo_modulation(lfo("sine", 5.0, 0.5), "pitch", 2.0)  # 2 semitone vibrato
    /// lfo_modulation(lfo("triangle", 4.0, 0.3), "volume", 0.5)  # Tremolo
    /// lfo_modulation(lfo("sine", 2.0, 0.7), "filter_cutoff", 2000)  # Filter wobble
    /// ```
    fn lfo_modulation<'v>(
        config: Value<'v>,
        target: &str,
        amount: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        const TARGETS: &[&str] = &[
            "pitch", "volume", "filter_cutoff", "pan", "pulse_width",
            "fm_index", "grain_size", "grain_density", "delay_time",
            "reverb_size", "distortion_drive"
        ];
        validate_enum(target, TARGETS, "lfo_modulation", "target")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "config"),
            config,
        );

        // Build target dict based on target type
        let mut target_dict = new_dict(heap);
        target_dict.insert_hashed(
            hashed_key(heap, "target"),
            heap.alloc_str(target).to_value(),
        );

        // Different targets use different amount field names
        let amount_key = match target {
            "pitch" => "semitones",
            "grain_size" | "delay_time" => "amount_ms",
            _ => "amount",
        };
        target_dict.insert_hashed(
            hashed_key(heap, amount_key),
            heap.alloc(amount).to_value(),
        );

        dict.insert_hashed(
            hashed_key(heap, "target"),
            heap.alloc(target_dict).to_value(),
        );

        Ok(dict)
    }

    // ========================================================================
    // Additional Synthesis Types
    // ========================================================================

    /// Creates an AM (Amplitude Modulation) synthesis block.
    ///
    /// # Arguments
    /// * `carrier` - Carrier frequency in Hz
    /// * `modulator` - Modulator frequency in Hz
    /// * `depth` - Modulation depth 0.0-1.0
    /// * `sweep_to` - Optional target carrier frequency for sweep
    ///
    /// # Returns
    /// A dict matching the Synthesis::AmSynth IR structure.
    ///
    /// # Example
    /// ```starlark
    /// am_synth(440, 110, 0.5)
    /// am_synth(440, 110, 0.5, 220)  # Sweep carrier to 220 Hz
    /// ```
    fn pitch_envelope<'v>(
        #[starlark(default = 0.01)] attack: f64,
        #[starlark(default = 0.1)] decay: f64,
        #[starlark(default = 0.5)] sustain: f64,
        #[starlark(default = 0.2)] release: f64,
        #[starlark(default = 0.0)] depth: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(attack, "pitch_envelope", "attack")
            .or_else(|_| if attack == 0.0 { Ok(()) } else { Err("".to_string()) })
            .map_err(|_| anyhow::anyhow!("S103: pitch_envelope(): 'attack' must be >= 0"))?;
        validate_positive(decay, "pitch_envelope", "decay")
            .or_else(|_| if decay == 0.0 { Ok(()) } else { Err("".to_string()) })
            .map_err(|_| anyhow::anyhow!("S103: pitch_envelope(): 'decay' must be >= 0"))?;
        validate_unit_range(sustain, "pitch_envelope", "sustain")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(release, "pitch_envelope", "release")
            .or_else(|_| if release == 0.0 { Ok(()) } else { Err("".to_string()) })
            .map_err(|_| anyhow::anyhow!("S103: pitch_envelope(): 'release' must be >= 0"))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "attack"),
            heap.alloc(attack).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "decay"),
            heap.alloc(decay).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "sustain"),
            heap.alloc(sustain).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "release"),
            heap.alloc(release).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "depth"),
            heap.alloc(depth).to_value(),
        );

        Ok(dict)
    }
}
