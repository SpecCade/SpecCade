//! Audio SFX recipe types.

pub mod common;
pub mod envelope;
pub mod filter;
pub mod layer;
pub mod noise;
pub mod oscillators;
pub mod synthesis;

pub use common::{FreqSweep, SweepCurve, Waveform};
pub use envelope::{Envelope, PitchEnvelope};
pub use filter::Filter;
pub use layer::AudioLayer;
pub use noise::NoiseType;
pub use oscillators::OscillatorConfig;
pub use synthesis::Synthesis;

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
