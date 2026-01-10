//! Audio instrument recipe types.

use serde::{Deserialize, Serialize};

use super::audio_sfx::{Envelope, OscillatorConfig, PitchEnvelope, Synthesis, Waveform};

/// Parameters for the `audio_instrument.synth_patch_v1` recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioInstrumentSynthPatchV1Params {
    /// Duration of the instrument sample in seconds.
    #[serde(default = "default_note_duration")]
    pub note_duration_seconds: f64,
    /// Sample rate in Hz (22050, 44100, or 48000).
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    /// Synthesis configuration.
    #[serde(flatten)]
    pub synthesis: InstrumentSynthesis,
    /// ADSR envelope.
    pub envelope: Envelope,
    /// Optional pitch envelope for frequency modulation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch_envelope: Option<PitchEnvelope>,
    /// Optional notes to generate (MIDI note numbers or note names).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<NoteSpec>>,
    /// Whether to generate loop points.
    #[serde(default)]
    pub generate_loop_points: bool,
}

/// Synthesis configuration for instruments (supports both simple and advanced FM).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InstrumentSynthesis {
    /// Advanced FM synthesis with multiple operators.
    FmOperators {
        /// FM operators configuration.
        operators: Vec<FmOperator>,
    },
    /// Simple synthesis types (reuses audio_sfx Synthesis).
    Simple {
        /// Simple synthesis configuration.
        synthesis: Synthesis,
    },
}

/// FM operator configuration for advanced FM synthesis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FmOperator {
    /// Operator waveform.
    #[serde(default = "default_fm_waveform")]
    pub waveform: Waveform,
    /// Frequency ratio relative to fundamental (e.g., 1.0, 2.0, 3.5).
    #[serde(default = "default_fm_ratio")]
    pub ratio: f64,
    /// Amplitude level (0.0 to 1.0).
    #[serde(default = "default_fm_level")]
    pub level: f64,
    /// Modulation index (depth of modulation).
    #[serde(default)]
    pub modulation_index: f64,
    /// Optional envelope for this operator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub envelope: Option<Envelope>,
    /// Detune in cents.
    #[serde(default)]
    pub detune: f64,
}

fn default_fm_waveform() -> Waveform {
    Waveform::Sine
}

fn default_fm_ratio() -> f64 {
    1.0
}

fn default_fm_level() -> f64 {
    1.0
}

fn default_note_duration() -> f64 {
    1.0
}

fn default_sample_rate() -> u32 {
    44100
}

/// Note specification for multi-note instrument samples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NoteSpec {
    /// MIDI note number (0-127).
    MidiNote(u8),
    /// Note name (e.g., "C4", "A#3").
    NoteName(String),
}

impl NoteSpec {
    /// Converts the note spec to a MIDI note number.
    pub fn to_midi_note(&self) -> Option<u8> {
        match self {
            NoteSpec::MidiNote(n) => Some(*n),
            NoteSpec::NoteName(name) => parse_note_name(name),
        }
    }

    /// Converts the note spec to a frequency in Hz.
    pub fn to_frequency(&self) -> Option<f64> {
        self.to_midi_note().map(midi_to_frequency)
    }
}

/// Converts a MIDI note number to frequency in Hz.
pub fn midi_to_frequency(midi_note: u8) -> f64 {
    440.0 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0)
}

