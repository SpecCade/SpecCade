//! Exotic and advanced synthesis functions

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

/// Registers exotic synthesis functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_exotic_synthesis(builder);
}

#[starlark_module]
fn register_exotic_synthesis(builder: &mut GlobalsBuilder) {
    /// Creates a Vocoder synthesis block.
    ///
    /// # Arguments
    /// * `carrier_freq` - Carrier frequency in Hz
    /// * `carrier_type` - Carrier type: "sawtooth", "pulse", "noise"
    /// * `num_bands` - Number of filter bands (8-32)
    /// * `band_spacing` - Band spacing: "linear" or "logarithmic"
    /// * `envelope_attack` - Envelope follower attack time
    /// * `envelope_release` - Envelope follower release time
    /// * `formant_rate` - Rate of formant transitions (default: 2.0)
    /// * `bands` - Optional custom band configurations
    fn vocoder<'v>(
        #[starlark(require = named)] carrier_freq: f64,
        #[starlark(require = named)] carrier_type: &str,
        #[starlark(require = named)] num_bands: i32,
        #[starlark(require = named)] band_spacing: &str,
        #[starlark(require = named)] envelope_attack: f64,
        #[starlark(require = named)] envelope_release: f64,
        #[starlark(require = named, default = 2.0)] formant_rate: f64,
        #[starlark(require = named, default = NoneType)] bands: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(carrier_freq, "vocoder", "carrier_freq")
            .map_err(|e| anyhow::anyhow!(e))?;

        const CARRIER_TYPES: &[&str] = &["sawtooth", "pulse", "noise"];
        validate_enum(carrier_type, CARRIER_TYPES, "vocoder", "carrier_type")
            .map_err(|e| anyhow::anyhow!(e))?;

        if !(8..=32).contains(&num_bands) {
            return Err(anyhow::anyhow!("S103: vocoder(): 'num_bands' must be 8-32, got {}", num_bands));
        }

        const BAND_SPACINGS: &[&str] = &["linear", "logarithmic"];
        validate_enum(band_spacing, BAND_SPACINGS, "vocoder", "band_spacing")
            .map_err(|e| anyhow::anyhow!(e))?;

        validate_positive(envelope_attack, "vocoder", "envelope_attack")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(envelope_release, "vocoder", "envelope_release")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(formant_rate, "vocoder", "formant_rate")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("vocoder").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "carrier_freq"),
            heap.alloc(carrier_freq).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "carrier_type"),
            heap.alloc_str(carrier_type).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "num_bands"),
            heap.alloc(num_bands).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "band_spacing"),
            heap.alloc_str(band_spacing).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "envelope_attack"),
            heap.alloc(envelope_attack).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "envelope_release"),
            heap.alloc(envelope_release).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "formant_rate"),
            heap.alloc(formant_rate).to_value(),
        );

        if !bands.is_none() {
            dict.insert_hashed(
                hashed_key(heap, "bands"),
                bands,
            );
        }

        Ok(dict)
    }

    /// Creates a vocoder band configuration.
    ///
    /// # Arguments
    /// * `center_freq` - Center frequency of the band in Hz
    /// * `bandwidth` - Bandwidth (Q factor) of the band filter
    /// * `envelope_pattern` - Optional list of amplitude values over time (0.0-1.0)
    ///
    /// # Example
    /// ```starlark
    /// vocoder_band(200, 100)
    /// vocoder_band(400, 150, [0.0, 0.5, 1.0, 0.5, 0.0])
    /// ```
    #[starlark(speculative_exec_safe)]
    fn vocoder_band<'v>(
        center_freq: f64,
        bandwidth: f64,
        #[starlark(default = NoneType)] envelope_pattern: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(center_freq, "vocoder_band", "center_freq")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(bandwidth, "vocoder_band", "bandwidth")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "center_freq"),
            heap.alloc(center_freq).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "bandwidth"),
            heap.alloc(bandwidth).to_value(),
        );

        if !envelope_pattern.is_none() {
            dict.insert_hashed(
                hashed_key(heap, "envelope_pattern"),
                envelope_pattern,
            );
        }

        Ok(dict)
    }

    /// Creates a Formant synthesis block for vowel/voice sounds.
    ///
    /// # Arguments
    /// * `frequency` - Base pitch frequency in Hz
    /// * `formants` - Optional list of formant configs from formant_config()
    /// * `vowel` - Optional vowel preset: "a", "i", "u", "e", "o"
    /// * `vowel_morph` - Optional second vowel for morphing
    /// * `morph_amount` - Morph amount 0.0-1.0 (0.0 = first vowel, 1.0 = second)
    /// * `breathiness` - Noise amount for breathiness 0.0-1.0
    ///
    /// # Example
    /// ```starlark
    /// formant_synth(frequency = 220, vowel = "a")
    /// formant_synth(frequency = 220, vowel = "a", vowel_morph = "i", morph_amount = 0.5)
    /// ```
    #[starlark(speculative_exec_safe)]
    fn formant_synth<'v>(
        #[starlark(require = named)] frequency: f64,
        #[starlark(require = named, default = NoneType)] formants: Value<'v>,
        #[starlark(require = named, default = NoneType)] vowel: Value<'v>,
        #[starlark(require = named, default = NoneType)] vowel_morph: Value<'v>,
        #[starlark(require = named, default = 0.0)] morph_amount: f64,
        #[starlark(require = named, default = 0.0)] breathiness: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "formant_synth", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(morph_amount, "formant_synth", "morph_amount")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(breathiness, "formant_synth", "breathiness")
            .map_err(|e| anyhow::anyhow!(e))?;

        const VOWELS: &[&str] = &["a", "i", "u", "e", "o"];

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("formant").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );

        if !formants.is_none() {
            dict.insert_hashed(
                hashed_key(heap, "formants"),
                formants,
            );
        }

        if !vowel.is_none() {
            let vowel_str = vowel.unpack_str()
                .ok_or_else(|| anyhow::anyhow!("S102: formant_synth(): 'vowel' must be a string"))?;
            validate_enum(vowel_str, VOWELS, "formant_synth", "vowel")
                .map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(
                hashed_key(heap, "vowel"),
                heap.alloc_str(vowel_str).to_value(),
            );
        }

        if !vowel_morph.is_none() {
            let vowel_morph_str = vowel_morph.unpack_str()
                .ok_or_else(|| anyhow::anyhow!("S102: formant_synth(): 'vowel_morph' must be a string"))?;
            validate_enum(vowel_morph_str, VOWELS, "formant_synth", "vowel_morph")
                .map_err(|e| anyhow::anyhow!(e))?;
            dict.insert_hashed(
                hashed_key(heap, "vowel_morph"),
                heap.alloc_str(vowel_morph_str).to_value(),
            );
        }

        dict.insert_hashed(
            hashed_key(heap, "morph_amount"),
            heap.alloc(morph_amount).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "breathiness"),
            heap.alloc(breathiness).to_value(),
        );

        Ok(dict)
    }

    /// Creates a formant configuration.
    ///
    /// # Arguments
    /// * `frequency` - Center frequency of the formant in Hz
    /// * `amplitude` - Amplitude/gain 0.0-1.0
    /// * `bandwidth` - Bandwidth (Q factor)
    ///
    /// # Example
    /// ```starlark
    /// formant_config(800, 1.0, 100)
    /// ```
    #[starlark(speculative_exec_safe)]
    fn formant_config<'v>(
        frequency: f64,
        amplitude: f64,
        bandwidth: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "formant_config", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(amplitude, "formant_config", "amplitude")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(bandwidth, "formant_config", "bandwidth")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "amplitude"),
            heap.alloc(amplitude).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "bandwidth"),
            heap.alloc(bandwidth).to_value(),
        );

        Ok(dict)
    }

    /// Creates a Vector synthesis block with 2D crossfading.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `sources` - List of exactly 4 source dicts from vector_source()
    /// * `position_x` - Static X position 0.0-1.0 (default: 0.5)
    /// * `position_y` - Static Y position 0.0-1.0 (default: 0.5)
    /// * `path` - Optional list of path points from vector_path_point()
    /// * `path_loop` - Whether path should loop (default: False)
    /// * `path_curve` - Path interpolation: "linear", "exponential", "logarithmic"
    ///
    /// # Example
    /// ```starlark
    /// vector_synth(
    ///     frequency = 440,
    ///     sources = [
    ///         vector_source("sine"),
    ///         vector_source("saw"),
    ///         vector_source("square"),
    ///         vector_source("triangle"),
    ///     ],
    ///     position_x = 0.25,
    ///     position_y = 0.75,
    /// )
    /// ```
    #[starlark(speculative_exec_safe)]
    fn vector_synth<'v>(
        #[starlark(require = named)] frequency: f64,
        #[starlark(require = named)] sources: Value<'v>,
        #[starlark(require = named, default = 0.5)] position_x: f64,
        #[starlark(require = named, default = 0.5)] position_y: f64,
        #[starlark(require = named, default = NoneType)] path: Value<'v>,
        #[starlark(require = named, default = false)] path_loop: bool,
        #[starlark(require = named, default = "linear")] path_curve: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "vector_synth", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(position_x, "vector_synth", "position_x")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(position_y, "vector_synth", "position_y")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(path_curve, SWEEP_CURVES, "vector_synth", "path_curve")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Verify sources is a list of 4 items
        let sources_list: Vec<Value<'v>> = sources.iterate(heap)
            .map_err(|_| anyhow::anyhow!("S102: vector_synth(): 'sources' must be a list"))?
            .collect();
        if sources_list.len() != 4 {
            return Err(anyhow::anyhow!("S103: vector_synth(): 'sources' must have exactly 4 elements, got {}", sources_list.len()));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("vector").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "sources"),
            sources,
        );
        dict.insert_hashed(
            hashed_key(heap, "position_x"),
            heap.alloc(position_x).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "position_y"),
            heap.alloc(position_y).to_value(),
        );

        if !path.is_none() {
            dict.insert_hashed(
                hashed_key(heap, "path"),
                path,
            );
        }

        dict.insert_hashed(
            hashed_key(heap, "path_loop"),
            heap.alloc(path_loop).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "path_curve"),
            heap.alloc_str(path_curve).to_value(),
        );

        Ok(dict)
    }

    /// Creates a vector source configuration.
    ///
    /// # Arguments
    /// * `source_type` - Source waveform: "sine", "saw", "square", "triangle", "noise", "wavetable"
    /// * `frequency_ratio` - Frequency ratio relative to base (default: 1.0)
    ///
    /// # Example
    /// ```starlark
    /// vector_source("sine")
    /// vector_source("saw", 2.0)  # One octave up
    /// ```
    #[starlark(speculative_exec_safe)]
    fn vector_source<'v>(
        source_type: &str,
        #[starlark(default = 1.0)] frequency_ratio: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        const VECTOR_SOURCES: &[&str] = &["sine", "saw", "square", "triangle", "noise", "wavetable"];
        validate_enum(source_type, VECTOR_SOURCES, "vector_source", "source_type")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(frequency_ratio, "vector_source", "frequency_ratio")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "source_type"),
            heap.alloc_str(source_type).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency_ratio"),
            heap.alloc(frequency_ratio).to_value(),
        );

        Ok(dict)
    }

    /// Creates a vector path point for animation.
    ///
    /// # Arguments
    /// * `x` - X position 0.0-1.0
    /// * `y` - Y position 0.0-1.0
    /// * `duration` - Duration in seconds to reach this point
    ///
    /// # Example
    /// ```starlark
    /// vector_path_point(0.0, 0.0, 0.5)  # Move to top-left over 0.5 seconds
    /// ```
    #[starlark(speculative_exec_safe)]
    fn vector_path_point<'v>(
        x: f64,
        y: f64,
        duration: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_unit_range(x, "vector_path_point", "x")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(y, "vector_path_point", "y")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(duration, "vector_path_point", "duration")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "x"),
            heap.alloc(x).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "y"),
            heap.alloc(y).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "duration"),
            heap.alloc(duration).to_value(),
        );

        Ok(dict)
    }

    /// Creates a Waveguide synthesis block for wind/brass sounds.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `breath` - Breath/excitation strength 0.0-1.0
    /// * `noise` - Noise mix in excitation 0.0-1.0
    /// * `damping` - High-frequency absorption 0.0-1.0
    /// * `resonance` - Feedback/resonance amount 0.0-1.0
    ///
    /// # Example
    /// ```starlark
    /// waveguide(
    ///     frequency = 440,
    ///     breath = 0.7,
    ///     noise = 0.3,
    ///     damping = 0.5,
    ///     resonance = 0.8,
    /// )
    /// ```
    #[starlark(speculative_exec_safe)]
    fn waveguide<'v>(
        #[starlark(require = named)] frequency: f64,
        #[starlark(require = named)] breath: f64,
        #[starlark(require = named)] noise: f64,
        #[starlark(require = named)] damping: f64,
        #[starlark(require = named)] resonance: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "waveguide", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(breath, "waveguide", "breath")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(noise, "waveguide", "noise")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(damping, "waveguide", "damping")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(resonance, "waveguide", "resonance")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("waveguide").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "breath"),
            heap.alloc(breath).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "noise"),
            heap.alloc(noise).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "damping"),
            heap.alloc(damping).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "resonance"),
            heap.alloc(resonance).to_value(),
        );

        Ok(dict)
    }

    /// Creates a Bowed String synthesis block for violin/cello sounds.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `bow_pressure` - Bow pressure/force 0.0-1.0
    /// * `bow_position` - Bow position on string 0.0-1.0 (0 = bridge, 1 = nut)
    /// * `damping` - String damping 0.0-1.0
    ///
    /// # Example
    /// ```starlark
    /// bowed_string(
    ///     frequency = 440,
    ///     bow_pressure = 0.6,
    ///     bow_position = 0.3,
    ///     damping = 0.2,
    /// )
    /// ```
    #[starlark(speculative_exec_safe)]
    fn bowed_string<'v>(
        #[starlark(require = named)] frequency: f64,
        #[starlark(require = named)] bow_pressure: f64,
        #[starlark(require = named)] bow_position: f64,
        #[starlark(require = named)] damping: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "bowed_string", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(bow_pressure, "bowed_string", "bow_pressure")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(bow_position, "bowed_string", "bow_position")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(damping, "bowed_string", "damping")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("bowed_string").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "bow_pressure"),
            heap.alloc(bow_pressure).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "bow_position"),
            heap.alloc(bow_position).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "damping"),
            heap.alloc(damping).to_value(),
        );

        Ok(dict)
    }

    /// Creates a Pulsar synthesis block (synchronized grain trains).
    ///
    /// # Arguments
    /// * `frequency` - Fundamental frequency of each grain in Hz
    /// * `pulse_rate` - Grains per second (pulsaret rate)
    /// * `grain_size_ms` - Duration of each grain in milliseconds
    /// * `shape` - Waveform shape: "sine", "square", "sawtooth", "triangle", "pulse"
    ///
    /// # Example
    /// ```starlark
    /// pulsar(
    ///     frequency = 440,
    ///     pulse_rate = 20,
    ///     grain_size_ms = 50,
    ///     shape = "sine",
    /// )
    /// ```
    #[starlark(speculative_exec_safe)]
    fn pulsar<'v>(
        #[starlark(require = named)] frequency: f64,
        #[starlark(require = named)] pulse_rate: f64,
        #[starlark(require = named)] grain_size_ms: f64,
        #[starlark(require = named)] shape: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "pulsar", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(pulse_rate, "pulsar", "pulse_rate")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(grain_size_ms, "pulsar", "grain_size_ms")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_enum(shape, WAVEFORMS, "pulsar", "shape")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("pulsar").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "pulse_rate"),
            heap.alloc(pulse_rate).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "grain_size_ms"),
            heap.alloc(grain_size_ms).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "shape"),
            heap.alloc_str(shape).to_value(),
        );

        Ok(dict)
    }

    /// Creates a VOSIM synthesis block (voice simulation).
    ///
    /// # Arguments
    /// * `frequency` - Fundamental frequency (pitch) in Hz
    /// * `formant_freq` - Formant frequency (spectral peak) in Hz
    /// * `pulses` - Number of pulses per period (1-16)
    /// * `breathiness` - Noise amount for breathiness 0.0-1.0 (default: 0.0)
    ///
    /// # Example
    /// ```starlark
    /// vosim(
    ///     frequency = 220,
    ///     formant_freq = 880,
    ///     pulses = 4,
    ///     breathiness = 0.1,
    /// )
    /// ```
    #[starlark(speculative_exec_safe)]
    fn vosim<'v>(
        #[starlark(require = named)] frequency: f64,
        #[starlark(require = named)] formant_freq: f64,
        #[starlark(require = named)] pulses: i32,
        #[starlark(require = named, default = 0.0)] breathiness: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "vosim", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(formant_freq, "vosim", "formant_freq")
            .map_err(|e| anyhow::anyhow!(e))?;
        if !(1..=16).contains(&pulses) {
            return Err(anyhow::anyhow!("S103: vosim(): 'pulses' must be 1-16, got {}", pulses));
        }
        validate_unit_range(breathiness, "vosim", "breathiness")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("vosim").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "formant_freq"),
            heap.alloc(formant_freq).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "pulses"),
            heap.alloc(pulses).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "breathiness"),
            heap.alloc(breathiness).to_value(),
        );

        Ok(dict)
    }

    /// Creates a Spectral Freeze synthesis block using FFT.
    ///
    /// # Arguments
    /// * `source` - Source material dict from spectral_source()
    ///
    /// # Example
    /// ```starlark
    /// spectral_freeze(source = spectral_source("noise", "pink"))
    /// spectral_freeze(source = spectral_source("tone", "sawtooth", 440))
    /// ```
    #[starlark(speculative_exec_safe)]
    fn spectral_freeze<'v>(
        #[starlark(require = named)] source: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("spectral_freeze").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "source"),
            source,
        );

        Ok(dict)
    }

    /// Creates a spectral source configuration.
    ///
    /// # Arguments
    /// * `source_type` - Source type: "noise" or "tone"
    /// * `param1` - For noise: noise_type. For tone: waveform.
    /// * `param2` - For tone: frequency. Ignored for noise.
    ///
    /// # Example
    /// ```starlark
    /// spectral_source("noise", "pink")
    /// spectral_source("tone", "sawtooth", 440)
    /// ```
    #[starlark(speculative_exec_safe)]
    fn spectral_source<'v>(
        source_type: &str,
        param1: Value<'v>,
        #[starlark(default = NoneType)] param2: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        let mut dict = new_dict(heap);

        match source_type {
            "noise" => {
                let noise_type = param1.unpack_str()
                    .ok_or_else(|| anyhow::anyhow!("S102: spectral_source(): noise type must be a string"))?;
                validate_enum(noise_type, NOISE_TYPES, "spectral_source", "noise_type")
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
                    .ok_or_else(|| anyhow::anyhow!("S102: spectral_source(): waveform must be a string"))?;
                validate_enum(waveform, WAVEFORMS, "spectral_source", "waveform")
                    .map_err(|e| anyhow::anyhow!(e))?;

                let frequency = extract_float(param2, "spectral_source", "frequency")?;
                validate_positive(frequency, "spectral_source", "frequency")
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
            _ => {
                return Err(anyhow::anyhow!("S104: spectral_source(): 'source_type' must be one of: noise, tone"));
            }
        }

        Ok(dict)
    }

    /// Creates a Pitched Body synthesis block (impact sounds with frequency sweep).
    ///
    /// # Arguments
    /// * `start_freq` - Starting frequency in Hz
    /// * `end_freq` - Ending frequency in Hz
    ///
    /// # Example
    /// ```starlark
    /// pitched_body(start_freq = 880, end_freq = 110)
    /// ```
    #[starlark(speculative_exec_safe)]
    fn pitched_body<'v>(
        #[starlark(require = named)] start_freq: f64,
        #[starlark(require = named)] end_freq: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(start_freq, "pitched_body", "start_freq")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(end_freq, "pitched_body", "end_freq")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("pitched_body").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "start_freq"),
            heap.alloc(start_freq).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "end_freq"),
            heap.alloc(end_freq).to_value(),
        );

        Ok(dict)
    }
}
