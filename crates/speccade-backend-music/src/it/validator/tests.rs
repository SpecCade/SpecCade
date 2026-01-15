//! Tests for IT validator.

use super::error::{ItErrorCategory, ItFormatError};
use super::validator::ItValidator;

/// Create a minimal valid IT file for testing.
fn create_minimal_it() -> Vec<u8> {
    let mut it = vec![0u8; 192];

    // Magic
    it[0..4].copy_from_slice(b"IMPM");

    // Name
    it[4..14].copy_from_slice(b"Test Song ");

    // OrdNum = 1
    it[0x20..0x22].copy_from_slice(&1u16.to_le_bytes());

    // InsNum = 0
    it[0x22..0x24].copy_from_slice(&0u16.to_le_bytes());

    // SmpNum = 0
    it[0x24..0x26].copy_from_slice(&0u16.to_le_bytes());

    // PatNum = 0
    it[0x26..0x28].copy_from_slice(&0u16.to_le_bytes());

    // Cwt/v = 0x0214
    it[0x28..0x2A].copy_from_slice(&0x0214u16.to_le_bytes());

    // Cmwt = 0x0200
    it[0x2A..0x2C].copy_from_slice(&0x0200u16.to_le_bytes());

    // Flags
    it[0x2C..0x2E].copy_from_slice(&0x0009u16.to_le_bytes());

    // Global volume = 128
    it[0x30] = 128;

    // Mix volume = 48
    it[0x31] = 48;

    // Initial speed = 6
    it[0x32] = 6;

    // Initial tempo = 125
    it[0x33] = 125;

    // Pan separation = 128
    it[0x34] = 128;

    // Channel pan (all center)
    it[0x40..0x80].fill(32);

    // Channel volume
    it[0x80..0xC0].fill(64);

    // Order: 255 (end)
    it.push(255);

    it
}

/// Create an IT file with an instrument.
fn create_it_with_instrument() -> Vec<u8> {
    let mut it = create_minimal_it();

    // Update header for 1 instrument
    it[0x22..0x24].copy_from_slice(&1u16.to_le_bytes());

    // Add instrument offset (right after order + offset table)
    let inst_offset = (it.len() + 4) as u32; // 4 bytes for offset entry
    it.extend_from_slice(&inst_offset.to_le_bytes());

    // Create instrument header (554 bytes)
    let mut inst = vec![0u8; 554];
    inst[0..4].copy_from_slice(b"IMPI");
    // Name
    inst[0x20..0x30].copy_from_slice(b"Test Inst       ");
    // Global volume
    inst[0x18] = 128;

    it.extend(inst);

    it
}

/// Create an IT file with a sample.
fn create_it_with_sample() -> Vec<u8> {
    let mut it = create_minimal_it();

    // Update header for 1 sample
    it[0x24..0x26].copy_from_slice(&1u16.to_le_bytes());

    // Add sample offset
    let sample_offset = (it.len() + 4) as u32;
    it.extend_from_slice(&sample_offset.to_le_bytes());

    // Create sample header (80 bytes)
    let mut smp = vec![0u8; 80];
    smp[0..4].copy_from_slice(b"IMPS");
    // Global volume
    smp[0x11] = 64;
    // Flags (has data, 16-bit)
    smp[0x12] = 0x03;
    // Default volume
    smp[0x13] = 64;
    // Name
    smp[0x14..0x24].copy_from_slice(b"Test Sample     ");
    // C5 speed
    smp[0x3C..0x40].copy_from_slice(&22050u32.to_le_bytes());

    it.extend(smp);

    it
}

