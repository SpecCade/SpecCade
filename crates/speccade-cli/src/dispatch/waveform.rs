//! Audio waveform visualization generator.
//!
//! Generates deterministic waveform preview images from audio data.

use png::{BitDepth, ColorType, Compression, Encoder, FilterType};
use std::io::Write;

/// Waveform image dimensions.
pub const WAVEFORM_WIDTH: u32 = 1024;
pub const WAVEFORM_HEIGHT: u32 = 256;

/// Colors for the waveform visualization.
const BACKGROUND_COLOR: [u8; 3] = [32, 32, 32]; // Dark gray background
const WAVEFORM_COLOR: [u8; 3] = [64, 192, 255]; // Cyan/blue waveform
const CENTER_LINE_COLOR: [u8; 3] = [64, 64, 64]; // Darker gray center line
const PEAK_COLOR: [u8; 3] = [255, 128, 64]; // Orange for peak markers

/// Result of waveform generation.
pub struct WaveformResult {
    /// PNG image data.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
}

/// Generates a waveform visualization PNG from 16-bit PCM audio data.
///
/// The waveform shows amplitude over time, with the audio centered vertically.
/// For stereo audio, both channels are mixed to mono for visualization.
///
/// # Arguments
/// * `pcm_data` - Raw 16-bit little-endian PCM data
/// * `is_stereo` - Whether the audio is stereo (interleaved L/R samples)
///
/// # Returns
/// A `WaveformResult` containing the PNG data and its hash.
pub fn generate_waveform_png(pcm_data: &[u8], is_stereo: bool) -> WaveformResult {
    // Convert PCM bytes to mono f32 samples
    let samples = pcm_to_mono_f32(pcm_data, is_stereo);

    // Calculate per-column min/max amplitudes
    let column_data = compute_column_data(&samples, WAVEFORM_WIDTH as usize);

    // Render to RGB image buffer
    let rgb_buffer = render_waveform(&column_data, WAVEFORM_WIDTH, WAVEFORM_HEIGHT);

    // Encode as PNG
    let png_data = encode_png(&rgb_buffer, WAVEFORM_WIDTH, WAVEFORM_HEIGHT);
    let hash = blake3::hash(&png_data).to_hex().to_string();

    WaveformResult { png_data, hash }
}

/// Converts 16-bit PCM data to mono f32 samples in range [-1.0, 1.0].
fn pcm_to_mono_f32(pcm_data: &[u8], is_stereo: bool) -> Vec<f32> {
    let bytes_per_sample = 2; // 16-bit
    let channels = if is_stereo { 2 } else { 1 };
    let frame_size = bytes_per_sample * channels;

    let num_frames = pcm_data.len() / frame_size;
    let mut samples = Vec::with_capacity(num_frames);

    for frame_idx in 0..num_frames {
        let frame_start = frame_idx * frame_size;

        if is_stereo {
            // Read left and right channels, average for mono
            let left = i16::from_le_bytes([pcm_data[frame_start], pcm_data[frame_start + 1]]);
            let right = i16::from_le_bytes([pcm_data[frame_start + 2], pcm_data[frame_start + 3]]);
            let mono = ((left as i32 + right as i32) / 2) as f32 / 32768.0;
            samples.push(mono);
        } else {
            // Mono: just read the sample
            let sample = i16::from_le_bytes([pcm_data[frame_start], pcm_data[frame_start + 1]]);
            samples.push(sample as f32 / 32768.0);
        }
    }

    samples
}

/// Per-column amplitude data for waveform rendering.
#[derive(Clone)]
struct ColumnData {
    min: f32,
    max: f32,
    rms: f32,
}

