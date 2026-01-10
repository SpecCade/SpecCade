//! Comprehensive tests for the IT (Impulse Tracker) writer module.
//!
//! These tests verify binary format correctness, header structure, pattern encoding,
//! sample handling, and IT-specific features like NNA modes and envelope loops.

use speccade_backend_music::it::{
    dca, dct, env_flags, flags, nna, sample_flags, convert_flags,
    ItEnvelope, ItEnvelopePoint, ItHeader, ItInstrument, ItModule, ItNote, ItPattern,
    ItSample, IT_INSTRUMENT_MAGIC, IT_INSTRUMENT_SIZE, IT_MAGIC, IT_SAMPLE_HEADER_SIZE,
    IT_SAMPLE_MAGIC, effects,
};
use speccade_backend_music::it::{validate_it_bytes, ItValidationError};
use speccade_backend_music::note::{note_name_to_it, it as it_notes};

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
// 1. Header Validation Tests
// =============================================================================

#[test]
fn test_it_header_magic() {
    let it = generate_minimal_it();
    assert_eq!(&it[0..4], b"IMPM", "IT file must start with 'IMPM' magic bytes");
}

#[test]
fn test_it_header_magic_constant() {
    assert_eq!(IT_MAGIC, b"IMPM", "IT_MAGIC constant should be 'IMPM'");
}

#[test]
fn test_it_header_version() {
    let it = generate_minimal_it();
    // Cwt/v (created with tracker version) is at offset 0x28 (40)
    let cwtt = read_u16_le(&it, 0x28);
    // Cmwt (compatible with version) is at offset 0x2A (42)
    let cmwt = read_u16_le(&it, 0x2A);

    // Should be compatible with IT 2.00+ format
    assert!(cmwt >= 0x0200, "Cmwt should be at least 0x0200 (IT 2.00)");
    // Cwt/v should indicate version 2.14 or similar
    assert!(cwtt >= 0x0200, "Cwt/v should be at least 0x0200");
}

#[test]
fn test_it_header_size() {
    let header = ItHeader::new("Test Song", 8, 6, 125);
    let mut buf = Vec::new();
    header.write(&mut buf).unwrap();

    // IT header is exactly 192 bytes
    assert_eq!(buf.len(), 192, "IT header must be exactly 192 bytes");
}

#[test]
fn test_it_header_song_name() {
    let name = "My Test Song";
    let it = generate_it_with_params(name, 4, 6, 125);

    // Song name is at offset 0x04, 26 bytes
    let name_bytes = &it[0x04..0x04 + 26];
    let name_str = std::str::from_utf8(&name_bytes[..name.len()]).unwrap();
    assert_eq!(name_str, name, "Song name should be correctly stored");

    // Rest should be null-padded
    for &b in &name_bytes[name.len()..] {
        assert_eq!(b, 0, "Name should be null-padded");
    }
}

#[test]
fn test_it_header_pattern_highlight() {
    let it = generate_minimal_it();
    // Pattern highlight is at offset 0x1E (30) - minor, 0x1F (31) - major
    let minor = it[0x1E];
    let major = it[0x1F];

    // Default highlight is (4, 16)
    assert_eq!(minor, 4, "Minor pattern highlight should be 4");
    assert_eq!(major, 16, "Major pattern highlight should be 16");
}

#[test]
fn test_it_header_counts() {
    let mut module = ItModule::new("Test", 4, 6, 125);

    // Add 3 patterns
    for _ in 0..3 {
        module.add_pattern(ItPattern::empty(64, 4));
    }

    // Add 2 instruments
    for i in 0..2 {
        module.add_instrument(ItInstrument::new(&format!("Inst{}", i)));
    }

    // Add 2 samples
    for i in 0..2 {
        module.add_sample(ItSample::new(&format!("Sample{}", i), vec![0u8; 100], 22050));
    }

    module.set_orders(&[0, 1, 2]);

    let it = module.to_bytes().unwrap();

    // Order count at 0x20
    let order_count = read_u16_le(&it, 0x20);
    assert_eq!(order_count, 3, "Order count should be 3");

    // Instrument count at 0x22
    let inst_count = read_u16_le(&it, 0x22);
    assert_eq!(inst_count, 2, "Instrument count should be 2");

    // Sample count at 0x24
    let sample_count = read_u16_le(&it, 0x24);
    assert_eq!(sample_count, 2, "Sample count should be 2");

    // Pattern count at 0x26
    let pattern_count = read_u16_le(&it, 0x26);
    assert_eq!(pattern_count, 3, "Pattern count should be 3");
}