/// Create an IT file with a pattern.
fn create_it_with_pattern() -> Vec<u8> {
    let mut it = create_minimal_it();

    // Update header for 1 pattern
    it[0x26..0x28].copy_from_slice(&1u16.to_le_bytes());

    // Add pattern offset
    let pattern_offset = (it.len() + 4) as u32;
    it.extend_from_slice(&pattern_offset.to_le_bytes());

    // Create minimal pattern
    // Header: 8 bytes
    let mut pattern = Vec::new();
    // Packed length (just end-of-row markers for 64 rows)
    pattern.extend_from_slice(&64u16.to_le_bytes());
    // Rows = 64
    pattern.extend_from_slice(&64u16.to_le_bytes());
    // Reserved
    pattern.extend_from_slice(&[0u8; 4]);
    // Packed data: 64 end-of-row markers
    pattern.extend(vec![0u8; 64]);

    it.extend(pattern);

    it
}

#[test]
fn test_it_header_magic() {
    let it = create_minimal_it();
    let report = ItValidator::validate(&it).unwrap();

    assert!(report.is_valid);
    assert!(report.header.is_some());

    let header = report.header.unwrap();
    assert_eq!(header.name, "Test Song");
}

#[test]
fn test_it_header_magic_invalid() {
    let mut it = create_minimal_it();
    it[0..4].copy_from_slice(b"XXXX");

    let result = ItValidator::validate(&it);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.category, ItErrorCategory::Header);
    assert!(err.message.contains("magic"));
}

#[test]
fn test_it_header_fields() {
    let it = create_minimal_it();
    let report = ItValidator::validate(&it).unwrap();

    let header = report.header.unwrap();
    assert_eq!(header.initial_speed, 6);
    assert_eq!(header.initial_tempo, 125);
    assert_eq!(header.global_volume, 128);
    assert_eq!(header.mix_volume, 48);
    assert_eq!(header.panning_separation, 128);
    assert!(header.flags.stereo);
    assert!(header.flags.linear_slides);
}

#[test]
fn test_it_header_global_volume_invalid() {
    let mut it = create_minimal_it();
    it[0x30] = 200; // Invalid: > 128

    let report = ItValidator::validate(&it).unwrap();
    assert!(!report.is_valid);
    assert!(!report.errors.is_empty());
    assert!(report.errors[0].message.contains("Global volume"));
}

#[test]
fn test_it_header_speed_zero() {
    let mut it = create_minimal_it();
    it[0x32] = 0; // Invalid: speed cannot be 0

    let report = ItValidator::validate(&it).unwrap();
    assert!(!report.is_valid);
    assert!(report.errors.iter().any(|e| e.message.contains("speed")));
}

#[test]
fn test_it_instrument_nna() {
    let it = create_it_with_instrument();
    let report = ItValidator::validate(&it).unwrap();

    assert!(report.is_valid);
    assert_eq!(report.instruments.len(), 1);
    assert_eq!(report.instruments[0].nna, 0); // Default: cut
}

#[test]
fn test_it_instrument_invalid_magic() {
    let mut it = create_it_with_instrument();
    // Find instrument data and corrupt magic
    let inst_offset = u32::from_le_bytes([it[193], it[194], it[195], it[196]]) as usize;
    it[inst_offset..inst_offset + 4].copy_from_slice(b"XXXX");

    let report = ItValidator::validate(&it).unwrap();
    assert!(!report.is_valid);
    assert!(report.errors.iter().any(|e| e.message.contains("magic")));
}

#[test]
fn test_it_sample_validation() {
    let it = create_it_with_sample();
    let report = ItValidator::validate(&it).unwrap();

    assert!(report.is_valid);
    assert_eq!(report.samples.len(), 1);

    let sample = &report.samples[0];
    assert!(sample.flags.has_data);
    assert!(sample.flags.is_16bit);
    assert_eq!(sample.c5_speed, 22050);
    assert_eq!(sample.global_volume, 64);
}

#[test]
fn test_it_sample_invalid_magic() {
    let mut it = create_it_with_sample();
    let sample_offset = u32::from_le_bytes([it[193], it[194], it[195], it[196]]) as usize;
    it[sample_offset..sample_offset + 4].copy_from_slice(b"XXXX");

    let report = ItValidator::validate(&it).unwrap();
    assert!(!report.is_valid);
}

