//! Audio SFX recipe types.

use serde::{Deserialize, Serialize};

/// Parameters for the `audio_sfx.layered_synth_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSfxLayeredSynthV1Params {
    /// Duration of the sound effect in seconds.
    pub duration_seconds: f64,
    /// Sample rate in Hz (22050, 44100, or 48000).
    pub sample_rate: u32,
    /// Synthesis layers to combine.
    pub layers: Vec<AudioLayer>,
    /// Optional master filter applied after mixing all layers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub master_filter: Option<Filter>,
}

/// A single synthesis layer in an audio SFX.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioLayer {
    /// Synthesis parameters.
    pub synthesis: Synthesis,
    /// ADSR envelope.
    pub envelope: Envelope,
    /// Volume level (0.0 to 1.0).
    pub volume: f64,
    /// Stereo pan (-1.0 = left, 0.0 = center, 1.0 = right).
    pub pan: f64,
    /// Layer start delay in seconds (default: 0.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay: Option<f64>,
}

/// Synthesis type configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Synthesis {
    /// FM synthesis.
    FmSynth {
        /// Carrier frequency in Hz.
        carrier_freq: f64,
        /// Modulator frequency in Hz.
        modulator_freq: f64,
        /// Modulation index.
        modulation_index: f64,
        /// Optional frequency sweep.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
    },
    /// Karplus-Strong plucked string synthesis.
    KarplusStrong {
        /// Base frequency in Hz.
        frequency: f64,
        /// Decay factor (0.0 to 1.0).
        decay: f64,
        /// Blend factor for the lowpass filter.
        blend: f64,
    },
    /// Noise burst.
    NoiseBurst {
        /// Type of noise.
        noise_type: NoiseType,
        /// Optional filter.
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<Filter>,
    },
    /// Additive synthesis with multiple harmonics.
    Additive {
        /// Base frequency in Hz.
        base_freq: f64,
        /// Harmonic amplitudes (index 0 = fundamental).
        harmonics: Vec<f64>,
    },
    /// Simple waveform oscillator.
    Oscillator {
        /// Waveform type.
        waveform: Waveform,
        /// Frequency in Hz.
        frequency: f64,
        /// Optional frequency sweep.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
        /// Detune amount in cents (100 cents = 1 semitone).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detune: Option<f64>,
        /// Duty cycle for square/pulse waves (0.0 to 1.0, default 0.5).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duty: Option<f64>,
    },
    /// Multi-oscillator stack (subtractive synthesis).
    MultiOscillator {
        /// Base frequency in Hz.
        frequency: f64,
        /// Stack of oscillators to mix additively.
        oscillators: Vec<OscillatorConfig>,
        /// Optional frequency sweep applied to all oscillators.
        #[serde(skip_serializing_if = "Option::is_none")]
        freq_sweep: Option<FreqSweep>,
    },
    /// Pitched body synthesis (impact sounds with frequency sweep).
    PitchedBody {
        /// Starting frequency in Hz.
        start_freq: f64,
        /// Ending frequency in Hz.
        end_freq: f64,
    },
    /// Metallic synthesis with inharmonic partials.
    Metallic {
        /// Base frequency in Hz.
        base_freq: f64,
        /// Number of inharmonic partials.
        num_partials: usize,
        /// Inharmonicity factor (1.0 = harmonic, >1.0 = increasingly inharmonic).
        inharmonicity: f64,
    },
}

/// Frequency sweep parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreqSweep {
    /// Target frequency at end of sweep.
    pub end_freq: f64,
    /// Sweep curve type.
    pub curve: SweepCurve,
}

/// Sweep curve type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SweepCurve {
    /// Linear interpolation.
    Linear,
    /// Exponential interpolation.
    Exponential,
    /// Logarithmic interpolation.
    Logarithmic,
}

/// Noise type for noise-based synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseType {
    /// White noise (equal energy per frequency).
    White,
    /// Pink noise (1/f spectrum).
    Pink,
    /// Brown noise (1/f^2 spectrum).
    Brown,
}

/// Basic waveform types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Waveform {
    /// Sine wave.
    Sine,
    /// Square wave.
    Square,
    /// Sawtooth wave.
    Sawtooth,
    /// Triangle wave.
    Triangle,
    /// Pulse wave with variable duty cycle.
    Pulse,
}

