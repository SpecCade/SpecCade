//! Compare command implementation
//!
//! Compares two asset files (audio or texture) and outputs perceptual difference metrics.
//! Supports SSIM, DeltaE, histogram deltas for images, and spectral/loudness metrics for audio.

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use crate::analysis::{audio, detect_asset_type, perceptual, texture, AssetAnalysisType};

use super::json_output::{
    error_codes, AudioCompareMetrics, CompareMetrics, CompareOutput, CompareResult,
    HistogramDiffMetrics, JsonError, TextureCompareMetrics,
};

/// Run the compare command
///
/// # Arguments
/// * `path_a` - Path to the first file (reference)
/// * `path_b` - Path to the second file (comparison target)
/// * `json_output` - Whether to output machine-readable JSON
///
/// # Returns
/// Exit code: 0 on success, 1 on error
pub fn run(path_a: &str, path_b: &str, json_output: bool) -> Result<ExitCode> {
    if json_output {
        run_json(path_a, path_b)
    } else {
        run_human(path_a, path_b)
    }
}

/// Run compare with human-readable (colored) output
fn run_human(path_a: &str, path_b: &str) -> Result<ExitCode> {
    let file_a = Path::new(path_a);
    let file_b = Path::new(path_b);

    // Detect asset types
    let type_a = detect_asset_type(file_a).ok_or_else(|| {
        anyhow::anyhow!(
            "Unsupported file format for A: {}",
            file_a
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("(none)")
        )
    })?;

    let type_b = detect_asset_type(file_b).ok_or_else(|| {
        anyhow::anyhow!(
            "Unsupported file format for B: {}",
            file_b
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("(none)")
        )
    })?;

    if type_a != type_b {
        anyhow::bail!(
            "Cannot compare different asset types: {} vs {}",
            type_a,
            type_b
        );
    }

    println!("{}", "Comparing files:".cyan().bold());
    println!("  {} {}", "A:".dimmed(), path_a);
    println!("  {} {}", "B:".dimmed(), path_b);
    println!("{} {}", "Type:".dimmed(), type_a);

    // Read files
    let data_a = fs::read(file_a).with_context(|| format!("Failed to read file A: {}", path_a))?;
    let data_b = fs::read(file_b).with_context(|| format!("Failed to read file B: {}", path_b))?;

    // Compute hashes
    let hash_a = blake3::hash(&data_a).to_hex().to_string();
    let hash_b = blake3::hash(&data_b).to_hex().to_string();

    println!("{} {}", "Hash A:".dimmed(), &hash_a[..16]);
    println!("{} {}", "Hash B:".dimmed(), &hash_b[..16]);

    let identical = data_a == data_b;
    if identical {
        println!("\n{}", "Files are byte-identical!".green().bold());
    }

    // Perform type-specific comparison
    println!("\n{}", "Comparison Metrics:".cyan().bold());

    match type_a {
        AssetAnalysisType::Texture => {
            let result = compare_textures(&data_a, &data_b)?;
            print_texture_metrics(&result);
        }
        AssetAnalysisType::Audio => {
            let result = compare_audio(&data_a, &data_b)?;
            print_audio_metrics(&result);
        }
    }

    Ok(ExitCode::SUCCESS)
}