#[test]
fn test_it_sample_compression() {
    let mut it = create_it_with_sample();
    let sample_offset = u32::from_le_bytes([it[193], it[194], it[195], it[196]]) as usize;
    // Set compressed flag
    it[sample_offset + 0x12] |= 0x08;

    let report = ItValidator::validate(&it).unwrap();
    assert!(report.samples[0].flags.is_compressed);
}

#[test]
fn test_it_pattern_packing() {
    let it = create_it_with_pattern();
    let report = ItValidator::validate(&it).unwrap();

    assert!(report.is_valid);
    assert_eq!(report.patterns.len(), 1);
    assert_eq!(report.patterns[0].num_rows, 64);
}

#[test]
fn test_it_pattern_zero_rows() {
    let mut it = create_it_with_pattern();
    let pattern_offset = u32::from_le_bytes([it[193], it[194], it[195], it[196]]) as usize;
    // Set rows to 0
    it[pattern_offset + 2..pattern_offset + 4].copy_from_slice(&0u16.to_le_bytes());

    let report = ItValidator::validate(&it).unwrap();
    assert!(!report.is_valid);
    assert!(report.errors.iter().any(|e| e.message.contains("0 rows")));
}

#[test]
fn test_it_full_validation() {
    // Create a more complete IT file
    let mut it = create_minimal_it();

    // Add 1 instrument
    it[0x22..0x24].copy_from_slice(&1u16.to_le_bytes());
    // Add 1 sample
    it[0x24..0x26].copy_from_slice(&1u16.to_le_bytes());
    // Add 1 pattern
    it[0x26..0x28].copy_from_slice(&1u16.to_le_bytes());

    // Calculate offsets
    let base = it.len();
    let inst_offset = (base + 12) as u32; // After 3 offset entries
    let sample_offset = (inst_offset as usize + 554) as u32;
    let pattern_offset = (sample_offset as usize + 80) as u32;

    // Add offset table
    it.extend_from_slice(&inst_offset.to_le_bytes());
    it.extend_from_slice(&sample_offset.to_le_bytes());
    it.extend_from_slice(&pattern_offset.to_le_bytes());

    // Add instrument
    let mut inst = vec![0u8; 554];
    inst[0..4].copy_from_slice(b"IMPI");
    inst[0x18] = 128;
    it.extend(inst);

    // Add sample
    let mut smp = vec![0u8; 80];
    smp[0..4].copy_from_slice(b"IMPS");
    smp[0x11] = 64;
    smp[0x12] = 0x03;
    smp[0x13] = 64;
    smp[0x3C..0x40].copy_from_slice(&22050u32.to_le_bytes());
    it.extend(smp);

    // Add pattern
    let mut pattern = Vec::new();
    pattern.extend_from_slice(&64u16.to_le_bytes()); // packed length
    pattern.extend_from_slice(&64u16.to_le_bytes()); // rows
    pattern.extend_from_slice(&[0u8; 4]); // reserved
    pattern.extend(vec![0u8; 64]); // end-of-row markers
    it.extend(pattern);

    let report = ItValidator::validate(&it).unwrap();

    assert!(report.is_valid, "Errors: {:?}", report.errors);
    assert!(report.header.is_some());
    assert_eq!(report.instruments.len(), 1);
    assert_eq!(report.samples.len(), 1);
    assert_eq!(report.patterns.len(), 1);
}

#[test]
fn test_it_file_too_small() {
    let data = vec![0u8; 100];
    let result = ItValidator::validate(&data);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.category, ItErrorCategory::Structure);
    assert!(err.message.contains("too small"));
}

