//! Integration tests for IT (Impulse Tracker) writer.
//!
//! This module tests format comparisons with XM, validation logic,
//! module assembly, deterministic hashing, and edge cases.

use speccade_backend_music::it::{
    effects, validate_it_bytes, ItInstrument, ItModule, ItNote, ItPattern, ItSample,
    ItValidationError, IT_INSTRUMENT_SIZE, IT_MAX_INSTRUMENTS, IT_MAX_SAMPLES,
};
use speccade_backend_music::note::{it as it_notes, note_name_to_it};

// =============================================================================
// Helper Functions
// =============================================================================

/// Generate a minimal valid IT module for testing.
fn generate_minimal_it() -> Vec<u8> {
    let mut module = ItModule::new("Test", 4, 6, 125);
    module.add_pattern(ItPattern::empty(64, 4));
    let instrument = ItInstrument::new("Inst1");
    module.add_instrument(instrument);
    let sample = ItSample::new("Sample1", vec![0u8; 100], 22050);
    module.add_sample(sample);
    module.set_orders(&[0]);
    module.to_bytes().unwrap()
}

/// Generate an IT module with specific parameters for testing.
fn generate_it_with_params(name: &str, channels: u8, speed: u8, bpm: u8) -> Vec<u8> {
    let mut module = ItModule::new(name, channels, speed, bpm);
    module.add_pattern(ItPattern::empty(64, channels));
    let instrument = ItInstrument::new("Test");
    module.add_instrument(instrument);
    let sample = ItSample::new("TestSample", vec![0u8; 200], 22050);
    module.add_sample(sample);
    module.set_orders(&[0]);
    module.to_bytes().unwrap()
}

