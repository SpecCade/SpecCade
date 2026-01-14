//! XM format header validation tests.
//!
//! Tests covering:
//! - Basic header structure (offsets 0x00-0x3C)
//! - Extended header fields (song length, channels, patterns, etc.)
//! - Pattern order table validation

use speccade_backend_music::xm::{
    XmFormatError, XmValidator, XmWarning, XM_ID_TEXT, XM_MAGIC_BYTE, XM_STANDARD_HEADER_SIZE,
    XM_VERSION_104,
};

// ============================================================================
// Test Helper Functions
// ============================================================================

/// Create a minimal valid XM file header for testing.
fn create_test_xm_bytes(
    name: &str,
    channels: u16,
    patterns: u16,
    instruments: u16,
    tempo: u16,
    bpm: u16,
) -> Vec<u8> {
    let mut data = vec![0u8; 336]; // Minimum header size (60 + 276)

    // Offset 0x00-0x10: ID text "Extended Module: "
    data[0..17].copy_from_slice(XM_ID_TEXT);

    // Offset 0x11-0x24: Module name (20 bytes)
    let name_bytes = name.as_bytes();
    let copy_len = name_bytes.len().min(20);
    data[17..17 + copy_len].copy_from_slice(&name_bytes[..copy_len]);

    // Offset 0x25: Magic byte 0x1A
    data[37] = XM_MAGIC_BYTE;

    // Offset 0x26-0x39: Tracker name (20 bytes)
    data[38..58].copy_from_slice(b"FastTracker v2.00   ");

    // Offset 0x3A-0x3B: Version (0x0104)
    data[58..60].copy_from_slice(&XM_VERSION_104.to_le_bytes());

    // Offset 0x3C-0x3F: Header size
    data[60..64].copy_from_slice(&XM_STANDARD_HEADER_SIZE.to_le_bytes());

    // Offset 0x40-0x41: Song length
    data[64..66].copy_from_slice(&1u16.to_le_bytes());

    // Offset 0x42-0x43: Restart position
    data[66..68].copy_from_slice(&0u16.to_le_bytes());

    // Offset 0x44-0x45: Number of channels
    data[68..70].copy_from_slice(&channels.to_le_bytes());

    // Offset 0x46-0x47: Number of patterns
    data[70..72].copy_from_slice(&patterns.to_le_bytes());

    // Offset 0x48-0x49: Number of instruments
    data[72..74].copy_from_slice(&instruments.to_le_bytes());

    // Offset 0x4A-0x4B: Flags (1 = linear frequency table)
    data[74..76].copy_from_slice(&1u16.to_le_bytes());

    // Offset 0x4C-0x4D: Default tempo
    data[76..78].copy_from_slice(&tempo.to_le_bytes());

    // Offset 0x4E-0x4F: Default BPM
    data[78..80].copy_from_slice(&bpm.to_le_bytes());

    // Offset 0x50-0x14F: Pattern order table (256 bytes) - already zeroed

    data
}

/// Add an empty pattern header to XM data.
fn add_empty_pattern(data: &mut Vec<u8>, rows: u16) {
    // Pattern header length (4 bytes)
    data.extend_from_slice(&9u32.to_le_bytes());
    // Packing type (1 byte, always 0)
    data.push(0);
    // Number of rows (2 bytes)
    data.extend_from_slice(&rows.to_le_bytes());
    // Packed pattern data size (2 bytes, 0 for empty)
    data.extend_from_slice(&0u16.to_le_bytes());
}

// ============================================================================
// Header Validation Tests (Offset 0x00-0x3C)
// ============================================================================

#[test]
fn test_xm_header_id_text_validation() {
    let data = create_test_xm_bytes("Test", 4, 0, 0, 6, 125);
    let result = XmValidator::validate_header_only(&data);
    assert!(result.is_ok(), "Valid ID text should pass");

    // Test invalid ID text
    let mut bad_data = data.clone();
    bad_data[0..17].copy_from_slice(b"Invalid Module:  ");
    let result = XmValidator::validate_header_only(&bad_data);
    assert!(matches!(result, Err(XmFormatError::InvalidIdText { .. })));
}

#[test]
fn test_xm_header_module_name() {
    let data = create_test_xm_bytes("Test Song Name 12345", 4, 0, 0, 6, 125);
    let result = XmValidator::validate_header_only(&data).unwrap();
    assert_eq!(result.name, "Test Song Name 12345");

    // Test truncated name
    let data = create_test_xm_bytes("This Name Is Way Too Long For XM", 4, 0, 0, 6, 125);
    let result = XmValidator::validate_header_only(&data).unwrap();
    assert_eq!(result.name.len(), 20);
}

