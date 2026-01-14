//! Tests for XM validator.

use super::*;

/// Create a minimal valid XM header for testing.
fn create_minimal_xm(
    name: &str,
    channels: u16,
    patterns: u16,
    instruments: u16,
    tempo: u16,
    bpm: u16,
) -> Vec<u8> {
    let mut data = vec![0u8; constants::XM_FULL_HEADER_SIZE];

    // ID text
    data[0..17].copy_from_slice(constants::XM_ID_TEXT);

    // Module name
    let name_bytes = name.as_bytes();
    let copy_len = name_bytes.len().min(20);
    data[17..17 + copy_len].copy_from_slice(&name_bytes[..copy_len]);

    // Magic byte
    data[37] = constants::XM_MAGIC_BYTE;

    // Tracker name
    data[38..58].copy_from_slice(b"FastTracker v2.00   ");

    // Version
    data[58..60].copy_from_slice(&constants::XM_VERSION_104.to_le_bytes());

    // Header size
    data[60..64].copy_from_slice(&constants::XM_STANDARD_HEADER_SIZE.to_le_bytes());

    // Song length
    data[64..66].copy_from_slice(&1u16.to_le_bytes());

    // Restart position
    data[66..68].copy_from_slice(&0u16.to_le_bytes());

    // Number of channels
    data[68..70].copy_from_slice(&channels.to_le_bytes());

    // Number of patterns
    data[70..72].copy_from_slice(&patterns.to_le_bytes());

    // Number of instruments
    data[72..74].copy_from_slice(&instruments.to_le_bytes());

    // Flags
    data[74..76].copy_from_slice(&1u16.to_le_bytes());

    // Tempo
    data[76..78].copy_from_slice(&tempo.to_le_bytes());

    // BPM
    data[78..80].copy_from_slice(&bpm.to_le_bytes());

    data
}

/// Add an empty pattern to XM data.
fn add_empty_pattern(data: &mut Vec<u8>, rows: u16) {
    // Pattern header length
    data.extend_from_slice(&9u32.to_le_bytes());
    // Packing type
    data.push(0);
    // Number of rows
    data.extend_from_slice(&rows.to_le_bytes());
    // Packed size (0 for empty)
    data.extend_from_slice(&0u16.to_le_bytes());
}

#[test]
fn test_xm_header_format() {
    let data = create_minimal_xm("Test Song", 8, 1, 0, 6, 125);
    add_empty_pattern(&mut data.clone(), 64);

    let result = XmValidator::validate_header_only(&data);
    assert!(result.is_ok());

    let header = result.unwrap();
    assert_eq!(header.name, "Test Song");
    assert_eq!(header.num_channels, 8);
    assert_eq!(header.num_patterns, 1);
    assert_eq!(header.num_instruments, 0);
    assert_eq!(header.default_tempo, 6);
    assert_eq!(header.default_bpm, 125);
    assert!(header.linear_frequency_table);
}

#[test]
fn test_xm_invalid_magic() {
    let mut data = create_minimal_xm("Test", 4, 1, 0, 6, 125);
    data[0..17].copy_from_slice(b"Not an XM file!! ");

    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(result, Err(XmFormatError::InvalidIdText { .. })));
}

#[test]
fn test_xm_missing_magic_byte() {
    let mut data = create_minimal_xm("Test", 4, 1, 0, 6, 125);
    data[37] = 0x00;

    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::MissingMagicByte { .. })
    ));
}

#[test]
fn test_xm_invalid_version() {
    let mut data = create_minimal_xm("Test", 4, 1, 0, 6, 125);
    data[58..60].copy_from_slice(&0x0200u16.to_le_bytes());

    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::UnsupportedVersion { .. })
    ));
}

#[test]
fn test_xm_invalid_channel_count() {
    // Too few channels
    let data = create_minimal_xm("Test", 0, 1, 0, 6, 125);
    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidChannelCount { .. })
    ));

    // Too many channels
    let data = create_minimal_xm("Test", 64, 1, 0, 6, 125);
    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidChannelCount { .. })
    ));
}

#[test]
fn test_xm_invalid_tempo() {
    let data = create_minimal_xm("Test", 4, 1, 0, 0, 125);
    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(result, Err(XmFormatError::InvalidTempo { .. })));

    let data = create_minimal_xm("Test", 4, 1, 0, 50, 125);
    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(result, Err(XmFormatError::InvalidTempo { .. })));
}

#[test]
fn test_xm_invalid_bpm() {
    let data = create_minimal_xm("Test", 4, 1, 0, 6, 20);
    let result = XmValidator::validate_header_only(&data);
    assert!(matches!(result, Err(XmFormatError::InvalidBpm { .. })));
}

#[test]
fn test_xm_pattern_encoding() {
    let mut data = create_minimal_xm("Test", 4, 1, 0, 6, 125);
    add_empty_pattern(&mut data, 64);

    let result = XmValidator::validate(&data);
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(report.valid);
    assert_eq!(report.patterns.len(), 1);
    assert!(report.patterns[0].is_empty);
}

