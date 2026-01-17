//! Basic synthesis functions for audio generation

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

/// Valid sweep curves.
const SWEEP_CURVES: &[&str] = &["linear", "exponential", "logarithmic"];

/// Registers basic synthesis functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_basic_synthesis(builder);
}

#[starlark_module]
fn register_basic_synthesis(builder: &mut GlobalsBuilder) {
    fn oscillator<'v>(
        frequency: f64,
        #[starlark(default = "sine")] waveform: &str,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        #[starlark(default = "linear")] curve: &str,
        #[starlark(default = NoneType)] detune: Value<'v>,
        #[starlark(default = NoneType)] duty: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "oscillator", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(waveform, WAVEFORMS, "oscillator", "waveform")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(curve, SWEEP_CURVES, "oscillator", "curve")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("oscillator").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "waveform"),
            heap.alloc_str(waveform).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );

        // Add freq_sweep if sweep_to is provided
        if !sweep_to.is_none() {
            let end_freq = extract_float(sweep_to, "oscillator", "sweep_to")?;
            validate_positive(end_freq, "oscillator", "sweep_to")
                .map_err(|e| anyhow::anyhow!(e))?;

            let mut sweep_dict = new_dict(heap);
            sweep_dict.insert_hashed(
                hashed_key(heap, "end_freq"),
                heap.alloc(end_freq).to_value(),
            );
            sweep_dict.insert_hashed(
                hashed_key(heap, "curve"),
                heap.alloc_str(curve).to_value(),
            );
            dict.insert_hashed(
                hashed_key(heap, "freq_sweep"),
                heap.alloc(sweep_dict).to_value(),
            );
        }

        // Add detune if provided
        if !detune.is_none() {
            let detune_val = extract_float(detune, "oscillator", "detune")?;
            dict.insert_hashed(
                hashed_key(heap, "detune"),
                heap.alloc(detune_val).to_value(),
            );
        }

        // Add duty if provided
        if !duty.is_none() {
            let duty_val = extract_float(duty, "oscillator", "duty")?;
            validate_unit_range(duty_val, "oscillator", "duty")
                .map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(
                hashed_key(heap, "duty"),
                heap.alloc(duty_val).to_value(),
            );
        }

        Ok(dict)
    }

    /// Creates an FM synthesis block.
    ///
    /// # Arguments
    /// * `carrier` - Carrier frequency in Hz
    /// * `modulator` - Modulator frequency in Hz
    /// * `index` - Modulation index
    /// * `sweep_to` - Optional target carrier frequency for sweep
    ///
    /// # Returns
    /// A dict matching the Synthesis::FmSynth IR structure.
    ///
    /// # Example
    /// ```starlark
    /// fm_synth(440, 880, 5.0)
    /// fm_synth(440, 880, 5.0, 220)  # Sweep carrier to 220 Hz
    /// ```
    fn fm_synth<'v>(
        carrier: f64,
        modulator: f64,
        index: f64,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(carrier, "fm_synth", "carrier")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(modulator, "fm_synth", "modulator")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(index, "fm_synth", "index")
            .or_else(|_| if index == 0.0 { Ok(()) } else { Err("".to_string()) })
            .map_err(|_| anyhow::anyhow!("S103: fm_synth(): 'index' must be >= 0, got {}", index))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("fm_synth").to_value(),
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
            hashed_key(heap, "modulation_index"),
            heap.alloc(index).to_value(),
        );

        // Add freq_sweep if sweep_to is provided
        if !sweep_to.is_none() {
            let end_freq = extract_float(sweep_to, "fm_synth", "sweep_to")?;
            validate_positive(end_freq, "fm_synth", "sweep_to")
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

    /// Creates an AM synthesis block.
    ///
    /// # Arguments
    /// * `carrier` - Carrier frequency in Hz
    /// * `modulator` - Modulator frequency in Hz
    /// * `depth` - Modulation depth 0.0-1.0
    /// * `sweep_to` - Optional target carrier frequency for sweep
    ///
    /// # Returns
    /// A dict matching the Synthesis::AmSynth IR structure.
    fn am_synth<'v>(
        carrier: f64,
        modulator: f64,
        depth: f64,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(carrier, "am_synth", "carrier")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(modulator, "am_synth", "modulator")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(depth, "am_synth", "depth")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("am_synth").to_value(),
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
            hashed_key(heap, "modulation_depth"),
            heap.alloc(depth).to_value(),
        );

        // Add freq_sweep if sweep_to is provided
        if !sweep_to.is_none() {
            let end_freq = extract_float(sweep_to, "am_synth", "sweep_to")?;
            validate_positive(end_freq, "am_synth", "sweep_to")
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

    /// Creates a noise burst synthesis block.
    ///
    /// # Arguments
    /// * `noise_type` - Noise type: "white", "pink", or "brown" (default: "white")
    /// * `filter` - Optional filter dict from lowpass()/highpass()
    ///
    /// # Returns
    /// A dict matching the Synthesis::NoiseBurst IR structure.
    ///
    /// # Example
    /// ```starlark
    /// noise_burst()  # White noise
    /// noise_burst("pink")
    /// noise_burst("white", lowpass(5000))
    /// ```
    fn noise_burst<'v>(
        #[starlark(default = "white")] noise_type: &str,
        #[starlark(default = NoneType)] filter: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_enum(noise_type, NOISE_TYPES, "noise_burst", "noise_type")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("noise_burst").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "noise_type"),
            heap.alloc_str(noise_type).to_value(),
        );

        // Add filter if provided
        if !filter.is_none() {
            dict.insert_hashed(
                hashed_key(heap, "filter"),
                filter,
            );
        }

        Ok(dict)
    }

    /// Creates a Karplus-Strong plucked string synthesis.
    ///
    /// # Arguments
    /// * `frequency` - Fundamental frequency in Hz
    /// * `damping` - Damping factor 0.0-1.0
    /// * `blend` - Blend factor for averaging 0.0-1.0
    ///
    /// # Returns
    /// A dict matching the Synthesis::KarplusStrong IR structure.
    ///
    /// # Example
    /// ```starlark
    /// karplus_strong(440, 0.5, 0.5)  # A4 with medium damping
    /// ```
    fn karplus_strong<'v>(
        frequency: f64,
        damping: f64,
        blend: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "karplus_strong", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(damping, "karplus_strong", "damping")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(blend, "karplus_strong", "blend")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("karplus_strong").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "damping"),
            heap.alloc(damping).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "blend"),
            heap.alloc(blend).to_value(),
        );

        Ok(dict)
    }
}
