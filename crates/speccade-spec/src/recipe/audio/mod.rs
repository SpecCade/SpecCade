//! Unified audio recipe types.
//!
//! This module consolidates audio SFX and instrument types into a unified structure.
//!
//! ## Structure
//!
//! - `synthesis` - Shared synthesis types (waveforms, envelopes, filters, etc.)
//!
//! ## Recipe Kind
//!
//! - `audio_v1` - Unified audio recipe with layered synthesis

pub mod synthesis;

use serde::{Deserialize, Serialize};

// Re-export synthesis types
pub use synthesis::{
    midi_to_frequency, parse_note_name, Envelope, Filter, FreqSweep, NoiseType, NoteSpec,
    OscillatorConfig, PitchEnvelope, SweepCurve, Synthesis, Waveform,
};

/// A single synthesis layer in an audio recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

/// Parameters for the `audio_v1` unified audio recipe.
///
/// This is the unified audio recipe type, consolidating SFX and instrument approaches.
///
/// ## Base Note Semantics
///
/// The `base_note` field tells the tracker what pitch this sample is recorded at.
/// The tracker uses this for transposition when playing pattern notes:
///
/// - `base_note: None`, pattern note: None → play tracker's native base (C5 for IT, C4 for XM)
/// - `base_note: Some(note)`, pattern note: None → play the sample's base note
/// - `base_note: None`, pattern note: Some → play tracker's native base
/// - `base_note: Some(note)`, pattern note: Some → play the pattern note (transposed relative to base)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioV1Params {
    /// Base note this sample is tuned to (MIDI note or note name like "C4").
    /// If None, the tracker uses its native default (C5 for IT, C4 for XM).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_note: Option<NoteSpec>,
    /// Duration of the audio in seconds.
    pub duration_seconds: f64,
    /// Sample rate in Hz (22050, 44100, or 48000).
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    /// Synthesis layers to combine.
    pub layers: Vec<AudioLayer>,
    /// Optional pitch envelope for frequency modulation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch_envelope: Option<PitchEnvelope>,
    /// Whether to generate loop points.
    #[serde(default)]
    pub generate_loop_points: bool,
    /// Optional master filter applied after mixing all layers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub master_filter: Option<Filter>,
}

fn default_sample_rate() -> u32 {
    44100
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // AudioLayer Tests
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
    // AudioV1Params Tests
    // ========================================================================

    #[test]
    fn test_audio_v1_params_defaults() {
        let params = AudioV1Params {
            base_note: None,
            duration_seconds: 1.0,
            sample_rate: default_sample_rate(),
            layers: vec![],
            pitch_envelope: None,
            generate_loop_points: false,
            master_filter: None,
        };

        assert_eq!(params.base_note, None);
        assert_eq!(params.sample_rate, 44100);
        assert!(!params.generate_loop_points);
    }

    #[test]
    fn test_audio_v1_params_with_base_note() {
        let params = AudioV1Params {
            base_note: Some(NoteSpec::NoteName("A4".to_string())),
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
            pitch_envelope: None,
            generate_loop_points: false,
            master_filter: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("base_note"));
        let parsed: AudioV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn test_audio_v1_params_without_base_note() {
        // When base_note is None, it should be omitted from JSON
        let params = AudioV1Params {
            base_note: None,
            duration_seconds: 0.5,
            sample_rate: 44100,
            layers: vec![],
            pitch_envelope: None,
            generate_loop_points: false,
            master_filter: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(!json.contains("base_note"));
        let parsed: AudioV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn test_audio_v1_params_with_pitch_envelope() {
        let params = AudioV1Params {
            base_note: Some(NoteSpec::MidiNote(60)),
            duration_seconds: 1.5,
            sample_rate: 48000,
            layers: vec![AudioLayer {
                synthesis: Synthesis::KarplusStrong {
                    frequency: 440.0,
                    decay: 0.996,
                    blend: 0.7,
                },
                envelope: Envelope::default(),
                volume: 1.0,
                pan: 0.0,
                delay: None,
            }],
            pitch_envelope: Some(PitchEnvelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.5,
                release: 0.2,
                depth: 2.0,
            }),
            generate_loop_points: true,
            master_filter: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("pitch_envelope"));
        assert!(json.contains("generate_loop_points"));
        let parsed: AudioV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn test_audio_v1_params_with_master_filter() {
        let params = AudioV1Params {
            base_note: Some(NoteSpec::NoteName("C4".to_string())),
            duration_seconds: 1.0,
            sample_rate: 22050,
            layers: vec![],
            pitch_envelope: None,
            generate_loop_points: false,
            master_filter: Some(Filter::Lowpass {
                cutoff: 2000.0,
                resonance: 0.707,
                cutoff_end: None,
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("master_filter"));
        let parsed: AudioV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn test_complex_multi_layer_sound() {
        let params = AudioV1Params {
            base_note: Some(NoteSpec::NoteName("D4".to_string())),
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
            pitch_envelope: Some(PitchEnvelope {
                attack: 0.02,
                decay: 0.1,
                sustain: 0.0,
                release: 0.0,
                depth: 3.0,
            }),
            generate_loop_points: false,
            master_filter: Some(Filter::Highpass {
                cutoff: 100.0,
                resonance: 0.5,
                cutoff_end: None,
            }),
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: AudioV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn test_multi_oscillator_instrument_style() {
        let params = AudioV1Params {
            base_note: Some(NoteSpec::NoteName("A3".to_string())),
            duration_seconds: 2.0,
            sample_rate: 44100,
            layers: vec![AudioLayer {
                synthesis: Synthesis::MultiOscillator {
                    frequency: 440.0,
                    oscillators: vec![
                        OscillatorConfig {
                            waveform: Waveform::Sawtooth,
                            volume: 1.0,
                            detune: Some(0.0),
                            phase: None,
                            duty: None,
                        },
                        OscillatorConfig {
                            waveform: Waveform::Sawtooth,
                            volume: 0.8,
                            detune: Some(5.0),
                            phase: None,
                            duty: None,
                        },
                    ],
                    freq_sweep: None,
                },
                envelope: Envelope {
                    attack: 0.05,
                    decay: 0.2,
                    sustain: 0.7,
                    release: 0.3,
                },
                volume: 1.0,
                pan: 0.0,
                delay: None,
            }],
            pitch_envelope: None,
            generate_loop_points: true,
            master_filter: None,
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: AudioV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn test_deny_unknown_fields_audio_layer() {
        let json = r#"{
            "synthesis": {
                "type": "oscillator",
                "waveform": "sine",
                "frequency": 440.0
            },
            "envelope": {
                "attack": 0.01,
                "decay": 0.1,
                "sustain": 0.5,
                "release": 0.2
            },
            "volume": 0.8,
            "pan": 0.0,
            "unknown_field": "should_fail"
        }"#;

        let result: Result<AudioLayer, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deny_unknown_fields_audio_v1_params() {
        let json = r#"{
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [],
            "unknown_field": "should_fail"
        }"#;

        let result: Result<AudioV1Params, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
