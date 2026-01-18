//! Analyze command implementation
//!
//! Analyzes generated assets (audio/texture) and outputs deterministic quality metrics.

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use crate::analysis::{audio, detect_asset_type, texture, AssetAnalysisType};

use super::json_output::{error_codes, AnalyzeOutput, AnalyzeResult, JsonError};

/// Run the analyze command
///
/// # Arguments
/// * `input_path` - Path to the input file to analyze (WAV or PNG)
/// * `spec_path` - Optional path to spec file (generate then analyze)
/// * `output_path` - Optional output file path (default: stdout)
/// * `json_output` - Whether to output machine-readable JSON
///
/// # Returns
/// Exit code: 0 on success, 1 on error
pub fn run(
    input_path: Option<&str>,
    spec_path: Option<&str>,
    output_path: Option<&str>,
    json_output: bool,
) -> Result<ExitCode> {
    // Currently we only support --input mode
    // --spec mode (generate then analyze) will be implemented when needed
    if spec_path.is_some() {
        if json_output {
            let error = JsonError::new(
                error_codes::UNSUPPORTED_FORMAT,
                "--spec mode is not yet implemented. Use --input to analyze an existing file.",
            );
            let output = AnalyzeOutput::failure(vec![error]);
            let json = serde_json::to_string_pretty(&output)?;
            println!("{}", json);
            return Ok(ExitCode::from(1));
        } else {
            anyhow::bail!(
                "--spec mode is not yet implemented. Use --input to analyze an existing file."
            );
        }
    }

    let input = input_path.ok_or_else(|| anyhow::anyhow!("--input is required"))?;

    if json_output {
        run_json(input, output_path)
    } else {
        run_human(input, output_path)
    }
}

/// Run analyze with human-readable (colored) output
fn run_human(input_path: &str, output_path: Option<&str>) -> Result<ExitCode> {
    let path = Path::new(input_path);

    // Detect asset type
    let asset_type = detect_asset_type(path).ok_or_else(|| {
        anyhow::anyhow!(
            "Unsupported file format. Expected .wav or .png, got: {}",
            path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("(none)")
        )
    })?;

    println!("{} {}", "Analyzing:".cyan().bold(), input_path);
    println!("{} {}", "Type:".dimmed(), asset_type);

    // Read input file
    let data = fs::read(path).with_context(|| format!("Failed to read file: {}", input_path))?;

    // Compute input hash
    let input_hash = blake3::hash(&data).to_hex().to_string();
    println!("{} {}", "Hash:".dimmed(), &input_hash[..16]);

    // Analyze based on type
    let metrics = match asset_type {
        AssetAnalysisType::Audio => {
            let audio_metrics = audio::analyze_wav(&data)
                .map_err(|e| anyhow::anyhow!("Audio analysis failed: {}", e))?;
            audio::metrics_to_btree(&audio_metrics)
        }
        AssetAnalysisType::Texture => {
            let texture_metrics = texture::analyze_png(&data)
                .map_err(|e| anyhow::anyhow!("Texture analysis failed: {}", e))?;
            texture::metrics_to_btree(&texture_metrics)
        }
    };

    // Build result
    let result = AnalyzeResult {
        input: input_path.to_string(),
        asset_type: asset_type.as_str().to_string(),
        input_hash,
        metrics,
    };

    // Serialize to JSON
    let output = AnalyzeOutput::success(result);
    let json = serde_json::to_string_pretty(&output)?;

    // Output
    if let Some(out_path) = output_path {
        fs::write(out_path, &json).with_context(|| format!("Failed to write to: {}", out_path))?;
        println!("\n{} {}", "Output written to:".green().bold(), out_path);
    } else {
        println!("\n{}", json);
    }

    Ok(ExitCode::SUCCESS)
}

