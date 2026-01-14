//! Integration tests for XM writer, including validation, effects, and edge cases.
//!
//! These tests validate complete XM modules, effect codes, format constants,
//! and edge case scenarios.

use speccade_backend_music::xm::effects;
use speccade_backend_music::xm::{
    validate_xm_bytes, XmEnvelope, XmInstrument, XmModule, XmNote, XmPattern, XmSample,
    XmValidationError, XM_MAX_CHANNELS, XM_MAX_INSTRUMENTS, XM_MAX_PATTERNS, XM_MAX_PATTERN_ROWS,
};

// =============================================================================
// Helper Functions
// =============================================================================

/// Generate a minimal valid XM module for testing.
fn generate_minimal_xm() -> Vec<u8> {
    let mut module = XmModule::new("Test", 4, 6, 125);
    let pattern = XmPattern::empty(64, 4);
    module.add_pattern(pattern);
    module.set_order_table(&[0]);
    module.to_bytes().unwrap()
}

/// Generate an XM module with a single instrument.
fn generate_xm_with_instrument() -> Vec<u8> {
    let mut module = XmModule::new("With Instrument", 4, 6, 125);

    let pattern = XmPattern::empty(64, 4);
    module.add_pattern(pattern);

    // Create a simple instrument with minimal sample
    let sample_data = vec![0u8; 100]; // 50 samples of 16-bit silence
    let sample = XmSample::new("TestSample", sample_data, true);
    let instrument = XmInstrument::new("TestInstr", sample);
    module.add_instrument(instrument);

    module.set_order_table(&[0]);
    module.to_bytes().unwrap()
}

/// Generate an XM module with notes in patterns.
fn generate_xm_with_notes() -> Vec<u8> {
    let mut module = XmModule::new("With Notes", 2, 6, 120);

    // Create pattern with notes
    let mut pattern = XmPattern::empty(16, 2);
    pattern.set_note(0, 0, XmNote::from_name("C4", 1, Some(64)));
    pattern.set_note(4, 0, XmNote::from_name("E4", 1, Some(48)));
    pattern.set_note(8, 0, XmNote::from_name("G4", 1, Some(32)));
    pattern.set_note(12, 0, XmNote::note_off());
    module.add_pattern(pattern);

    // Add instrument
    let sample_data = vec![0u8; 200];
    let sample = XmSample::new("Lead", sample_data, true);
    let instrument = XmInstrument::new("Lead", sample);
    module.add_instrument(instrument);

    module.set_order_table(&[0]);
    module.to_bytes().unwrap()
}

// =============================================================================
// Integration / Validation Tests
// =============================================================================

#[test]
fn test_xm_validation_success() {
    let xm = generate_minimal_xm();
    assert!(validate_xm_bytes(&xm).is_ok());
}

#[test]
fn test_xm_validation_with_instrument() {
    let xm = generate_xm_with_instrument();
    assert!(validate_xm_bytes(&xm).is_ok());
}

#[test]
fn test_xm_validation_with_notes() {
    let xm = generate_xm_with_notes();
    assert!(validate_xm_bytes(&xm).is_ok());
}

#[test]
fn test_xm_validation_file_too_small() {
    let small_data = vec![0u8; 30];
    match validate_xm_bytes(&small_data) {
        Err(XmValidationError::FileTooSmall(size)) => {
            assert_eq!(size, 30);
        }
        _ => panic!("Expected FileTooSmall error"),
    }
}

#[test]
fn test_xm_validation_invalid_magic() {
    let mut xm = generate_minimal_xm();
    // Corrupt the magic
    xm[0] = b'X';

    match validate_xm_bytes(&xm) {
        Err(XmValidationError::InvalidMagic) => {}
        _ => panic!("Expected InvalidMagic error"),
    }
}