#[test]
fn test_it_header_speed_tempo() {
    let speed = 8u8;
    let bpm = 140u8;
    let it = generate_it_with_params("Test", 4, speed, bpm);

    // Initial speed at 0x32
    assert_eq!(it[0x32], speed, "Initial speed should match");

    // Initial tempo at 0x33
    assert_eq!(it[0x33], bpm, "Initial tempo should match");
}

#[test]
fn test_it_header_global_volume() {
    let it = generate_minimal_it();

    // Global volume at 0x30
    let global_vol = it[0x30];
    assert_eq!(global_vol, 128, "Default global volume should be 128");
}

#[test]
fn test_it_header_mix_volume() {
    let it = generate_minimal_it();

    // Mix volume at 0x31
    let mix_vol = it[0x31];
    assert_eq!(mix_vol, 48, "Default mix volume should be 48");
}

#[test]
fn test_it_header_flags() {
    let it = generate_minimal_it();

    // Flags at 0x2C
    let header_flags = read_u16_le(&it, 0x2C);

    // Default flags should include STEREO, USE_INSTRUMENTS, LINEAR_SLIDES
    assert!(
        header_flags & flags::STEREO != 0,
        "STEREO flag should be set"
    );
    assert!(
        header_flags & flags::USE_INSTRUMENTS != 0,
        "USE_INSTRUMENTS flag should be set"
    );
    assert!(
        header_flags & flags::LINEAR_SLIDES != 0,
        "LINEAR_SLIDES flag should be set"
    );
}

#[test]
fn test_it_header_offsets() {
    let mut module = ItModule::new("Test", 4, 6, 125);
    module.add_pattern(ItPattern::empty(64, 4));
    module.add_instrument(ItInstrument::new("Inst1"));
    module.add_sample(ItSample::new("Sample1", vec![0u8; 100], 22050));
    module.set_orders(&[0]);

    let it = module.to_bytes().unwrap();

    // After header (192 bytes) comes orders
    let order_count = read_u16_le(&it, 0x20) as usize;
    let offset_table_start = 192 + order_count;

    // Instrument offset table
    let inst_offset = read_u32_le(&it, offset_table_start) as usize;
    assert!(inst_offset > 0, "Instrument offset should be non-zero");

    // Verify instrument magic at that offset
    assert_eq!(
        &it[inst_offset..inst_offset + 4],
        IT_INSTRUMENT_MAGIC,
        "Instrument magic should be at calculated offset"
    );
}

#[test]
fn test_it_header_channel_pan() {
    let it = generate_it_with_params("Test", 8, 6, 125);

    // Channel pan starts at 0x40 (64), 64 bytes
    for i in 0..8 {
        let pan = it[0x40 + i];
        assert_eq!(pan, 32, "Active channel {} should have center pan (32)", i);
    }

    // Disabled channels should have pan 128
    for i in 8..64 {
        let pan = it[0x40 + i];
        assert_eq!(pan, 128, "Disabled channel {} should have pan 128", i);
    }
}

#[test]
fn test_it_header_channel_volume() {
    let it = generate_it_with_params("Test", 8, 6, 125);

    // Channel volume starts at 0x80 (128), 64 bytes
    for i in 0..8 {
        let vol = it[0x80 + i];
        assert_eq!(
            vol, 64,
            "Active channel {} should have full volume (64)",
            i
        );
    }

    // Disabled channels should have volume 0
    for i in 8..64 {
        let vol = it[0x80 + i];
        assert_eq!(vol, 0, "Disabled channel {} should have volume 0", i);
    }
}

// =============================================================================
// 2. IT-Specific Features Tests
// =============================================================================

#[test]
fn test_it_nna_modes() {
    // Test New Note Action modes
    assert_eq!(nna::CUT, 0, "NNA CUT should be 0");
    assert_eq!(nna::CONTINUE, 1, "NNA CONTINUE should be 1");
    assert_eq!(nna::OFF, 2, "NNA OFF should be 2");
    assert_eq!(nna::FADE, 3, "NNA FADE should be 3");

    // Create instrument with each NNA mode
    for nna_mode in [nna::CUT, nna::CONTINUE, nna::OFF, nna::FADE] {
        let instrument = ItInstrument::new("Test").with_nna(nna_mode, dct::OFF, dca::CUT);
        assert_eq!(instrument.nna, nna_mode, "NNA mode should match");
    }
}

