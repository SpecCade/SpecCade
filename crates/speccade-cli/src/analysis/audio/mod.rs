//! Audio analysis module for extracting quality metrics from WAV files.
//!
//! This module provides deterministic audio analysis for LLM-driven iteration loops
//! and quality gating. All metrics are computed to produce byte-identical JSON output
//! across runs on the same input.

mod quality;
mod spectral;
mod temporal;
mod types;

#[cfg(test)]
mod tests;

use serde_json;
use std::collections::BTreeMap;

// Re-export public types
pub use types::{
    AudioAnalysisError, AudioFormatMetadata, AudioMetrics, AudioQualityMetrics,
    AudioSpectralMetrics, AudioTemporalMetrics,
};

// Re-export internal types for submodules
use types::WavHeader;

// Import metric calculation functions
use quality::{
    calculate_dc_offset, calculate_peak, calculate_rms, calculate_silence_ratio, detect_clipping,
};
use spectral::{calculate_dominant_frequency, calculate_spectral_centroid};
use temporal::{calculate_attack_ms, calculate_zero_crossing_rate};

/// Precision for floating point values in output (6 decimal places).
const FLOAT_PRECISION: i32 = 6;

/// Round a float to the specified number of decimal places.
fn round_f64(value: f64, decimals: i32) -> f64 {
    let multiplier = 10_f64.powi(decimals);
    (value * multiplier).round() / multiplier
}

/// Parse WAV header and return metadata.
fn parse_wav_header(wav_data: &[u8]) -> Result<WavHeader, AudioAnalysisError> {
    if wav_data.len() < 44 {
        return Err(AudioAnalysisError::DataTooShort {
            expected: 44,
            actual: wav_data.len(),
        });
    }

    if &wav_data[0..4] != b"RIFF" {
        return Err(AudioAnalysisError::InvalidRiffHeader);
    }

    if &wav_data[8..12] != b"WAVE" {
        return Err(AudioAnalysisError::InvalidWaveFormat);
    }

    let (fmt_offset, _) =
        find_chunk(wav_data, b"fmt ")?.ok_or(AudioAnalysisError::MissingFmtChunk)?;

    let fmt_data = &wav_data[fmt_offset..];
    if fmt_data.len() < 16 {
        return Err(AudioAnalysisError::InvalidChunk {
            message: "fmt chunk too short".to_string(),
        });
    }

    let audio_format = u16::from_le_bytes([fmt_data[0], fmt_data[1]]);
    let channels = u16::from_le_bytes([fmt_data[2], fmt_data[3]]);
    let sample_rate = u32::from_le_bytes([fmt_data[4], fmt_data[5], fmt_data[6], fmt_data[7]]);
    let bits_per_sample = u16::from_le_bytes([fmt_data[14], fmt_data[15]]);

    if audio_format != 1 {
        return Err(AudioAnalysisError::UnsupportedAudioFormat {
            format_code: audio_format,
        });
    }

    let (data_offset, data_size) =
        find_chunk(wav_data, b"data")?.ok_or(AudioAnalysisError::MissingDataChunk)?;

    let bytes_per_sample = (bits_per_sample / 8) as usize;
    let num_samples = if bytes_per_sample > 0 && channels > 0 {
        data_size / (bytes_per_sample * channels as usize)
    } else {
        0
    };

    if data_offset + data_size > wav_data.len() {
        return Err(AudioAnalysisError::InvalidChunk {
            message: format!(
                "data chunk extends beyond file: offset {} + size {} > file length {}",
                data_offset,
                data_size,
                wav_data.len()
            ),
        });
    }

    Ok(WavHeader {
        channels,
        sample_rate,
        bits_per_sample,
        num_samples,
    })
}