/// Computes min/max/RMS amplitude for each horizontal column.
fn compute_column_data(samples: &[f32], num_columns: usize) -> Vec<ColumnData> {
    if samples.is_empty() {
        return vec![
            ColumnData {
                min: 0.0,
                max: 0.0,
                rms: 0.0
            };
            num_columns
        ];
    }

    let samples_per_column = samples.len() as f64 / num_columns as f64;
    let mut columns = Vec::with_capacity(num_columns);

    for col in 0..num_columns {
        let start = (col as f64 * samples_per_column).floor() as usize;
        let end = ((col + 1) as f64 * samples_per_column).ceil() as usize;
        let end = end.min(samples.len());

        if start >= end {
            columns.push(ColumnData {
                min: 0.0,
                max: 0.0,
                rms: 0.0,
            });
            continue;
        }

        let mut min = f32::MAX;
        let mut max = f32::MIN;
        let mut sum_sq = 0.0f64;

        for &sample in &samples[start..end] {
            min = min.min(sample);
            max = max.max(sample);
            sum_sq += (sample as f64) * (sample as f64);
        }

        let count = (end - start) as f64;
        let rms = (sum_sq / count).sqrt() as f32;

        columns.push(ColumnData { min, max, rms });
    }

    columns
}

/// Renders the waveform to an RGB buffer.
fn render_waveform(columns: &[ColumnData], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let center_y = h / 2;

    // Initialize with background color
    let mut buffer = vec![0u8; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 3;
            buffer[idx] = BACKGROUND_COLOR[0];
            buffer[idx + 1] = BACKGROUND_COLOR[1];
            buffer[idx + 2] = BACKGROUND_COLOR[2];
        }
    }

    // Draw center line
    for x in 0..w {
        let idx = (center_y * w + x) * 3;
        buffer[idx] = CENTER_LINE_COLOR[0];
        buffer[idx + 1] = CENTER_LINE_COLOR[1];
        buffer[idx + 2] = CENTER_LINE_COLOR[2];
    }

    // Draw waveform columns
    for (x, col) in columns.iter().enumerate() {
        if x >= w {
            break;
        }

        // Map amplitude range [-1, 1] to pixel range [0, height-1]
        // Note: y=0 is top of image, so we flip the coordinates
        let max_y = amplitude_to_y(col.max, h);
        let min_y = amplitude_to_y(col.min, h);

        // Ensure min_y >= max_y (since y increases downward)
        let top_y = max_y.min(min_y);
        let bottom_y = max_y.max(min_y);

        // Draw the waveform column
        for y in top_y..=bottom_y {
            if y < h {
                let idx = (y * w + x) * 3;

                // Use peak color for extreme values, otherwise waveform color
                let is_peak = col.max > 0.95 || col.min < -0.95;
                let color = if is_peak && (y == top_y || y == bottom_y) {
                    PEAK_COLOR
                } else {
                    WAVEFORM_COLOR
                };

                buffer[idx] = color[0];
                buffer[idx + 1] = color[1];
                buffer[idx + 2] = color[2];
            }
        }

        // Draw RMS envelope as slightly brighter inner area
        if col.rms > 0.01 {
            let rms_top = amplitude_to_y(col.rms, h);
            let rms_bottom = amplitude_to_y(-col.rms, h);
            let rms_top = rms_top.min(rms_bottom);
            let rms_bottom = rms_top.max(rms_bottom);

            for y in rms_top..=rms_bottom {
                if y < h && y >= top_y && y <= bottom_y {
                    let idx = (y * w + x) * 3;
                    // Brighten the RMS region slightly
                    buffer[idx] = ((buffer[idx] as u16 * 5 / 4).min(255)) as u8;
                    buffer[idx + 1] = ((buffer[idx + 1] as u16 * 5 / 4).min(255)) as u8;
                    buffer[idx + 2] = ((buffer[idx + 2] as u16 * 5 / 4).min(255)) as u8;
                }
            }
        }
    }

    buffer
}

/// Converts amplitude [-1, 1] to y pixel coordinate [0, height-1].
/// Note: y=0 is top of image, so positive amplitude maps to lower y values.
fn amplitude_to_y(amplitude: f32, height: usize) -> usize {
    let clamped = amplitude.clamp(-1.0, 1.0);
    // Map [-1, 1] to [height-1, 0]
    let normalized = (-clamped + 1.0) / 2.0;
    let y = (normalized * (height - 1) as f32).round() as usize;
    y.min(height - 1)
}

/// Encodes RGB buffer as PNG with deterministic settings.
fn encode_png(rgb_data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut buffer = Vec::new();
    encode_png_to_writer(rgb_data, width, height, &mut buffer)
        .expect("PNG encoding to Vec should not fail");
    buffer
}