/// Pitch envelope for modulating frequency over time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PitchEnvelope {
    /// Attack time in seconds.
    pub attack: f64,
    /// Decay time in seconds.
    pub decay: f64,
    /// Sustain level (0.0 to 1.0).
    pub sustain: f64,
    /// Release time in seconds.
    pub release: f64,
    /// Pitch depth in semitones (can be positive or negative).
    pub depth: f64,
}

impl Default for PitchEnvelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
            depth: 0.0,
        }
    }
}

/// Configuration for a single oscillator in a multi-oscillator stack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OscillatorConfig {
    /// Waveform type.
    pub waveform: Waveform,
    /// Volume/amplitude of this oscillator (0.0 to 1.0).
    #[serde(default = "default_oscillator_volume")]
    pub volume: f64,
    /// Detune amount in cents (100 cents = 1 semitone).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detune: Option<f64>,
    /// Phase offset in radians (0 to 2*PI).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<f64>,
    /// Duty cycle for square/pulse waves (0.0 to 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duty: Option<f64>,
}

fn default_oscillator_volume() -> f64 {
    1.0
}

/// Filter configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Filter {
    /// Low-pass filter.
    Lowpass {
        /// Cutoff frequency in Hz.
        cutoff: f64,
        /// Resonance (Q factor).
        resonance: f64,
        /// Optional target cutoff frequency for sweep.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cutoff_end: Option<f64>,
    },
    /// High-pass filter.
    Highpass {
        /// Cutoff frequency in Hz.
        cutoff: f64,
        /// Resonance (Q factor).
        resonance: f64,
        /// Optional target cutoff frequency for sweep.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cutoff_end: Option<f64>,
    },
    /// Band-pass filter.
    Bandpass {
        /// Center frequency in Hz.
        center: f64,
        /// Bandwidth in Hz.
        bandwidth: f64,
        /// Resonance (Q factor).
        resonance: f64,
        /// Optional target center frequency for sweep.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        center_end: Option<f64>,
    },
}