/// Run compare with machine-readable JSON output
fn run_json(path_a: &str, path_b: &str) -> Result<ExitCode> {
    let file_a = Path::new(path_a);
    let file_b = Path::new(path_b);

    // Detect asset types
    let type_a = match detect_asset_type(file_a) {
        Some(t) => t,
        None => {
            let error = JsonError::new(
                error_codes::UNSUPPORTED_FORMAT,
                format!(
                    "Unsupported file format for A: {}",
                    file_a
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("(none)")
                ),
            )
            .with_file(path_a);
            let output = CompareOutput::failure(vec![error]);
            println!("{}", serde_json::to_string_pretty(&output)?);
            return Ok(ExitCode::from(1));
        }
    };

    let type_b = match detect_asset_type(file_b) {
        Some(t) => t,
        None => {
            let error = JsonError::new(
                error_codes::UNSUPPORTED_FORMAT,
                format!(
                    "Unsupported file format for B: {}",
                    file_b
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("(none)")
                ),
            )
            .with_file(path_b);
            let output = CompareOutput::failure(vec![error]);
            println!("{}", serde_json::to_string_pretty(&output)?);
            return Ok(ExitCode::from(1));
        }
    };

    if type_a != type_b {
        let error = JsonError::new(
            error_codes::UNSUPPORTED_FORMAT,
            format!(
                "Cannot compare different asset types: {} vs {}",
                type_a, type_b
            ),
        );
        let output = CompareOutput::failure(vec![error]);
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(ExitCode::from(1));
    }

    // Read files
    let data_a = match fs::read(file_a) {
        Ok(d) => d,
        Err(e) => {
            let error = JsonError::new(
                error_codes::FILE_READ,
                format!("Failed to read file A: {}", e),
            )
            .with_file(path_a);
            let output = CompareOutput::failure(vec![error]);
            println!("{}", serde_json::to_string_pretty(&output)?);
            return Ok(ExitCode::from(1));
        }
    };

    let data_b = match fs::read(file_b) {
        Ok(d) => d,
        Err(e) => {
            let error = JsonError::new(
                error_codes::FILE_READ,
                format!("Failed to read file B: {}", e),
            )
            .with_file(path_b);
            let output = CompareOutput::failure(vec![error]);
            println!("{}", serde_json::to_string_pretty(&output)?);
            return Ok(ExitCode::from(1));
        }
    };

    // Compute hashes
    let hash_a = blake3::hash(&data_a).to_hex().to_string();
    let hash_b = blake3::hash(&data_b).to_hex().to_string();
    let identical = data_a == data_b;

    // Perform comparison
    let metrics = match type_a {
        AssetAnalysisType::Texture => match compare_textures(&data_a, &data_b) {
            Ok(m) => CompareMetrics::Texture(m),
            Err(e) => {
                let error = JsonError::new(
                    error_codes::TEXTURE_ANALYSIS,
                    format!("Texture comparison failed: {}", e),
                );
                let output = CompareOutput::failure(vec![error]);
                println!("{}", serde_json::to_string_pretty(&output)?);
                return Ok(ExitCode::from(1));
            }
        },
        AssetAnalysisType::Audio => match compare_audio(&data_a, &data_b) {
            Ok(m) => CompareMetrics::Audio(m),
            Err(e) => {
                let error = JsonError::new(
                    error_codes::AUDIO_ANALYSIS,
                    format!("Audio comparison failed: {}", e),
                );
                let output = CompareOutput::failure(vec![error]);
                println!("{}", serde_json::to_string_pretty(&output)?);
                return Ok(ExitCode::from(1));
            }
        },
    };

    let result = CompareResult {
        path_a: path_a.to_string(),
        path_b: path_b.to_string(),
        asset_type: type_a.as_str().to_string(),
        hash_a,
        hash_b,
        identical,
        metrics,
    };

    let output = CompareOutput::success(result);
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(ExitCode::SUCCESS)
}

/// Compare two texture files and return metrics.
fn compare_textures(data_a: &[u8], data_b: &[u8]) -> Result<TextureCompareMetrics> {
    // Analyze both textures
    let metrics_a = texture::analyze_png(data_a)
        .map_err(|e| anyhow::anyhow!("Failed to analyze texture A: {}", e))?;
    let metrics_b = texture::analyze_png(data_b)
        .map_err(|e| anyhow::anyhow!("Failed to analyze texture B: {}", e))?;

    // Check dimensions match
    if metrics_a.format.width != metrics_b.format.width
        || metrics_a.format.height != metrics_b.format.height
    {
        anyhow::bail!(
            "Texture dimensions do not match: {}x{} vs {}x{}",
            metrics_a.format.width,
            metrics_a.format.height,
            metrics_b.format.width,
            metrics_b.format.height
        );
    }

    if metrics_a.format.channels != metrics_b.format.channels {
        anyhow::bail!(
            "Texture channel counts do not match: {} vs {}",
            metrics_a.format.channels,
            metrics_b.format.channels
        );
    }

    // Extract pixels
    let (pixels_a, width, height, channels) = texture::extract_png_pixels(data_a)
        .map_err(|e| anyhow::anyhow!("Failed to extract pixels from A: {}", e))?;
    let (pixels_b, _, _, _) = texture::extract_png_pixels(data_b)
        .map_err(|e| anyhow::anyhow!("Failed to extract pixels from B: {}", e))?;

    // Compute perceptual metrics
    let result = perceptual::compare_images(
        &pixels_a, &pixels_b, width, height, channels, &metrics_a, &metrics_b,
    );

    Ok(TextureCompareMetrics {
        ssim: result.ssim,
        delta_e_mean: result.delta_e_mean,
        delta_e_max: result.delta_e_max,
        histogram_diff: HistogramDiffMetrics {
            red: result.histogram_diff.red,
            green: result.histogram_diff.green,
            blue: result.histogram_diff.blue,
            alpha: result.histogram_diff.alpha,
        },
    })
}

