//! Tests for IT (Impulse Tracker) pattern encoding and sample handling.
//!
//! This module verifies pattern compression, note encoding, effect handling,
//! sample data structures, loop configuration, and sample quality.

use speccade_backend_music::it::{
    convert_flags, effects, sample_flags, ItNote, ItPattern, ItSample, IT_SAMPLE_HEADER_SIZE,
    IT_SAMPLE_MAGIC,
};

// =============================================================================
// Helper Functions
// =============================================================================

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
// 3. Pattern Encoding Tests
// =============================================================================

#[test]
fn test_it_pattern_empty() {
    let pattern = ItPattern::empty(64, 4);

    assert_eq!(pattern.num_rows, 64);
    assert_eq!(pattern.notes.len(), 64);
    for row in &pattern.notes {
        assert_eq!(row.len(), 4);
        for note in row {
            assert!(note.is_empty(), "Empty pattern should have empty notes");
        }
    }
}

#[test]
fn test_it_pattern_compression_empty() {
    let pattern = ItPattern::empty(64, 4);
    let packed = pattern.pack(4);

    // Empty pattern should only have row end markers (one 0x00 per row)
    assert_eq!(
        packed.len(),
        64,
        "Empty pattern should pack to 64 bytes (row markers)"
    );

    for &b in &packed {
        assert_eq!(b, 0, "All bytes should be row end markers");
    }
}

#[test]
fn test_it_pattern_compression_with_notes() {
    let mut pattern = ItPattern::empty(4, 2);
    pattern.set_note(0, 0, ItNote::from_name("C4", 1, 64));
    pattern.set_note(2, 1, ItNote::from_name("E4", 1, 48));

    let packed = pattern.pack(2);

    // Should have data bytes + row end markers
    assert!(!packed.is_empty());
    assert!(
        packed.len() > 4,
        "Packed data with notes should be larger than just row markers"
    );

    // Count row end markers (zeros at expected positions)
    let row_markers: Vec<_> = packed.iter().enumerate().filter(|(_, &b)| b == 0).collect();
    assert!(
        row_markers.len() >= 4,
        "Should have at least 4 row end markers"
    );
}

#[test]
fn test_it_pattern_compression_channel_byte() {
    let mut pattern = ItPattern::empty(1, 4);
    pattern.set_note(0, 0, ItNote::from_name("C4", 1, 64));

    let packed = pattern.pack(4);

    // First byte should be channel number (1-based) with bit 7 set
    assert_eq!(
        packed[0] & 0x7F,
        1,
        "Channel byte should be 1 (0-indexed + 1)"
    );
    assert!(
        packed[0] & 0x80 != 0,
        "Channel byte should have bit 7 set for mask follows"
    );
}

#[test]
fn test_it_pattern_effects() {
    let mut pattern = ItPattern::empty(4, 2);

    // Add note with effect
    let note = ItNote::from_name("C4", 1, 64).with_effect(effects::VIBRATO, 0x48);
    pattern.set_note(0, 0, note);

    let packed = pattern.pack(2);
    assert!(
        packed.len() > 5,
        "Pattern with effect should have more packed data"
    );

    // The mask byte should indicate effect data is present (bit 3)
    let mask = packed[1];
    assert!(mask & 0x08 != 0, "Mask should indicate effect data (bit 3)");
}

#[test]
fn test_it_pattern_note_off() {
    let mut pattern = ItPattern::empty(2, 1);
    pattern.set_note(0, 0, ItNote::from_name("C4", 1, 64));
    pattern.set_note(1, 0, ItNote::note_off());

    let packed = pattern.pack(1);

    // Note off should be encoded
    assert!(packed.len() > 2, "Pattern with note-off should have data");
}

#[test]
fn test_it_pattern_note_cut() {
    let mut pattern = ItPattern::empty(2, 1);
    pattern.set_note(0, 0, ItNote::from_name("C4", 1, 64));
    pattern.set_note(1, 0, ItNote::note_cut());

    let packed = pattern.pack(1);
    assert!(packed.len() > 2, "Pattern with note-cut should have data");
}

