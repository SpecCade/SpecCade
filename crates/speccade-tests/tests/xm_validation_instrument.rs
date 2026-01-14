//! XM format instrument and sample validation tests.
//!
//! Tests covering:
//! - Instrument structure validation
//! - Volume and panning envelopes
//! - Vibrato settings
//! - Sample data validation
//! - Sample loop types and bounds

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

/// Add a full instrument with samples to XM data.
fn add_instrument_with_sample(data: &mut Vec<u8>, name: &str, sample_data: &[u8]) {
    // Instrument size (4 bytes) - standard size with samples
    data.extend_from_slice(&263u32.to_le_bytes());
    // Instrument name (22 bytes)
    let mut name_buf = [0u8; 22];
    let name_bytes = name.as_bytes();
    let copy_len = name_bytes.len().min(22);
    name_buf[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
    data.extend_from_slice(&name_buf);
    // Instrument type (1 byte, always 0)
    data.push(0);
    // Number of samples (2 bytes)
    data.extend_from_slice(&1u16.to_le_bytes());
    // Sample header size (4 bytes)
    data.extend_from_slice(&40u32.to_le_bytes());
    // Note-sample mapping table (96 bytes)
    data.extend_from_slice(&[0u8; 96]);
    // Volume envelope points (48 bytes = 12 points * 4 bytes)
    data.extend_from_slice(&[0u8; 48]);
    // Panning envelope points (48 bytes)
    data.extend_from_slice(&[0u8; 48]);
    // Number of volume envelope points (1 byte)
    data.push(0);
    // Number of panning envelope points (1 byte)
    data.push(0);
    // Volume sustain point (1 byte)
    data.push(0);
    // Volume loop start (1 byte)
    data.push(0);
    // Volume loop end (1 byte)
    data.push(0);
    // Panning sustain point (1 byte)
    data.push(0);
    // Panning loop start (1 byte)
    data.push(0);
    // Panning loop end (1 byte)
    data.push(0);
    // Volume envelope flags (1 byte)
    data.push(0);
    // Panning envelope flags (1 byte)
    data.push(0);
    // Vibrato type (1 byte)
    data.push(0);
    // Vibrato sweep (1 byte)
    data.push(0);
    // Vibrato depth (1 byte)
    data.push(0);
    // Vibrato rate (1 byte)
    data.push(0);
    // Volume fadeout (2 bytes)
    data.extend_from_slice(&0u16.to_le_bytes());
    // Reserved (22 bytes)
    data.extend_from_slice(&[0u8; 22]);

    // Sample header (40 bytes)
    data.extend_from_slice(&(sample_data.len() as u32).to_le_bytes()); // Length
    data.extend_from_slice(&0u32.to_le_bytes()); // Loop start
    data.extend_from_slice(&0u32.to_le_bytes()); // Loop length
    data.push(64); // Volume
    data.push(0); // Finetune
    data.push(0); // Flags (no loop, 8-bit)
    data.push(128); // Panning
    data.push(0); // Relative note
    data.push(0); // Reserved
                  // Sample name (22 bytes)
    data.extend_from_slice(b"Sample\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");

    // Sample data
    data.extend_from_slice(sample_data);
}

// ============================================================================
// Instrument Structure Validation Tests
// ============================================================================

#[test]
fn test_xm_instrument_basic_structure() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);
    add_empty_instrument(&mut data, "Test Instrument");

    let report = XmValidator::validate(&data).unwrap();
    assert_eq!(report.instruments.len(), 1);
    assert_eq!(report.instruments[0].name, "Test Instrument");
    assert_eq!(report.instruments[0].num_samples, 0);
}

#[test]
fn test_xm_instrument_with_samples() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);
    add_instrument_with_sample(&mut data, "Lead Synth", &[128u8; 100]);

    let report = XmValidator::validate(&data).unwrap();
    assert!(report.valid);
    assert_eq!(report.instruments.len(), 1);
}

#[test]
fn test_xm_instrument_no_samples_warning() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);
    add_empty_instrument(&mut data, "Empty Instrument");

    let report = XmValidator::validate(&data).unwrap();
    assert!(report
        .warnings
        .iter()
        .any(|w| matches!(w, XmWarning::InstrumentWithoutSamples { .. })));
}