#[test]
fn test_xm_validation_wrong_version() {
    let mut xm = generate_minimal_xm();
    // Change version to something unsupported
    xm[58] = 0x00;
    xm[59] = 0x02; // version 2.00

    match validate_xm_bytes(&xm) {
        Err(XmValidationError::UnsupportedVersion(v)) => {
            assert_eq!(v, 0x0200);
        }
        _ => panic!("Expected UnsupportedVersion error"),
    }
}

#[test]
fn test_xm_hash_determinism() {
    // Generate the same module twice
    let mut module1 = XmModule::new("HashTest", 4, 6, 125);
    module1.add_pattern(XmPattern::empty(64, 4));
    module1.set_order_table(&[0]);

    let mut module2 = XmModule::new("HashTest", 4, 6, 125);
    module2.add_pattern(XmPattern::empty(64, 4));
    module2.set_order_table(&[0]);

    let hash1 = module1.compute_hash().unwrap();
    let hash2 = module2.compute_hash().unwrap();

    assert_eq!(hash1, hash2, "Same module should produce same hash");
}

#[test]
fn test_xm_hash_different_content() {
    let mut module1 = XmModule::new("HashTest1", 4, 6, 125);
    module1.add_pattern(XmPattern::empty(64, 4));
    module1.set_order_table(&[0]);

    let mut module2 = XmModule::new("HashTest2", 4, 6, 125);
    module2.add_pattern(XmPattern::empty(64, 4));
    module2.set_order_table(&[0]);

    let hash1 = module1.compute_hash().unwrap();
    let hash2 = module2.compute_hash().unwrap();

    assert_ne!(
        hash1, hash2,
        "Different modules should produce different hashes"
    );
}