/// ADSR envelope parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    /// Attack time in seconds.
    pub attack: f64,
    /// Decay time in seconds.
    pub decay: f64,
    /// Sustain level (0.0 to 1.0).
    pub sustain: f64,
    /// Release time in seconds.
    pub release: f64,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Top-Level AudioSfxLayeredSynthV1Params Tests
    // ========================================================================

    #[test]
    fn test_audio_sfx_params_serde() {
        let params = AudioSfxLayeredSynthV1Params {
            duration_seconds: 2.5,
            sample_rate: 44100,
            layers: vec![AudioLayer {
                synthesis: Synthesis::Oscillator {
                    waveform: Waveform::Sine,
                    frequency: 440.0,
                    freq_sweep: None,
                    detune: None,
                    duty: None,
                },
                envelope: Envelope::default(),
                volume: 0.8,
                pan: 0.0,
                delay: Some(0.1),
            }],
            master_filter: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        let parsed: AudioSfxLayeredSynthV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn test_audio_sfx_params_with_master_filter() {
        let params = AudioSfxLayeredSynthV1Params {
            duration_seconds: 1.0,
            sample_rate: 22050,
            layers: vec![],
            master_filter: Some(Filter::Lowpass {
                cutoff: 2000.0,
                resonance: 0.707,
                cutoff_end: None,
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        let parsed: AudioSfxLayeredSynthV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    // ========================================================================
    // AudioLayer Tests (amplitude, delay, envelope, volume, pan)
    // ========================================================================

    #[test]
    fn test_audio_layer_serde() {
        let layer = AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Square,
                frequency: 880.0,
                freq_sweep: None,
                detune: None,
                duty: Some(0.5),
            },
            envelope: Envelope {
                attack: 0.02,
                decay: 0.15,
                sustain: 0.6,
                release: 0.3,
            },
            volume: 0.75,
            pan: -0.5,
            delay: Some(0.25),
        };

        let json = serde_json::to_string(&layer).unwrap();
        let parsed: AudioLayer = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, layer);
    }

    #[test]
    fn test_audio_layer_no_delay() {
        let layer = AudioLayer {
            synthesis: Synthesis::Oscillator {
                waveform: Waveform::Sine,
                frequency: 440.0,
                freq_sweep: None,
                detune: None,
                duty: None,
            },
            envelope: Envelope::default(),
            volume: 1.0,
            pan: 0.0,
            delay: None,
        };

        let json = serde_json::to_string(&layer).unwrap();
        assert!(!json.contains("delay"));
        let parsed: AudioLayer = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, layer);
    }

    // ========================================================================
    // Synthesis Type Tests
    // ========================================================================

    #[test]
    fn test_synthesis_oscillator_sine() {
        let osc = Synthesis::Oscillator {
            waveform: Waveform::Sine,
            frequency: 440.0,
            freq_sweep: None,
            detune: None,
            duty: None,
        };

        let json = serde_json::to_string(&osc).unwrap();
        assert!(json.contains("oscillator"));
        assert!(json.contains("sine"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, osc);
    }

    #[test]
    fn test_synthesis_oscillator_square_with_duty() {
        let osc = Synthesis::Oscillator {
            waveform: Waveform::Square,
            frequency: 220.0,
            freq_sweep: None,
            detune: None,
            duty: Some(0.25),
        };

        let json = serde_json::to_string(&osc).unwrap();
        assert!(json.contains("duty"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, osc);
    }

    #[test]
    fn test_synthesis_oscillator_with_freq_sweep() {
        let osc = Synthesis::Oscillator {
            waveform: Waveform::Sawtooth,
            frequency: 880.0,
            freq_sweep: Some(FreqSweep {
                end_freq: 110.0,
                curve: SweepCurve::Linear,
            }),
            detune: Some(5.0),
            duty: None,
        };

        let json = serde_json::to_string(&osc).unwrap();
        assert!(json.contains("freq_sweep"));
        assert!(json.contains("detune"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, osc);
    }

    #[test]
    fn test_synthesis_fm_serde() {
        let fm = Synthesis::FmSynth {
            carrier_freq: 440.0,
            modulator_freq: 880.0,
            modulation_index: 2.5,
            freq_sweep: Some(FreqSweep {
                end_freq: 110.0,
                curve: SweepCurve::Exponential,
            }),
        };

        let json = serde_json::to_string(&fm).unwrap();
        assert!(json.contains("fm_synth"));
        assert!(json.contains("carrier_freq"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, fm);
    }

    #[test]
    fn test_synthesis_karplus_strong() {
        let ks = Synthesis::KarplusStrong {
            frequency: 220.0,
            decay: 0.996,
            blend: 0.7,
        };

        let json = serde_json::to_string(&ks).unwrap();
        assert!(json.contains("karplus_strong"));
        assert!(json.contains("decay"));
        assert!(json.contains("blend"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ks);
    }

    #[test]
    fn test_synthesis_noise_burst_white() {
        let noise = Synthesis::NoiseBurst {
            noise_type: NoiseType::White,
            filter: None,
        };

        let json = serde_json::to_string(&noise).unwrap();
        assert!(json.contains("noise_burst"));
        assert!(json.contains("white"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, noise);
    }

    #[test]
    fn test_synthesis_noise_burst_pink_with_filter() {
        let noise = Synthesis::NoiseBurst {
            noise_type: NoiseType::Pink,
            filter: Some(Filter::Lowpass {
                cutoff: 1000.0,
                resonance: 0.5,
                cutoff_end: None,
            }),
        };

        let json = serde_json::to_string(&noise).unwrap();
        assert!(json.contains("pink"));
        assert!(json.contains("filter"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, noise);
    }

    #[test]
    fn test_synthesis_noise_burst_brown() {
        let noise = Synthesis::NoiseBurst {
            noise_type: NoiseType::Brown,
            filter: None,
        };

        let json = serde_json::to_string(&noise).unwrap();
        assert!(json.contains("brown"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, noise);
    }

    #[test]
    fn test_synthesis_additive() {
        let additive = Synthesis::Additive {
            base_freq: 440.0,
            harmonics: vec![1.0, 0.5, 0.25, 0.125],
        };

        let json = serde_json::to_string(&additive).unwrap();
        assert!(json.contains("additive"));
        assert!(json.contains("harmonics"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, additive);
    }

    #[test]
    fn test_synthesis_multi_oscillator() {
        let multi = Synthesis::MultiOscillator {
            frequency: 440.0,
            oscillators: vec![
                OscillatorConfig {
                    waveform: Waveform::Sawtooth,
                    volume: 1.0,
                    detune: None,
                    phase: None,
                    duty: None,
                },
                OscillatorConfig {
                    waveform: Waveform::Square,
                    volume: 0.5,
                    detune: Some(5.0),
                    phase: Some(1.57),
                    duty: Some(0.5),
                },
            ],
            freq_sweep: None,
        };

        let json = serde_json::to_string(&multi).unwrap();
        assert!(json.contains("multi_oscillator"));
        assert!(json.contains("oscillators"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, multi);
    }

    #[test]
    fn test_synthesis_pitched_body() {
        let pitched = Synthesis::PitchedBody {
            start_freq: 200.0,
            end_freq: 50.0,
        };

        let json = serde_json::to_string(&pitched).unwrap();
        assert!(json.contains("pitched_body"));
        assert!(json.contains("start_freq"));
        assert!(json.contains("end_freq"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, pitched);
    }

    #[test]
    fn test_synthesis_metallic() {
        let metallic = Synthesis::Metallic {
            base_freq: 800.0,
            num_partials: 6,
            inharmonicity: 1.414,
        };

        let json = serde_json::to_string(&metallic).unwrap();
        assert!(json.contains("metallic"));
        assert!(json.contains("base_freq"));
        assert!(json.contains("num_partials"));
        assert!(json.contains("inharmonicity"));

        let parsed: Synthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, metallic);
    }

    // ========================================================================
    // Envelope Tests (ADSR)
    // ========================================================================

    #[test]
    fn test_envelope_default() {
        let env = Envelope::default();
        assert_eq!(env.attack, 0.01);
        assert_eq!(env.decay, 0.1);
        assert_eq!(env.sustain, 0.5);
        assert_eq!(env.release, 0.2);
    }

    #[test]
    fn test_envelope_custom_serde() {
        let env = Envelope {
            attack: 0.05,
            decay: 0.2,
            sustain: 0.7,
            release: 0.3,
        };

        let json = serde_json::to_string(&env).unwrap();
        let parsed: Envelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, env);
    }

    // ========================================================================
    // Filter Tests (lowpass, highpass, bandpass, cutoff sweeps)
    // ========================================================================

    #[test]
    fn test_filter_lowpass_basic() {
        let filter = Filter::Lowpass {
            cutoff: 2000.0,
            resonance: 0.707,
            cutoff_end: None,
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("lowpass"));
        assert!(json.contains("2000"));
        assert!(json.contains("0.707"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    #[test]
    fn test_filter_lowpass_with_sweep() {
        let filter = Filter::Lowpass {
            cutoff: 5000.0,
            resonance: 1.0,
            cutoff_end: Some(500.0),
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("cutoff_end"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    #[test]
    fn test_filter_highpass() {
        let filter = Filter::Highpass {
            cutoff: 2000.0,
            resonance: 0.5,
            cutoff_end: None,
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("highpass"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    #[test]
    fn test_filter_highpass_with_sweep() {
        let filter = Filter::Highpass {
            cutoff: 100.0,
            resonance: 0.8,
            cutoff_end: Some(2000.0),
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("cutoff_end"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    #[test]
    fn test_filter_bandpass() {
        let filter = Filter::Bandpass {
            center: 1000.0,
            bandwidth: 500.0,
            resonance: 0.707,
            center_end: None,
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("bandpass"));
        assert!(json.contains("center"));
        assert!(json.contains("bandwidth"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    #[test]
    fn test_filter_bandpass_with_sweep() {
        let filter = Filter::Bandpass {
            center: 2000.0,
            bandwidth: 300.0,
            resonance: 1.2,
            center_end: Some(500.0),
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("center_end"));

        let parsed: Filter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    // ========================================================================
    // Waveform Tests
    // ========================================================================

    #[test]
    fn test_waveform_serde() {
        let waveforms = vec![
            Waveform::Sine,
            Waveform::Square,
            Waveform::Sawtooth,
            Waveform::Triangle,
            Waveform::Pulse,
        ];

        for waveform in waveforms {
            let json = serde_json::to_string(&waveform).unwrap();
            let parsed: Waveform = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, waveform);
        }
    }

    // ========================================================================
    // NoiseType Tests
    // ========================================================================

    #[test]
    fn test_noise_type_serde() {
        let noise_types = vec![NoiseType::White, NoiseType::Pink, NoiseType::Brown];

        for noise_type in noise_types {
            let json = serde_json::to_string(&noise_type).unwrap();
            let parsed: NoiseType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, noise_type);
        }
    }

    // ========================================================================
    // FreqSweep Tests
    // ========================================================================

    #[test]
    fn test_freq_sweep_linear() {
        let sweep = FreqSweep {
            end_freq: 220.0,
            curve: SweepCurve::Linear,
        };

        let json = serde_json::to_string(&sweep).unwrap();
        assert!(json.contains("linear"));

        let parsed: FreqSweep = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, sweep);
    }

    #[test]
    fn test_freq_sweep_exponential() {
        let sweep = FreqSweep {
            end_freq: 880.0,
            curve: SweepCurve::Exponential,
        };

        let json = serde_json::to_string(&sweep).unwrap();
        assert!(json.contains("exponential"));

        let parsed: FreqSweep = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, sweep);
    }

    #[test]
    fn test_freq_sweep_logarithmic() {
        let sweep = FreqSweep {
            end_freq: 110.0,
            curve: SweepCurve::Logarithmic,
        };

        let json = serde_json::to_string(&sweep).unwrap();
        assert!(json.contains("logarithmic"));

        let parsed: FreqSweep = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, sweep);
    }

    // ========================================================================
    // OscillatorConfig Tests (volume, detune, phase, duty)
    // ========================================================================

    #[test]
    fn test_oscillator_config_default_volume() {
        let osc = OscillatorConfig {
            waveform: Waveform::Sine,
            volume: default_oscillator_volume(),
            detune: None,
            phase: None,
            duty: None,
        };

        assert_eq!(osc.volume, 1.0);
    }

    #[test]
    fn test_oscillator_config_with_all_fields() {
        let osc = OscillatorConfig {
            waveform: Waveform::Square,
            volume: 0.75,
            detune: Some(10.0),
            phase: Some(3.14),
            duty: Some(0.3),
        };

        let json = serde_json::to_string(&osc).unwrap();
        assert!(json.contains("detune"));
        assert!(json.contains("phase"));
        assert!(json.contains("duty"));

        let parsed: OscillatorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, osc);
    }

    // ========================================================================
    // PitchEnvelope Tests
    // ========================================================================

    #[test]
    fn test_pitch_envelope_default() {
        let env = PitchEnvelope::default();
        assert_eq!(env.attack, 0.01);
        assert_eq!(env.decay, 0.1);
        assert_eq!(env.sustain, 0.5);
        assert_eq!(env.release, 0.2);
        assert_eq!(env.depth, 0.0);
    }

    #[test]
    fn test_pitch_envelope_custom_serde() {
        let env = PitchEnvelope {
            attack: 0.02,
            decay: 0.15,
            sustain: 0.7,
            release: 0.25,
            depth: 12.0,
        };

        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("depth"));

        let parsed: PitchEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, env);
    }

    // ========================================================================
    // Integration Tests: Complex Multi-Layer Sounds
    // ========================================================================

    #[test]
    fn test_complex_multi_layer_sound() {
        let params = AudioSfxLayeredSynthV1Params {
            duration_seconds: 1.5,
            sample_rate: 44100,
            layers: vec![
                AudioLayer {
                    synthesis: Synthesis::Oscillator {
                        waveform: Waveform::Sine,
                        frequency: 440.0,
                        freq_sweep: Some(FreqSweep {
                            end_freq: 220.0,
                            curve: SweepCurve::Exponential,
                        }),
                        detune: None,
                        duty: None,
                    },
                    envelope: Envelope {
                        attack: 0.01,
                        decay: 0.2,
                        sustain: 0.6,
                        release: 0.3,
                    },
                    volume: 0.8,
                    pan: -0.3,
                    delay: None,
                },
                AudioLayer {
                    synthesis: Synthesis::NoiseBurst {
                        noise_type: NoiseType::White,
                        filter: Some(Filter::Lowpass {
                            cutoff: 5000.0,
                            resonance: 0.707,
                            cutoff_end: Some(500.0),
                        }),
                    },
                    envelope: Envelope {
                        attack: 0.001,
                        decay: 0.05,
                        sustain: 0.0,
                        release: 0.1,
                    },
                    volume: 0.4,
                    pan: 0.0,
                    delay: Some(0.05),
                },
            ],
            master_filter: Some(Filter::Highpass {
                cutoff: 100.0,
                resonance: 0.5,
                cutoff_end: None,
            }),
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: AudioSfxLayeredSynthV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }
}
