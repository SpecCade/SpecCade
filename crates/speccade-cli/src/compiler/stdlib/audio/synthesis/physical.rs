//! Physical modeling synthesis functions

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

/// Registers physical modeling synthesis functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_physical_synthesis(builder);
}

#[starlark_module]
fn register_physical_synthesis(builder: &mut GlobalsBuilder) {
    /// Creates a Membrane Drum synthesis block.
    ///
    /// # Arguments
    /// * `frequency` - Fundamental frequency in Hz
    /// * `decay` - Decay rate 0.0-1.0 (higher = faster decay)
    /// * `tone` - Tone/brightness 0.0-1.0 (low = fundamental, high = overtones)
    /// * `strike` - Strike strength 0.0-1.0
    fn membrane_drum<'v>(
        frequency: f64,
        decay: f64,
        tone: f64,
        strike: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "membrane_drum", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(decay, "membrane_drum", "decay")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(tone, "membrane_drum", "tone")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(strike, "membrane_drum", "strike")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("membrane_drum").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "decay"),
            heap.alloc(decay).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "tone"),
            heap.alloc(tone).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "strike"),
            heap.alloc(strike).to_value(),
        );

        Ok(dict)
    }

    /// Creates a Feedback FM synthesis block.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `feedback` - Self-modulation amount 0.0-1.0
    /// * `modulation_index` - Modulation depth/index
    /// * `sweep_to` - Optional target frequency for sweep
    fn feedback_fm<'v>(
        frequency: f64,
        feedback: f64,
        modulation_index: f64,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "feedback_fm", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(feedback, "feedback_fm", "feedback")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(modulation_index, "feedback_fm", "modulation_index")
            .or_else(|_| if modulation_index == 0.0 { Ok(()) } else { Err("".to_string()) })
            .map_err(|_| anyhow::anyhow!("S103: feedback_fm(): 'modulation_index' must be >= 0"))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("feedback_fm").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "feedback"),
            heap.alloc(feedback).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "modulation_index"),
            heap.alloc(modulation_index).to_value(),
        );

        if !sweep_to.is_none() {
            let end_freq = extract_float(sweep_to, "feedback_fm", "sweep_to")?;
            validate_positive(end_freq, "feedback_fm", "sweep_to")
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

    /// Creates a Phase Distortion synthesis block.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `distortion` - Distortion amount (0.0 = pure sine, higher = more harmonics)
    /// * `distortion_decay` - How fast distortion decays to pure sine
    /// * `waveform` - PD waveform: "resonant", "sawtooth", "pulse"
    /// * `sweep_to` - Optional target frequency for sweep
    fn pd_synth<'v>(
        frequency: f64,
        distortion: f64,
        #[starlark(default = 0.0)] distortion_decay: f64,
        #[starlark(default = "resonant")] waveform: &str,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "pd_synth", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(distortion, "pd_synth", "distortion")
            .or_else(|_| if distortion == 0.0 { Ok(()) } else { Err("".to_string()) })
            .map_err(|_| anyhow::anyhow!("S103: pd_synth(): 'distortion' must be >= 0"))?;

        const PD_WAVEFORMS: &[&str] = &["resonant", "sawtooth", "pulse"];
        validate_enum(waveform, PD_WAVEFORMS, "pd_synth", "waveform")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("pd_synth").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "distortion"),
            heap.alloc(distortion).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "distortion_decay"),
            heap.alloc(distortion_decay).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "waveform"),
            heap.alloc_str(waveform).to_value(),
        );

        if !sweep_to.is_none() {
            let end_freq = extract_float(sweep_to, "pd_synth", "sweep_to")?;
            validate_positive(end_freq, "pd_synth", "sweep_to")
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

    /// Creates a Modal synthesis block.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `modes` - List of mode dicts from modal_mode()
    /// * `excitation` - Excitation type: "impulse", "noise", "pluck"
    /// * `sweep_to` - Optional target frequency for sweep
    fn modal<'v>(
        frequency: f64,
        modes: Value<'v>,
        #[starlark(default = "impulse")] excitation: &str,
        #[starlark(default = NoneType)] sweep_to: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "modal", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;

        const EXCITATIONS: &[&str] = &["impulse", "noise", "pluck"];
        validate_enum(excitation, EXCITATIONS, "modal", "excitation")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("modal").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "modes"),
            modes,
        );
        dict.insert_hashed(
            hashed_key(heap, "excitation"),
            heap.alloc_str(excitation).to_value(),
        );

        if !sweep_to.is_none() {
            let end_freq = extract_float(sweep_to, "modal", "sweep_to")?;
            validate_positive(end_freq, "modal", "sweep_to")
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

    /// Creates a modal mode configuration.
    ///
    /// # Arguments
    /// * `freq_ratio` - Frequency ratio relative to fundamental (1.0 = fundamental)
    /// * `amplitude` - Amplitude of this mode 0.0-1.0
    /// * `decay_time` - Decay time in seconds
    fn modal_mode<'v>(
        freq_ratio: f64,
        amplitude: f64,
        decay_time: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(freq_ratio, "modal_mode", "freq_ratio")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(amplitude, "modal_mode", "amplitude")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_positive(decay_time, "modal_mode", "decay_time")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "freq_ratio"),
            heap.alloc(freq_ratio).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "amplitude"),
            heap.alloc(amplitude).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "decay_time"),
            heap.alloc(decay_time).to_value(),
        );

        Ok(dict)
    }

    /// Creates a Metallic synthesis block.
    ///
    /// # Arguments
    /// * `base_freq` - Base frequency in Hz
    /// * `num_partials` - Number of inharmonic partials
    /// * `inharmonicity` - Inharmonicity factor (1.0 = harmonic, >1.0 = inharmonic)
    fn metallic<'v>(
        base_freq: f64,
        num_partials: i32,
        inharmonicity: f64,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(base_freq, "metallic", "base_freq")
            .map_err(|e| anyhow::anyhow!(e))?;
        if num_partials < 1 {
            return Err(anyhow::anyhow!("S103: metallic(): 'num_partials' must be >= 1"));
        }
        if inharmonicity < 1.0 {
            return Err(anyhow::anyhow!("S103: metallic(): 'inharmonicity' must be >= 1.0"));
        }

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("metallic").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "base_freq"),
            heap.alloc(base_freq).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "num_partials"),
            heap.alloc(num_partials).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "inharmonicity"),
            heap.alloc(inharmonicity).to_value(),
        );

        Ok(dict)
    }

    /// Creates a Comb Filter synthesis block.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz (determines delay line length)
    /// * `decay` - Feedback decay 0.0-1.0 (higher = longer resonance)
    /// * `excitation` - Excitation type: "impulse", "noise", "saw"
    fn comb_filter_synth<'v>(
        frequency: f64,
        decay: f64,
        #[starlark(default = "impulse")] excitation: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        validate_positive(frequency, "comb_filter_synth", "frequency")
            .map_err(|e| anyhow::anyhow!(e))?;
        validate_unit_range(decay, "comb_filter_synth", "decay")
            .map_err(|e| anyhow::anyhow!(e))?;

        const EXCITATIONS: &[&str] = &["impulse", "noise", "saw"];
        validate_enum(excitation, EXCITATIONS, "comb_filter_synth", "excitation")
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut dict = new_dict(heap);

        dict.insert_hashed(
            hashed_key(heap, "type"),
            heap.alloc_str("comb_filter_synth").to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "frequency"),
            heap.alloc(frequency).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "decay"),
            heap.alloc(decay).to_value(),
        );
        dict.insert_hashed(
            hashed_key(heap, "excitation"),
            heap.alloc_str(excitation).to_value(),
        );

        Ok(dict)
    }
}