#[test]
fn test_xm_multiple_patterns() {
    let mut module = XmModule::new("MultiPattern", 4, 6, 125);

    for i in 0..10 {
        let mut pattern = XmPattern::empty(32, 4);
        pattern.set_note(0, 0, XmNote::from_name(&format!("C{}", i % 8), 1, Some(64)));
        module.add_pattern(pattern);
    }

    module.set_order_table(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let xm = module.to_bytes().unwrap();
    assert!(validate_xm_bytes(&xm).is_ok());

    let num_patterns = u16::from_le_bytes([xm[70], xm[71]]);
    assert_eq!(num_patterns, 10);
}

#[test]
fn test_xm_multiple_instruments() {
    let mut module = XmModule::new("MultiInstr", 4, 6, 125);
    module.add_pattern(XmPattern::empty(64, 4));

    for i in 0..5 {
        let sample = XmSample::new(&format!("Sample{}", i), vec![0u8; 100 + i * 50], true);
        let instrument = XmInstrument::new(&format!("Instr{}", i), sample);
        module.add_instrument(instrument);
    }

    module.set_order_table(&[0]);

    let xm = module.to_bytes().unwrap();
    assert!(validate_xm_bytes(&xm).is_ok());

    let num_instruments = u16::from_le_bytes([xm[72], xm[73]]);
    assert_eq!(num_instruments, 5);
}

#[test]
fn test_xm_complex_module() {
    let mut module = XmModule::new("Complex Song", 8, 6, 140);

    // Add multiple patterns with various content
    for p in 0..4 {
        let mut pattern = XmPattern::empty(64, 8);

        for row in (0..64).step_by(4) {
            for ch in 0..4 {
                let note_names = ["C4", "E4", "G4", "B4"];
                let note = XmNote::from_name(note_names[(p + ch as usize) % 4], ch + 1, Some(48));
                pattern.set_note(row, ch, note);
            }
        }

        module.add_pattern(pattern);
    }

    // Add instruments with envelopes
    for i in 0..4 {
        let sample_data: Vec<u8> = (0..(500 + i * 100))
            .map(|x| ((x as f32 * 0.1).sin() * 127.0 + 128.0) as u8)
            .collect();
        let sample = XmSample::new(&format!("Wave{}", i), sample_data, false);
        let env = XmEnvelope::adsr(10, 30, 40, 50);
        let instrument =
            XmInstrument::new(&format!("Synth{}", i), sample).with_volume_envelope(env);
        module.add_instrument(instrument);
    }

    module.set_order_table(&[0, 1, 2, 3, 0, 1, 2, 3]);
    module.set_restart_position(4);

    let xm = module.to_bytes().unwrap();
    assert!(validate_xm_bytes(&xm).is_ok());
}

// =============================================================================
// Effect Code Tests
// =============================================================================

#[test]
fn test_effect_constants() {
    assert_eq!(effects::ARPEGGIO, 0x0);
    assert_eq!(effects::PORTA_UP, 0x1);
    assert_eq!(effects::PORTA_DOWN, 0x2);
    assert_eq!(effects::TONE_PORTA, 0x3);
    assert_eq!(effects::VIBRATO, 0x4);
    assert_eq!(effects::VOL_SLIDE, 0xA);
    assert_eq!(effects::SET_VOLUME, 0xC);
    assert_eq!(effects::PATTERN_BREAK, 0xD);
    assert_eq!(effects::SET_SPEED_TEMPO, 0xF);
}

#[test]
fn test_note_with_various_effects() {
    let effects_to_test = [
        (effects::ARPEGGIO, 0x37),
        (effects::PORTA_UP, 0x10),
        (effects::VIBRATO, 0x44),
        (effects::VOL_SLIDE, 0x0F),
        (effects::SET_SPEED_TEMPO, 0x80),
    ];

    for (effect, param) in effects_to_test {
        let note = XmNote::from_name("C4", 1, None).with_effect(effect, param);
        assert_eq!(note.effect, effect);
        assert_eq!(note.effect_param, param);
    }
}

// =============================================================================
// Constants Tests
// =============================================================================

#[test]
fn test_format_limits() {
    assert_eq!(XM_MAX_CHANNELS, 32);
    assert_eq!(XM_MAX_PATTERNS, 256);
    assert_eq!(XM_MAX_INSTRUMENTS, 128);
    assert_eq!(XM_MAX_PATTERN_ROWS, 256);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_module() {
    let mut module = XmModule::new("Empty", 1, 6, 125);
    module.add_pattern(XmPattern::empty(1, 1));
    module.set_order_table(&[0]);

    let xm = module.to_bytes().unwrap();
    assert!(validate_xm_bytes(&xm).is_ok());
}

#[test]
fn test_max_channels() {
    let mut module = XmModule::new("MaxCh", 32, 6, 125);
    module.add_pattern(XmPattern::empty(64, 32));
    module.set_order_table(&[0]);

    let xm = module.to_bytes().unwrap();
    assert!(validate_xm_bytes(&xm).is_ok());

    let num_channels = u16::from_le_bytes([xm[68], xm[69]]);
    assert_eq!(num_channels, 32);
}

#[test]
fn test_special_characters_in_name() {
    let mut module = XmModule::new("Test!@#$%", 4, 6, 125);
    module.add_pattern(XmPattern::empty(64, 4));
    module.set_order_table(&[0]);

    let xm = module.to_bytes().unwrap();
    assert!(validate_xm_bytes(&xm).is_ok());
}

#[test]
fn test_long_order_table() {
    let mut module = XmModule::new("LongOrder", 4, 6, 125);

    for _ in 0..16 {
        module.add_pattern(XmPattern::empty(64, 4));
    }

    // Create a long order table (max 256 entries)
    let order: Vec<u8> = (0..200).map(|i| (i % 16) as u8).collect();
    module.set_order_table(&order);

    let xm = module.to_bytes().unwrap();
    assert!(validate_xm_bytes(&xm).is_ok());

    let song_length = u16::from_le_bytes([xm[64], xm[65]]);
    assert_eq!(song_length, 200);
}

#[test]
fn test_note_is_empty() {
    let empty = XmNote::empty();
    assert!(empty.is_empty());

    let with_note = XmNote::from_name("C4", 0, None);
    assert!(!with_note.is_empty());

    let note_off = XmNote::note_off();
    assert!(!note_off.is_empty());
}
