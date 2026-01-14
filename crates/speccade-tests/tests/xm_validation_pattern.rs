//! XM format pattern data validation tests.
//!
//! Tests covering:
//! - Pattern header structure
//! - Pattern packing format
//! - Row count validation
//! - Note value ranges
//! - Packed data encoding

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

/// Add a pattern with packed data to XM data.
fn add_pattern_with_data(data: &mut Vec<u8>, rows: u16, _channels: u16, notes: &[u8]) {
    // Pattern header length (4 bytes)
    data.extend_from_slice(&9u32.to_le_bytes());
    // Packing type (1 byte, always 0)
    data.push(0);
    // Number of rows (2 bytes)
    data.extend_from_slice(&rows.to_le_bytes());
    // Packed pattern data size (2 bytes)
    data.extend_from_slice(&(notes.len() as u16).to_le_bytes());
    // Pattern data
    data.extend_from_slice(notes);
}

// ============================================================================
// Pattern Data Validation Tests
// ============================================================================

#[test]
fn test_xm_pattern_header_format() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    add_empty_pattern(&mut data, 64);

    let report = XmValidator::validate(&data).unwrap();
    assert_eq!(report.patterns.len(), 1);
    assert_eq!(report.patterns[0].header_length, 9);
    assert_eq!(report.patterns[0].packing_type, 0);
    assert_eq!(report.patterns[0].num_rows, 64);
}

#[test]
fn test_xm_pattern_packing_type() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);

    // Invalid packing type
    data.extend_from_slice(&9u32.to_le_bytes()); // Header length
    data.push(1); // Invalid packing type (should be 0)
    data.extend_from_slice(&64u16.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());

    let result = XmValidator::validate(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidPackingType { found: 1, .. })
    ));
}

#[test]
fn test_xm_pattern_row_count() {
    // Valid row counts
    for rows in [1, 32, 64, 128, 256] {
        let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
        add_empty_pattern(&mut data, rows);
        let result = XmValidator::validate(&data);
        assert!(result.is_ok(), "Row count {} should be valid", rows);
    }

    // Invalid: 0 rows
    let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    data.extend_from_slice(&9u32.to_le_bytes());
    data.push(0);
    data.extend_from_slice(&0u16.to_le_bytes()); // 0 rows - invalid
    data.extend_from_slice(&0u16.to_le_bytes());

    let result = XmValidator::validate(&data);
    assert!(matches!(result, Err(XmFormatError::PatternError { .. })));
}

#[test]
fn test_xm_pattern_packed_data() {
    let mut data = create_test_xm_bytes("Test", 2, 1, 0, 6, 125);

    // Create packed pattern data for 2 rows, 2 channels
    let notes = vec![
        // Row 0, Channel 0: Note C-4, Instrument 1
        0x80 | 0x01 | 0x02, // Packing: has note + has instrument
        49,                 // C-4
        1,                  // Instrument 1
        // Row 0, Channel 1: Empty
        0x80, // Empty packed note
        // Row 1, Channel 0: Note off
        0x80 | 0x01,
        97, // Note off
        // Row 1, Channel 1: Empty
        0x80,
    ];

    add_pattern_with_data(&mut data, 2, 2, &notes);

    let report = XmValidator::validate(&data).unwrap();
    assert!(report.valid);
    assert!(!report.patterns[0].is_empty);
}

#[test]
fn test_xm_pattern_empty() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    add_empty_pattern(&mut data, 64);

    let report = XmValidator::validate(&data).unwrap();
    assert!(report.patterns[0].is_empty);
    assert!(report
        .warnings
        .iter()
        .any(|w| matches!(w, XmWarning::EmptyPattern { .. })));
}

#[test]
fn test_xm_pattern_note_values() {
    let mut data = create_test_xm_bytes("Test", 1, 1, 0, 6, 125);

    // Valid notes: 1-96 (C-0 to B-7) and 97 (note off)
    let notes = vec![
        0x80 | 0x01,
        1, // C-0 (lowest)
        0x80 | 0x01,
        96, // B-7 (highest)
        0x80 | 0x01,
        97,   // Note off
        0x80, // Empty
    ];

    add_pattern_with_data(&mut data, 4, 1, &notes);

    let report = XmValidator::validate(&data).unwrap();
    assert!(report.valid);
}

#[test]
fn test_xm_pattern_invalid_note() {
    let mut data = create_test_xm_bytes("Test", 1, 1, 0, 6, 125);

    // Invalid note value (>97)
    let notes = vec![0x80 | 0x01, 100]; // Invalid note

    add_pattern_with_data(&mut data, 1, 1, &notes);

    let report = XmValidator::validate(&data).unwrap();
    assert!(!report.valid);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, XmFormatError::InvalidNoteValue { value: 100, .. })));
}