/// Find a chunk in WAV data and return its data offset and size.
fn find_chunk(
    wav_data: &[u8],
    chunk_id: &[u8; 4],
) -> Result<Option<(usize, usize)>, AudioAnalysisError> {
    let mut offset = 12;

    while offset + 8 <= wav_data.len() {
        let id = &wav_data[offset..offset + 4];
        let size = u32::from_le_bytes([
            wav_data[offset + 4],
            wav_data[offset + 5],
            wav_data[offset + 6],
            wav_data[offset + 7],
        ]) as usize;

        if id == chunk_id {
            return Ok(Some((offset + 8, size)));
        }

        let chunk_total_size = 8 + size + (size % 2);
        offset += chunk_total_size;
    }

    Ok(None)
}

/// Parse WAV samples to f32 normalized to [-1.0, 1.0].
fn parse_wav_samples(wav_data: &[u8]) -> Result<Vec<f32>, AudioAnalysisError> {
    let header = parse_wav_header(wav_data)?;

    let (data_offset, data_size) =
        find_chunk(wav_data, b"data")?.ok_or(AudioAnalysisError::MissingDataChunk)?;

    let data = &wav_data[data_offset..data_offset + data_size];

    match header.bits_per_sample {
        8 => Ok(data.iter().map(|&b| (b as f32 - 128.0) / 128.0).collect()),
        16 => Ok(data
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / 32768.0
            })
            .collect()),
        24 => Ok(data
            .chunks_exact(3)
            .map(|chunk| {
                let sample = if chunk[2] & 0x80 != 0 {
                    i32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0xFF])
                } else {
                    i32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0x00])
                };
                sample as f32 / 8388608.0
            })
            .collect()),
        32 => Ok(data
            .chunks_exact(4)
            .map(|chunk| {
                let sample = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                sample as f32 / 2147483648.0
            })
            .collect()),
        bits => Err(AudioAnalysisError::UnsupportedBitsPerSample { bits }),
    }
}

/// Convert stereo samples to mono by averaging channels.
fn stereo_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels == 1 {
        return samples.to_vec();
    }

    samples
        .chunks_exact(channels as usize)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Extract mono samples from WAV data for embedding computation.
///
/// Returns normalized samples in the range [-1.0, 1.0] and the sample rate.
pub fn extract_wav_samples(wav_data: &[u8]) -> Result<(Vec<f32>, u32), AudioAnalysisError> {
    let header = parse_wav_header(wav_data)?;
    let samples = parse_wav_samples(wav_data)?;

    if samples.is_empty() {
        return Err(AudioAnalysisError::EmptyAudio);
    }

    let mono_samples = stereo_to_mono(&samples, header.channels);
    Ok((mono_samples, header.sample_rate))
}

