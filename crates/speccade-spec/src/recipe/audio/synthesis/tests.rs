//! Tests for synthesis types.

use super::*;

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
// Envelope Tests
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
fn test_envelope_serde() {
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
// PitchEnvelope Tests
// ========================================================================

#[test]
fn test_pitch_envelope_default() {
    let env = PitchEnvelope::default();
    assert_eq!(env.depth, 0.0);
}

#[test]
fn test_pitch_envelope_serde() {
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
// Filter Tests
// ========================================================================

#[test]
fn test_filter_lowpass() {
    let filter = Filter::Lowpass {
        cutoff: 2000.0,
        resonance: 0.707,
        cutoff_end: None,
    };

    let json = serde_json::to_string(&filter).unwrap();
    assert!(json.contains("lowpass"));
    let parsed: Filter = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, filter);
}

#[test]
fn test_filter_highpass() {
    let filter = Filter::Highpass {
        cutoff: 500.0,
        resonance: 0.5,
        cutoff_end: Some(2000.0),
    };

    let json = serde_json::to_string(&filter).unwrap();
    assert!(json.contains("highpass"));
    let parsed: Filter = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, filter);
}

#[test]
fn test_filter_allpass() {
    let filter = Filter::Allpass {
        frequency: 1000.0,
        resonance: 2.0,
        frequency_end: Some(2000.0),
    };

    let json = serde_json::to_string(&filter).unwrap();
    assert!(json.contains("allpass"));
    assert!(json.contains("frequency"));
    let parsed: Filter = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, filter);
}

#[test]
fn test_filter_allpass_no_sweep() {
    let filter = Filter::Allpass {
        frequency: 800.0,
        resonance: 0.707,
        frequency_end: None,
    };

    let json = serde_json::to_string(&filter).unwrap();
    assert!(json.contains("allpass"));
    assert!(!json.contains("frequency_end"));
    let parsed: Filter = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, filter);
}

// ========================================================================
// Synthesis Tests
// ========================================================================

#[test]
fn test_synthesis_oscillator() {
    let synth = Synthesis::Oscillator {
        waveform: Waveform::Sine,
        frequency: 440.0,
        freq_sweep: None,
        detune: None,
        duty: None,
    };

    let json = serde_json::to_string(&synth).unwrap();
    assert!(json.contains("oscillator"));
    let parsed: Synthesis = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, synth);
}

#[test]
fn test_synthesis_karplus_strong() {
    let synth = Synthesis::KarplusStrong {
        frequency: 220.0,
        decay: 0.996,
        blend: 0.7,
    };

    let json = serde_json::to_string(&synth).unwrap();
    assert!(json.contains("karplus_strong"));
    let parsed: Synthesis = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, synth);
}

#[test]
fn test_synthesis_noise_burst() {
    let synth = Synthesis::NoiseBurst {
        noise_type: NoiseType::White,
        filter: Some(Filter::Lowpass {
            cutoff: 1000.0,
            resonance: 0.5,
            cutoff_end: None,
        }),
    };

    let json = serde_json::to_string(&synth).unwrap();
    assert!(json.contains("noise_burst"));
    let parsed: Synthesis = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, synth);
}

// ========================================================================
// NoteSpec Tests
// ========================================================================

#[test]
fn test_note_spec_midi_note() {
    let note = NoteSpec::MidiNote(69);
    let freq = note.to_frequency();
    assert!((freq - 440.0).abs() < 0.001);

    let json = serde_json::to_string(&note).unwrap();
    let parsed: NoteSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, note);
}

#[test]
fn test_note_spec_note_name() {
    let note = NoteSpec::NoteName("C4".to_string());
    let freq = note.to_frequency();
    assert!((freq - 261.63).abs() < 0.1);

    let json = serde_json::to_string(&note).unwrap();
    let parsed: NoteSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, note);
}

#[test]
fn test_note_spec_default() {
    let note = NoteSpec::default();
    assert_eq!(note, NoteSpec::NoteName("C4".to_string()));
}