#[test]
fn test_it_pattern_write() {
    let mut pattern = ItPattern::empty(16, 4);
    pattern.set_note(0, 0, ItNote::from_name("C4", 1, 64));

    let mut buf = Vec::new();
    pattern.write(&mut buf, 4).unwrap();

    // Pattern header is 8 bytes
    assert!(buf.len() >= 8, "Pattern should have at least 8-byte header");

    // First 2 bytes are packed data length
    let packed_len = read_u16_le(&buf, 0);
    assert!(packed_len > 0, "Packed length should be positive");

    // Next 2 bytes are row count
    let rows = read_u16_le(&buf, 2);
    assert_eq!(rows, 16, "Row count should be 16");

    // Bytes 4-7 are reserved
    assert_eq!(&buf[4..8], &[0, 0, 0, 0], "Reserved bytes should be zero");
}

#[test]
fn test_it_effect_codes() {
    assert_eq!(effects::SET_SPEED, 1, "Set speed is A (1)");
    assert_eq!(effects::POSITION_JUMP, 2, "Position jump is B (2)");
    assert_eq!(effects::PATTERN_BREAK, 3, "Pattern break is C (3)");
    assert_eq!(effects::VOLUME_SLIDE, 4, "Volume slide is D (4)");
    assert_eq!(effects::PORTA_DOWN, 5, "Portamento down is E (5)");
    assert_eq!(effects::PORTA_UP, 6, "Portamento up is F (6)");
    assert_eq!(effects::TONE_PORTA, 7, "Tone portamento is G (7)");
    assert_eq!(effects::VIBRATO, 8, "Vibrato is H (8)");
    assert_eq!(effects::ARPEGGIO, 10, "Arpeggio is J (10)");
    assert_eq!(effects::TEMPO, 20, "Tempo is T (20)");
    assert_eq!(effects::SET_PANNING, 24, "Set panning is X (24)");
}

// =============================================================================
// 4. Sample Data Tests
// =============================================================================

#[test]
fn test_it_sample_creation() {
    let data = vec![0u8; 1000];
    let sample = ItSample::new("Test", data.clone(), 44100);

    assert_eq!(sample.name, "Test");
    assert_eq!(
        sample.length, 500,
        "Length should be in samples (1000 bytes / 2)"
    );
    assert_eq!(sample.c5_speed, 44100);
    assert!(sample.flags & sample_flags::HAS_DATA != 0);
    assert!(sample.flags & sample_flags::BITS_16 != 0);
}

#[test]
fn test_it_sample_flags() {
    assert_eq!(sample_flags::HAS_DATA, 0x01);
    assert_eq!(sample_flags::BITS_16, 0x02);
    assert_eq!(sample_flags::STEREO, 0x04);
    assert_eq!(sample_flags::COMPRESSED, 0x08);
    assert_eq!(sample_flags::LOOP, 0x10);
    assert_eq!(sample_flags::SUSTAIN_LOOP, 0x20);
    assert_eq!(sample_flags::LOOP_PINGPONG, 0x40);
    assert_eq!(sample_flags::SUSTAIN_PINGPONG, 0x80);
}

#[test]
fn test_it_sample_convert_flags() {
    assert_eq!(convert_flags::SIGNED, 0x01);
    assert_eq!(convert_flags::BIG_ENDIAN, 0x02);
    assert_eq!(convert_flags::DELTA, 0x04);
    assert_eq!(convert_flags::BYTE_DELTA, 0x08);
}

#[test]
fn test_it_sample_header_size() {
    let sample = ItSample::new("Test", vec![0u8; 100], 22050);

    let mut buf = Vec::new();
    sample.write_header(&mut buf, 1000).unwrap();

    assert_eq!(
        buf.len(),
        IT_SAMPLE_HEADER_SIZE,
        "Sample header should be 80 bytes"
    );
}

