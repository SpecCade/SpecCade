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