#[test]
fn test_xm_pattern_with_data() {
    let mut data = create_minimal_xm("Test", 2, 1, 0, 6, 125);

    // Pattern header
    data.extend_from_slice(&9u32.to_le_bytes()); // Header length
    data.push(0); // Packing type
    data.extend_from_slice(&4u16.to_le_bytes()); // 4 rows
    data.extend_from_slice(&8u16.to_le_bytes()); // Packed size (2 channels * 4 rows = 8 empty notes)

    // Packed pattern data (all empty notes with packing flag)
    data.extend(std::iter::repeat_n(0x80, 8));

    let result = XmValidator::validate(&data);
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(report.valid);
    assert!(!report.patterns[0].is_empty);
}

#[test]
fn test_xm_invalid_packing_type() {
    let mut data = create_minimal_xm("Test", 2, 1, 0, 6, 125);

    // Pattern header with invalid packing type
    data.extend_from_slice(&9u32.to_le_bytes());
    data.push(1); // Invalid packing type (should be 0)
    data.extend_from_slice(&4u16.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());

    let result = XmValidator::validate(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidPackingType { .. })
    ));
}

#[test]
fn test_xm_is_xm_quick_check() {
    let data = create_minimal_xm("Test", 4, 1, 0, 6, 125);
    assert!(XmValidator::is_xm(&data));

    let bad_data = vec![0u8; 100];
    assert!(!XmValidator::is_xm(&bad_data));

    let short_data = vec![0u8; 10];
    assert!(!XmValidator::is_xm(&short_data));
}

#[test]
fn test_xm_file_too_small() {
    let data = vec![0u8; 50];
    let result = XmValidator::validate(&data);
    assert!(matches!(result, Err(XmFormatError::FileTooSmall { .. })));
}