#[test]
fn test_it_order_list_validation() {
    let mut it = create_minimal_it();

    // Add more orders
    it[0x20..0x22].copy_from_slice(&5u16.to_le_bytes());
    // Remove the existing order (255) and add new ones
    it.pop();
    it.push(0); // Pattern 0
    it.push(254); // Skip marker
    it.push(0); // Pattern 0
    it.push(100); // Invalid: no pattern 100
    it.push(255); // End marker

    let report = ItValidator::validate(&it).unwrap();

    assert_eq!(report.orders.len(), 5);
    assert_eq!(report.orders[0], 0);
    assert_eq!(report.orders[1], 254);
    assert_eq!(report.orders[4], 255);

    // Should have a warning about referencing non-existent pattern
    assert!(!report.warnings.is_empty());
    assert!(report
        .warnings
        .iter()
        .any(|w| w.message.contains("pattern 100")));
}

#[test]
fn test_it_channel_pan_validation() {
    let mut it = create_minimal_it();

    // Set invalid panning value
    it[0x45] = 80; // Invalid: between 65-99

    let report = ItValidator::validate(&it).unwrap();

    assert!(report
        .warnings
        .iter()
        .any(|w| w.message.contains("panning")));
}

#[test]
fn test_it_channel_volume_validation() {
    let mut it = create_minimal_it();

    // Set invalid volume
    it[0x85] = 100; // Invalid: > 64

    let report = ItValidator::validate(&it).unwrap();

    assert!(report.warnings.iter().any(|w| w.message.contains("volume")));
}

#[test]
fn test_format_error_display() {
    let err = ItFormatError::new(ItErrorCategory::Header, "test error");
    assert_eq!(format!("{}", err), "IT header error: test error");

    let err = ItFormatError::at_offset(ItErrorCategory::Sample, "at offset", 0x100);
    assert_eq!(
        format!("{}", err),
        "IT sample error at offset 0x0100: at offset"
    );

    let err = ItFormatError::field_at_offset(ItErrorCategory::Pattern, "num_rows", "invalid", 0x50);
    assert!(format!("{}", err).contains("field 'num_rows'"));
    assert!(format!("{}", err).contains("0x0050"));
}

#[test]
fn test_it_header_only_validation() {
    let it = create_minimal_it();
    let header = ItValidator::validate_header_only(&it).unwrap();

    assert_eq!(header.name, "Test Song");
    assert_eq!(header.initial_speed, 6);
    assert_eq!(header.initial_tempo, 125);
}

#[test]
fn test_sample_loop_validation() {
    let mut it = create_it_with_sample();
    let sample_offset = u32::from_le_bytes([it[193], it[194], it[195], it[196]]) as usize;

    // Enable loop
    it[sample_offset + 0x12] |= 0x10;
    // Set loop begin > end
    it[sample_offset + 0x34..sample_offset + 0x38].copy_from_slice(&100u32.to_le_bytes());
    it[sample_offset + 0x38..sample_offset + 0x3C].copy_from_slice(&50u32.to_le_bytes());

    let report = ItValidator::validate(&it).unwrap();
    assert!(!report.is_valid);
    assert!(report
        .errors
        .iter()
        .any(|e| e.message.contains("loop begin")));
}

#[test]
fn test_envelope_validation() {
    let it = create_it_with_instrument();
    let report = ItValidator::validate(&it).unwrap();

    let inst = &report.instruments[0];
    // Default envelope should not be enabled
    assert!(!inst.volume_envelope.enabled);
    assert!(!inst.panning_envelope.enabled);
    assert!(!inst.pitch_envelope.enabled);
}

#[test]
fn test_instrument_fadeout_warning() {
    let mut it = create_it_with_instrument();
    let inst_offset = u32::from_le_bytes([it[193], it[194], it[195], it[196]]) as usize;

    // Set fadeout > 1024
    it[inst_offset + 0x14..inst_offset + 0x16].copy_from_slice(&2000u16.to_le_bytes());

    let report = ItValidator::validate(&it).unwrap();
    assert!(report
        .warnings
        .iter()
        .any(|w| w.message.contains("fadeout")));
}