#[test]
fn test_note_spec_invalid_name() {
    let note = NoteSpec::NoteName("InvalidNote".to_string());
    let freq = note.to_frequency();
    // Should fall back to C4 (261.63 Hz)
    assert!((freq - 261.63).abs() < 0.1);
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

// ========================================================================
// ModulationTarget Tests
// ========================================================================

#[test]
fn test_modulation_target_pulse_width_serde() {
    let target = ModulationTarget::PulseWidth { amount: 0.2 };

    let json = serde_json::to_string(&target).unwrap();
    assert!(json.contains(r#""target":"pulse_width""#));
    assert!(json.contains(r#""amount":0.2"#));

    let parsed: ModulationTarget = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, target);
}

#[test]
fn test_modulation_target_pulse_width_from_json() {
    let json = r#"{"target":"pulse_width","amount":0.3}"#;
    let target: ModulationTarget = serde_json::from_str(json).unwrap();

    match target {
        ModulationTarget::PulseWidth { amount } => {
            assert!((amount - 0.3).abs() < 0.0001);
        }
        _ => panic!("Expected PulseWidth variant"),
    }
}

#[test]
fn test_modulation_target_all_variants_serde() {
    let targets = vec![
        ModulationTarget::Pitch { semitones: 2.0 },
        ModulationTarget::Volume { amount: 0.5 },
        ModulationTarget::FilterCutoff { amount: 1000.0 },
        ModulationTarget::Pan { amount: 0.8 },
        ModulationTarget::PulseWidth { amount: 0.2 },
    ];

    for target in targets {
        let json = serde_json::to_string(&target).unwrap();
        let parsed: ModulationTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, target);
    }
}

// ========================================================================
// DetuneCurve Tests
// ========================================================================

#[test]
fn test_detune_curve_default() {
    let curve: DetuneCurve = Default::default();
    assert_eq!(curve, DetuneCurve::Linear);
}

#[test]
fn test_detune_curve_serde() {
    let curves = vec![DetuneCurve::Linear, DetuneCurve::Exp2];

    for curve in curves {
        let json = serde_json::to_string(&curve).unwrap();
        let parsed: DetuneCurve = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, curve);
    }
}

#[test]
fn test_detune_curve_from_json() {
    let json_linear = r#""linear""#;
    let json_exp2 = r#""exp2""#;

    let linear: DetuneCurve = serde_json::from_str(json_linear).unwrap();
    let exp2: DetuneCurve = serde_json::from_str(json_exp2).unwrap();

    assert_eq!(linear, DetuneCurve::Linear);
    assert_eq!(exp2, DetuneCurve::Exp2);
}

// ========================================================================
// SupersawUnison Tests
// ========================================================================

#[test]
fn test_synthesis_supersaw_unison() {
    let synth = Synthesis::SupersawUnison {
        frequency: 440.0,
        voices: 7,
        detune_cents: 25.0,
        spread: 0.8,
        detune_curve: DetuneCurve::Linear,
    };

    let json = serde_json::to_string(&synth).unwrap();
    assert!(json.contains("supersaw_unison"));
    assert!(json.contains("frequency"));
    assert!(json.contains("voices"));
    assert!(json.contains("detune_cents"));
    assert!(json.contains("spread"));

    let parsed: Synthesis = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, synth);
}

#[test]
fn test_synthesis_supersaw_unison_from_json() {
    let json = r#"{
        "type": "supersaw_unison",
        "frequency": 220.0,
        "voices": 5,
        "detune_cents": 20.0,
        "spread": 1.0,
        "detune_curve": "exp2"
    }"#;

    let synth: Synthesis = serde_json::from_str(json).unwrap();

    match synth {
        Synthesis::SupersawUnison {
            frequency,
            voices,
            detune_cents,
            spread,
            detune_curve,
        } => {
            assert!((frequency - 220.0).abs() < 0.001);
            assert_eq!(voices, 5);
            assert!((detune_cents - 20.0).abs() < 0.001);
            assert!((spread - 1.0).abs() < 0.001);
            assert_eq!(detune_curve, DetuneCurve::Exp2);
        }
        _ => panic!("Expected SupersawUnison variant"),
    }
}

#[test]
fn test_synthesis_supersaw_unison_default_curve() {
    // Test that detune_curve defaults to Linear when omitted
    let json = r#"{
        "type": "supersaw_unison",
        "frequency": 440.0,
        "voices": 3,
        "detune_cents": 15.0,
        "spread": 0.5
    }"#;

    let synth: Synthesis = serde_json::from_str(json).unwrap();

    match synth {
        Synthesis::SupersawUnison { detune_curve, .. } => {
            assert_eq!(detune_curve, DetuneCurve::Linear);
        }
        _ => panic!("Expected SupersawUnison variant"),
    }
}