#[test]
fn test_xm_volume_envelope_structure() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);

    // Add instrument with volume envelope
    data.extend_from_slice(&263u32.to_le_bytes()); // Instrument size
    data.extend_from_slice(b"Envelope Test\0\0\0\0\0\0\0\0\0"); // Name
    data.push(0); // Instrument type
    data.extend_from_slice(&1u16.to_le_bytes()); // 1 sample
    data.extend_from_slice(&40u32.to_le_bytes()); // Sample header size
    data.extend_from_slice(&[0u8; 96]); // Note-sample mapping

    // Volume envelope: 4 points for ADSR
    let mut vol_env = [0u8; 48];
    // Point 0: (0, 0) - start
    vol_env[0..4].copy_from_slice(&[0, 0, 0, 0]);
    // Point 1: (10, 64) - attack peak
    vol_env[4..8].copy_from_slice(&[10, 0, 64, 0]);
    // Point 2: (30, 48) - decay to sustain
    vol_env[8..12].copy_from_slice(&[30, 0, 48, 0]);
    // Point 3: (100, 0) - release to 0
    vol_env[12..16].copy_from_slice(&[100, 0, 0, 0]);
    data.extend_from_slice(&vol_env);

    // Panning envelope (empty)
    data.extend_from_slice(&[0u8; 48]);

    // Envelope parameters
    data.push(4); // 4 volume points
    data.push(0); // 0 panning points
    data.push(2); // Volume sustain at point 2
    data.push(0); // Vol loop start
    data.push(0); // Vol loop end
    data.push(0); // Pan sustain
    data.push(0); // Pan loop start
    data.push(0); // Pan loop end
    data.push(0b011); // Volume envelope: enabled + sustain
    data.push(0); // Panning envelope disabled
    data.extend_from_slice(&[0u8; 4]); // Vibrato
    data.extend_from_slice(&0u16.to_le_bytes()); // Fadeout
    data.extend_from_slice(&[0u8; 22]); // Reserved

    // Sample header
    data.extend_from_slice(&100u32.to_le_bytes()); // Length
    data.extend_from_slice(&[0u8; 8]); // Loop start, length
    data.push(64); // Volume
    data.push(0); // Finetune
    data.push(0); // Flags
    data.push(128); // Panning
    data.push(0); // Relative note
    data.push(0); // Reserved
    data.extend_from_slice(&[0u8; 22]); // Name

    // Sample data
    data.extend_from_slice(&[128u8; 100]);

    let report = XmValidator::validate(&data).unwrap();
    assert!(report.valid);
    let inst = &report.instruments[0];
    assert!(inst.volume_envelope.enabled);
    assert!(inst.volume_envelope.sustain_enabled);
    assert!(!inst.volume_envelope.loop_enabled);
    assert_eq!(inst.volume_envelope.num_points, 4);
}

#[test]
fn test_xm_envelope_max_points() {
    // Envelope can have max 12 points
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);

    // Instrument with 12 envelope points (maximum valid)
    data.extend_from_slice(&263u32.to_le_bytes());
    data.extend_from_slice(&[0u8; 22]); // Name
    data.push(0);
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&40u32.to_le_bytes());
    data.extend_from_slice(&[0u8; 96]);
    data.extend_from_slice(&[0u8; 48]); // Vol envelope (12 points)
    data.extend_from_slice(&[0u8; 48]); // Pan envelope
    data.push(12); // 12 volume points (max)
    data.push(0);
    data.extend_from_slice(&[0u8; 8]); // Sustain/loop
    data.push(1); // Vol enabled
    data.push(0);
    data.extend_from_slice(&[0u8; 4]); // Vibrato
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&[0u8; 22]);
    data.extend_from_slice(&100u32.to_le_bytes());
    data.extend_from_slice(&[0u8; 8]);
    data.push(64);
    data.push(0);
    data.push(0);
    data.push(128);
    data.push(0);
    data.push(0);
    data.extend_from_slice(&[0u8; 22]);
    data.extend_from_slice(&[128u8; 100]);

    let report = XmValidator::validate(&data).unwrap();
    assert!(report.valid);
}

