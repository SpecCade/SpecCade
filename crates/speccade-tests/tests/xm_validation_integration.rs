//! XM format integration and full validation tests.
//!
//! Tests covering:
//! - Complete file validation
//! - Quick check/identification
//! - File truncation detection
//! - Validation report structure
//! - Integration with XM writer

use speccade_backend_music::xm::{
    XmEnvelope, XmInstrument, XmModule, XmNote, XmPattern, XmSample, XmValidator, XM_ID_TEXT,
    XM_MAGIC_BYTE, XM_STANDARD_HEADER_SIZE, XM_VERSION_104,
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

/// Add a minimal instrument with no samples to XM data.
fn add_empty_instrument(data: &mut Vec<u8>, name: &str) {
    // Instrument size (4 bytes)
    data.extend_from_slice(&29u32.to_le_bytes());
    // Instrument name (22 bytes)
    let mut name_buf = [0u8; 22];
    let name_bytes = name.as_bytes();
    let copy_len = name_bytes.len().min(22);
    name_buf[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
    data.extend_from_slice(&name_buf);
    // Instrument type (1 byte, always 0)
    data.push(0);
    // Number of samples (2 bytes)
    data.extend_from_slice(&0u16.to_le_bytes());
}

// ============================================================================
// Full Validation Tests
// ============================================================================

#[test]
fn test_xm_full_validation_complete_file() {
    // Create a complete XM file using the writer
    let mut module = XmModule::new("Full Test Song", 4, 6, 120);

    // Add patterns
    let mut pattern = XmPattern::empty(64, 4);
    pattern.set_note(0, 0, XmNote::from_name("C4", 1, Some(64)));
    pattern.set_note(16, 0, XmNote::from_name("E4", 1, Some(64)));
    pattern.set_note(32, 0, XmNote::from_name("G4", 1, Some(64)));
    pattern.set_note(48, 0, XmNote::note_off());
    module.add_pattern(pattern);

    // Add instrument with sample
    let sample_data = vec![0u8; 1000];
    let sample = XmSample::new("Lead Sample", sample_data, true);
    let envelope = XmEnvelope::adsr(10, 20, 48, 30);
    let instrument = XmInstrument::new("Lead Synth", sample).with_volume_envelope(envelope);
    module.add_instrument(instrument);

    module.set_order_table(&[0]);

    // Generate XM bytes
    let bytes = module.to_bytes().unwrap();

    // Validate
    let report = XmValidator::validate(&bytes).unwrap();
    assert!(report.valid, "Generated XM should be valid");
    assert!(report.errors.is_empty(), "Should have no errors");

    // Verify header info
    let header = report.header.as_ref().unwrap();
    assert_eq!(header.name, "Full Test Song");
    assert_eq!(header.num_channels, 4);
    assert_eq!(header.num_patterns, 1);
    assert_eq!(header.num_instruments, 1);
    assert_eq!(header.default_tempo, 6);
    assert_eq!(header.default_bpm, 120);

    // Verify pattern info
    assert_eq!(report.patterns.len(), 1);
    assert!(!report.patterns[0].is_empty);

    // Verify instrument info
    assert_eq!(report.instruments.len(), 1);
}

#[test]
fn test_xm_quick_check() {
    let valid_data = create_test_xm_bytes("Test", 4, 0, 0, 6, 125);
    assert!(XmValidator::is_xm(&valid_data));

    let invalid_data = vec![0u8; 100];
    assert!(!XmValidator::is_xm(&invalid_data));

    let short_data = vec![0u8; 50];
    assert!(!XmValidator::is_xm(&short_data));

    // Not XM header but right length
    let mut wrong_magic = valid_data.clone();
    wrong_magic[0..17].copy_from_slice(b"RIFF chunk size  "); // 17 bytes
    assert!(!XmValidator::is_xm(&wrong_magic));
}

#[test]
fn test_xm_file_truncation_detection() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 0, 6, 125);
    add_empty_pattern(&mut data, 64);

    // Truncate file
    data.truncate(300);

    let result = XmValidator::validate(&data);
    // Should fail with truncation error during pattern parsing
    assert!(result.is_err() || !result.unwrap().valid);
}

#[test]
fn test_xm_validation_report_structure() {
    let mut data = create_test_xm_bytes("Test", 4, 2, 1, 6, 125);
    add_empty_pattern(&mut data, 64);
    add_empty_pattern(&mut data, 32);
    add_empty_instrument(&mut data, "Test Inst");

    let report = XmValidator::validate(&data).unwrap();

    // Verify report structure
    assert!(report.header.is_some());
    assert_eq!(report.patterns.len(), 2);
    assert_eq!(report.instruments.len(), 1);
    assert_eq!(report.file_size, data.len());

    // Check for expected warnings
    assert!(report.has_warnings()); // Empty patterns + instrument without samples
}
