//! Complex synthesis functions for audio generation

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{dict::Dict, none::NoneType, Heap, Value, ValueLike};

use super::super::super::validation::{
    validate_enum, validate_positive, validate_unit_range,
};

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

/// Valid noise types.
const NOISE_TYPES: &[&str] = &["white", "pink", "brown"];

/// Registers complex synthesis functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_complex_synthesis(builder);
}

#[starlark_module]
fn register_complex_synthesis(builder: &mut GlobalsBuilder) {
    /// Creates an additive synthesis block with harmonics.
    ///
    /// # Arguments
    /// * `base_freq` - Base frequency in Hz
    /// * `harmonics` - List of harmonic amplitudes (index 0 = fundamental)
    ///
    /// # Returns
    /// A dict matching the Synthesis::Additive IR structure.
    ///
    /// # Example
    /// ```starlark
    /// additive(440, [1.0, 0.5, 0.25, 0.125])  # Fundamental + 3 harmonics
    /// additive(220, [1.0, 0.0, 0.33, 0.0, 0.2])  # Odd harmonics only
    /// ```
    fn additive<'v>(
        base_freq: f64,
        harmonics: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(base_freq, "additive", "base_freq")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Extract harmonics list
        let harmonics_list = harmonics.iterate(heap)
            .map_err(|_| anyhow::anyhow!("S102: additive(): 'harmonics' must be a list"))?;

        let mut harmonic_values: Vec<f64> = Vec::new();
        for item in harmonics_list {
            let val = extract_float(item, "additive", "harmonics item")?;
            harmonic_values.push(val);
        }

        if harmonic_values.is_empty() {
            return Err(anyhow::anyhow!("S103: additive(): 'harmonics' must not be empty"));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("additive").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "base_freq"),
            heap.alloc(base_freq).to_value(),
        );

        // Build harmonics list
        let harmonics_alloc: Vec<Value<'v>> = harmonic_values
            .iter()
            .map(|&h| heap.alloc(h).to_value())
            .collect();
        dict.insert_hashed(
            hashed_key(heap, "harmonics"),
            heap.alloc(harmonics_alloc).to_value(),
        );

        Ok(dict)
    }

    /// Creates a Supersaw/Unison synthesis block.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `voices` - Number of unison voices (1-16)
    /// * `detune_cents` - Maximum detune in cents (100 = 1 semitone)
    /// * `spread` - Stereo spread 0.0-1.0
    /// * `detune_curve` - Detune distribution: "linear" or "exp2" (default: "linear")
    ///
    /// # Returns
    /// A dict matching the Synthesis::SupersawUnison IR structure.
    ///
    /// # Example
    /// ```starlark
    /// supersaw_unison(440, 7, 20, 0.8)  # Classic supersaw
    /// supersaw_unison(440, 5, 15, 1.0, "exp2")  # With exponential detune
    /// ```
    fn supersaw_unison<'v>(
        frequency: f64,
        voices: i32,
        detune_cents: f64,
        spread: f64,
        #[starlark(default = "linear")] detune_curve: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "supersaw_unison", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        if !(1..=16).contains(&voices) {
            return Err(anyhow::anyhow!("S103: supersaw_unison(): 'voices' must be 1-16, got {}", voices));
        }
        validate_positive(detune_cents, "supersaw_unison", "detune_cents")
            .or_else(|_| if detune_cents == 0.0 { Ok(()) } else { Err("".to_string()) })
            .map_err(|_| anyhow::anyhow!("S103: supersaw_unison(): 'detune_cents' must be >= 0, got {}", detune_cents))?;
        validate_unit_range(spread, "supersaw_unison", "spread")
            .map_err(|e| anyhow::anyhow!(e))?;

        const DETUNE_CURVES: &[&str] = &["linear", "exp2"];
        validate_enum(detune_curve, DETUNE_CURVES, "supersaw_unison", "detune_curve")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("supersaw_unison").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "voices"),
            heap.alloc(voices).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "detune_cents"),
            heap.alloc(detune_cents).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "spread"),
            heap.alloc(spread).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "detune_curve"),
            heap.alloc_str(detune_curve).to_value(),
        );

        Ok(dict)
    }

    /// Creates a Wavetable synthesis block.
    ///
    /// # Arguments
    /// * `table` - Wavetable source: "basic", "analog", "digital", "pwm", "formant", "organ"
    /// * `frequency` - Base frequency in Hz
    /// * `position` - Position in wavetable 0.0-1.0 (default: 0.0)
    /// * `position_end` - Optional end position for sweep
    /// * `voices` - Optional number of unison voices (1-8)
    /// * `detune` - Optional detune amount in cents for unison
    ///
    /// # Returns
    /// A dict matching the Synthesis::Wavetable IR structure.
    ///
    /// # Example
    /// ```starlark
    /// wavetable("basic", 440)
    /// wavetable("analog", 440, 0.5)  # Start at middle of wavetable
    /// wavetable("digital", 440, 0.0, 1.0)  # Sweep through entire table
    /// wavetable("basic", 440, 0.0, None, 4, 10)  # 4-voice unison
    /// ```
    fn wavetable<'v>(
        table: &str,
        frequency: f64,
        #[starlark(default = 0.0)] position: f64,
        #[starlark(default = NoneType)] position_end: Value<'v>,
        #[starlark(default = NoneType)] voices: Value<'v>,
        #[starlark(default = NoneType)] detune: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        const TABLES: &[&str] = &["basic", "analog", "digital", "pwm", "formant", "organ"];
        validate_enum(table, TABLES, "wavetable", "table")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(frequency, "wavetable", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(position, "wavetable", "position")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("wavetable").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "table"),
            heap.alloc_str(table).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "position"),
            heap.alloc(position).to_value(),
        );

        // Add position_sweep if position_end is provided
        if !position_end.is_none() {
            let end_pos = extract_float(position_end, "wavetable", "position_end")?;
            validate_unit_range(end_pos, "wavetable", "position_end")
                .map_err(|e| anyhow::anyhow!(e))?;

            let mut sweep_dict = new_dict(heap);
            sweep_dict.insert_hashed(
                hashed_key(heap, "end_position"),
                heap.alloc(end_pos).to_value(),
            );
            sweep_dict.insert_hashed(
                hashed_key(heap, "curve"),
                heap.alloc_str("linear").to_value(),
            );
            dict.insert_hashed(
                hashed_key(heap, "position_sweep"),
                heap.alloc(sweep_dict).to_value(),
            );
        }

        // Add voices if provided
        if !voices.is_none() {
            let voices_val = voices.unpack_i32()
                .ok_or_else(|| anyhow::anyhow!("S102: wavetable(): 'voices' expected int"))?;
            if !(1..=8).contains(&voices_val) {
                return Err(anyhow::anyhow!("S103: wavetable(): 'voices' must be 1-8, got {}", voices_val));
            }
            dict.insert_hashed(
                hashed_key(heap, "voices"),
                heap.alloc(voices_val).to_value(),
            );
        }

        // Add detune if provided
        if !detune.is_none() {
            let detune_val = extract_float(detune, "wavetable", "detune")?;
            dict.insert_hashed(
                hashed_key(heap, "detune"),
                heap.alloc(detune_val).to_value(),
            );
        }

        Ok(dict)
    }

    /// Creates a Granular synthesis block.
    ///
    /// # Arguments
    /// * `source` - Source material dict (from granular_source())
    /// * `grain_size_ms` - Grain size in milliseconds (10-500)
    /// * `grain_density` - Grains per second (1-100)
    /// * `pitch_spread` - Random pitch variation in semitones (default: 0)
    /// * `position_spread` - Random position jitter 0.0-1.0 (default: 0)
    /// * `pan_spread` - Stereo spread 0.0-1.0 (default: 0)
    ///
    /// # Returns
    /// A dict matching the Synthesis::Granular IR structure.
    ///
    /// # Example
    /// ```starlark
    /// granular(granular_source("noise", "white"), 50, 20)
    /// granular(granular_source("tone", "sine", 440), 100, 30, 2.0, 0.5, 0.8)
    /// ```
    fn granular<'v>(
        source: Value<'v>,
        grain_size_ms: f64,
        grain_density: f64,
        #[starlark(default = 0.0)] pitch_spread: f64,
        #[starlark(default = 0.0)] position_spread: f64,
        #[starlark(default = 0.0)] pan_spread: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        if !(10.0..=500.0).contains(&grain_size_ms) {
            return Err(anyhow::anyhow!("S103: granular(): 'grain_size_ms' must be 10-500, got {}", grain_size_ms));
        }
        if !(1.0..=100.0).contains(&grain_density) {
            return Err(anyhow::anyhow!("S103: granular(): 'grain_density' must be 1-100, got {}", grain_density));
        }
        validate_unit_range(position_spread, "granular", "position_spread")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(pan_spread, "granular", "pan_spread")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("granular").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "source"),
            source,
        );
        dict.insert_hashed(
            hashed_key(heap, "grain_size_ms"),
            heap.alloc(grain_size_ms).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "grain_density"),
            heap.alloc(grain_density).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "pitch_spread"),
            heap.alloc(pitch_spread).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "position_spread"),
            heap.alloc(position_spread).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "pan_spread"),
            heap.alloc(pan_spread).to_value(),
        );

        Ok(dict)
    }

    /// Creates a granular source configuration.
    ///
    /// # Arguments
    /// * `source_type` - Source type: "noise", "tone", or "formant"
    /// * `param1` - For noise: noise_type. For tone: waveform. For formant: frequency
    /// * `param2` - For tone: frequency. For formant: formant_freq. Ignored for noise.
    ///
    /// # Returns
    /// A dict matching the GranularSource IR structure.
    ///
    /// # Example
    /// ```starlark
    /// granular_source("noise", "white")
    /// granular_source("tone", "sine", 440)
    /// granular_source("formant", 220, 880)
    /// ```
    fn granular_source<'v>(
        source_type: &str,
        param1: Value<'v>,
        #[starlark(default = NoneType)] param2: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        let mut dict = new_dict(heap);

        match source_type {
            "noise" => {
                let noise_type = param1.unpack_str()
                    .ok_or_else(|| anyhow::anyhow!("S102: granular_source(): noise type must be a string"))?;
                validate_enum(noise_type, NOISE_TYPES, "granular_source", "noise_type")
                    .map_err(|e| anyhow::anyhow!(e))?;

                dict.insert_hashed(
                    hashed_key(heap, "type"),
                    heap.alloc_str("noise").to_value(),
                );
                dict.insert_hashed(
                    hashed_key(heap, "noise_type"),
                    heap.alloc_str(noise_type).to_value(),
                );
            }
            "tone" => {
                let waveform = param1.unpack_str()
                    .ok_or_else(|| anyhow::anyhow!("S102: granular_source(): waveform must be a string"))?;
                validate_enum(waveform, WAVEFORMS, "granular_source", "waveform")
                    .map_err(|e| anyhow::anyhow!(e))?;

                let frequency = extract_float(param2, "granular_source", "frequency")?;
                validate_positive(frequency, "granular_source", "frequency")
                    .map_err(|e| anyhow::anyhow!(e))?;

                dict.insert_hashed(
                    hashed_key(heap, "type"),
                    heap.alloc_str("tone").to_value(),
                );
                dict.insert_hashed(
                    hashed_key(heap, "waveform"),
                    heap.alloc_str(waveform).to_value(),
                );
                dict.insert_hashed(
                    hashed_key(heap, "frequency"),
                    heap.alloc(frequency).to_value(),
                );
            }
            "formant" => {
                let frequency = extract_float(param1, "granular_source", "frequency")?;
                validate_positive(frequency, "granular_source", "frequency")
                    .map_err(|e| anyhow::anyhow!(e))?;

                let formant_freq = extract_float(param2, "granular_source", "formant_freq")?;
                validate_positive(formant_freq, "granular_source", "formant_freq")
                    .map_err(|e| anyhow::anyhow!(e))?;

                dict.insert_hashed(
                    hashed_key(heap, "type"),
                    heap.alloc_str("formant").to_value(),
                );
                dict.insert_hashed(
                    hashed_key(heap, "frequency"),
                    heap.alloc(frequency).to_value(),
                );
                dict.insert_hashed(
                    hashed_key(heap, "formant_freq"),
                    heap.alloc(formant_freq).to_value(),
                );
            }
            _ => {
                return Err(anyhow::anyhow!("S104: granular_source(): 'source_type' must be one of: noise, tone, formant"));
            }
        }

        Ok(dict)
    }

    /// Creates a Ring Modulation synthesis block.
    ///
    /// # Arguments
    /// * `carrier` - Carrier frequency in Hz
    /// * `modulator` - Modulator frequency in Hz
    /// * `mix` - Wet/dry mix 0.0-1.0 (0.0 = pure carrier, 1.0 = pure ring mod)
    /// * `sweep_to` - Optional target carrier frequency for sweep
    ///
    /// # Returns
    /// A dict matching the Synthesis::RingModSynth IR structure.
    fn ring_mod_synth<'v>(
        carrier: f64,
        modulator: f64,
        mix: f64,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(carrier, "ring_mod_synth", "carrier")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(modulator, "ring_mod_synth", "modulator")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(mix, "ring_mod_synth", "mix")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("ring_mod_synth").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "carrier_freq"),
            heap.alloc(carrier).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "modulator_freq"),
            heap.alloc(modulator).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "mix"),
            heap.alloc(mix).to_value(),
        );

        if !sweep_to.is_none() {
            let end_freq = extract_float(sweep_to, "ring_mod_synth", "sweep_to")?;
            validate_positive(end_freq, "ring_mod_synth", "sweep_to")
                .map_err(|e| anyhow::anyhow!(e))?;

            let mut sweep_dict = new_dict(heap);
            sweep_dict.insert_hashed(
                hashed_key(heap, "end_freq"),
                heap.alloc(end_freq).to_value(),
            );
            sweep_dict.insert_hashed(
                hashed_key(heap, "curve"),
                heap.alloc_str("linear").to_value(),
            );
            dict.insert_hashed(
                hashed_key(heap, "freq_sweep"),
                heap.alloc(sweep_dict).to_value(),
            );
        }

        Ok(dict)
    }

    /// Creates a Multi-Oscillator synthesis block.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `oscillators` - List of oscillator config dicts from oscillator_config()
    /// * `sweep_to` - Optional target frequency for sweep
    ///
    /// # Returns
    /// A dict matching the Synthesis::MultiOscillator IR structure.
    fn multi_oscillator<'v>(
        frequency: f64,
        oscillators: Value<'v>,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "multi_oscillator", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("multi_oscillator").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "oscillators"),
            oscillators,
        );

        if !sweep_to.is_none() {
            let end_freq = extract_float(sweep_to, "multi_oscillator", "sweep_to")?;
            validate_positive(end_freq, "multi_oscillator", "sweep_to")
                .map_err(|e| anyhow::anyhow!(e))?;

            let mut sweep_dict = new_dict(heap);
            sweep_dict.insert_hashed(
                hashed_key(heap, "end_freq"),
                heap.alloc(end_freq).to_value(),
            );
            sweep_dict.insert_hashed(
                hashed_key(heap, "curve"),
                heap.alloc_str("linear").to_value(),
            );
            dict.insert_hashed(
                hashed_key(heap, "freq_sweep"),
                heap.alloc(sweep_dict).to_value(),
            );
        }

        Ok(dict)
    }

    /// Creates an oscillator configuration for multi_oscillator().
    ///
    /// # Arguments
    /// * `waveform` - Waveform type
    /// * `volume` - Volume 0.0-1.0 (default: 1.0)
    /// * `detune` - Detune in cents (optional)
    /// * `phase` - Phase offset 0.0-1.0 (optional)
    /// * `duty` - Duty cycle for pulse waves 0.0-1.0 (optional)
    fn oscillator_config<'v>(
        waveform: &str,
        #[starlark(default = 1.0)] volume: f64,
        #[starlark(default = NoneType)] detune: Value<'v>,
        #[starlark(default = NoneType)] phase: Value<'v>,
        #[starlark(default = NoneType)] duty: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_enum(waveform, WAVEFORMS, "oscillator_config", "waveform")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(volume, "oscillator_config", "volume")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "waveform"),
            heap.alloc_str(waveform).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "volume"),
            heap.alloc(volume).to_value(),
        );

        if !detune.is_none() {
            let detune_val = extract_float(detune, "oscillator_config", "detune")?;
            dict.insert_hashed(
                hashed_key(heap, "detune"),
                heap.alloc(detune_val).to_value(),
            );
        }

        if !phase.is_none() {
            let phase_val = extract_float(phase, "oscillator_config", "phase")?;
            validate_unit_range(phase_val, "oscillator_config", "phase")
                .map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(
                hashed_key(heap, "phase"),
                heap.alloc(phase_val).to_value(),
            );
        }

        if !duty.is_none() {
            let duty_val = extract_float(duty, "oscillator_config", "duty")?;
            validate_unit_range(duty_val, "oscillator_config", "duty")
                .map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(
                hashed_key(heap, "duty"),
                heap.alloc(duty_val).to_value(),
            );
        }

        Ok(dict)
    }
}