/// Analyze a WAV file and return metrics.
pub fn analyze_wav(wav_data: &[u8]) -> Result<AudioMetrics, AudioAnalysisError> {
    let header = parse_wav_header(wav_data)?;
    let samples = parse_wav_samples(wav_data)?;

    if samples.is_empty() {
        return Err(AudioAnalysisError::EmptyAudio);
    }

    // Convert to mono for analysis
    let mono_samples = stereo_to_mono(&samples, header.channels);

    // Calculate format metadata
    let duration_ms = if header.sample_rate > 0 {
        round_f64(
            header.num_samples as f64 / header.sample_rate as f64 * 1000.0,
            FLOAT_PRECISION,
        )
    } else {
        0.0
    };

    // Calculate quality metrics
    let peak = calculate_peak(&mono_samples);
    let peak_db = if peak > 0.0 {
        round_f64(20.0 * (peak as f64).log10(), FLOAT_PRECISION)
    } else {
        -100.0
    };

    let rms = calculate_rms(&mono_samples);
    let rms_db = if rms > 0.0 {
        round_f64(20.0 * rms.log10(), FLOAT_PRECISION)
    } else {
        -100.0
    };

    let clipping_detected = detect_clipping(&mono_samples);
    let dc_offset = round_f64(calculate_dc_offset(&mono_samples), FLOAT_PRECISION);
    let silence_ratio = round_f64(calculate_silence_ratio(&mono_samples), FLOAT_PRECISION);

    // Calculate temporal metrics
    let attack_ms = round_f64(
        calculate_attack_ms(&mono_samples, header.sample_rate),
        FLOAT_PRECISION,
    );
    let zero_crossing_rate =
        round_f64(calculate_zero_crossing_rate(&mono_samples), FLOAT_PRECISION);

    // Calculate spectral metrics
    let centroid_hz = round_f64(
        calculate_spectral_centroid(&mono_samples, header.sample_rate),
        FLOAT_PRECISION,
    );
    let dominant_frequency_hz = round_f64(
        calculate_dominant_frequency(&mono_samples, header.sample_rate),
        FLOAT_PRECISION,
    );

    Ok(AudioMetrics {
        format: AudioFormatMetadata {
            sample_rate: header.sample_rate,
            channels: header.channels,
            bits_per_sample: header.bits_per_sample,
            duration_ms,
            num_samples: header.num_samples,
        },
        quality: AudioQualityMetrics {
            peak_db,
            rms_db,
            clipping_detected,
            dc_offset,
            silence_ratio,
        },
        temporal: AudioTemporalMetrics {
            attack_ms,
            zero_crossing_rate,
        },
        spectral: AudioSpectralMetrics {
            centroid_hz,
            dominant_frequency_hz,
        },
    })
}

/// Convert AudioMetrics to a BTreeMap for deterministic JSON serialization.
pub fn metrics_to_btree(metrics: &AudioMetrics) -> BTreeMap<String, serde_json::Value> {
    let mut map = BTreeMap::new();

    // Format section
    let mut format = BTreeMap::new();
    format.insert(
        "bits_per_sample".to_string(),
        serde_json::json!(metrics.format.bits_per_sample),
    );
    format.insert(
        "channels".to_string(),
        serde_json::json!(metrics.format.channels),
    );
    format.insert(
        "duration_ms".to_string(),
        serde_json::json!(metrics.format.duration_ms),
    );
    format.insert(
        "num_samples".to_string(),
        serde_json::json!(metrics.format.num_samples),
    );
    format.insert(
        "sample_rate".to_string(),
        serde_json::json!(metrics.format.sample_rate),
    );
    map.insert("format".to_string(), serde_json::json!(format));

    // Quality section
    let mut quality = BTreeMap::new();
    quality.insert(
        "clipping_detected".to_string(),
        serde_json::json!(metrics.quality.clipping_detected),
    );
    quality.insert(
        "dc_offset".to_string(),
        serde_json::json!(metrics.quality.dc_offset),
    );
    quality.insert(
        "peak_db".to_string(),
        serde_json::json!(metrics.quality.peak_db),
    );
    quality.insert(
        "rms_db".to_string(),
        serde_json::json!(metrics.quality.rms_db),
    );
    quality.insert(
        "silence_ratio".to_string(),
        serde_json::json!(metrics.quality.silence_ratio),
    );
    map.insert("quality".to_string(), serde_json::json!(quality));

    // Spectral section
    let mut spectral = BTreeMap::new();
    spectral.insert(
        "centroid_hz".to_string(),
        serde_json::json!(metrics.spectral.centroid_hz),
    );
    spectral.insert(
        "dominant_frequency_hz".to_string(),
        serde_json::json!(metrics.spectral.dominant_frequency_hz),
    );
    map.insert("spectral".to_string(), serde_json::json!(spectral));

    // Temporal section
    let mut temporal = BTreeMap::new();
    temporal.insert(
        "attack_ms".to_string(),
        serde_json::json!(metrics.temporal.attack_ms),
    );
    temporal.insert(
        "zero_crossing_rate".to_string(),
        serde_json::json!(metrics.temporal.zero_crossing_rate),
    );
    map.insert("temporal".to_string(), serde_json::json!(temporal));

    map
}