#[test]
fn test_xm_header_magic_byte_0x1a() {
    let data = create_test_xm_bytes("Test", 4, 0, 0, 6, 125);
    assert_eq!(data[37], 0x1A, "Magic byte should be at offset 37");

    let mut bad_data = data.clone();
    bad_data[37] = 0x00;
    let result = XmValidator::validate_header_only(&bad_data);
    assert!(matches!(
        result,
        Err(XmFormatError::MissingMagicByte {
            found: 0x00,
            offset: 37
        })
    ));
}

#[test]
fn test_xm_header_tracker_name() {
    let data = create_test_xm_bytes("Test", 4, 0, 0, 6, 125);
    let result = XmValidator::validate_header_only(&data).unwrap();
    assert!(result.tracker_name.starts_with("FastTracker"));

    // Test non-standard tracker name generates warning
    let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    data[38..58].copy_from_slice(b"SpecCade XM Writer  ");
    add_empty_pattern(&mut data, 64);

    let report = XmValidator::validate(&data).unwrap();
    assert!(report
        .warnings
        .iter()
        .any(|w| matches!(w, XmWarning::NonStandardTrackerName { .. })));
}

#[test]
fn test_xm_header_version_0x0104() {
    let data = create_test_xm_bytes("Test", 4, 0, 0, 6, 125);
    let result = XmValidator::validate_header_only(&data).unwrap();
    assert_eq!(result.version, 0x0104);

    // Test unsupported version
    let mut bad_data = data.clone();
    bad_data[58..60].copy_from_slice(&0x0200u16.to_le_bytes());
    let result = XmValidator::validate_header_only(&bad_data);
    assert!(matches!(
        result,
        Err(XmFormatError::UnsupportedVersion { version: 0x0200 })
    ));
}

#[test]
fn test_xm_header_size() {
    let data = create_test_xm_bytes("Test", 4, 0, 0, 6, 125);
    let header_size = u32::from_le_bytes([data[60], data[61], data[62], data[63]]);
    assert_eq!(header_size, 276, "Standard header size should be 276");

    // Non-standard header size should generate warning
    let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    data[60..64].copy_from_slice(&280u32.to_le_bytes());
    // Pad data to account for larger header
    data.resize(data.len() + 4, 0);
    add_empty_pattern(&mut data, 64);

    let report = XmValidator::validate(&data).unwrap();
    assert!(report
        .warnings
        .iter()
        .any(|w| matches!(w, XmWarning::NonStandardHeaderSize { .. })));
}

// ============================================================================
// Extended Header Field Tests
// ============================================================================

#[test]
fn test_xm_song_length_validation() {
    // Valid song length
    let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    data[64..66].copy_from_slice(&10u16.to_le_bytes());
    add_empty_pattern(&mut data, 64);
    let result = XmValidator::validate(&data);
    assert!(result.is_ok());

    // Invalid song length (0)
    let mut bad_data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    bad_data[64..66].copy_from_slice(&0u16.to_le_bytes());
    let result = XmValidator::validate_header_only(&bad_data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidSongLength { .. })
    ));

    // Invalid song length (>256)
    let mut bad_data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    bad_data[64..66].copy_from_slice(&300u16.to_le_bytes());
    let result = XmValidator::validate_header_only(&bad_data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidSongLength { .. })
    ));
}

#[test]
fn test_xm_restart_position_validation() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    data[64..66].copy_from_slice(&10u16.to_le_bytes()); // Song length = 10
    data[66..68].copy_from_slice(&5u16.to_le_bytes()); // Restart at 5 (valid)
    add_empty_pattern(&mut data, 64);
    let result = XmValidator::validate(&data);
    assert!(result.is_ok());

    // Invalid restart position (>= song length)
    let mut bad_data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    bad_data[64..66].copy_from_slice(&10u16.to_le_bytes());
    bad_data[66..68].copy_from_slice(&10u16.to_le_bytes());
    let result = XmValidator::validate_header_only(&bad_data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidRestartPosition { .. })
    ));
}

#[test]
fn test_xm_channel_count_validation() {
    // Valid channel counts
    for channels in [1, 4, 8, 16, 32] {
        let mut data = create_test_xm_bytes("Test", channels, 1, 0, 6, 125);
        add_empty_pattern(&mut data, 64);
        let result = XmValidator::validate(&data);
        assert!(result.is_ok(), "Channel count {} should be valid", channels);
    }

    // Invalid: 0 channels
    let data = create_test_xm_bytes("Test", 0, 1, 0, 6, 125);
    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidChannelCount { channels: 0, .. })
    ));

    // Invalid: too many channels
    let data = create_test_xm_bytes("Test", 64, 1, 0, 6, 125);
    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidChannelCount { channels: 64, .. })
    ));
}