/// Extract a little-endian u16 from bytes.
fn read_u16_le(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

/// Extract a little-endian u32 from bytes.
fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

// =============================================================================
// 5. IT vs XM Comparison Tests
// =============================================================================

#[test]
fn test_it_note_values() {
    // IT notes are 0-indexed (0 = C-0), XM are 1-indexed (1 = C-0)
    assert_eq!(note_name_to_it("C0"), 0, "C-0 should be 0 in IT");
    assert_eq!(note_name_to_it("C4"), 48, "C-4 should be 48 in IT");
    assert_eq!(note_name_to_it("B9"), 119, "B-9 should be 119 in IT");
}

#[test]
fn test_it_special_notes() {
    assert_eq!(it_notes::NOTE_OFF, 255, "IT note off should be 255");
    assert_eq!(it_notes::NOTE_CUT, 254, "IT note cut should be 254");
    assert_eq!(it_notes::NOTE_FADE, 253, "IT note fade should be 253");

    assert_eq!(note_name_to_it("^^^"), 255, "^^^ should be note off");
    assert_eq!(note_name_to_it("==="), 254, "=== should be note cut");
    assert_eq!(note_name_to_it("~~~"), 253, "~~~ should be note fade");
}

#[test]
fn test_it_vs_xm_channel_limit() {
    // IT supports up to 64 channels, XM only 32
    use speccade_backend_music::it::IT_MAX_CHANNELS;
    use speccade_backend_music::xm::XM_MAX_CHANNELS;

    assert_eq!(IT_MAX_CHANNELS, 64, "IT should support 64 channels");
    assert_eq!(XM_MAX_CHANNELS, 32, "XM should support 32 channels");
}

#[test]
fn test_it_vs_xm_instrument_limit() {
    use speccade_backend_music::xm::XM_MAX_INSTRUMENTS;

    assert_eq!(IT_MAX_INSTRUMENTS, 99, "IT should support 99 instruments");
    assert_eq!(XM_MAX_INSTRUMENTS, 128, "XM should support 128 instruments");
}

#[test]
fn test_it_vs_xm_sample_limit() {
    // IT has separate samples from instruments
    assert_eq!(IT_MAX_SAMPLES, 99, "IT should support 99 samples");
}

#[test]
fn test_it_has_separate_samples() {
    // IT separates instruments and samples, XM bundles them
    let mut module = ItModule::new("Test", 4, 6, 125);

    // Add 2 instruments
    module.add_instrument(ItInstrument::new("Inst1"));
    module.add_instrument(ItInstrument::new("Inst2"));

    // Add 3 samples (independent of instrument count)
    module.add_sample(ItSample::new("Sample1", vec![0u8; 100], 22050));
    module.add_sample(ItSample::new("Sample2", vec![0u8; 100], 22050));
    module.add_sample(ItSample::new("Sample3", vec![0u8; 100], 22050));

    assert_eq!(module.instruments.len(), 2);
    assert_eq!(module.samples.len(), 3);

    let it = module.to_bytes().unwrap();
    let inst_count = read_u16_le(&it, 0x22);
    let sample_count = read_u16_le(&it, 0x24);

    assert_eq!(inst_count, 2);
    assert_eq!(sample_count, 3);
}

#[test]
fn test_it_instrument_sample_mapping() {
    let mut instrument = ItInstrument::new("Test");

    // Map different notes to different samples
    for i in 0..60 {
        instrument.note_sample_table[i] = (i as u8, 1);
    }
    for i in 60..120 {
        instrument.note_sample_table[i] = (i as u8, 2);
    }

    // Check mappings
    assert_eq!(instrument.note_sample_table[30].1, 1);
    assert_eq!(instrument.note_sample_table[80].1, 2);
}

// =============================================================================
// 6. Validation Tests
// =============================================================================

#[test]
fn test_it_validation_success() {
    let it = generate_minimal_it();
    assert!(validate_it_bytes(&it).is_ok());
}

#[test]
fn test_it_validation_too_small() {
    let data = vec![0u8; 3];
    let result = validate_it_bytes(&data);

    assert!(result.is_err());
    match result.unwrap_err() {
        ItValidationError::FileTooSmall(size) => assert_eq!(size, 3),
        _ => panic!("Expected FileTooSmall error"),
    }
}

#[test]
fn test_it_validation_invalid_magic() {
    let mut it = generate_minimal_it();
    it[0] = b'X';

    let result = validate_it_bytes(&it);
    assert!(result.is_err());
    match result.unwrap_err() {
        ItValidationError::InvalidMagic => {}
        _ => panic!("Expected InvalidMagic error"),
    }
}

#[test]
fn test_it_validation_error_display() {
    let err1 = ItValidationError::FileTooSmall(10);
    assert!(err1.to_string().contains("10"));

    let err2 = ItValidationError::InvalidMagic;
    assert!(err2.to_string().contains("magic"));
}

// =============================================================================
// 7. Module Assembly Tests
// =============================================================================

#[test]
fn test_it_module_message() {
    let mut module = ItModule::new("Test", 4, 6, 125);
    module.add_pattern(ItPattern::empty(64, 4));
    module.set_message("Created by SpecCade");
    module.set_orders(&[0]);

    let it = module.to_bytes().unwrap();

    // Special flags at 0x2E should have MESSAGE bit set
    let special = read_u16_le(&it, 0x2E);
    assert!(
        special & 0x01 != 0,
        "Special flags should have message bit set"
    );

    // Message length at 0x36-0x37
    let msg_len = read_u16_le(&it, 0x36);
    assert!(msg_len > 0, "Message length should be positive");

    // Message offset at 0x38-0x3B
    let msg_offset = read_u32_le(&it, 0x38);
    assert!(msg_offset > 0, "Message offset should be positive");

    // Verify message content at offset
    let msg_bytes = &it[msg_offset as usize..msg_offset as usize + 19];
    assert_eq!(msg_bytes, b"Created by SpecCade");
}

#[test]
fn test_it_hash_determinism() {
    let create_module = || {
        let mut module = ItModule::new("Hash Test", 4, 6, 125);
        module.add_pattern(ItPattern::empty(64, 4));
        module.add_instrument(ItInstrument::new("Test"));
        module.add_sample(ItSample::new("Test", vec![0u8; 100], 22050));
        module.set_orders(&[0]);
        module
    };

    let module1 = create_module();
    let module2 = create_module();

    let hash1 = module1.compute_hash().unwrap();
    let hash2 = module2.compute_hash().unwrap();

    assert_eq!(hash1, hash2, "Same input should produce same hash");
    assert_eq!(hash1.len(), 64, "BLAKE3 hash should be 64 hex characters");
}

#[test]
fn test_it_module_order_table() {
    let mut module = ItModule::new("Test", 4, 6, 125);

    for i in 0..5 {
        module.add_pattern(ItPattern::empty(64, 4));
        module.add_instrument(ItInstrument::new(&format!("Inst{}", i)));
        module.add_sample(ItSample::new(
            &format!("Sample{}", i),
            vec![0u8; 100],
            22050,
        ));
    }

    // Set custom order: 0, 2, 1, 4, 3, 0
    module.set_orders(&[0, 2, 1, 4, 3, 0]);

    let it = module.to_bytes().unwrap();

    // Orders start at byte 192
    assert_eq!(it[192], 0);
    assert_eq!(it[193], 2);
    assert_eq!(it[194], 1);
    assert_eq!(it[195], 4);
    assert_eq!(it[196], 3);
    assert_eq!(it[197], 0);
}

#[test]
fn test_it_instrument_size() {
    assert_eq!(IT_INSTRUMENT_SIZE, 554, "IT instrument should be 554 bytes");

    let instrument = ItInstrument::new("Test Instrument");
    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    assert_eq!(buf.len(), IT_INSTRUMENT_SIZE);
}

#[test]
fn test_it_instrument_magic() {
    use speccade_backend_music::it::IT_INSTRUMENT_MAGIC;
    let instrument = ItInstrument::new("Test");
    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    assert_eq!(&buf[0..4], IT_INSTRUMENT_MAGIC);
}

// =============================================================================
// 8. Edge Case Tests
// =============================================================================

#[test]
fn test_it_max_channels() {
    let module = ItModule::new("Test", 64, 6, 125);
    assert!(
        module.header.channel_pan.iter().take(64).all(|&p| p == 32),
        "All 64 channels should be enabled"
    );
}

#[test]
fn test_it_long_song_name() {
    let long_name = "This is a very long song name that exceeds the 26 character limit";
    let it = generate_it_with_params(long_name, 4, 6, 125);

    // Name should be truncated to 26 bytes
    let name_bytes = &it[0x04..0x04 + 26];
    let expected = &long_name.as_bytes()[..26];
    assert_eq!(name_bytes, expected);
}

#[test]
fn test_it_empty_pattern_module() {
    let mut module = ItModule::new("Empty Test", 4, 6, 125);
    module.add_pattern(ItPattern::empty(64, 4));
    module.set_orders(&[0]);

    let it = module.to_bytes().unwrap();
    assert!(validate_it_bytes(&it).is_ok());
}

#[test]
fn test_it_multiple_patterns() {
    let mut module = ItModule::new("Multi Pattern", 4, 6, 125);

    for i in 0..10 {
        let mut pattern = ItPattern::empty(64, 4);
        pattern.set_note(0, 0, ItNote::from_midi(48 + i, 1, 64));
        module.add_pattern(pattern);
    }

    module.add_instrument(ItInstrument::new("Test"));
    module.add_sample(ItSample::new("Test", vec![0u8; 100], 22050));
    module.set_orders(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let it = module.to_bytes().unwrap();
    let pattern_count = read_u16_le(&it, 0x26);
    assert_eq!(pattern_count, 10);
}

#[test]
fn test_it_note_from_midi() {
    let note = ItNote::from_midi(60, 1, 64); // Middle C
    assert_eq!(note.note, 60);
    assert_eq!(note.instrument, 1);
    assert_eq!(note.volume, 64);
}

#[test]
fn test_it_note_with_effect() {
    let note = ItNote::from_name("C4", 1, 64).with_effect(effects::VIBRATO, 0x48);
    assert_eq!(note.effect, effects::VIBRATO);
    assert_eq!(note.effect_param, 0x48);
}
