//! Audio synthesis, filters, effects, modulation, and layer functions

use super::{func, param, FunctionInfo};

const WAVEFORMS: &[&str] = &["sine", "square", "sawtooth", "triangle", "pulse"];
const NOISE_TYPES: &[&str] = &["white", "pink", "brown"];
const SWEEP_CURVES: &[&str] = &["linear", "exponential", "logarithmic"];

pub(super) fn register_functions() -> Vec<FunctionInfo> {
    vec![
        // === AUDIO SYNTHESIS (Basic) ===
        func!(
            "oscillator",
            "audio.synthesis",
            "Creates a basic oscillator synthesis block with optional frequency sweep.",
            vec![
                param!("frequency", "float", req, range: Some(0.0), None),
                param!("waveform", "string", opt, "sine", enum: WAVEFORMS),
                param!("sweep_to", "float", opt_none, range: Some(0.0), None),
                param!("curve", "string", opt, "linear", enum: SWEEP_CURVES),
                param!("detune", "float", opt_none),
                param!("duty", "float", opt_none, range: Some(0.0), Some(1.0)),
            ],
            "A dict matching the Synthesis::Oscillator IR structure.",
            r#"oscillator(440, "sawtooth", 220, "exponential")"#
        ),
        func!(
            "fm_synth",
            "audio.synthesis",
            "Creates an FM synthesis block.",
            vec![
                param!("carrier", "float", req, range: Some(0.0), None),
                param!("modulator", "float", req, range: Some(0.0), None),
                param!("index", "float", req, range: Some(0.0), None),
                param!("sweep_to", "float", opt_none, range: Some(0.0), None),
            ],
            "A dict matching the Synthesis::FmSynth IR structure.",
            "fm_synth(440, 880, 5.0)"
        ),
        func!(
            "am_synth",
            "audio.synthesis",
            "Creates an AM synthesis block.",
            vec![
                param!("carrier", "float", req, range: Some(0.0), None),
                param!("modulator", "float", req, range: Some(0.0), None),
                param!("depth", "float", req, range: Some(0.0), Some(1.0)),
                param!("sweep_to", "float", opt_none, range: Some(0.0), None),
            ],
            "A dict matching the Synthesis::AmSynth IR structure.",
            "am_synth(440, 880, 0.5)"
        ),
        func!(
            "noise_burst",
            "audio.synthesis",
            "Creates a noise burst synthesis block.",
            vec![
                param!("noise_type", "string", opt, "white", enum: NOISE_TYPES),
                param!("filter", "dict", opt_none),
            ],
            "A dict matching the Synthesis::NoiseBurst IR structure.",
            r#"noise_burst("pink", lowpass(5000))"#
        ),
        func!(
            "karplus_strong",
            "audio.synthesis",
            "Creates a Karplus-Strong plucked string synthesis block.",
            vec![
                param!("frequency", "float", req, range: Some(0.0), None),
                param!("damping", "float", req, range: Some(0.0), Some(1.0)),
                param!("blend", "float", req, range: Some(0.0), Some(1.0)),
            ],
            "A dict matching the Synthesis::KarplusStrong IR structure.",
            "karplus_strong(440, 0.5, 0.5)"
        ),
        // === AUDIO SYNTHESIS (Complex) ===
        func!(
            "additive",
            "audio.synthesis",
            "Creates an additive synthesis block with harmonics.",
            vec![
                param!("base_freq", "float", req, range: Some(0.0), None),
                param!("harmonics", "list", req),
            ],
            "A dict matching the Synthesis::Additive IR structure.",
            "additive(440, [1.0, 0.5, 0.25, 0.125])"
        ),
        func!(
            "supersaw_unison",
            "audio.synthesis",
            "Creates a Supersaw/Unison synthesis block.",
            vec![
                param!("frequency", "float", req, range: Some(0.0), None),
                param!("voices", "int", req, range: Some(1.0), Some(16.0)),
                param!("detune_cents", "float", req, range: Some(0.0), None),
                param!("spread", "float", req, range: Some(0.0), Some(1.0)),
                param!("detune_curve", "string", opt, "linear", enum: &["linear", "exp2"]),
            ],
            "A dict matching the Synthesis::SupersawUnison IR structure.",
            "supersaw_unison(440, 7, 20, 0.8)"
        ),
        func!(
            "wavetable",
            "audio.synthesis",
            "Creates a Wavetable synthesis block.",
            vec![
                param!("table", "string", req, enum: &["basic", "analog", "digital", "pwm", "formant", "organ"]),
                param!("frequency", "float", req, range: Some(0.0), None),
                param!("position", "float", opt, 0.0, range: Some(0.0), Some(1.0)),
                param!("position_end", "float", opt_none, range: Some(0.0), Some(1.0)),
                param!("voices", "int", opt_none, range: Some(1.0), Some(8.0)),
                param!("detune", "float", opt_none),
            ],
            "A dict matching the Synthesis::Wavetable IR structure.",
            r#"wavetable("analog", 440, 0.5)"#
        ),
        func!(
            "granular",
            "audio.synthesis",
            "Creates a Granular synthesis block.",
            vec![
                param!("source", "dict", req),
                param!("grain_size_ms", "float", req, range: Some(10.0), Some(500.0)),
                param!("grain_density", "float", req, range: Some(1.0), Some(100.0)),
                param!("pitch_spread", "float", opt, 0.0),
                param!("position_spread", "float", opt, 0.0, range: Some(0.0), Some(1.0)),
                param!("pan_spread", "float", opt, 0.0, range: Some(0.0), Some(1.0)),
            ],
            "A dict matching the Synthesis::Granular IR structure.",
            r#"granular(granular_source("noise", "white"), 50, 20)"#
        ),
        func!(
            "granular_source",
            "audio.synthesis",
            "Creates a granular source configuration.",
            vec![
                param!("source_type", "string", req, enum: &["noise", "tone", "formant"]),
                param!("param1", "string|float", req),
                param!("param2", "float", opt_none),
            ],
            "A dict matching the GranularSource IR structure.",
            r#"granular_source("tone", "sine", 440)"#
        ),
        // === AUDIO SYNTHESIS (Physical) ===
        func!(
            "membrane_drum",
            "audio.synthesis",
            "Creates a Membrane Drum physical modeling synthesis block.",
            vec![
                param!("frequency", "float", req, range: Some(0.0), None),
                param!("decay", "float", req, range: Some(0.0), Some(1.0)),
                param!("tone", "float", req, range: Some(0.0), Some(1.0)),
                param!("strike", "float", req, range: Some(0.0), Some(1.0)),
            ],
            "A dict matching the Synthesis::MembraneDrum IR structure.",
            "membrane_drum(100, 0.5, 0.3, 0.8)"
        ),
        func!(
            "modal",
            "audio.synthesis",
            "Creates a Modal synthesis block.",
            vec![
                param!("frequency", "float", req, range: Some(0.0), None),
                param!("modes", "list", req),
                param!("excitation", "string", opt, "impulse", enum: &["impulse", "noise", "pluck"]),
            ],
            "A dict matching the Synthesis::Modal IR structure.",
            "modal(440, [modal_mode(1.0, 1.0, 0.5)])"
        ),
        func!(
            "modal_mode",
            "audio.synthesis",
            "Creates a modal mode configuration.",
            vec![
                param!("freq_ratio", "float", req, range: Some(0.0), None),
                param!("amplitude", "float", req, range: Some(0.0), Some(1.0)),
                param!("decay_time", "float", req, range: Some(0.0), None),
            ],
            "A modal mode dict.",
            "modal_mode(1.0, 1.0, 0.5)"
        ),
        func!(
            "metallic",
            "audio.synthesis",
            "Creates a Metallic synthesis block.",
            vec![
                param!("base_freq", "float", req, range: Some(0.0), None),
                param!("num_partials", "int", req, range: Some(1.0), None),
                param!("inharmonicity", "float", req, range: Some(1.0), None),
            ],
            "A dict matching the Synthesis::Metallic IR structure.",
            "metallic(440, 8, 1.5)"
        ),
        // === AUDIO SYNTHESIS (Exotic) ===
        func!(
            "vocoder",
            "audio.synthesis",
            "Creates a Vocoder synthesis block.",
            vec![
                param!("carrier_freq", "float", req, range: Some(0.0), None),
                param!("carrier_type", "string", req, enum: &["sawtooth", "pulse", "noise"]),
                param!("num_bands", "int", req, range: Some(8.0), Some(32.0)),
                param!("band_spacing", "string", req, enum: &["linear", "logarithmic"]),
                param!("envelope_attack", "float", req, range: Some(0.0), None),
                param!("envelope_release", "float", req, range: Some(0.0), None),
                param!("formant_rate", "float", opt, 2.0, range: Some(0.0), None),
            ],
            "A dict matching the Synthesis::Vocoder IR structure."
        ),
        func!(
            "formant_synth",
            "audio.synthesis",
            "Creates a Formant synthesis block for vowel/voice sounds.",
            vec![
                param!("frequency", "float", req, range: Some(0.0), None),
                param!("vowel", "string", opt_none, enum: &["a", "i", "u", "e", "o"]),
                param!("vowel_morph", "string", opt_none, enum: &["a", "i", "u", "e", "o"]),
                param!("morph_amount", "float", opt, 0.0, range: Some(0.0), Some(1.0)),
                param!("breathiness", "float", opt, 0.0, range: Some(0.0), Some(1.0)),
            ],
            "A dict matching the Synthesis::Formant IR structure.",
            r#"formant_synth(frequency=220, vowel="a")"#
        ),
        func!(
            "waveguide",
            "audio.synthesis",
            "Creates a Waveguide synthesis block for wind/brass sounds.",
            vec![
                param!("frequency", "float", req, range: Some(0.0), None),
                param!("breath", "float", req, range: Some(0.0), Some(1.0)),
                param!("noise", "float", req, range: Some(0.0), Some(1.0)),
                param!("damping", "float", req, range: Some(0.0), Some(1.0)),
                param!("resonance", "float", req, range: Some(0.0), Some(1.0)),
            ],
            "A dict matching the Synthesis::Waveguide IR structure."
        ),
        func!(
            "bowed_string",
            "audio.synthesis",
            "Creates a Bowed String synthesis block for violin/cello sounds.",
            vec![
                param!("frequency", "float", req, range: Some(0.0), None),
                param!("bow_pressure", "float", req, range: Some(0.0), Some(1.0)),
                param!("bow_position", "float", req, range: Some(0.0), Some(1.0)),
                param!("damping", "float", req, range: Some(0.0), Some(1.0)),
            ],
            "A dict matching the Synthesis::BowedString IR structure."
        ),
        func!(
            "pulsar",
            "audio.synthesis",
            "Creates a Pulsar synthesis block.",
            vec![
                param!("frequency", "float", req, range: Some(0.0), None),
                param!("pulse_rate", "float", req, range: Some(0.0), None),
                param!("grain_size_ms", "float", req, range: Some(0.0), None),
                param!("shape", "string", req, enum: WAVEFORMS),
            ],
            "A dict matching the Synthesis::Pulsar IR structure."
        ),
        func!(
            "vosim",
            "audio.synthesis",
            "Creates a VOSIM synthesis block (voice simulation).",
            vec![
                param!("frequency", "float", req, range: Some(0.0), None),
                param!("formant_freq", "float", req, range: Some(0.0), None),
                param!("pulses", "int", req, range: Some(1.0), Some(16.0)),
                param!("breathiness", "float", opt, 0.0, range: Some(0.0), Some(1.0)),
            ],
            "A dict matching the Synthesis::Vosim IR structure."
        ),
        func!(
            "spectral_freeze",
            "audio.synthesis",
            "Creates a Spectral Freeze synthesis block.",
            vec![param!("source", "dict", req)],
            "A dict matching the Synthesis::SpectralFreeze IR structure.",
            r#"spectral_freeze(source=spectral_source("noise", "pink"))"#
        ),
        func!(
            "pitched_body",
            "audio.synthesis",
            "Creates a Pitched Body synthesis block.",
            vec![
                param!("start_freq", "float", req, range: Some(0.0), None),
                param!("end_freq", "float", req, range: Some(0.0), None),
            ],
            "A dict matching the Synthesis::PitchedBody IR structure.",
            "pitched_body(start_freq=880, end_freq=110)"
        ),
        // === AUDIO FILTERS ===
        func!(
            "lowpass",
            "audio.filters",
            "Creates a lowpass filter configuration.",
            vec![
                param!("cutoff", "float", req, range: Some(0.0), None),
                param!("resonance", "float", opt, 0.707, range: Some(0.0), None),
                param!("sweep_to", "float", opt_none, range: Some(0.0), None),
            ],
            "A filter dict.",
            "lowpass(5000, 0.707)"
        ),
        func!(
            "highpass",
            "audio.filters",
            "Creates a highpass filter configuration.",
            vec![
                param!("cutoff", "float", req, range: Some(0.0), None),
                param!("resonance", "float", opt, 0.707, range: Some(0.0), None),
            ],
            "A filter dict.",
            "highpass(200)"
        ),
        func!(
            "bandpass",
            "audio.filters",
            "Creates a bandpass filter configuration.",
            vec![
                param!("center", "float", req, range: Some(0.0), None),
                param!("bandwidth", "float", req, range: Some(0.0), None),
            ],
            "A filter dict.",
            "bandpass(1000, 200)"
        ),
        // === AUDIO EFFECTS ===
        func!(
            "reverb",
            "audio.effects",
            "Creates a reverb effect configuration.",
            vec![
                param!("decay", "float", opt, 0.5),
                param!("wet", "float", opt, 0.3, range: Some(0.0), Some(1.0)),
                param!("room_size", "float", opt, 0.8, range: Some(0.0), Some(1.0)),
                param!("width", "float", opt, 1.0, range: Some(0.0), Some(1.0)),
            ],
            "An effect dict.",
            "reverb(0.5, 0.3)"
        ),
        func!(
            "delay",
            "audio.effects",
            "Creates a delay effect configuration.",
            vec![
                param!("time_ms", "float", opt, 250.0, range: Some(0.0), None),
                param!("feedback", "float", opt, 0.4, range: Some(0.0), Some(1.0)),
                param!("wet", "float", opt, 0.3, range: Some(0.0), Some(1.0)),
                param!("ping_pong", "bool", opt, false),
            ],
            "An effect dict.",
            "delay(500, 0.5, 0.4)"
        ),
        func!(
            "compressor",
            "audio.effects",
            "Creates a compressor effect configuration.",
            vec![
                param!("threshold_db", "float", opt, -12.0),
                param!("ratio", "float", opt, 4.0, range: Some(0.0), None),
                param!("attack_ms", "float", opt, 10.0, range: Some(0.0), None),
                param!("release_ms", "float", opt, 100.0, range: Some(0.0), None),
                param!("makeup_db", "float", opt, 0.0),
            ],
            "An effect dict.",
            "compressor(-18, 6, 5, 50)"
        ),
        func!(
            "chorus",
            "audio.effects",
            "Creates a chorus effect configuration.",
            vec![
                param!("rate", "float", req, range: Some(0.0), None),
                param!("depth", "float", req, range: Some(0.0), Some(1.0)),
                param!("wet", "float", req, range: Some(0.0), Some(1.0)),
                param!("voices", "int", opt, 2, range: Some(1.0), Some(4.0)),
            ],
            "An effect dict.",
            "chorus(1.5, 0.3, 0.25)"
        ),
        func!(
            "bitcrush",
            "audio.effects",
            "Creates a bitcrusher effect configuration.",
            vec![
                param!("bits", "int", req, range: Some(1.0), Some(16.0)),
                param!("sample_rate_reduction", "float", opt, 1.0, range: Some(1.0), None),
            ],
            "An effect dict.",
            "bitcrush(8, 4.0)"
        ),
        // === AUDIO MODULATION ===
        func!(
            "lfo",
            "audio.modulation",
            "Creates an LFO configuration.",
            vec![
                param!("rate", "float", req, range: Some(0.0), None),
                param!("depth", "float", req, range: Some(0.0), Some(1.0)),
                param!("waveform", "string", opt, "sine", enum: &["sine", "triangle", "square", "sawtooth", "random"]),
            ],
            "An LFO dict.",
            r#"lfo(5.0, 0.3, "sine")"#
        ),
        func!(
            "pitch_envelope",
            "audio.modulation",
            "Creates a pitch envelope configuration.",
            vec![
                param!("start_semitones", "float", req),
                param!("end_semitones", "float", req),
                param!("time", "float", req, range: Some(0.0), None),
                param!("curve", "string", opt, "linear", enum: &["linear", "exponential"]),
            ],
            "A pitch envelope dict.",
            r#"pitch_envelope(12, 0, 0.1, "exponential")"#
        ),
        // === AUDIO LAYERS ===
        func!(
            "audio_layer",
            "audio.layers",
            "Creates a complete audio synthesis layer.",
            vec![
                param!("synthesis", "dict", req),
                param!("envelope", "dict", req),
                param!("filter", "dict", opt_none),
                param!("volume", "float", opt, 1.0, range: Some(0.0), Some(1.0)),
                param!("pan", "float", opt, 0.0, range: Some(-1.0), Some(1.0)),
            ],
            "A layer dict.",
            "audio_layer(oscillator(440), envelope(0.01, 0.1, 0.7, 0.2))"
        ),
    ]
}