#[test]
fn test_it_sample_header_magic() {
    let sample = ItSample::new("Test", vec![0u8; 100], 22050);

    let mut buf = Vec::new();
    sample.write_header(&mut buf, 1000).unwrap();

    assert_eq!(&buf[0..4], IT_SAMPLE_MAGIC, "Sample magic should be 'IMPS'");
}

#[test]
fn test_it_sample_header_data_pointer() {
    let sample = ItSample::new("Test", vec![0u8; 100], 22050);
    let data_offset = 12345u32;

    let mut buf = Vec::new();
    sample.write_header(&mut buf, data_offset).unwrap();

    // Sample data pointer is at offset 72
    let stored_offset = read_u32_le(&buf, 72);
    assert_eq!(
        stored_offset, data_offset,
        "Sample data pointer should match"
    );
}

#[test]
fn test_it_sample_with_loop() {
    let data = vec![0u8; 1000];
    let sample = ItSample::new("Test", data, 22050).with_loop(0, 400, false);

    assert!(
        sample.flags & sample_flags::LOOP != 0,
        "Loop flag should be set"
    );
    assert_eq!(sample.loop_begin, 0);
    assert_eq!(sample.loop_end, 400);
}

#[test]
fn test_it_sample_with_pingpong_loop() {
    let data = vec![0u8; 1000];
    let sample = ItSample::new("Test", data, 22050).with_loop(100, 400, true);

    assert!(
        sample.flags & sample_flags::LOOP != 0,
        "Loop flag should be set"
    );
    assert!(
        sample.flags & sample_flags::LOOP_PINGPONG != 0,
        "Pingpong flag should be set"
    );
    assert_eq!(sample.loop_begin, 100);
    assert_eq!(sample.loop_end, 400);
}

#[test]
fn test_it_sample_c5_speed() {
    let sample = ItSample::new("Test", vec![0u8; 100], 44100);

    let mut buf = Vec::new();
    sample.write_header(&mut buf, 0).unwrap();

    // C-5 speed is at offset 60
    let c5_speed = read_u32_le(&buf, 60);
    assert_eq!(c5_speed, 44100, "C-5 speed should match");
}

#[test]
fn test_it_sample_default_volume() {
    let mut sample = ItSample::new("Test", vec![0u8; 100], 22050);
    sample.default_volume = 48;

    let mut buf = Vec::new();
    sample.write_header(&mut buf, 0).unwrap();

    // Default volume is at offset 19
    assert_eq!(buf[19], 48, "Default volume should match");
}

#[test]
fn test_it_sample_data_write() {
    let data = vec![0x12, 0x34, 0x56, 0x78];
    let sample = ItSample::new("Test", data.clone(), 22050);

    let mut buf = Vec::new();
    sample.write_data(&mut buf).unwrap();

    assert_eq!(buf, data, "Sample data should be written unchanged");
}

#[test]
fn test_it_sample_quality_16bit() {
    // Generate a simple sine wave
    let sample_count = 100;
    let mut data = Vec::with_capacity(sample_count * 2);
    for i in 0..sample_count {
        let t = i as f64 / sample_count as f64;
        let value = (t * std::f64::consts::TAU).sin();
        let sample = (value * 32767.0) as i16;
        data.extend_from_slice(&sample.to_le_bytes());
    }

    let sample = ItSample::new("Sine", data.clone(), 44100);
    assert_eq!(sample.data, data);
    assert!(sample.flags & sample_flags::BITS_16 != 0);
}

#[test]
fn test_it_sample_vibrato() {
    let mut sample = ItSample::new("Test", vec![0u8; 100], 22050);
    sample.vibrato_speed = 10;
    sample.vibrato_depth = 5;
    sample.vibrato_rate = 2;
    sample.vibrato_type = 1; // Ramp down

    let mut buf = Vec::new();
    sample.write_header(&mut buf, 0).unwrap();

    // Vibrato parameters are at offsets 76-79
    assert_eq!(buf[76], 10, "Vibrato speed");
    assert_eq!(buf[77], 5, "Vibrato depth");
    assert_eq!(buf[78], 2, "Vibrato rate");
    assert_eq!(buf[79], 1, "Vibrato type");
}