#[test]
fn test_it_nna_in_instrument_header() {
    let instrument = ItInstrument::new("Test").with_nna(nna::FADE, dct::NOTE, dca::FADE);

    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    // NNA is at offset 17 (after magic + filename + reserved)
    assert_eq!(buf[17], nna::FADE, "NNA mode should be written correctly");
    // DCT is at offset 18
    assert_eq!(buf[18], dct::NOTE, "DCT mode should be written correctly");
    // DCA is at offset 19
    assert_eq!(buf[19], dca::FADE, "DCA mode should be written correctly");
}

#[test]
fn test_it_dct_modes() {
    // Test Duplicate Check Type modes
    assert_eq!(dct::OFF, 0, "DCT OFF should be 0");
    assert_eq!(dct::NOTE, 1, "DCT NOTE should be 1");
    assert_eq!(dct::SAMPLE, 2, "DCT SAMPLE should be 2");
    assert_eq!(dct::INSTRUMENT, 3, "DCT INSTRUMENT should be 3");
}

#[test]
fn test_it_dca_modes() {
    // Test Duplicate Check Action modes
    assert_eq!(dca::CUT, 0, "DCA CUT should be 0");
    assert_eq!(dca::OFF, 1, "DCA OFF should be 1");
    assert_eq!(dca::FADE, 2, "DCA FADE should be 2");
}

#[test]
fn test_it_envelope_flags() {
    assert_eq!(env_flags::ENABLED, 0x01, "Envelope ENABLED flag");
    assert_eq!(env_flags::LOOP, 0x02, "Envelope LOOP flag");
    assert_eq!(env_flags::SUSTAIN_LOOP, 0x04, "Envelope SUSTAIN_LOOP flag");
    assert_eq!(env_flags::CARRY, 0x08, "Envelope CARRY flag");
    assert_eq!(env_flags::FILTER, 0x80, "Envelope FILTER flag");
}

#[test]
fn test_it_envelope_basic() {
    let mut env = ItEnvelope::default();
    env.flags = env_flags::ENABLED;
    env.points = vec![
        ItEnvelopePoint { tick: 0, value: 0 },
        ItEnvelopePoint { tick: 10, value: 64 },
        ItEnvelopePoint { tick: 50, value: 32 },
    ];

    let mut buf = Vec::new();
    env.write(&mut buf, false).unwrap();

    // Volume/panning envelope is 82 bytes
    assert_eq!(buf.len(), 82, "Volume envelope should be 82 bytes");

    // Check flags at offset 0
    assert_eq!(buf[0], env_flags::ENABLED, "Envelope flags should match");

    // Check number of points at offset 1
    assert_eq!(buf[1], 3, "Should have 3 envelope points");
}

#[test]
fn test_it_envelope_loops() {
    let mut env = ItEnvelope::default();
    env.flags = env_flags::ENABLED | env_flags::LOOP | env_flags::SUSTAIN_LOOP;
    env.loop_begin = 1;
    env.loop_end = 3;
    env.sustain_begin = 2;
    env.sustain_end = 4;
    env.points = vec![
        ItEnvelopePoint { tick: 0, value: 0 },
        ItEnvelopePoint { tick: 10, value: 64 },
        ItEnvelopePoint { tick: 20, value: 48 },
        ItEnvelopePoint { tick: 30, value: 32 },
        ItEnvelopePoint { tick: 40, value: 0 },
    ];

    let mut buf = Vec::new();
    env.write(&mut buf, false).unwrap();

    // Check loop points at offsets 2-5
    assert_eq!(buf[2], 1, "Loop begin index");
    assert_eq!(buf[3], 3, "Loop end index");
    assert_eq!(buf[4], 2, "Sustain begin index");
    assert_eq!(buf[5], 4, "Sustain end index");
}

#[test]
fn test_it_pitch_envelope() {
    let env = ItEnvelope::default();

    let mut buf = Vec::new();
    env.write(&mut buf, true).unwrap();

    // Pitch envelope is 83 bytes (one extra reserved byte)
    assert_eq!(buf.len(), 83, "Pitch envelope should be 83 bytes");
}

#[test]
fn test_it_adsr_envelope() {
    let env = ItEnvelope::adsr_volume(10, 20, 32, 30);

    assert!(
        env.flags & env_flags::ENABLED != 0,
        "ADSR envelope should be enabled"
    );
    assert!(
        env.flags & env_flags::SUSTAIN_LOOP != 0,
        "ADSR envelope should have sustain loop"
    );
    assert!(env.points.len() >= 4, "ADSR should have at least 4 points");

    // First point should be at 0 with value 0 (attack start)
    assert_eq!(env.points[0].tick, 0);
    assert_eq!(env.points[0].value, 0);

    // Second point should be at attack time with max value
    assert_eq!(env.points[1].tick, 10);
    assert_eq!(env.points[1].value, 64);
}

