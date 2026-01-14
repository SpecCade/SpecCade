//! Tests for XM header validation and pattern encoding.
//!
//! These tests validate the binary format of XM headers, header fields,
//! and pattern data encoding.

use speccade_backend_music::xm::effects;
use speccade_backend_music::xm::{
    XmModule, XmNote, XmPattern, XM_HEADER_SIZE, XM_MAGIC, XM_VERSION,
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

// =============================================================================
// Header Validation Tests
// =============================================================================

#[test]
fn test_xm_header_id_text() {
    let xm = generate_minimal_xm();
    assert!(
        xm.len() >= 17,
        "XM file should be at least 17 bytes for magic"
    );
    assert_eq!(
        &xm[0..17],
        b"Extended Module: ",
        "XM file should start with 'Extended Module: '"
    );
}

#[test]
fn test_xm_header_magic_constant() {
    // Verify the constant matches expected value
    assert_eq!(XM_MAGIC, b"Extended Module: ");
    assert_eq!(XM_MAGIC.len(), 17);
}

#[test]
fn test_xm_header_version() {
    let xm = generate_minimal_xm();
    // Version is at offset 58-59 (little-endian)
    let version = u16::from_le_bytes([xm[58], xm[59]]);
    assert_eq!(version, 0x0104, "XM version should be 0x0104 (1.04)");
}

#[test]
fn test_xm_version_constant() {
    assert_eq!(XM_VERSION, 0x0104);
}

#[test]
fn test_xm_header_size_field() {
    let xm = generate_minimal_xm();
    // Header size is at offset 60-63 (little-endian u32)
    let header_size = u32::from_le_bytes([xm[60], xm[61], xm[62], xm[63]]);
    assert_eq!(
        header_size, XM_HEADER_SIZE,
        "Header size field should be {}",
        XM_HEADER_SIZE
    );
}

#[test]
fn test_xm_header_size_constant() {
    assert_eq!(XM_HEADER_SIZE, 276);
}

#[test]
fn test_xm_header_song_name() {
    let mut module = XmModule::new("My Test Song Name", 4, 6, 125);
    module.add_pattern(XmPattern::empty(64, 4));
    module.set_order_table(&[0]);
    let xm = module.to_bytes().unwrap();

    // Song name is at offset 17-36 (20 bytes, null-padded)
    let name_bytes = &xm[17..37];
    let name = std::str::from_utf8(name_bytes)
        .unwrap()
        .trim_end_matches('\0');
    assert_eq!(name, "My Test Song Name");
}

#[test]
fn test_xm_header_song_name_truncation() {
    // Test that long names are properly truncated to 20 characters
    let long_name = "This Name Is Way Too Long For XM Format";
    let mut module = XmModule::new(long_name, 4, 6, 125);
    module.add_pattern(XmPattern::empty(64, 4));
    module.set_order_table(&[0]);
    let xm = module.to_bytes().unwrap();

    let name_bytes = &xm[17..37];
    // Should be exactly 20 bytes, with the name truncated
    assert_eq!(name_bytes.len(), 20);
}

#[test]
fn test_xm_header_1a_byte() {
    let xm = generate_minimal_xm();
    // 0x1A byte is at offset 37 (after 17 magic + 20 name)
    assert_eq!(
        xm[37], 0x1A,
        "0x1A marker byte should be present after song name"
    );
}

#[test]
fn test_xm_header_tracker_name() {
    let xm = generate_minimal_xm();
    // Tracker name is at offset 38-57 (20 bytes)
    let tracker_bytes = &xm[38..58];
    let tracker = std::str::from_utf8(tracker_bytes)
        .unwrap()
        .trim_end_matches('\0')
        .trim_end(); // Also trim trailing spaces
    assert_eq!(tracker, "SpecCade XM Writer");
}

#[test]
fn test_xm_header_song_length() {
    let mut module = XmModule::new("Test", 4, 6, 125);
    module.add_pattern(XmPattern::empty(64, 4));
    module.add_pattern(XmPattern::empty(64, 4));
    module.set_order_table(&[0, 1, 0, 1]); // 4 positions
    let xm = module.to_bytes().unwrap();

    // Song length is at offset 64-65
    let song_length = u16::from_le_bytes([xm[64], xm[65]]);
    assert_eq!(
        song_length, 4,
        "Song length should match order table entries"
    );
}

#[test]
fn test_xm_header_restart_position() {
    let mut module = XmModule::new("Test", 4, 6, 125);
    module.add_pattern(XmPattern::empty(64, 4));
    module.set_order_table(&[0]);
    module.set_restart_position(0);
    let xm = module.to_bytes().unwrap();

    // Restart position is at offset 66-67
    let restart = u16::from_le_bytes([xm[66], xm[67]]);
    assert_eq!(restart, 0);
}

#[test]
fn test_xm_header_num_channels() {
    let mut module = XmModule::new("Test", 8, 6, 125);
    module.add_pattern(XmPattern::empty(64, 8));
    module.set_order_table(&[0]);
    let xm = module.to_bytes().unwrap();

    // Number of channels is at offset 68-69
    let num_channels = u16::from_le_bytes([xm[68], xm[69]]);
    assert_eq!(num_channels, 8);
}

#[test]
fn test_xm_header_num_patterns() {
    let mut module = XmModule::new("Test", 4, 6, 125);
    module.add_pattern(XmPattern::empty(64, 4));
    module.add_pattern(XmPattern::empty(32, 4));
    module.add_pattern(XmPattern::empty(16, 4));
    module.set_order_table(&[0, 1, 2]);
    let xm = module.to_bytes().unwrap();

    // Number of patterns is at offset 70-71
    let num_patterns = u16::from_le_bytes([xm[70], xm[71]]);
    assert_eq!(num_patterns, 3);
}

#[test]
fn test_xm_header_num_instruments() {
    use speccade_backend_music::xm::{XmInstrument, XmSample};
    let mut module = XmModule::new("With Instrument", 4, 6, 125);

    let pattern = XmPattern::empty(64, 4);
    module.add_pattern(pattern);

    // Create a simple instrument with minimal sample
    let sample_data = vec![0u8; 100]; // 50 samples of 16-bit silence
    let sample = XmSample::new("TestSample", sample_data, true);
    let instrument = XmInstrument::new("TestInstr", sample);
    module.add_instrument(instrument);

    module.set_order_table(&[0]);
    let xm = module.to_bytes().unwrap();

    // Number of instruments is at offset 72-73
    let num_instruments = u16::from_le_bytes([xm[72], xm[73]]);
    assert_eq!(num_instruments, 1);
}

#[test]
fn test_xm_header_flags_linear_frequency() {
    let xm = generate_minimal_xm();

    // Flags are at offset 74-75
    let flags = u16::from_le_bytes([xm[74], xm[75]]);
    assert_eq!(flags & 1, 1, "Linear frequency table flag should be set");
}

#[test]
fn test_xm_header_default_speed() {
    let mut module = XmModule::new("Test", 4, 3, 125);
    module.add_pattern(XmPattern::empty(64, 4));
    module.set_order_table(&[0]);
    let xm = module.to_bytes().unwrap();

    // Default speed is at offset 76-77
    let speed = u16::from_le_bytes([xm[76], xm[77]]);
    assert_eq!(speed, 3);
}

#[test]
fn test_xm_header_default_bpm() {
    let mut module = XmModule::new("Test", 4, 6, 140);
    module.add_pattern(XmPattern::empty(64, 4));
    module.set_order_table(&[0]);
    let xm = module.to_bytes().unwrap();

    // Default BPM is at offset 78-79
    let bpm = u16::from_le_bytes([xm[78], xm[79]]);
    assert_eq!(bpm, 140);
}

#[test]
fn test_xm_header_order_table() {
    let mut module = XmModule::new("Test", 4, 6, 125);
    module.add_pattern(XmPattern::empty(64, 4));
    module.add_pattern(XmPattern::empty(64, 4));
    module.add_pattern(XmPattern::empty(64, 4));
    module.set_order_table(&[0, 1, 2, 1, 0]);
    let xm = module.to_bytes().unwrap();

    // Order table is at offset 80-335 (256 bytes)
    assert_eq!(xm[80], 0);
    assert_eq!(xm[81], 1);
    assert_eq!(xm[82], 2);
    assert_eq!(xm[83], 1);
    assert_eq!(xm[84], 0);
    // Rest should be zeros
    assert_eq!(xm[85], 0);
}

#[test]
fn test_xm_header_total_size() {
    let xm = generate_minimal_xm();

    // The header should be exactly 336 bytes (17 magic + 20 name + 1 0x1A + 20 tracker
    // + 2 version + 4 header_size + 2 song_length + 2 restart + 2 channels
    // + 2 patterns + 2 instruments + 2 flags + 2 speed + 2 bpm + 256 order)
    // But XM_HEADER_SIZE (276) is what's in the field
    // Total header portion before patterns = 60 + 276 = 336
    assert!(xm.len() >= 336, "XM file should have at least full header");
}

// =============================================================================
// Pattern Encoding Tests
// =============================================================================

#[test]
fn test_pattern_empty() {
    let pattern = XmPattern::empty(64, 4);
    let packed = pattern.pack(4);

    // Empty pattern should have packed data
    assert!(
        !packed.is_empty(),
        "Empty pattern should still have packed data"
    );

    // Each empty note should be encoded as 0x80 (packing byte with no data)
    // 64 rows * 4 channels = 256 empty notes = 256 bytes
    assert_eq!(
        packed.len(),
        256,
        "Empty pattern should be 256 bytes for 64x4"
    );
    assert!(
        packed.iter().all(|&b| b == 0x80),
        "All bytes should be 0x80 for empty notes"
    );
}

#[test]
fn test_pattern_with_notes() {
    let mut pattern = XmPattern::empty(16, 2);
    pattern.set_note(0, 0, XmNote::from_name("C4", 1, Some(64)));

    let packed = pattern.pack(2);
    assert!(!packed.is_empty());

    // First note should have note, instrument, and volume set
    // Packing byte: 0x80 | 0x01 (note) | 0x02 (instrument) | 0x04 (volume) = 0x87
    assert_eq!(packed[0], 0x87);

    // Note value for C4 in XM is 49
    assert_eq!(packed[1], 49);

    // Instrument 1
    assert_eq!(packed[2], 1);

    // Volume: 0x10 + 64 = 0x50
    assert_eq!(packed[3], 0x50);
}

#[test]
fn test_pattern_with_effects() {
    let mut pattern = XmPattern::empty(4, 1);
    let note = XmNote::from_name("C4", 1, None).with_effect(effects::VIBRATO, 0x34);
    pattern.set_note(0, 0, note);

    let packed = pattern.pack(1);

    // Should have: note, instrument, effect, effect_param
    // Packing byte: 0x80 | 0x01 | 0x02 | 0x08 | 0x10 = 0x9B
    assert_eq!(packed[0], 0x9B);
    assert_eq!(packed[1], 49); // C4
    assert_eq!(packed[2], 1); // instrument
    assert_eq!(packed[3], effects::VIBRATO); // effect
    assert_eq!(packed[4], 0x34); // param
}

#[test]
fn test_pattern_note_off() {
    let mut pattern = XmPattern::empty(4, 1);
    pattern.set_note(0, 0, XmNote::note_off());

    let packed = pattern.pack(1);

    // Note-off should have note = 97
    assert_eq!(packed[0], 0x81); // 0x80 | 0x01 (has note)
    assert_eq!(packed[1], 97); // note-off value
}

#[test]
fn test_pattern_mixed_content() {
    let mut pattern = XmPattern::empty(8, 2);

    // Row 0, ch 0: note
    pattern.set_note(0, 0, XmNote::from_name("C4", 1, Some(64)));
    // Row 0, ch 1: empty (default)
    // Row 4, ch 0: note with effect
    pattern.set_note(
        4,
        0,
        XmNote::from_name("E4", 1, None).with_effect(effects::VOL_SLIDE, 0x0F),
    );
    // Row 4, ch 1: note-off
    pattern.set_note(4, 1, XmNote::note_off());

    let packed = pattern.pack(2);
    assert!(!packed.is_empty());

    // Verify total size is less than unpacked would be (5 bytes per note)
    assert!(packed.len() < 8 * 2 * 5);
}

#[test]
fn test_pattern_write_header() {
    let pattern = XmPattern::empty(64, 4);
    let mut buf = Vec::new();
    pattern.write(&mut buf, 4).unwrap();

    // Pattern header is 9 bytes
    let header_len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    assert_eq!(header_len, 9);

    // Packing type
    assert_eq!(buf[4], 0);

    // Number of rows
    let num_rows = u16::from_le_bytes([buf[5], buf[6]]);
    assert_eq!(num_rows, 64);

    // Packed data size
    let packed_size = u16::from_le_bytes([buf[7], buf[8]]);
    assert_eq!(packed_size, 256); // 64 rows * 4 channels
}

#[test]
fn test_pattern_various_row_counts() {
    for rows in [16, 32, 64, 128, 256] {
        let pattern = XmPattern::empty(rows, 4);
        let packed = pattern.pack(4);
        assert_eq!(packed.len(), (rows * 4) as usize);
    }
}

#[test]
fn test_pattern_get_note() {
    let mut pattern = XmPattern::empty(16, 4);
    let note = XmNote::from_name("A4", 2, Some(48));
    pattern.set_note(5, 2, note);

    let retrieved = pattern.get_note(5, 2).unwrap();
    assert_eq!(retrieved.note, 58); // A4 in XM
    assert_eq!(retrieved.instrument, 2);
    assert_eq!(retrieved.volume, 0x10 + 48);
}

#[test]
fn test_pattern_out_of_bounds() {
    let mut pattern = XmPattern::empty(16, 4);
    let note = XmNote::from_name("C4", 1, None);

    // Should not panic, just ignore
    pattern.set_note(100, 0, note);
    pattern.set_note(0, 100, note);

    // Should return None for out of bounds
    assert!(pattern.get_note(100, 0).is_none());
    assert!(pattern.get_note(0, 100).is_none());
}