/// Parses a note name (e.g., "C4", "A#3", "Bb5") to a MIDI note number.
pub fn parse_note_name(name: &str) -> Option<u8> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    let mut chars = name.chars();
    let note_letter = chars.next()?.to_ascii_uppercase();

    let base_semitone = match note_letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };

    let rest: String = chars.collect();
    let (accidental_offset, octave_str) = if let Some(stripped) = rest.strip_prefix('#') {
        (1i32, stripped)
    } else if let Some(stripped) = rest.strip_prefix('s') {
        (1i32, stripped)
    } else if let Some(stripped) = rest.strip_prefix('b') {
        (-1i32, stripped)
    } else {
        (0i32, rest.as_str())
    };

    let octave: i32 = octave_str.parse().ok()?;

    // MIDI note = (octave + 1) * 12 + semitone
    // C4 = 60, A4 = 69
    let midi = (octave + 1) * 12 + base_semitone + accidental_offset;

    if (0..=127).contains(&midi) {
        Some(midi as u8)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Top-Level AudioInstrumentSynthPatchV1Params Tests
    // ========================================================================

    #[test]
    fn test_instrument_params_default_values() {
        let params = AudioInstrumentSynthPatchV1Params {
            note_duration_seconds: default_note_duration(),
            sample_rate: default_sample_rate(),
            synthesis: InstrumentSynthesis::Simple {
                synthesis: Synthesis::Oscillator {
                    waveform: Waveform::Sine,
                    frequency: 440.0,
                    freq_sweep: None,
                    detune: None,
                    duty: None,
                },
            },
            envelope: Envelope::default(),
            pitch_envelope: None,
            notes: None,
            generate_loop_points: false,
        };

        assert_eq!(params.note_duration_seconds, 1.0);
        assert_eq!(params.sample_rate, 44100);
        assert!(!params.generate_loop_points);
    }

    #[test]
    fn test_instrument_params_serde() {
        let params = AudioInstrumentSynthPatchV1Params {
            note_duration_seconds: 1.5,
            sample_rate: 22050,
            synthesis: InstrumentSynthesis::Simple {
                synthesis: Synthesis::KarplusStrong {
                    frequency: 440.0,
                    decay: 0.996,
                    blend: 0.7,
                },
            },
            envelope: Envelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.7,
                release: 0.3,
            },
            pitch_envelope: Some(PitchEnvelope {
                attack: 0.02,
                decay: 0.1,
                sustain: 0.5,
                release: 0.2,
                depth: 2.0,
            }),
            notes: Some(vec![
                NoteSpec::MidiNote(60),
                NoteSpec::NoteName("A4".to_string()),
            ]),
            generate_loop_points: true,
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: AudioInstrumentSynthPatchV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn test_instrument_params_with_pitch_envelope() {
        let params = AudioInstrumentSynthPatchV1Params {
            note_duration_seconds: 1.0,
            sample_rate: 44100,
            synthesis: InstrumentSynthesis::Simple {
                synthesis: Synthesis::Oscillator {
                    waveform: Waveform::Sine,
                    frequency: 440.0,
                    freq_sweep: None,
                    detune: None,
                    duty: None,
                },
            },
            envelope: Envelope::default(),
            pitch_envelope: Some(PitchEnvelope {
                attack: 0.05,
                decay: 0.15,
                sustain: 0.0,
                release: 0.2,
                depth: 12.0,
            }),
            notes: None,
            generate_loop_points: false,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("pitch_envelope"));
        assert!(json.contains("depth"));

        let parsed: AudioInstrumentSynthPatchV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    // ========================================================================
    // InstrumentSynthesis Tests (Simple vs FM Operators)
    // ========================================================================

    #[test]
    fn test_instrument_synthesis_simple_karplus() {
        let synth = InstrumentSynthesis::Simple {
            synthesis: Synthesis::KarplusStrong {
                frequency: 440.0,
                decay: 0.996,
                blend: 0.7,
            },
        };

        let json = serde_json::to_string(&synth).unwrap();
        assert!(json.contains("karplus_strong"));

        let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, synth);
    }

    #[test]
    fn test_instrument_synthesis_fm_operators() {
        let synth = InstrumentSynthesis::FmOperators {
            operators: vec![
                FmOperator {
                    waveform: Waveform::Sine,
                    ratio: 1.0,
                    level: 1.0,
                    modulation_index: 2.0,
                    envelope: Some(Envelope {
                        attack: 0.01,
                        decay: 0.1,
                        sustain: 0.7,
                        release: 0.2,
                    }),
                    detune: 0.0,
                },
                FmOperator {
                    waveform: Waveform::Sine,
                    ratio: 2.0,
                    level: 0.5,
                    modulation_index: 1.5,
                    envelope: None,
                    detune: 5.0,
                },
            ],
        };

        let json = serde_json::to_string(&synth).unwrap();
        assert!(json.contains("operators"));

        let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, synth);
    }

    #[test]
    fn test_instrument_synthesis_simple_additive() {
        let synth = InstrumentSynthesis::Simple {
            synthesis: Synthesis::Additive {
                base_freq: 440.0,
                harmonics: vec![1.0, 0.5, 0.25],
            },
        };

        let json = serde_json::to_string(&synth).unwrap();
        assert!(json.contains("additive"));

        let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, synth);
    }

    #[test]
    fn test_instrument_synthesis_simple_multi_oscillator() {
        let synth = InstrumentSynthesis::Simple {
            synthesis: Synthesis::MultiOscillator {
                frequency: 440.0,
                oscillators: vec![
                    OscillatorConfig {
                        waveform: Waveform::Sawtooth,
                        volume: 1.0,
                        detune: None,
                        phase: None,
                        duty: None,
                    },
                ],
                freq_sweep: None,
            },
        };

        let json = serde_json::to_string(&synth).unwrap();
        assert!(json.contains("multi_oscillator"));

        let parsed: InstrumentSynthesis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, synth);
    }

    // ========================================================================
    // FmOperator Tests (waveform, ratio, level, modulation_index, envelope, detune)
    // ========================================================================

    #[test]
    fn test_fm_operator_defaults() {
        let op = FmOperator {
            waveform: default_fm_waveform(),
            ratio: default_fm_ratio(),
            level: default_fm_level(),
            modulation_index: 0.0,
            envelope: None,
            detune: 0.0,
        };

        assert_eq!(op.waveform, Waveform::Sine);
        assert_eq!(op.ratio, 1.0);
        assert_eq!(op.level, 1.0);
        assert_eq!(op.modulation_index, 0.0);
        assert_eq!(op.detune, 0.0);
    }

    #[test]
    fn test_fm_operator_custom_values() {
        let op = FmOperator {
            waveform: Waveform::Square,
            ratio: 3.5,
            level: 0.75,
            modulation_index: 2.5,
            envelope: Some(Envelope {
                attack: 0.02,
                decay: 0.15,
                sustain: 0.6,
                release: 0.25,
            }),
            detune: 10.0,
        };

        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("3.5"));
        assert!(json.contains("0.75"));
        assert!(json.contains("2.5"));
        assert!(json.contains("envelope"));
        assert!(json.contains("10"));

        let parsed: FmOperator = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, op);
    }

    #[test]
    fn test_fm_operator_with_envelope() {
        let op = FmOperator {
            waveform: Waveform::Sine,
            ratio: 2.0,
            level: 1.0,
            modulation_index: 3.0,
            envelope: Some(Envelope {
                attack: 0.001,
                decay: 0.05,
                sustain: 0.8,
                release: 0.1,
            }),
            detune: 0.0,
        };

        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("envelope"));

        let parsed: FmOperator = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, op);
    }

    #[test]
    fn test_fm_operator_without_envelope() {
        let op = FmOperator {
            waveform: Waveform::Sine,
            ratio: 1.0,
            level: 1.0,
            modulation_index: 2.0,
            envelope: None,
            detune: 0.0,
        };

        let json = serde_json::to_string(&op).unwrap();
        assert!(!json.contains("envelope"));

        let parsed: FmOperator = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, op);
    }

    #[test]
    fn test_fm_operator_all_waveforms() {
        let waveforms = vec![
            Waveform::Sine,
            Waveform::Square,
            Waveform::Sawtooth,
            Waveform::Triangle,
        ];

        for waveform in waveforms {
            let op = FmOperator {
                waveform,
                ratio: 1.0,
                level: 1.0,
                modulation_index: 2.0,
                envelope: None,
                detune: 0.0,
            };

            let json = serde_json::to_string(&op).unwrap();
            let parsed: FmOperator = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.waveform, waveform);
        }
    }

    // ========================================================================
    // NoteSpec Tests (MIDI note and note name)
    // ========================================================================

    #[test]
    fn test_note_spec_midi_note() {
        let note = NoteSpec::MidiNote(69);
        let json = serde_json::to_string(&note).unwrap();
        let parsed: NoteSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, note);
    }

    #[test]
    fn test_note_spec_note_name() {
        let note = NoteSpec::NoteName("C4".to_string());
        let json = serde_json::to_string(&note).unwrap();
        let parsed: NoteSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, note);
    }

    #[test]
    fn test_note_spec_to_midi_note() {
        let note = NoteSpec::MidiNote(60);
        assert_eq!(note.to_midi_note(), Some(60));

        let note = NoteSpec::NoteName("C4".to_string());
        assert_eq!(note.to_midi_note(), Some(60));

        let note = NoteSpec::NoteName("A4".to_string());
        assert_eq!(note.to_midi_note(), Some(69));
    }

    #[test]
    fn test_note_spec_to_frequency() {
        let note = NoteSpec::NoteName("A4".to_string());
        let freq = note.to_frequency().unwrap();
        assert!((freq - 440.0).abs() < 0.001);

        let note = NoteSpec::MidiNote(69);
        let freq = note.to_frequency().unwrap();
        assert!((freq - 440.0).abs() < 0.001);
    }

    // ========================================================================
    // parse_note_name Tests
    // ========================================================================

    #[test]
    fn test_parse_note_name() {
        assert_eq!(parse_note_name("C4"), Some(60));
        assert_eq!(parse_note_name("A4"), Some(69));
        assert_eq!(parse_note_name("C#4"), Some(61));
        assert_eq!(parse_note_name("Db4"), Some(61));
        assert_eq!(parse_note_name("C0"), Some(12));
        assert_eq!(parse_note_name("G9"), Some(127));
        assert_eq!(parse_note_name(""), None);
        assert_eq!(parse_note_name("X4"), None);
    }

    #[test]
    fn test_parse_note_name_with_sharp() {
        assert_eq!(parse_note_name("C#5"), Some(73));
        assert_eq!(parse_note_name("F#4"), Some(66));
        assert_eq!(parse_note_name("A#3"), Some(58));
    }

    #[test]
    fn test_parse_note_name_with_flat() {
        assert_eq!(parse_note_name("Db5"), Some(73));
        assert_eq!(parse_note_name("Gb4"), Some(66));
        assert_eq!(parse_note_name("Bb3"), Some(58));
    }

    #[test]
    fn test_parse_note_name_edge_cases() {
        // Lowest MIDI note
        assert_eq!(parse_note_name("C-1"), Some(0));
        // Highest MIDI note
        assert_eq!(parse_note_name("G9"), Some(127));
        // Out of range
        assert_eq!(parse_note_name("C10"), None);
        assert_eq!(parse_note_name("C-2"), None);
    }

    // ========================================================================
    // midi_to_frequency Tests
    // ========================================================================

    #[test]
    fn test_midi_to_frequency() {
        // A4 = 440 Hz
        assert!((midi_to_frequency(69) - 440.0).abs() < 0.001);
        // C4 ~= 261.63 Hz
        assert!((midi_to_frequency(60) - 261.63).abs() < 0.1);
        // A3 = 220 Hz
        assert!((midi_to_frequency(57) - 220.0).abs() < 0.001);
        // A5 = 880 Hz
        assert!((midi_to_frequency(81) - 880.0).abs() < 0.001);
    }

    #[test]
    fn test_midi_to_frequency_octaves() {
        // Each octave should double the frequency
        let a3 = midi_to_frequency(57);
        let a4 = midi_to_frequency(69);
        let a5 = midi_to_frequency(81);

        assert!((a4 / a3 - 2.0).abs() < 0.001);
        assert!((a5 / a4 - 2.0).abs() < 0.001);
    }

    // ========================================================================
    // Integration Tests: Complex Instrument Configurations
    // ========================================================================

    #[test]
    fn test_complex_fm_instrument() {
        let params = AudioInstrumentSynthPatchV1Params {
            note_duration_seconds: 2.0,
            sample_rate: 48000,
            synthesis: InstrumentSynthesis::FmOperators {
                operators: vec![
                    FmOperator {
                        waveform: Waveform::Sine,
                        ratio: 1.0,
                        level: 1.0,
                        modulation_index: 3.0,
                        envelope: Some(Envelope {
                            attack: 0.01,
                            decay: 0.2,
                            sustain: 0.7,
                            release: 0.3,
                        }),
                        detune: 0.0,
                    },
                    FmOperator {
                        waveform: Waveform::Sine,
                        ratio: 2.0,
                        level: 0.5,
                        modulation_index: 2.0,
                        envelope: Some(Envelope {
                            attack: 0.005,
                            decay: 0.15,
                            sustain: 0.5,
                            release: 0.2,
                        }),
                        detune: 5.0,
                    },
                    FmOperator {
                        waveform: Waveform::Sine,
                        ratio: 3.5,
                        level: 0.25,
                        modulation_index: 1.0,
                        envelope: None,
                        detune: -5.0,
                    },
                ],
            },
            envelope: Envelope {
                attack: 0.02,
                decay: 0.3,
                sustain: 0.6,
                release: 0.4,
            },
            pitch_envelope: Some(PitchEnvelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.0,
                release: 0.0,
                depth: 2.0,
            }),
            notes: Some(vec![
                NoteSpec::NoteName("C3".to_string()),
                NoteSpec::NoteName("C4".to_string()),
                NoteSpec::NoteName("C5".to_string()),
            ]),
            generate_loop_points: true,
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: AudioInstrumentSynthPatchV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn test_complex_karplus_instrument() {
        let params = AudioInstrumentSynthPatchV1Params {
            note_duration_seconds: 1.5,
            sample_rate: 22050,
            synthesis: InstrumentSynthesis::Simple {
                synthesis: Synthesis::KarplusStrong {
                    frequency: 440.0,
                    decay: 0.998,
                    blend: 0.5,
                },
            },
            envelope: Envelope {
                attack: 0.001,
                decay: 0.05,
                sustain: 0.9,
                release: 0.5,
            },
            pitch_envelope: None,
            notes: Some(vec![
                NoteSpec::MidiNote(48),
                NoteSpec::MidiNote(60),
                NoteSpec::MidiNote(72),
            ]),
            generate_loop_points: false,
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: AudioInstrumentSynthPatchV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn test_subtractive_multi_oscillator_instrument() {
        let params = AudioInstrumentSynthPatchV1Params {
            note_duration_seconds: 1.0,
            sample_rate: 44100,
            synthesis: InstrumentSynthesis::Simple {
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
                        OscillatorConfig {
                            waveform: Waveform::Square,
                            volume: 0.5,
                            detune: Some(-5.0),
                            phase: Some(1.57),
                            duty: Some(0.5),
                        },
                    ],
                    freq_sweep: None,
                },
            },
            envelope: Envelope {
                attack: 0.05,
                decay: 0.2,
                sustain: 0.7,
                release: 0.3,
            },
            pitch_envelope: None,
            notes: None,
            generate_loop_points: true,
        };

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: AudioInstrumentSynthPatchV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }
}