#[test]
fn test_xm_envelope_too_many_points() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);

    // Instrument with 15 envelope points (invalid, max is 12)
    data.extend_from_slice(&263u32.to_le_bytes());
    data.extend_from_slice(&[0u8; 22]);
    data.push(0);
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&40u32.to_le_bytes());
    data.extend_from_slice(&[0u8; 96]);
    data.extend_from_slice(&[0u8; 48]);
    data.extend_from_slice(&[0u8; 48]);
    data.push(15); // 15 volume points - invalid!
    data.push(0);
    data.extend_from_slice(&[0u8; 8]);
    data.push(1);
    data.push(0);
    data.extend_from_slice(&[0u8; 4]);
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&[0u8; 22]);
    data.extend_from_slice(&100u32.to_le_bytes());
    data.extend_from_slice(&[0u8; 8]);
    data.push(64);
    data.push(0);
    data.push(0);
    data.push(128);
    data.push(0);
    data.push(0);
    data.extend_from_slice(&[0u8; 22]);
    data.extend_from_slice(&[128u8; 100]);

    let result = XmValidator::validate(&data);
    assert!(matches!(result, Err(XmFormatError::EnvelopeError { .. })));
}

#[test]
fn test_xm_vibrato_settings() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);

    // Instrument with vibrato settings
    data.extend_from_slice(&263u32.to_le_bytes());
    data.extend_from_slice(&[0u8; 22]);
    data.push(0);
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&40u32.to_le_bytes());
    data.extend_from_slice(&[0u8; 96]);
    data.extend_from_slice(&[0u8; 96]); // Envelopes
    data.extend_from_slice(&[0u8; 10]); // Points + sustain/loop
    data.push(0); // Vibrato type: sine
    data.push(10); // Vibrato sweep
    data.push(8); // Vibrato depth
    data.push(4); // Vibrato rate
    data.extend_from_slice(&1000u16.to_le_bytes()); // Fadeout
    data.extend_from_slice(&[0u8; 22]);
    data.extend_from_slice(&100u32.to_le_bytes());
    data.extend_from_slice(&[0u8; 8]);
    data.push(64);
    data.push(0);
    data.push(0);
    data.push(128);
    data.push(0);
    data.push(0);
    data.extend_from_slice(&[0u8; 22]);
    data.extend_from_slice(&[128u8; 100]);

    let report = XmValidator::validate(&data).unwrap();
    assert!(report.valid);
    let inst = &report.instruments[0];
    assert_eq!(inst.vibrato_type, 0);
    assert_eq!(inst.vibrato_sweep, 10);
    assert_eq!(inst.vibrato_depth, 8);
    assert_eq!(inst.vibrato_rate, 4);
    assert_eq!(inst.volume_fadeout, 1000);
}

// ============================================================================
// Sample Data Validation Tests
// ============================================================================

#[test]
fn test_xm_sample_basic_structure() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);
    add_instrument_with_sample(&mut data, "Test", &[128u8; 200]);

    let report = XmValidator::validate(&data).unwrap();
    assert!(report.valid);
}

#[test]
fn test_xm_sample_volume_range() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);

    // Build complete instrument header (263 bytes) with invalid sample volume
    data.extend_from_slice(&263u32.to_le_bytes()); // Instrument size
    data.extend_from_slice(&[0u8; 22]); // Name
    data.push(0); // Type
    data.extend_from_slice(&1u16.to_le_bytes()); // 1 sample
    data.extend_from_slice(&40u32.to_le_bytes()); // Sample header size
    data.extend_from_slice(&[0u8; 96]); // Note-sample mapping
    data.extend_from_slice(&[0u8; 48]); // Volume envelope points
    data.extend_from_slice(&[0u8; 48]); // Panning envelope points
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
    data.extend_from_slice(&0u16.to_le_bytes()); // Fadeout
    data.extend_from_slice(&[0u8; 22]); // Reserved

    // Sample header (40 bytes)
    data.extend_from_slice(&100u32.to_le_bytes()); // Length
    data.extend_from_slice(&0u32.to_le_bytes()); // Loop start
    data.extend_from_slice(&0u32.to_le_bytes()); // Loop length
    data.push(100); // Invalid volume (max is 64)
    data.push(0); // Finetune
    data.push(0); // Flags
    data.push(128); // Panning
    data.push(0); // Relative note
    data.push(0); // Reserved
    data.extend_from_slice(&[0u8; 22]); // Sample name

    // Sample data
    data.extend_from_slice(&[128u8; 100]);

    let report = XmValidator::validate(&data).unwrap();
    assert!(!report.valid);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, XmFormatError::InvalidVolume { value: 100, .. })));
}

