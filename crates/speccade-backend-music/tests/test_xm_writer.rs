//! Comprehensive tests for the XM (FastTracker II) writer.
//!
//! These tests validate the binary format, headers, patterns, instruments,
//! and sample data of generated XM files.

use speccade_backend_music::xm::effects;
use speccade_backend_music::xm::{
    validate_xm_bytes, XmEnvelope, XmEnvelopePoint, XmInstrument, XmModule, XmNote, XmPattern,
    XmSample, XmValidationError, XM_HEADER_SIZE, XM_INSTRUMENT_HEADER_SIZE, XM_MAGIC,
    XM_MAX_CHANNELS, XM_MAX_INSTRUMENTS, XM_MAX_PATTERNS, XM_MAX_PATTERN_ROWS,
    XM_SAMPLE_HEADER_SIZE, XM_VERSION,
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
    let xm = generate_xm_with_instrument();

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

// =============================================================================
// Instrument Data Tests
// =============================================================================

#[test]
fn test_instrument_header_size() {
    assert_eq!(XM_INSTRUMENT_HEADER_SIZE, 263);
}

#[test]
fn test_sample_header_size() {
    assert_eq!(XM_SAMPLE_HEADER_SIZE, 40);
}

#[test]
fn test_instrument_write() {
    let sample_data = vec![0u8; 100];
    let sample = XmSample::new("TestSample", sample_data, true);
    let instrument = XmInstrument::new("TestInstrument", sample);

    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    // Check header size field
    let header_size = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    assert_eq!(header_size, XM_INSTRUMENT_HEADER_SIZE);

    // Check instrument name (22 bytes at offset 4)
    let name = std::str::from_utf8(&buf[4..26])
        .unwrap()
        .trim_end_matches('\0');
    assert_eq!(name, "TestInstrument");

    // Check instrument type (should be 0)
    assert_eq!(buf[26], 0);

    // Check number of samples (should be 1)
    let num_samples = u16::from_le_bytes([buf[27], buf[28]]);
    assert_eq!(num_samples, 1);
}

#[test]
fn test_instrument_envelope_points() {
    let env = XmEnvelope {
        enabled: true,
        points: vec![
            XmEnvelopePoint { frame: 0, value: 0 },
            XmEnvelopePoint {
                frame: 10,
                value: 64,
            },
            XmEnvelopePoint {
                frame: 50,
                value: 32,
            },
        ],
        sustain_point: 2,
        ..Default::default()
    };

    let sample = XmSample::new("Test", vec![0; 100], true);
    let instrument = XmInstrument::new("EnvTest", sample).with_volume_envelope(env);

    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    // Volume envelope starts at offset 33 (after 4+22+1+2+4 = 33) + 96 (note mapping) = 129
    let env_offset = 129;

    // First point: frame 0, value 0
    let frame0 = u16::from_le_bytes([buf[env_offset], buf[env_offset + 1]]);
    let value0 = u16::from_le_bytes([buf[env_offset + 2], buf[env_offset + 3]]);
    assert_eq!(frame0, 0);
    assert_eq!(value0, 0);

    // Second point: frame 10, value 64
    let frame1 = u16::from_le_bytes([buf[env_offset + 4], buf[env_offset + 5]]);
    let value1 = u16::from_le_bytes([buf[env_offset + 6], buf[env_offset + 7]]);
    assert_eq!(frame1, 10);
    assert_eq!(value1, 64);
}

#[test]
fn test_instrument_envelope_flags() {
    let mut env = XmEnvelope::default();
    assert_eq!(env.flags(), 0);

    env.enabled = true;
    assert_eq!(env.flags(), 1);

    env.sustain_enabled = true;
    assert_eq!(env.flags(), 3);

    env.loop_enabled = true;
    assert_eq!(env.flags(), 7);
}

#[test]
fn test_instrument_adsr_envelope() {
    let env = XmEnvelope::adsr(10, 20, 32, 30);

    assert!(env.enabled);
    assert!(env.sustain_enabled);
    assert!(!env.loop_enabled);
    assert!(env.points.len() >= 4);

    // Check attack starts at 0
    assert_eq!(env.points[0].frame, 0);
    assert_eq!(env.points[0].value, 0);

    // Check attack reaches max
    assert_eq!(env.points[1].frame, 10);
    assert_eq!(env.points[1].value, 64);
}

#[test]
fn test_instrument_vibrato_settings() {
    let sample = XmSample::new("Vib", vec![0; 100], true);
    let mut instrument = XmInstrument::new("VibTest", sample);
    instrument.vibrato_type = 1;
    instrument.vibrato_sweep = 10;
    instrument.vibrato_depth = 5;
    instrument.vibrato_rate = 20;

    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    // Vibrato settings are after the envelope data
    // Need to find the exact offset (after envelope flags)
    // envelope ends at offset 129 + 48 + 48 = 225
    // then: vol_num(1) + pan_num(1) + vol_sustain(1) + vol_loop_start(1) + vol_loop_end(1)
    //       + pan_sustain(1) + pan_loop_start(1) + pan_loop_end(1) + vol_flags(1) + pan_flags(1)
    //       = 10 bytes, so vibrato starts at 235
    let vib_offset = 235;
    assert_eq!(buf[vib_offset], 1); // type
    assert_eq!(buf[vib_offset + 1], 10); // sweep
    assert_eq!(buf[vib_offset + 2], 5); // depth
    assert_eq!(buf[vib_offset + 3], 20); // rate
}

#[test]
fn test_instrument_sample_data_follows() {
    let sample_data: Vec<u8> = (0..200).map(|i| (i % 256) as u8).collect();
    let sample = XmSample::new("DataTest", sample_data.clone(), true);
    let instrument = XmInstrument::new("DataTest", sample);

    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    // Sample header is 40 bytes, sample data follows
    let sample_header_offset = XM_INSTRUMENT_HEADER_SIZE as usize;
    let sample_data_offset = sample_header_offset + XM_SAMPLE_HEADER_SIZE as usize;

    // Verify sample data length field
    let sample_len = u32::from_le_bytes([
        buf[sample_header_offset],
        buf[sample_header_offset + 1],
        buf[sample_header_offset + 2],
        buf[sample_header_offset + 3],
    ]);
    assert_eq!(sample_len, 200);

    // Sample data should be delta-encoded for 16-bit
    assert!(buf.len() >= sample_data_offset + 200);
}

// =============================================================================
// Sample Data Tests
// =============================================================================

#[test]
fn test_sample_delta_encoding() {
    // Create a simple ascending sequence
    let mut sample_data = Vec::new();
    for i in 0..10i16 {
        sample_data.extend_from_slice(&(i * 100).to_le_bytes());
    }

    let sample = XmSample::new("DeltaTest", sample_data, true);
    let instrument = XmInstrument::new("DeltaTest", sample);

    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    let sample_data_offset = XM_INSTRUMENT_HEADER_SIZE as usize + XM_SAMPLE_HEADER_SIZE as usize;

    // First delta should be 0 (0 - 0 = 0)
    let d0 = i16::from_le_bytes([buf[sample_data_offset], buf[sample_data_offset + 1]]);
    assert_eq!(d0, 0);

    // Second delta should be 100 (100 - 0 = 100)
    let d1 = i16::from_le_bytes([buf[sample_data_offset + 2], buf[sample_data_offset + 3]]);
    assert_eq!(d1, 100);

    // Third delta should be 100 (200 - 100 = 100)
    let d2 = i16::from_le_bytes([buf[sample_data_offset + 4], buf[sample_data_offset + 5]]);
    assert_eq!(d2, 100);
}

#[test]
fn test_sample_delta_encoding_negative() {
    // Create a sequence that goes up then down
    let mut sample_data = Vec::new();
    let values: Vec<i16> = vec![0, 1000, 2000, 1000, 0, -1000];
    for v in &values {
        sample_data.extend_from_slice(&v.to_le_bytes());
    }

    let sample = XmSample::new("NegTest", sample_data, true);
    let instrument = XmInstrument::new("NegTest", sample);

    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    let offset = XM_INSTRUMENT_HEADER_SIZE as usize + XM_SAMPLE_HEADER_SIZE as usize;

    // Verify deltas: 0, 1000, 1000, -1000, -1000, -1000
    let d0 = i16::from_le_bytes([buf[offset], buf[offset + 1]]);
    assert_eq!(d0, 0);

    let d1 = i16::from_le_bytes([buf[offset + 2], buf[offset + 3]]);
    assert_eq!(d1, 1000);

    let d2 = i16::from_le_bytes([buf[offset + 4], buf[offset + 5]]);
    assert_eq!(d2, 1000);

    let d3 = i16::from_le_bytes([buf[offset + 6], buf[offset + 7]]);
    assert_eq!(d3, -1000);
}

#[test]
fn test_sample_loop_points() {
    let sample_data = vec![0u8; 2000]; // 1000 samples
    let sample = XmSample::new("LoopTest", sample_data, true).with_loop(100, 500, 1); // forward loop

    let mut buf = Vec::new();
    sample.write_header(&mut buf).unwrap();

    // Loop start is at offset 4-7
    let loop_start = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
    assert_eq!(loop_start, 100);

    // Loop length is at offset 8-11
    let loop_length = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
    assert_eq!(loop_length, 500);

    // Type byte is at offset 14
    let type_byte = buf[14];
    assert_eq!(type_byte & 0x03, 1); // forward loop
}

#[test]
fn test_sample_loop_pingpong() {
    let sample_data = vec![0u8; 2000];
    let sample = XmSample::new("PingPong", sample_data, true).with_loop(0, 1000, 2); // ping-pong loop

    let mut buf = Vec::new();
    sample.write_header(&mut buf).unwrap();

    let type_byte = buf[14];
    assert_eq!(type_byte & 0x03, 2); // ping-pong
}

#[test]
fn test_sample_16bit_flag() {
    let sample = XmSample::new("16bit", vec![0u8; 100], true);
    let mut buf = Vec::new();
    sample.write_header(&mut buf).unwrap();

    let type_byte = buf[14];
    assert_eq!(type_byte & 0x10, 0x10); // 16-bit flag
}

#[test]
fn test_sample_8bit_flag() {
    let sample = XmSample::new("8bit", vec![0u8; 100], false);
    let mut buf = Vec::new();
    sample.write_header(&mut buf).unwrap();

    let type_byte = buf[14];
    assert_eq!(type_byte & 0x10, 0); // no 16-bit flag
}

#[test]
fn test_sample_volume() {
    let mut sample = XmSample::new("VolTest", vec![0u8; 100], true);
    sample.volume = 48;

    let mut buf = Vec::new();
    sample.write_header(&mut buf).unwrap();

    assert_eq!(buf[12], 48);
}

#[test]
fn test_sample_finetune() {
    let mut sample = XmSample::new("FineTest", vec![0u8; 100], true);
    sample.finetune = -16;

    let mut buf = Vec::new();
    sample.write_header(&mut buf).unwrap();

    assert_eq!(buf[13] as i8, -16);
}

#[test]
fn test_sample_relative_note() {
    let mut sample = XmSample::new("RelNote", vec![0u8; 100], true);
    sample.relative_note = 12; // one octave up

    let mut buf = Vec::new();
    sample.write_header(&mut buf).unwrap();

    assert_eq!(buf[16] as i8, 12);
}

#[test]
fn test_sample_panning() {
    let mut sample = XmSample::new("PanTest", vec![0u8; 100], true);
    sample.panning = 0; // full left

    let mut buf = Vec::new();
    sample.write_header(&mut buf).unwrap();

    assert_eq!(buf[15], 0);

    sample.panning = 255; // full right
    buf.clear();
    sample.write_header(&mut buf).unwrap();
    assert_eq!(buf[15], 255);
}

#[test]
fn test_sample_name() {
    let sample = XmSample::new("MySampleName12345", vec![0u8; 100], true);
    let mut buf = Vec::new();
    sample.write_header(&mut buf).unwrap();

    // Sample name is at offset 18-39 (22 bytes)
    let name = std::str::from_utf8(&buf[18..40])
        .unwrap()
        .trim_end_matches('\0');
    assert_eq!(name, "MySampleName12345");
}

#[test]
fn test_sample_length_samples() {
    let sample_16bit = XmSample::new("16bit", vec![0u8; 200], true);
    assert_eq!(sample_16bit.length_samples(), 100); // 200 bytes / 2

    let sample_8bit = XmSample::new("8bit", vec![0u8; 200], false);
    assert_eq!(sample_8bit.length_samples(), 200);
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
