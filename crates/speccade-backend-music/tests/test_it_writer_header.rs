//! Tests for IT (Impulse Tracker) header structure and IT-specific features.
//!
//! This module verifies header format correctness, version information, counts,
//! channel configuration, NNA modes, envelope handling, and instrument settings.

use speccade_backend_music::it::{
    dca, dct, env_flags, flags, nna, ItEnvelope, ItEnvelopePoint, ItHeader, ItInstrument,
    ItModule, ItPattern, ItSample, IT_INSTRUMENT_MAGIC,
};

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
    assert_eq!(
        &it[0..4],
        b"IMPM",
        "IT file must start with 'IMPM' magic bytes"
    );
}

#[test]
fn test_it_header_magic_constant() {
    use speccade_backend_music::it::IT_MAGIC;
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
        module.add_sample(ItSample::new(
            &format!("Sample{}", i),
            vec![0u8; 100],
            22050,
        ));
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
        assert_eq!(vol, 64, "Active channel {} should have full volume (64)", i);
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
    let env = ItEnvelope {
        flags: env_flags::ENABLED,
        points: vec![
            ItEnvelopePoint { tick: 0, value: 0 },
            ItEnvelopePoint {
                tick: 10,
                value: 64,
            },
            ItEnvelopePoint {
                tick: 50,
                value: 32,
            },
        ],
        ..Default::default()
    };

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
    let env = ItEnvelope {
        flags: env_flags::ENABLED | env_flags::LOOP | env_flags::SUSTAIN_LOOP,
        loop_begin: 1,
        loop_end: 3,
        sustain_begin: 2,
        sustain_end: 4,
        points: vec![
            ItEnvelopePoint { tick: 0, value: 0 },
            ItEnvelopePoint {
                tick: 10,
                value: 64,
            },
            ItEnvelopePoint {
                tick: 20,
                value: 48,
            },
            ItEnvelopePoint {
                tick: 30,
                value: 32,
            },
            ItEnvelopePoint { tick: 40, value: 0 },
        ],
    };

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
    assert_eq!(
        buf[58],
        64 | 0x80,
        "Filter cutoff should be set with enable bit"
    );
    assert_eq!(
        buf[59],
        32 | 0x80,
        "Filter resonance should be set with enable bit"
    );
}
