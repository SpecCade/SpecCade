//! Unit tests for audio analysis.

use super::*;

fn create_test_wav(samples: &[f32], sample_rate: u32, channels: u16) -> Vec<u8> {
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * channels as u32 * 2;
    let block_align = channels * 2;
    let data_size = samples.len() * 2;
    let file_size = 36 + data_size;

    let mut wav = Vec::new();

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(file_size as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_size as u32).to_le_bytes());
    for &s in samples {
        let sample_i16 = (s * 32767.0) as i16;
        wav.extend_from_slice(&sample_i16.to_le_bytes());
    }

    wav
}

#[test]
fn test_analyze_sine_wave() {
    let sample_rate = 44100;
    let frequency = 440.0;
    let duration = 0.1;
    let num_samples = (sample_rate as f64 * duration) as usize;

    let samples: Vec<f32> = (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5
        })
        .collect();

    let wav = create_test_wav(&samples, sample_rate, 1);
    let metrics = analyze_wav(&wav).unwrap();

    assert_eq!(metrics.format.sample_rate, sample_rate);
    assert_eq!(metrics.format.channels, 1);
    assert!(!metrics.quality.clipping_detected);
    assert!(metrics.quality.peak_db < 0.0);
    assert!(metrics.quality.rms_db < metrics.quality.peak_db);
}

#[test]
fn test_analyze_silence() {
    let samples = vec![0.0f32; 1000];
    let wav = create_test_wav(&samples, 44100, 1);
    let metrics = analyze_wav(&wav).unwrap();

    assert_eq!(metrics.quality.silence_ratio, 1.0);
    assert!(!metrics.quality.clipping_detected);
}

#[test]
fn test_analyze_clipped() {
    let samples = vec![1.0f32; 1000];
    let wav = create_test_wav(&samples, 44100, 1);
    let metrics = analyze_wav(&wav).unwrap();

    assert!(metrics.quality.clipping_detected);
}

#[test]
fn test_deterministic_output() {
    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav = create_test_wav(&samples, 44100, 1);

    let metrics1 = analyze_wav(&wav).unwrap();
    let metrics2 = analyze_wav(&wav).unwrap();

    let json1 = serde_json::to_string(&metrics_to_btree(&metrics1)).unwrap();
    let json2 = serde_json::to_string(&metrics_to_btree(&metrics2)).unwrap();

    assert_eq!(json1, json2);
}

#[test]
fn test_metrics_to_btree_sorted_keys() {
    let samples: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.01).sin()).collect();
    let wav = create_test_wav(&samples, 44100, 1);
    let metrics = analyze_wav(&wav).unwrap();

    let btree = metrics_to_btree(&metrics);
    let keys: Vec<_> = btree.keys().collect();

    // Keys should be alphabetically sorted
    assert_eq!(keys, vec!["format", "quality", "spectral", "temporal"]);
}
