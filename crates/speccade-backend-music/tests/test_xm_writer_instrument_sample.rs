//! Tests for XM instrument and sample data structures.
//!
//! These tests validate the binary format of instruments, samples,
//! envelopes, and sample data encoding.

use speccade_backend_music::xm::{
    XmEnvelope, XmEnvelopePoint, XmInstrument, XmSample, XM_INSTRUMENT_HEADER_SIZE,
    XM_SAMPLE_HEADER_SIZE,
};

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
    // XM stores loop offsets in bytes; for 16-bit samples each sample is 2 bytes.
    assert_eq!(loop_start, 200);

    // Loop length is at offset 8-11
    let loop_length = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
    assert_eq!(loop_length, 1000);

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
