//! Filter functions for audio processing

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
        function,
        param,
        value.get_type()
    ))
}

/// Registers filters functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_filters_functions(builder);
}

#[starlark_module]
fn register_filters_functions(builder: &mut GlobalsBuilder) {
    fn lowpass<'v>(
        cutoff: f64,
        #[starlark(default = 0.707)] resonance: f64,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(cutoff, "lowpass", "cutoff").map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(resonance, "lowpass", "resonance").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("lowpass").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "cutoff"), heap.alloc(cutoff).to_value());
        dict.insert_hashed(
            hashed_key(heap, "resonance"),
            heap.alloc(resonance).to_value(),
        );

        // Add cutoff_end if sweep_to is provided
        if !sweep_to.is_none() {
            let end_cutoff = extract_float(sweep_to, "lowpass", "sweep_to")?;
            validate_positive(end_cutoff, "lowpass", "sweep_to").map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(
                hashed_key(heap, "cutoff_end"),
                heap.alloc(end_cutoff).to_value(),
            );
        }

        Ok(dict)
    }

    /// Creates a highpass filter.
    ///
    /// # Arguments
    /// * `cutoff` - Cutoff frequency in Hz
    /// * `resonance` - Q factor / resonance (default: 0.707)
    /// * `sweep_to` - Optional target cutoff for sweep
    ///
    /// # Returns
    /// A dict matching the Filter::Highpass IR structure.
    ///
    /// # Example
    /// ```starlark
    /// highpass(100)
    /// highpass(500, 1.0, 2000)  # Sweep from 500 to 2000 Hz
    /// ```
    fn highpass<'v>(
        cutoff: f64,
        #[starlark(default = 0.707)] resonance: f64,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(cutoff, "highpass", "cutoff").map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(resonance, "highpass", "resonance").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("highpass").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "cutoff"), heap.alloc(cutoff).to_value());
        dict.insert_hashed(
            hashed_key(heap, "resonance"),
            heap.alloc(resonance).to_value(),
        );

        // Add cutoff_end if sweep_to is provided
        if !sweep_to.is_none() {
            let end_cutoff = extract_float(sweep_to, "highpass", "sweep_to")?;
            validate_positive(end_cutoff, "highpass", "sweep_to")
                .map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(
                hashed_key(heap, "cutoff_end"),
                heap.alloc(end_cutoff).to_value(),
            );
        }

        Ok(dict)
    }

    /// Creates a complete audio synthesis layer.
    ///
    /// # Arguments
    /// * `synthesis` - Synthesis dict from oscillator(), fm_synth(), etc.
    /// * `envelope` - Optional envelope dict (uses default if None)
    /// * `volume` - Layer volume 0.0-1.0 (default: 0.8)
    /// * `pan` - Stereo pan -1.0 to 1.0 (default: 0.0)
    /// * `filter` - Optional filter dict from lowpass()/highpass()
    /// * `lfo` - Optional LFO modulation dict
    /// * `delay` - Optional layer start delay in seconds
    ///
    /// # Returns
    /// A dict matching the AudioLayer IR structure.
    ///
    /// # Example
    /// ```starlark
    /// audio_layer(oscillator(440))
    /// audio_layer(
    ///     synthesis = oscillator(440, "sawtooth"),
    ///     envelope = envelope(0.01, 0.2, 0.6, 0.3),
    ///     volume = 0.7,
    ///     pan = -0.3,
    ///     filter = lowpass(2000, 0.707, 500)
    /// )
    /// ```
    fn bandpass<'v>(
        center: f64,
        #[starlark(default = 1.0)] resonance: f64,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(center, "bandpass", "center").map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(resonance, "bandpass", "resonance").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("bandpass").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "center"), heap.alloc(center).to_value());
        dict.insert_hashed(
            hashed_key(heap, "resonance"),
            heap.alloc(resonance).to_value(),
        );

        if !sweep_to.is_none() {
            let end_center = extract_float(sweep_to, "bandpass", "sweep_to")?;
            validate_positive(end_center, "bandpass", "sweep_to")
                .map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(
                hashed_key(heap, "center_end"),
                heap.alloc(end_center).to_value(),
            );
        }

        Ok(dict)
    }

    /// Creates a notch (band-reject) filter.
    ///
    /// # Arguments
    /// * `center` - Center frequency in Hz
    /// * `resonance` - Q factor / resonance (default: 1.0)
    /// * `sweep_to` - Optional target center frequency for sweep
    fn notch<'v>(
        center: f64,
        #[starlark(default = 1.0)] resonance: f64,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(center, "notch", "center").map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(resonance, "notch", "resonance").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "type"), heap.alloc_str("notch").to_value());
        dict.insert_hashed(hashed_key(heap, "center"), heap.alloc(center).to_value());
        dict.insert_hashed(
            hashed_key(heap, "resonance"),
            heap.alloc(resonance).to_value(),
        );

        if !sweep_to.is_none() {
            let end_center = extract_float(sweep_to, "notch", "sweep_to")?;
            validate_positive(end_center, "notch", "sweep_to").map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(
                hashed_key(heap, "center_end"),
                heap.alloc(end_center).to_value(),
            );
        }

        Ok(dict)
    }

    /// Creates an allpass filter.
    ///
    /// # Arguments
    /// * `frequency` - Center frequency in Hz
    /// * `resonance` - Q factor / resonance (default: 0.707)
    /// * `sweep_to` - Optional target frequency for sweep
    fn allpass<'v>(
        frequency: f64,
        #[starlark(default = 0.707)] resonance: f64,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "allpass", "frequency").map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(resonance, "allpass", "resonance").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("allpass").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "resonance"),
            heap.alloc(resonance).to_value(),
        );

        if !sweep_to.is_none() {
            let end_freq = extract_float(sweep_to, "allpass", "sweep_to")?;
            validate_positive(end_freq, "allpass", "sweep_to").map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(
                hashed_key(heap, "frequency_end"),
                heap.alloc(end_freq).to_value(),
            );
        }

        Ok(dict)
    }

    /// Creates a comb filter.
    ///
    /// # Arguments
    /// * `delay_ms` - Delay time in milliseconds
    /// * `feedback` - Feedback amount 0.0-0.99
    /// * `wet` - Wet/dry mix 0.0-1.0
    fn comb_filter<'v>(
        delay_ms: f64,
        feedback: f64,
        wet: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(delay_ms, "comb_filter", "delay_ms").map_err(|e| anyhow::anyhow!(e))?;
        if !(0.0..=0.99).contains(&feedback) {
            return Err(anyhow::anyhow!(
                "S103: comb_filter(): 'feedback' must be 0.0-0.99, got {}",
                feedback
            ));
        }
        validate_unit_range(wet, "comb_filter", "wet").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(hashed_key(heap, "type"), heap.alloc_str("comb").to_value());
        dict.insert_hashed(
            hashed_key(heap, "delay_ms"),
            heap.alloc(delay_ms).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "feedback"),
            heap.alloc(feedback).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "wet"), heap.alloc(wet).to_value());

        Ok(dict)
    }

    /// Creates a formant filter.
    ///
    /// # Arguments
    /// * `vowel` - Vowel preset: "a", "i", "u", "e", "o"
    /// * `intensity` - Intensity 0.0-1.0 (0.0 = dry, 1.0 = full vowel shape)
    fn formant_filter<'v>(vowel: &str, intensity: f64, heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        const VOWELS: &[&str] = &["a", "i", "u", "e", "o"];
        validate_enum(vowel, VOWELS, "formant_filter", "vowel").map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(intensity, "formant_filter", "intensity")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("formant").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "vowel"), heap.alloc_str(vowel).to_value());
        dict.insert_hashed(
            hashed_key(heap, "intensity"),
            heap.alloc(intensity).to_value(),
        );

        Ok(dict)
    }

    /// Creates a ladder filter (Moog-style 4-pole lowpass).
    ///
    /// # Arguments
    /// * `cutoff` - Cutoff frequency in Hz
    /// * `resonance` - Resonance 0.0-1.0 (maps to 0-4x feedback)
    /// * `sweep_to` - Optional target cutoff for sweep
    fn ladder<'v>(
        cutoff: f64,
        #[starlark(default = 0.0)] resonance: f64,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(cutoff, "ladder", "cutoff").map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(resonance, "ladder", "resonance").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("ladder").to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "cutoff"), heap.alloc(cutoff).to_value());
        dict.insert_hashed(
            hashed_key(heap, "resonance"),
            heap.alloc(resonance).to_value(),
        );

        if !sweep_to.is_none() {
            let end_cutoff = extract_float(sweep_to, "ladder", "sweep_to")?;
            validate_positive(end_cutoff, "ladder", "sweep_to").map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(
                hashed_key(heap, "cutoff_end"),
                heap.alloc(end_cutoff).to_value(),
            );
        }

        Ok(dict)
    }

    /// Creates a low shelf filter.
    ///
    /// # Arguments
    /// * `frequency` - Shelf frequency in Hz
    /// * `gain_db` - Gain in dB (positive for boost, negative for cut)
    fn shelf_low<'v>(frequency: f64, gain_db: f64, heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "shelf_low", "frequency").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("shelf_low").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "gain_db"), heap.alloc(gain_db).to_value());

        Ok(dict)
    }

    /// Creates a high shelf filter.
    ///
    /// # Arguments
    /// * `frequency` - Shelf frequency in Hz
    /// * `gain_db` - Gain in dB (positive for boost, negative for cut)
    fn shelf_high<'v>(frequency: f64, gain_db: f64, heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "shelf_high", "frequency").map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("shelf_high").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(hashed_key(heap, "gain_db"), heap.alloc(gain_db).to_value());

        Ok(dict)
    }
}