/// Compare two audio files and return metrics.
fn compare_audio(data_a: &[u8], data_b: &[u8]) -> Result<AudioCompareMetrics> {
    // Analyze both audio files
    let metrics_a = audio::analyze_wav(data_a)
        .map_err(|e| anyhow::anyhow!("Failed to analyze audio A: {}", e))?;
    let metrics_b = audio::analyze_wav(data_b)
        .map_err(|e| anyhow::anyhow!("Failed to analyze audio B: {}", e))?;

    // Extract samples
    let (samples_a, sample_rate_a) = audio::extract_wav_samples(data_a)
        .map_err(|e| anyhow::anyhow!("Failed to extract samples from A: {}", e))?;
    let (samples_b, sample_rate_b) = audio::extract_wav_samples(data_b)
        .map_err(|e| anyhow::anyhow!("Failed to extract samples from B: {}", e))?;

    if sample_rate_a != sample_rate_b {
        anyhow::bail!(
            "Sample rates do not match: {} vs {}",
            sample_rate_a,
            sample_rate_b
        );
    }

    // Compute perceptual metrics
    let result = perceptual::compare_audio(
        &samples_a,
        &samples_b,
        sample_rate_a,
        &metrics_a,
        &metrics_b,
    );

    Ok(AudioCompareMetrics {
        spectral_correlation: result.spectral_correlation,
        rms_delta_db: result.rms_delta_db,
        peak_delta_db: result.peak_delta_db,
        loudness_delta_percent: result.loudness_delta_percent,
    })
}

/// Print texture comparison metrics in human-readable format.
fn print_texture_metrics(metrics: &TextureCompareMetrics) {
    println!("  {} {:.6}", "SSIM:".cyan(), metrics.ssim);

    // Color code SSIM
    let ssim_status = if metrics.ssim > 0.99 {
        "excellent".green()
    } else if metrics.ssim > 0.95 {
        "good".yellow()
    } else if metrics.ssim > 0.90 {
        "fair".yellow()
    } else {
        "poor".red()
    };
    println!("         ({})", ssim_status);

    println!(
        "  {} mean={:.2}, max={:.2}",
        "DeltaE:".cyan(),
        metrics.delta_e_mean,
        metrics.delta_e_max
    );

    // Color code DeltaE
    let delta_e_status = if metrics.delta_e_mean < 1.0 {
        "imperceptible".green()
    } else if metrics.delta_e_mean < 3.0 {
        "barely perceptible".yellow()
    } else if metrics.delta_e_mean < 6.0 {
        "noticeable".yellow()
    } else {
        "significant".red()
    };
    println!("          ({})", delta_e_status);

    println!("  {}", "Histogram Diff:".cyan());
    println!("    {} {:.2}", "red:".dimmed(), metrics.histogram_diff.red);
    if let Some(g) = metrics.histogram_diff.green {
        println!("    {} {:.2}", "green:".dimmed(), g);
    }
    if let Some(b) = metrics.histogram_diff.blue {
        println!("    {} {:.2}", "blue:".dimmed(), b);
    }
    if let Some(a) = metrics.histogram_diff.alpha {
        println!("    {} {:.2}", "alpha:".dimmed(), a);
    }
}