/// Encodes RGB buffer as PNG to a writer.
fn encode_png_to_writer<W: Write>(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    writer: W,
) -> Result<(), png::EncodingError> {
    let mut encoder = Encoder::new(writer, width, height);
    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Eight);
    // Use fixed compression settings for determinism
    encoder.set_compression(Compression::Default);
    encoder.set_filter(FilterType::NoFilter);

    let mut png_writer = encoder.write_header()?;
    png_writer.write_image_data(rgb_data)?;

    Ok(())
}

/// Computes the preview path from the primary output path.
///
/// Given a path like "sounds/laser.wav", returns "sounds/laser_preview.waveform.png".
pub fn preview_path_from_primary(primary_path: &str) -> String {
    // Find the last dot to get the extension
    if let Some(dot_pos) = primary_path.rfind('.') {
        let base = &primary_path[..dot_pos];
        format!("{}_preview.waveform.png", base)
    } else {
        format!("{}_preview.waveform.png", primary_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcm_to_mono_f32_mono() {
        // Simple 16-bit PCM: [0, 32767, -32768]
        let pcm: [u8; 6] = [
            0x00, 0x00, // 0
            0xFF, 0x7F, // 32767
            0x00, 0x80, // -32768
        ];

        let samples = pcm_to_mono_f32(&pcm, false);
        assert_eq!(samples.len(), 3);
        assert!((samples[0] - 0.0).abs() < 0.001);
        assert!((samples[1] - 0.99997).abs() < 0.001);
        assert!((samples[2] - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_pcm_to_mono_f32_stereo() {
        // Stereo: L=32767, R=0 -> mono = 16383.5
        let pcm: [u8; 4] = [
            0xFF, 0x7F, // Left: 32767
            0x00, 0x00, // Right: 0
        ];

        let samples = pcm_to_mono_f32(&pcm, true);
        assert_eq!(samples.len(), 1);
        // (32767 + 0) / 2 / 32768 = ~0.5
        assert!((samples[0] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_amplitude_to_y() {
        let height = 256;
        // amplitude 1.0 should map to y=0 (top)
        assert_eq!(amplitude_to_y(1.0, height), 0);
        // amplitude -1.0 should map to y=height-1 (bottom)
        assert_eq!(amplitude_to_y(-1.0, height), height - 1);
        // amplitude 0.0 should map to center
        assert_eq!(amplitude_to_y(0.0, height), height / 2);
    }

    #[test]
    fn test_preview_path_from_primary() {
        assert_eq!(
            preview_path_from_primary("sounds/laser.wav"),
            "sounds/laser_preview.waveform.png"
        );
        assert_eq!(
            preview_path_from_primary("audio/sfx/explosion.wav"),
            "audio/sfx/explosion_preview.waveform.png"
        );
        assert_eq!(
            preview_path_from_primary("noextension"),
            "noextension_preview.waveform.png"
        );
    }

    #[test]
    fn test_waveform_generation_determinism() {
        // Create some test PCM data
        let mut pcm = Vec::new();
        for i in 0..1000 {
            let sample = ((i as f64 * 0.1).sin() * 16000.0) as i16;
            pcm.extend_from_slice(&sample.to_le_bytes());
        }

        let result1 = generate_waveform_png(&pcm, false);
        let result2 = generate_waveform_png(&pcm, false);

        assert_eq!(result1.hash, result2.hash);
        assert_eq!(result1.png_data, result2.png_data);
    }

    #[test]
    fn test_waveform_png_valid_header() {
        // Create minimal PCM data
        let pcm: [u8; 4] = [0x00, 0x00, 0xFF, 0x7F];
        let result = generate_waveform_png(&pcm, false);

        // Check PNG magic bytes
        assert!(result.png_data.len() > 8);
        assert_eq!(
            &result.png_data[0..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
    }

    #[test]
    fn test_empty_pcm_produces_valid_waveform() {
        let pcm: [u8; 0] = [];
        let result = generate_waveform_png(&pcm, false);

        // Should still produce a valid PNG (just showing silence)
        assert!(result.png_data.len() > 8);
        assert_eq!(
            &result.png_data[0..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
    }
}