/// Run analyze with machine-readable JSON output
fn run_json(input_path: &str, output_path: Option<&str>) -> Result<ExitCode> {
    let path = Path::new(input_path);

    // Detect asset type
    let asset_type = match detect_asset_type(path) {
        Some(t) => t,
        None => {
            let error = JsonError::new(
                error_codes::UNSUPPORTED_FORMAT,
                format!(
                    "Unsupported file format. Expected .wav or .png, got: {}",
                    path.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("(none)")
                ),
            )
            .with_file(input_path);
            let output = AnalyzeOutput::failure(vec![error]);
            output_json(&output, output_path)?;
            return Ok(ExitCode::from(1));
        }
    };

    // Read input file
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            let error = JsonError::new(
                error_codes::FILE_READ,
                format!("Failed to read file: {}", e),
            )
            .with_file(input_path);
            let output = AnalyzeOutput::failure(vec![error]);
            output_json(&output, output_path)?;
            return Ok(ExitCode::from(1));
        }
    };

    // Compute input hash
    let input_hash = blake3::hash(&data).to_hex().to_string();

    // Analyze based on type
    let metrics = match asset_type {
        AssetAnalysisType::Audio => match audio::analyze_wav(&data) {
            Ok(m) => audio::metrics_to_btree(&m),
            Err(e) => {
                let error = JsonError::new(
                    error_codes::AUDIO_ANALYSIS,
                    format!("Audio analysis failed: {}", e),
                )
                .with_file(input_path);
                let output = AnalyzeOutput::failure(vec![error]);
                output_json(&output, output_path)?;
                return Ok(ExitCode::from(1));
            }
        },
        AssetAnalysisType::Texture => match texture::analyze_png(&data) {
            Ok(m) => texture::metrics_to_btree(&m),
            Err(e) => {
                let error = JsonError::new(
                    error_codes::TEXTURE_ANALYSIS,
                    format!("Texture analysis failed: {}", e),
                )
                .with_file(input_path);
                let output = AnalyzeOutput::failure(vec![error]);
                output_json(&output, output_path)?;
                return Ok(ExitCode::from(1));
            }
        },
    };

    // Build result
    let result = AnalyzeResult {
        input: input_path.to_string(),
        asset_type: asset_type.as_str().to_string(),
        input_hash,
        metrics,
    };

    let output = AnalyzeOutput::success(result);
    output_json(&output, output_path)?;

    Ok(ExitCode::SUCCESS)
}

/// Output JSON to file or stdout
fn output_json(output: &AnalyzeOutput, output_path: Option<&str>) -> Result<()> {
    let json = serde_json::to_string_pretty(output)?;

    if let Some(out_path) = output_path {
        fs::write(out_path, &json).with_context(|| format!("Failed to write to: {}", out_path))?;
    } else {
        println!("{}", json);
    }

    Ok(())
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
    fn test_analyze_audio_file() {
        let tmp = tempfile::tempdir().unwrap();
        let wav_path = tmp.path().join("test.wav");

        let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let wav_data = create_test_wav(&samples, 44100);
        fs::write(&wav_path, &wav_data).unwrap();

        let code = run(Some(wav_path.to_str().unwrap()), None, None, true).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn test_analyze_texture_file() {
        let tmp = tempfile::tempdir().unwrap();
        let png_path = tmp.path().join("test.png");

        let pixels: Vec<u8> = vec![128, 128, 128, 255];
        let png_data = create_test_png(1, 1, &pixels);
        fs::write(&png_path, &png_data).unwrap();

        let code = run(Some(png_path.to_str().unwrap()), None, None, true).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn test_analyze_unsupported_format() {
        let tmp = tempfile::tempdir().unwrap();
        let txt_path = tmp.path().join("test.txt");
        fs::write(&txt_path, "hello").unwrap();

        let code = run(Some(txt_path.to_str().unwrap()), None, None, true).unwrap();
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn test_analyze_file_not_found() {
        let code = run(Some("/nonexistent/file.wav"), None, None, true).unwrap();
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn test_analyze_with_output_file() {
        let tmp = tempfile::tempdir().unwrap();
        let wav_path = tmp.path().join("test.wav");
        let out_path = tmp.path().join("metrics.json");

        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.01).sin()).collect();
        let wav_data = create_test_wav(&samples, 44100);
        fs::write(&wav_path, &wav_data).unwrap();

        let code = run(
            Some(wav_path.to_str().unwrap()),
            None,
            Some(out_path.to_str().unwrap()),
            true,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        // Check output file exists and is valid JSON
        let content = fs::read_to_string(&out_path).unwrap();
        let parsed: AnalyzeOutput = serde_json::from_str(&content).unwrap();
        assert!(parsed.success);
    }

    #[test]
    fn test_deterministic_output() {
        let tmp = tempfile::tempdir().unwrap();
        let wav_path = tmp.path().join("test.wav");
        let out1_path = tmp.path().join("metrics1.json");
        let out2_path = tmp.path().join("metrics2.json");

        let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let wav_data = create_test_wav(&samples, 44100);
        fs::write(&wav_path, &wav_data).unwrap();

        // Run twice
        run(
            Some(wav_path.to_str().unwrap()),
            None,
            Some(out1_path.to_str().unwrap()),
            true,
        )
        .unwrap();

        run(
            Some(wav_path.to_str().unwrap()),
            None,
            Some(out2_path.to_str().unwrap()),
            true,
        )
        .unwrap();

        // Compare outputs
        let content1 = fs::read_to_string(&out1_path).unwrap();
        let content2 = fs::read_to_string(&out2_path).unwrap();
        assert_eq!(content1, content2);
    }
}