#[test]
fn test_xm_sample_loop_types() {
    // Valid loop types: 0 (none), 1 (forward), 2 (ping-pong)
    for loop_type in [0u8, 1, 2] {
        let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
        add_empty_pattern(&mut data, 64);
        add_instrument_with_sample(&mut data, "Test", &[128u8; 100]);

        // Replace loop type in sample header
        // Sample header starts at: header (336) + pattern (9) + instrument header (263)
        // Loop type is in flags byte, which is at sample header offset + 14
        let sample_header_start = 336 + 9 + 263;
        data[sample_header_start + 14] = loop_type;

        let report = XmValidator::validate(&data).unwrap();
        assert!(report.valid, "Loop type {} should be valid", loop_type);
    }

    // Invalid loop type (3)
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);
    add_instrument_with_sample(&mut data, "Test", &[128u8; 100]);

    // Replace loop type in sample header with invalid value 3
    let sample_header_start = 336 + 9 + 263;
    data[sample_header_start + 14] = 3; // Invalid loop type

    let result = XmValidator::validate(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidLoopType { loop_type: 3, .. })
    ));
}

#[test]
fn test_xm_sample_loop_bounds() {
    // Valid loop bounds
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);
    add_instrument_with_sample(&mut data, "Test", &[128u8; 100]);

    // Modify sample header for valid loop bounds
    // Sample header starts at: header (336) + pattern (9) + instrument header (263)
    let sample_header_start = 336 + 9 + 263;
    // Length = 100 (already set)
    // Loop start = 10
    data[sample_header_start + 4..sample_header_start + 8].copy_from_slice(&10u32.to_le_bytes());
    // Loop length = 50 (10 + 50 = 60 <= 100, valid)
    data[sample_header_start + 8..sample_header_start + 12].copy_from_slice(&50u32.to_le_bytes());
    // Enable forward loop
    data[sample_header_start + 14] = 1;

    let report = XmValidator::validate(&data).unwrap();
    assert!(report.valid);

    // Invalid loop bounds (loop extends past end)
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);
    add_instrument_with_sample(&mut data, "Test", &[128u8; 100]);

    // Modify sample header for invalid loop bounds
    let sample_header_start = 336 + 9 + 263;
    // Length = 100 (already set)
    // Loop start = 50
    data[sample_header_start + 4..sample_header_start + 8].copy_from_slice(&50u32.to_le_bytes());
    // Loop length = 100 (50 + 100 = 150 > 100, INVALID!)
    data[sample_header_start + 8..sample_header_start + 12].copy_from_slice(&100u32.to_le_bytes());
    // Enable forward loop
    data[sample_header_start + 14] = 1;

    let result = XmValidator::validate(&data);
    assert!(matches!(
        result,
        Err(XmFormatError::InvalidLoopBounds { .. })
    ));
}

#[test]
fn test_xm_sample_16bit_flag() {
    let mut data = create_test_xm_bytes("Test", 4, 1, 1, 6, 125);
    add_empty_pattern(&mut data, 64);

    // 16-bit sample (flag bit 4 = 0x10)
    data.extend_from_slice(&263u32.to_le_bytes());
    data.extend_from_slice(&[0u8; 22]);
    data.push(0);
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&40u32.to_le_bytes());
    data.extend_from_slice(&[0u8; 96 + 96 + 28]);
    data.extend_from_slice(&200u32.to_le_bytes()); // Length (bytes)
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.push(64);
    data.push(0);
    data.push(0x10); // 16-bit flag
    data.push(128);
    data.push(0);
    data.push(0);
    data.extend_from_slice(&[0u8; 22]);
    data.extend_from_slice(&[0u8; 200]); // 16-bit sample data

    let report = XmValidator::validate(&data).unwrap();
    assert!(report.valid);
}