#[test]
fn test_it_instrument_fadeout() {
    let instrument = ItInstrument::new("Test");

    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    // Fadeout is at offset 20-21 (after NNA/DCT/DCA)
    let fadeout = read_u16_le(&buf, 20);
    assert_eq!(fadeout, 256, "Default fadeout should be 256");
}

#[test]
fn test_it_instrument_global_volume() {
    let mut instrument = ItInstrument::new("Test");
    instrument.global_volume = 100;

    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    // Global volume is at offset 24
    assert_eq!(buf[24], 100, "Instrument global volume should match");
}

#[test]
fn test_it_instrument_note_sample_table() {
    let mut instrument = ItInstrument::new("Test");
    instrument.map_all_to_sample(5);

    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    // Note-sample table starts at offset 64, 240 bytes (120 entries * 2)
    // Layout: magic(4) + filename(12) + reserved(1) + NNA/DCT/DCA(3) + fadeout(2)
    //       + pitch-pan(2) + global_vol(1) + default_pan(1) + random(2) + reserved(4)
    //       + name(26) + filter(2) + MIDI(4) = 64 bytes before note-sample table
    // Each entry is (note, sample)
    for i in 0..120 {
        let offset = 64 + i * 2;
        let note = buf[offset];
        let sample = buf[offset + 1];
        assert_eq!(note, i as u8, "Note {} should map correctly", i);
        assert_eq!(sample, 5, "Note {} should map to sample 5", i);
    }
}

#[test]
fn test_it_instrument_filter() {
    let mut instrument = ItInstrument::new("Test");
    instrument.filter_cutoff = Some(64);
    instrument.filter_resonance = Some(32);

    let mut buf = Vec::new();
    instrument.write(&mut buf).unwrap();

    // IFC (filter cutoff) at offset 58, IFR (filter resonance) at offset 59
    // Layout: magic(4) + filename(12) + reserved(1) + NNA/DCT/DCA(3) + fadeout(2)
    //       + pitch-pan(2) + global_vol(1) + default_pan(1) + random(2) + reserved(4)
    //       + name(26) = 58 bytes before filter cutoff
    // Bit 7 should be set to indicate filter is enabled
    assert_eq!(buf[58], 64 | 0x80, "Filter cutoff should be set with enable bit");
    assert_eq!(buf[59], 32 | 0x80, "Filter resonance should be set with enable bit");
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
    assert_eq!(packed.len(), 64, "Empty pattern should pack to 64 bytes (row markers)");

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
    let row_markers: Vec<_> = packed
        .iter()
        .enumerate()
        .filter(|(_, &b)| b == 0)
        .collect();
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
    assert!(
        mask & 0x08 != 0,
        "Mask should indicate effect data (bit 3)"
    );
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
    assert_eq!(sample.length, 500, "Length should be in samples (1000 bytes / 2)");
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

    assert_eq!(buf.len(), IT_SAMPLE_HEADER_SIZE, "Sample header should be 80 bytes");
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
    assert_eq!(stored_offset, data_offset, "Sample data pointer should match");
}

#[test]
fn test_it_sample_with_loop() {
    let data = vec![0u8; 1000];
    let sample = ItSample::new("Test", data, 22050).with_loop(0, 400, false);

    assert!(sample.flags & sample_flags::LOOP != 0, "Loop flag should be set");
    assert_eq!(sample.loop_begin, 0);
    assert_eq!(sample.loop_end, 400);
}

#[test]
fn test_it_sample_with_pingpong_loop() {
    let data = vec![0u8; 1000];
    let sample = ItSample::new("Test", data, 22050).with_loop(100, 400, true);

    assert!(sample.flags & sample_flags::LOOP != 0, "Loop flag should be set");
    assert!(sample.flags & sample_flags::LOOP_PINGPONG != 0, "Pingpong flag should be set");
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
    use speccade_backend_music::it::IT_MAX_INSTRUMENTS;
    use speccade_backend_music::xm::XM_MAX_INSTRUMENTS;

    assert_eq!(IT_MAX_INSTRUMENTS, 99, "IT should support 99 instruments");
    assert_eq!(XM_MAX_INSTRUMENTS, 128, "XM should support 128 instruments");
}

#[test]
fn test_it_vs_xm_sample_limit() {
    use speccade_backend_music::it::IT_MAX_SAMPLES;

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
        module.add_sample(ItSample::new(&format!("Sample{}", i), vec![0u8; 100], 22050));
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