#[test]
fn test_xm_pattern_count_validation() {
    // Valid pattern counts
    let mut data = create_test_xm_bytes("Test", 4, 10, 0, 6, 125);
    for _ in 0..10 {
        add_empty_pattern(&mut data, 64);
    }
    let result = XmValidator::validate(&data);
    assert!(result.is_ok());

    // Invalid: too many patterns
    let data = create_test_xm_bytes("Test", 4, 300, 0, 6, 125);
    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidPatternCount { patterns: 300, .. })
    ));
}

#[test]
fn test_xm_instrument_count_validation() {
    // Valid instrument count
    let data = create_test_xm_bytes("Test", 4, 1, 2, 6, 125);
    let result = XmValidator::validate_header_only(&data);
    assert!(result.is_ok());

    // Invalid: too many instruments
    let data = create_test_xm_bytes("Test", 4, 1, 200, 6, 125);
    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidInstrumentCount {
            instruments: 200,
            ..
        })
    ));
}

#[test]
fn test_xm_frequency_table_flag() {
    // Linear frequency table (bit 0 = 1)
    let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    data[74..76].copy_from_slice(&1u16.to_le_bytes());
    add_empty_pattern(&mut data, 64);
    let report = XmValidator::validate(&data).unwrap();
    assert!(report.header.as_ref().unwrap().linear_frequency_table);

    // Amiga frequency table (bit 0 = 0)
    let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    data[74..76].copy_from_slice(&0u16.to_le_bytes());
    add_empty_pattern(&mut data, 64);
    let report = XmValidator::validate(&data).unwrap();
    assert!(!report.header.as_ref().unwrap().linear_frequency_table);
}

#[test]
fn test_xm_tempo_validation() {
    // Valid tempo range (1-31)
    for tempo in [1, 6, 15, 31] {
        let mut data = create_test_xm_bytes("Test", 4, 1, 0, tempo, 125);
        add_empty_pattern(&mut data, 64);
        let result = XmValidator::validate(&data);
        assert!(result.is_ok(), "Tempo {} should be valid", tempo);
    }

    // Invalid: tempo 0
    let data = create_test_xm_bytes("Test", 4, 1, 0, 0, 125);
    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidTempo { tempo: 0, .. })
    ));

    // Invalid: tempo > 31
    let data = create_test_xm_bytes("Test", 4, 1, 0, 50, 125);
    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidTempo { tempo: 50, .. })
    ));
}

#[test]
fn test_xm_bpm_validation() {
    // Valid BPM range (32-255)
    for bpm in [32, 100, 125, 200, 255] {
        let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, bpm);
        add_empty_pattern(&mut data, 64);
        let result = XmValidator::validate(&data);
        assert!(result.is_ok(), "BPM {} should be valid", bpm);
    }

    // Invalid: BPM too low
    let data = create_test_xm_bytes("Test", 4, 1, 0, 6, 20);
    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidBpm { bpm: 20, .. })
    ));
}

#[test]
fn test_xm_pattern_order_table() {
    let mut data = create_test_xm_bytes("Test", 4, 3, 0, 6, 125);
    data[64..66].copy_from_slice(&3u16.to_le_bytes()); // Song length = 3
    data[80] = 0; // Order 0: pattern 0
    data[81] = 2; // Order 1: pattern 2
    data[82] = 1; // Order 2: pattern 1
    add_empty_pattern(&mut data, 64);
    add_empty_pattern(&mut data, 64);
    add_empty_pattern(&mut data, 64);

    let report = XmValidator::validate(&data).unwrap();
    let header = report.header.unwrap();
    assert_eq!(header.pattern_order[0], 0);
    assert_eq!(header.pattern_order[1], 2);
    assert_eq!(header.pattern_order[2], 1);
}

#[test]
fn test_xm_pattern_order_invalid_reference() {
    let mut data = create_test_xm_bytes("Test", 4, 2, 0, 6, 125);
    data[64..66].copy_from_slice(&2u16.to_le_bytes()); // Song length = 2
    data[80] = 0; // Valid
    data[81] = 5; // Invalid: references pattern 5 but only 2 patterns exist

    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidPatternOrder {
            position: 1,
            pattern: 5,
            ..
        })
    ));
}