/// Print audio comparison metrics in human-readable format.
fn print_audio_metrics(metrics: &AudioCompareMetrics) {
    println!(
        "  {} {:.6}",
        "Spectral Correlation:".cyan(),
        metrics.spectral_correlation
    );

    // Color code correlation
    let corr_status = if metrics.spectral_correlation > 0.95 {
        "very similar".green()
    } else if metrics.spectral_correlation > 0.80 {
        "similar".yellow()
    } else if metrics.spectral_correlation > 0.50 {
        "somewhat different".yellow()
    } else {
        "very different".red()
    };
    println!("                       ({})", corr_status);

    println!("  {} {:.2} dB", "RMS Delta:".cyan(), metrics.rms_delta_db);
    println!("  {} {:.2} dB", "Peak Delta:".cyan(), metrics.peak_delta_db);
    println!(
        "  {} {:.2}%",
        "Loudness Delta:".cyan(),
        metrics.loudness_delta_percent
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
        let channels: u16 = 1;
        let bits_per_sample: u16 = 16;
        let byte_rate = sample_rate * channels as u32 * 2;
        let block_align = channels * 2;
        let data_size = samples.len() * 2;
        let file_size = 36 + data_size;

        let mut wav = Vec::new();

        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(file_size as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(data_size as u32).to_le_bytes());

        for &s in samples {
            let sample_i16 = (s * 32767.0) as i16;
            wav.extend_from_slice(&sample_i16.to_le_bytes());
        }

        wav
    }

    fn create_test_png(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
        let mut png_data = Vec::new();
        let mut encoder = png::Encoder::new(&mut png_data, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(pixels).unwrap();
        drop(writer);
        png_data
    }

    #[test]
    fn test_compare_identical_textures() {
        let pixels: Vec<u8> = (0..16 * 16 * 4).map(|i| (i % 256) as u8).collect();
        let png = create_test_png(16, 16, &pixels);

        let metrics = compare_textures(&png, &png).unwrap();

        assert!((metrics.ssim - 1.0).abs() < 0.001);
        assert_eq!(metrics.delta_e_mean, 0.0);
        assert_eq!(metrics.delta_e_max, 0.0);
        assert_eq!(metrics.histogram_diff.red, 0.0);
    }

    #[test]
    fn test_compare_different_textures() {
        let pixels_a: Vec<u8> = vec![0u8; 16 * 16 * 4];
        let pixels_b: Vec<u8> = vec![255u8; 16 * 16 * 4];
        let png_a = create_test_png(16, 16, &pixels_a);
        let png_b = create_test_png(16, 16, &pixels_b);

        let metrics = compare_textures(&png_a, &png_b).unwrap();

        assert!(metrics.ssim < 0.1);
        assert!(metrics.delta_e_mean > 50.0);
    }

    #[test]
    fn test_compare_texture_dimension_mismatch() {
        let pixels_a: Vec<u8> = vec![128u8; 8 * 8 * 4];
        let pixels_b: Vec<u8> = vec![128u8; 16 * 16 * 4];
        let png_a = create_test_png(8, 8, &pixels_a);
        let png_b = create_test_png(16, 16, &pixels_b);

        let result = compare_textures(&png_a, &png_b);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("dimensions"));
    }

    #[test]
    fn test_compare_identical_audio() {
        let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin()).collect();
        let wav = create_test_wav(&samples, 44100);

        let metrics = compare_audio(&wav, &wav).unwrap();

        assert!((metrics.spectral_correlation - 1.0).abs() < 0.001);
        assert_eq!(metrics.rms_delta_db, 0.0);
        assert_eq!(metrics.peak_delta_db, 0.0);
    }

    #[test]
    fn test_compare_different_audio() {
        let samples_a: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let samples_b: Vec<f32> = (0..4410).map(|i| (i as f32 * 1.0).sin() * 0.8).collect();
        let wav_a = create_test_wav(&samples_a, 44100);
        let wav_b = create_test_wav(&samples_b, 44100);

        let metrics = compare_audio(&wav_a, &wav_b).unwrap();

        // Different amplitude and frequency should show differences
        assert!(metrics.rms_delta_db.abs() > 0.5);
    }

    #[test]
    fn test_compare_command_json_success() {
        let tmp = tempfile::tempdir().unwrap();
        let path_a = tmp.path().join("a.wav");
        let path_b = tmp.path().join("b.wav");

        let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin()).collect();
        let wav = create_test_wav(&samples, 44100);

        fs::write(&path_a, &wav).unwrap();
        fs::write(&path_b, &wav).unwrap();

        let code = run(path_a.to_str().unwrap(), path_b.to_str().unwrap(), true).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn test_compare_command_type_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let wav_path = tmp.path().join("test.wav");
        let png_path = tmp.path().join("test.png");

        let samples: Vec<f32> = vec![0.0; 1000];
        let wav = create_test_wav(&samples, 44100);
        let pixels: Vec<u8> = vec![128u8; 4 * 4 * 4];
        let png = create_test_png(4, 4, &pixels);

        fs::write(&wav_path, &wav).unwrap();
        fs::write(&png_path, &png).unwrap();

        let result = run(
            wav_path.to_str().unwrap(),
            png_path.to_str().unwrap(),
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_ssim_known_values() {
        // Test SSIM with known identical images
        let pixels: Vec<u8> = (0..32 * 32 * 4).map(|i| ((i * 7) % 256) as u8).collect();
        let png = create_test_png(32, 32, &pixels);

        let metrics = compare_textures(&png, &png).unwrap();
        // SSIM of identical images should be exactly 1.0
        assert!(
            (metrics.ssim - 1.0).abs() < 0.0001,
            "SSIM of identical images should be 1.0, got {}",
            metrics.ssim
        );
    }

    #[test]
    fn test_compare_delta_e_known_values() {
        // Black vs white should have DeltaE close to 100
        let black_pixel: Vec<u8> = vec![0, 0, 0, 255];
        let white_pixel: Vec<u8> = vec![255, 255, 255, 255];
        let black_png = create_test_png(1, 1, &black_pixel);
        let white_png = create_test_png(1, 1, &white_pixel);

        let metrics = compare_textures(&black_png, &white_png).unwrap();

        // DeltaE between black and white should be approximately 100
        assert!(
            metrics.delta_e_mean > 95.0 && metrics.delta_e_mean < 105.0,
            "DeltaE between black and white should be ~100, got {}",
            metrics.delta_e_mean
        );
    }

    #[test]
    fn test_compare_file_not_found() {
        let code = run("/nonexistent/file.wav", "/other/file.wav", true).unwrap();
        assert_eq!(code, ExitCode::from(1));
    }
}