#[test]
fn test_xm_instrument_structure() {
    let mut data = create_minimal_xm("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);

    // Add minimal instrument with no samples
    data.extend_from_slice(&29u32.to_le_bytes()); // Instrument size (minimal)
    data.extend_from_slice(b"Test Instrument\0\0\0\0\0\0\0"); // Name (22 bytes)
    data.push(0); // Instrument type
    data.extend_from_slice(&0u16.to_le_bytes()); // Number of samples

    let result = XmValidator::validate(&data);
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(report.valid);
    assert_eq!(report.instruments.len(), 1);
    assert!(report
        .warnings
        .iter()
        .any(|w| matches!(w, XmWarning::InstrumentWithoutSamples { .. })));
}

#[test]
fn test_xm_sample_data() {
    let mut data = create_minimal_xm("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);

    // Add instrument header with 1 sample
    data.extend_from_slice(&263u32.to_le_bytes()); // Standard instrument size
    data.extend_from_slice(b"Test Instrument\0\0\0\0\0\0\0"); // Name (22 bytes)
    data.push(0); // Instrument type
    data.extend_from_slice(&1u16.to_le_bytes()); // Number of samples
    data.extend_from_slice(&40u32.to_le_bytes()); // Sample header size

    // Note-sample mapping (96 bytes)
    data.extend_from_slice(&[0u8; 96]);

    // Volume envelope points (48 bytes)
    data.extend_from_slice(&[0u8; 48]);

    // Panning envelope points (48 bytes)
    data.extend_from_slice(&[0u8; 48]);

    // Envelope parameters
    data.push(0); // num vol points
    data.push(0); // num pan points
    data.push(0); // vol sustain
    data.push(0); // vol loop start
    data.push(0); // vol loop end
    data.push(0); // pan sustain
    data.push(0); // pan loop start
    data.push(0); // pan loop end
    data.push(0); // vol flags
    data.push(0); // pan flags

    // Vibrato
    data.push(0); // type
    data.push(0); // sweep
    data.push(0); // depth
    data.push(0); // rate

    // Volume fadeout
    data.extend_from_slice(&0u16.to_le_bytes());

    // Reserved (22 bytes)
    data.extend_from_slice(&[0u8; 22]);

    // Sample header (40 bytes)
    data.extend_from_slice(&100u32.to_le_bytes()); // Length
    data.extend_from_slice(&0u32.to_le_bytes()); // Loop start
    data.extend_from_slice(&0u32.to_le_bytes()); // Loop length
    data.push(64); // Volume
    data.push(0); // Finetune
    data.push(0); // Flags (no loop, 8-bit)
    data.push(128); // Panning
    data.push(0); // Relative note
    data.push(0); // Reserved
    data.extend_from_slice(b"Test Sample\0\0\0\0\0\0\0\0\0\0\0"); // Name (22 bytes)

    // Sample data (100 bytes)
    data.extend_from_slice(&[128u8; 100]);

    let result = XmValidator::validate(&data);
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(report.valid);
}

#[test]
fn test_xm_invalid_sample_loop_bounds() {
    let mut data = create_minimal_xm("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);

    // Add full instrument header with 1 sample (263 bytes)
    data.extend_from_slice(&263u32.to_le_bytes()); // Instrument size
    data.extend_from_slice(b"Test Instrument\0\0\0\0\0\0\0"); // Name (22 bytes)
    data.push(0); // Instrument type
    data.extend_from_slice(&1u16.to_le_bytes()); // Number of samples
    data.extend_from_slice(&40u32.to_le_bytes()); // Sample header size
    data.extend_from_slice(&[0u8; 96]); // Note-sample mapping (96 bytes)
    data.extend_from_slice(&[0u8; 48]); // Volume envelope points (48 bytes)
    data.extend_from_slice(&[0u8; 48]); // Panning envelope points (48 bytes)
    data.push(0); // Num vol points
    data.push(0); // Num pan points
    data.push(0); // Vol sustain
    data.push(0); // Vol loop start
    data.push(0); // Vol loop end
    data.push(0); // Pan sustain
    data.push(0); // Pan loop start
    data.push(0); // Pan loop end
    data.push(0); // Vol flags
    data.push(0); // Pan flags
    data.push(0); // Vibrato type
    data.push(0); // Vibrato sweep
    data.push(0); // Vibrato depth
    data.push(0); // Vibrato rate
    data.extend_from_slice(&0u16.to_le_bytes()); // Volume fadeout
    data.extend_from_slice(&[0u8; 22]); // Reserved (22 bytes)

    // Sample header with invalid loop (40 bytes)
    data.extend_from_slice(&100u32.to_le_bytes()); // Length: 100 bytes
    data.extend_from_slice(&50u32.to_le_bytes()); // Loop start: 50
    data.extend_from_slice(&100u32.to_le_bytes()); // Loop length: 100 (50 + 100 > 100, invalid!)
    data.push(64); // Volume
    data.push(0); // Finetune
    data.push(1); // Flags (forward loop enabled)
    data.push(128); // Panning
    data.push(0); // Relative note
    data.push(0); // Reserved
    data.extend_from_slice(&[0u8; 22]); // Sample name (22 bytes)

    // Sample data (100 bytes)
    data.extend_from_slice(&[128u8; 100]);

    let result = XmValidator::validate(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidLoopBounds { .. })
    ));
}

#[test]
fn test_xm_full_validation() {
    // Create a complete valid XM file
    let mut data = create_minimal_xm("Complete Test", 4, 2, 1, 6, 125);

    // Update song length and pattern order
    data[64..66].copy_from_slice(&2u16.to_le_bytes()); // 2 orders
    data[80] = 0; // Play pattern 0 first
    data[81] = 1; // Then pattern 1

    // Add two patterns
    add_empty_pattern(&mut data, 64);
    add_empty_pattern(&mut data, 32);

    // Add minimal instrument
    data.extend_from_slice(&29u32.to_le_bytes());
    data.extend_from_slice(b"Minimal Instrument\0\0\0\0");
    data.push(0);
    data.extend_from_slice(&0u16.to_le_bytes());

    let result = XmValidator::validate(&data);
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(report.valid);
    assert_eq!(report.patterns.len(), 2);
    assert_eq!(report.instruments.len(), 1);
    assert!(report.header.is_some());

    let header = report.header.unwrap();
    assert_eq!(header.song_length, 2);
    assert_eq!(header.num_patterns, 2);
    assert_eq!(header.pattern_order[0], 0);
    assert_eq!(header.pattern_order[1], 1);
}

#[test]
fn test_xm_warnings_generated() {
    // Use unusual but valid tempo (25 is > 20 which triggers warning) and BPM (45 is < 50 which triggers warning)
    // Both are within valid ranges (tempo 1-31, BPM 32-255)
    let mut data = create_minimal_xm("Test", 4, 1, 0, 25, 45);

    // Non-standard header size (280 instead of 276)
    data[60..64].copy_from_slice(&280u32.to_le_bytes());

    // Extend data to account for larger header size
    // Header now claims to be 280 bytes, so patterns start at offset 60 + 280 = 340 instead of 336
    let additional_bytes = 4; // 280 - 276 = 4 extra bytes
    data.resize(336 + additional_bytes, 0);

    add_empty_pattern(&mut data, 64);

    let result = XmValidator::validate(&data);
    assert!(
        result.is_ok(),
        "Validation should succeed with warnings: {:?}",
        result
    );

    let report = result.unwrap();
    // Should have warnings but still be valid
    assert!(report.has_warnings(), "Should have warnings");
    assert!(
        report
            .warnings
            .iter()
            .any(|w| matches!(w, XmWarning::NonStandardHeaderSize { .. })),
        "Should have non-standard header size warning"
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|w| matches!(w, XmWarning::UnusualTempoBpm { .. })),
        "Should have unusual tempo/BPM warning"
    );
}
