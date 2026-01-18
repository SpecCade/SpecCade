//! Analyze command implementation
//!
//! Analyzes generated assets (audio/texture) and outputs deterministic quality metrics.
//! Supports both single-file and batch mode (recursive directory scanning).

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use walkdir::WalkDir;

use crate::analysis::{audio, detect_asset_type, embeddings, texture, AssetAnalysisType};

use super::analyze_csv::format_csv;
use super::json_output::{
    error_codes, AnalyzeOutput, AnalyzeResult, BatchAnalyzeItem, BatchAnalyzeOutput, JsonError,
};

/// Output format for batch mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON array of results
    Json,
    /// One JSON object per line
    Jsonl,
    /// Flattened CSV with headers
    Csv,
}

impl OutputFormat {
    /// Parse output format from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "jsonl" => Some(Self::Jsonl),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }
}

/// Run the analyze command
///
/// # Arguments
/// * `input_path` - Path to the input file to analyze (WAV or PNG)
/// * `spec_path` - Optional path to spec file (generate then analyze)
/// * `input_dir` - Optional directory to recursively scan for assets (batch mode)
/// * `output_path` - Optional output file path (default: stdout)
/// * `json_output` - Whether to output machine-readable JSON
/// * `output_format` - Output format for batch mode (json, jsonl, csv)
/// * `include_embeddings` - Whether to include feature embeddings
///
/// # Returns
/// Exit code: 0 on success, 1 on error
pub fn run(
    input_path: Option<&str>,
    spec_path: Option<&str>,
    input_dir: Option<&str>,
    output_path: Option<&str>,
    json_output: bool,
    output_format: &str,
    include_embeddings: bool,
) -> Result<ExitCode> {
    // --spec mode is not supported
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

    // Batch mode: --input-dir takes precedence
    if let Some(dir) = input_dir {
        let format = OutputFormat::parse(output_format)
            .ok_or_else(|| anyhow::anyhow!("Invalid output format: {}", output_format))?;
        return run_batch(dir, output_path, format, include_embeddings);
    }

    // Single file mode
    let input =
        input_path.ok_or_else(|| anyhow::anyhow!("Either --input or --input-dir is required"))?;

    if json_output {
        run_json(input, output_path, include_embeddings)
    } else {
        run_human(input, output_path, include_embeddings)
    }
}

/// Run analyze with human-readable (colored) output
fn run_human(
    input_path: &str,
    output_path: Option<&str>,
    include_embeddings: bool,
) -> Result<ExitCode> {
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
    let (metrics, embedding) = match asset_type {
        AssetAnalysisType::Audio => {
            let audio_metrics = audio::analyze_wav(&data)
                .map_err(|e| anyhow::anyhow!("Audio analysis failed: {}", e))?;
            let emb = if include_embeddings {
                let (samples, sample_rate) = audio::extract_wav_samples(&data)
                    .map_err(|e| anyhow::anyhow!("Audio extraction failed: {}", e))?;
                Some(embeddings::compute_audio_embedding(&samples, sample_rate))
            } else {
                None
            };
            (audio::metrics_to_btree(&audio_metrics), emb)
        }
        AssetAnalysisType::Texture => {
            let texture_metrics = texture::analyze_png(&data)
                .map_err(|e| anyhow::anyhow!("Texture analysis failed: {}", e))?;
            let emb = if include_embeddings {
                let (pixels, width, height, channels) = texture::extract_png_pixels(&data)
                    .map_err(|e| anyhow::anyhow!("Texture extraction failed: {}", e))?;
                Some(embeddings::compute_texture_embedding(
                    &pixels, width, height, channels,
                ))
            } else {
                None
            };
            (texture::metrics_to_btree(&texture_metrics), emb)
        }
    };

    if include_embeddings {
        if let Some(ref emb) = embedding {
            println!("{} {} dimensions", "Embedding:".dimmed(), emb.len());
        }
    }

    // Build result
    let result = AnalyzeResult {
        input: input_path.to_string(),
        asset_type: asset_type.as_str().to_string(),
        input_hash,
        metrics,
        embedding,
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
fn run_json(
    input_path: &str,
    output_path: Option<&str>,
    include_embeddings: bool,
) -> Result<ExitCode> {
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
    let (metrics, embedding) = match asset_type {
        AssetAnalysisType::Audio => match audio::analyze_wav(&data) {
            Ok(m) => {
                let emb = if include_embeddings {
                    match audio::extract_wav_samples(&data) {
                        Ok((samples, sample_rate)) => {
                            Some(embeddings::compute_audio_embedding(&samples, sample_rate))
                        }
                        Err(e) => {
                            let error = JsonError::new(
                                error_codes::AUDIO_ANALYSIS,
                                format!("Audio extraction for embedding failed: {}", e),
                            )
                            .with_file(input_path);
                            let output = AnalyzeOutput::failure(vec![error]);
                            output_json(&output, output_path)?;
                            return Ok(ExitCode::from(1));
                        }
                    }
                } else {
                    None
                };
                (audio::metrics_to_btree(&m), emb)
            }
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
            Ok(m) => {
                let emb = if include_embeddings {
                    match texture::extract_png_pixels(&data) {
                        Ok((pixels, width, height, channels)) => Some(
                            embeddings::compute_texture_embedding(&pixels, width, height, channels),
                        ),
                        Err(e) => {
                            let error = JsonError::new(
                                error_codes::TEXTURE_ANALYSIS,
                                format!("Texture extraction for embedding failed: {}", e),
                            )
                            .with_file(input_path);
                            let output = AnalyzeOutput::failure(vec![error]);
                            output_json(&output, output_path)?;
                            return Ok(ExitCode::from(1));
                        }
                    }
                } else {
                    None
                };
                (texture::metrics_to_btree(&m), emb)
            }
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
        embedding,
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

/// Run batch analysis on a directory.
fn run_batch(
    dir_path: &str,
    output_path: Option<&str>,
    format: OutputFormat,
    include_embeddings: bool,
) -> Result<ExitCode> {
    let dir = Path::new(dir_path);
    if !dir.is_dir() {
        anyhow::bail!("--input-dir path is not a directory: {}", dir_path);
    }

    // Discover all .wav and .png files recursively
    let files: Vec<_> = WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| {
                    let ext_lower = ext.to_lowercase();
                    ext_lower == "wav" || ext_lower == "png"
                })
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    // Sort for deterministic output
    let mut files = files;
    files.sort();

    // Analyze each file
    let results: Vec<BatchAnalyzeItem> = files
        .iter()
        .map(|path| analyze_single_file(path, include_embeddings))
        .collect();

    // Output based on format
    let output_content = match format {
        OutputFormat::Json => format_json(&results)?,
        OutputFormat::Jsonl => format_jsonl(&results)?,
        OutputFormat::Csv => format_csv(&results, include_embeddings)?,
    };

    // Write to file or stdout
    if let Some(out_path) = output_path {
        fs::write(out_path, &output_content)
            .with_context(|| format!("Failed to write to: {}", out_path))?;
    } else {
        print!("{}", output_content);
    }

    Ok(ExitCode::SUCCESS)
}

/// Analyze a single file and return a BatchAnalyzeItem.
fn analyze_single_file(path: &Path, include_embeddings: bool) -> BatchAnalyzeItem {
    let path_str = path.display().to_string();

    // Detect asset type
    let asset_type = match detect_asset_type(path) {
        Some(t) => t,
        None => {
            let error = JsonError::new(
                error_codes::UNSUPPORTED_FORMAT,
                format!(
                    "Unsupported file format: {}",
                    path.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("(none)")
                ),
            );
            return BatchAnalyzeItem::failure(path_str, error);
        }
    };

    // Read file
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            let error = JsonError::new(error_codes::FILE_READ, format!("Failed to read: {}", e));
            return BatchAnalyzeItem::failure(path_str, error);
        }
    };

    // Compute hash
    let input_hash = blake3::hash(&data).to_hex().to_string();

    // Analyze
    let (metrics, embedding) = match asset_type {
        AssetAnalysisType::Audio => match audio::analyze_wav(&data) {
            Ok(m) => {
                let emb = if include_embeddings {
                    match audio::extract_wav_samples(&data) {
                        Ok((samples, sample_rate)) => {
                            Some(embeddings::compute_audio_embedding(&samples, sample_rate))
                        }
                        Err(e) => {
                            let error = JsonError::new(
                                error_codes::AUDIO_ANALYSIS,
                                format!("Embedding extraction failed: {}", e),
                            );
                            return BatchAnalyzeItem::failure(path_str, error);
                        }
                    }
                } else {
                    None
                };
                (audio::metrics_to_btree(&m), emb)
            }
            Err(e) => {
                let error = JsonError::new(
                    error_codes::AUDIO_ANALYSIS,
                    format!("Analysis failed: {}", e),
                );
                return BatchAnalyzeItem::failure(path_str, error);
            }
        },
        AssetAnalysisType::Texture => match texture::analyze_png(&data) {
            Ok(m) => {
                let emb = if include_embeddings {
                    match texture::extract_png_pixels(&data) {
                        Ok((pixels, width, height, channels)) => Some(
                            embeddings::compute_texture_embedding(&pixels, width, height, channels),
                        ),
                        Err(e) => {
                            let error = JsonError::new(
                                error_codes::TEXTURE_ANALYSIS,
                                format!("Embedding extraction failed: {}", e),
                            );
                            return BatchAnalyzeItem::failure(path_str, error);
                        }
                    }
                } else {
                    None
                };
                (texture::metrics_to_btree(&m), emb)
            }
            Err(e) => {
                let error = JsonError::new(
                    error_codes::TEXTURE_ANALYSIS,
                    format!("Analysis failed: {}", e),
                );
                return BatchAnalyzeItem::failure(path_str, error);
            }
        },
    };

    let result = AnalyzeResult {
        input: path_str,
        asset_type: asset_type.as_str().to_string(),
        input_hash,
        metrics,
        embedding,
    };

    BatchAnalyzeItem::success(result)
}

/// Format results as JSON (array with summary).
fn format_json(results: &[BatchAnalyzeItem]) -> Result<String> {
    let output = BatchAnalyzeOutput::new(results.to_vec());
    Ok(serde_json::to_string_pretty(&output)?)
}

/// Format results as JSONL (one object per line).
fn format_jsonl(results: &[BatchAnalyzeItem]) -> Result<String> {
    let mut output = String::new();
    for item in results {
        let line = serde_json::to_string(item)?;
        output.push_str(&line);
        output.push('\n');
    }
    Ok(output)
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

        let code = run(
            Some(wav_path.to_str().unwrap()),
            None,
            None,
            None,
            true,
            "json",
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn test_analyze_texture_file() {
        let tmp = tempfile::tempdir().unwrap();
        let png_path = tmp.path().join("test.png");

        let pixels: Vec<u8> = vec![128, 128, 128, 255];
        let png_data = create_test_png(1, 1, &pixels);
        fs::write(&png_path, &png_data).unwrap();

        let code = run(
            Some(png_path.to_str().unwrap()),
            None,
            None,
            None,
            true,
            "json",
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn test_analyze_unsupported_format() {
        let tmp = tempfile::tempdir().unwrap();
        let txt_path = tmp.path().join("test.txt");
        fs::write(&txt_path, "hello").unwrap();

        let code = run(
            Some(txt_path.to_str().unwrap()),
            None,
            None,
            None,
            true,
            "json",
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn test_analyze_file_not_found() {
        let code = run(
            Some("/nonexistent/file.wav"),
            None,
            None,
            None,
            true,
            "json",
            false,
        )
        .unwrap();
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
            None,
            Some(out_path.to_str().unwrap()),
            true,
            "json",
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        // Check output file exists and is valid JSON
        let content = fs::read_to_string(&out_path).unwrap();
        let parsed: AnalyzeOutput = serde_json::from_str(&content).unwrap();
        assert!(parsed.success);
        // No embedding when flag is false
        assert!(parsed.result.as_ref().unwrap().embedding.is_none());
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
            None,
            Some(out1_path.to_str().unwrap()),
            true,
            "json",
            false,
        )
        .unwrap();

        run(
            Some(wav_path.to_str().unwrap()),
            None,
            None,
            Some(out2_path.to_str().unwrap()),
            true,
            "json",
            false,
        )
        .unwrap();

        // Compare outputs
        let content1 = fs::read_to_string(&out1_path).unwrap();
        let content2 = fs::read_to_string(&out2_path).unwrap();
        assert_eq!(content1, content2);
    }

    #[test]
    fn test_analyze_audio_with_embeddings() {
        let tmp = tempfile::tempdir().unwrap();
        let wav_path = tmp.path().join("test.wav");
        let out_path = tmp.path().join("metrics.json");

        let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let wav_data = create_test_wav(&samples, 44100);
        fs::write(&wav_path, &wav_data).unwrap();

        let code = run(
            Some(wav_path.to_str().unwrap()),
            None,
            None,
            Some(out_path.to_str().unwrap()),
            true,
            "json",
            true, // include embeddings
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        let content = fs::read_to_string(&out_path).unwrap();
        let parsed: AnalyzeOutput = serde_json::from_str(&content).unwrap();
        assert!(parsed.success);

        // Check embedding is present and has correct dimension
        let embedding = parsed.result.as_ref().unwrap().embedding.as_ref().unwrap();
        assert_eq!(embedding.len(), embeddings::AUDIO_EMBEDDING_DIM);

        // Check all values are in valid range [0, 1]
        for &v in embedding {
            assert!(
                (0.0..=1.0).contains(&v),
                "Embedding value {} out of range",
                v
            );
        }
    }

    #[test]
    fn test_analyze_texture_with_embeddings() {
        let tmp = tempfile::tempdir().unwrap();
        let png_path = tmp.path().join("test.png");
        let out_path = tmp.path().join("metrics.json");

        // Create a 4x4 RGBA image
        let pixels: Vec<u8> = (0..4 * 4 * 4).map(|i| (i % 256) as u8).collect();
        let png_data = create_test_png(4, 4, &pixels);
        fs::write(&png_path, &png_data).unwrap();

        let code = run(
            Some(png_path.to_str().unwrap()),
            None,
            None,
            Some(out_path.to_str().unwrap()),
            true,
            "json",
            true, // include embeddings
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        let content = fs::read_to_string(&out_path).unwrap();
        let parsed: AnalyzeOutput = serde_json::from_str(&content).unwrap();
        assert!(parsed.success);

        // Check embedding is present and has correct dimension
        let embedding = parsed.result.as_ref().unwrap().embedding.as_ref().unwrap();
        assert_eq!(embedding.len(), embeddings::TEXTURE_EMBEDDING_DIM);

        // Check all values are in valid range [0, 1]
        for &v in embedding {
            assert!(
                (0.0..=1.0).contains(&v),
                "Embedding value {} out of range",
                v
            );
        }
    }

    #[test]
    fn test_embeddings_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        let wav_path = tmp.path().join("test.wav");
        let out1_path = tmp.path().join("metrics1.json");
        let out2_path = tmp.path().join("metrics2.json");

        let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let wav_data = create_test_wav(&samples, 44100);
        fs::write(&wav_path, &wav_data).unwrap();

        // Run twice with embeddings
        run(
            Some(wav_path.to_str().unwrap()),
            None,
            None,
            Some(out1_path.to_str().unwrap()),
            true,
            "json",
            true,
        )
        .unwrap();

        run(
            Some(wav_path.to_str().unwrap()),
            None,
            None,
            Some(out2_path.to_str().unwrap()),
            true,
            "json",
            true,
        )
        .unwrap();

        // Compare outputs - should be identical
        let content1 = fs::read_to_string(&out1_path).unwrap();
        let content2 = fs::read_to_string(&out2_path).unwrap();
        assert_eq!(content1, content2);
    }

    // Batch mode tests

    #[test]
    fn test_batch_analyze_json_format() {
        let tmp = tempfile::tempdir().unwrap();
        let subdir = tmp.path().join("assets");
        fs::create_dir(&subdir).unwrap();

        // Create test files
        let wav_path = subdir.join("test.wav");
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.01).sin()).collect();
        let wav_data = create_test_wav(&samples, 44100);
        fs::write(&wav_path, &wav_data).unwrap();

        let png_path = subdir.join("test.png");
        let pixels: Vec<u8> = vec![128, 128, 128, 255];
        let png_data = create_test_png(1, 1, &pixels);
        fs::write(&png_path, &png_data).unwrap();

        let out_path = tmp.path().join("results.json");

        let code = run(
            None,
            None,
            Some(subdir.to_str().unwrap()),
            Some(out_path.to_str().unwrap()),
            false,
            "json",
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        // Check output
        let content = fs::read_to_string(&out_path).unwrap();
        let parsed: BatchAnalyzeOutput = serde_json::from_str(&content).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.summary.total, 2);
        assert_eq!(parsed.summary.succeeded, 2);
        assert_eq!(parsed.summary.failed, 0);
        assert_eq!(parsed.results.len(), 2);
    }

    #[test]
    fn test_batch_analyze_jsonl_format() {
        let tmp = tempfile::tempdir().unwrap();
        let subdir = tmp.path().join("assets");
        fs::create_dir(&subdir).unwrap();

        // Create test file
        let wav_path = subdir.join("test.wav");
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.01).sin()).collect();
        let wav_data = create_test_wav(&samples, 44100);
        fs::write(&wav_path, &wav_data).unwrap();

        let out_path = tmp.path().join("results.jsonl");

        let code = run(
            None,
            None,
            Some(subdir.to_str().unwrap()),
            Some(out_path.to_str().unwrap()),
            false,
            "jsonl",
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        // Check output - each line should be valid JSON
        let content = fs::read_to_string(&out_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);
        let parsed: BatchAnalyzeItem = serde_json::from_str(lines[0]).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.asset_type.as_deref(), Some("audio"));
    }

    #[test]
    fn test_batch_analyze_csv_format() {
        let tmp = tempfile::tempdir().unwrap();
        let subdir = tmp.path().join("assets");
        fs::create_dir(&subdir).unwrap();

        // Create test file
        let wav_path = subdir.join("test.wav");
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.01).sin()).collect();
        let wav_data = create_test_wav(&samples, 44100);
        fs::write(&wav_path, &wav_data).unwrap();

        let out_path = tmp.path().join("results.csv");

        let code = run(
            None,
            None,
            Some(subdir.to_str().unwrap()),
            Some(out_path.to_str().unwrap()),
            false,
            "csv",
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        // Check output has header and data row
        let content = fs::read_to_string(&out_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2); // header + 1 data row
        assert!(lines[0].starts_with("input,success,asset_type,"));
        assert!(lines[1].contains("true")); // success column
    }

    #[test]
    fn test_batch_analyze_with_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let subdir = tmp.path().join("assets");
        fs::create_dir(&subdir).unwrap();

        // Create a valid file
        let wav_path = subdir.join("good.wav");
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.01).sin()).collect();
        let wav_data = create_test_wav(&samples, 44100);
        fs::write(&wav_path, &wav_data).unwrap();

        // Create an invalid WAV file
        let bad_path = subdir.join("bad.wav");
        fs::write(&bad_path, b"not a valid wav file").unwrap();

        let out_path = tmp.path().join("results.json");

        let code = run(
            None,
            None,
            Some(subdir.to_str().unwrap()),
            Some(out_path.to_str().unwrap()),
            false,
            "json",
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS); // batch always succeeds overall

        // Check output has both succeeded and failed
        let content = fs::read_to_string(&out_path).unwrap();
        let parsed: BatchAnalyzeOutput = serde_json::from_str(&content).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.summary.total, 2);
        assert_eq!(parsed.summary.succeeded, 1);
        assert_eq!(parsed.summary.failed, 1);

        // Check the failed item has error info
        let failed = parsed.results.iter().find(|r| !r.success).unwrap();
        assert!(failed.error.is_some());
    }

    #[test]
    fn test_batch_analyze_empty_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let subdir = tmp.path().join("empty");
        fs::create_dir(&subdir).unwrap();

        let out_path = tmp.path().join("results.json");

        let code = run(
            None,
            None,
            Some(subdir.to_str().unwrap()),
            Some(out_path.to_str().unwrap()),
            false,
            "json",
            false,
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        // Check output
        let content = fs::read_to_string(&out_path).unwrap();
        let parsed: BatchAnalyzeOutput = serde_json::from_str(&content).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.summary.total, 0);
        assert_eq!(parsed.summary.succeeded, 0);
        assert_eq!(parsed.summary.failed, 0);
    }

    #[test]
    fn test_batch_analyze_with_embeddings() {
        let tmp = tempfile::tempdir().unwrap();
        let subdir = tmp.path().join("assets");
        fs::create_dir(&subdir).unwrap();

        // Create test file
        let wav_path = subdir.join("test.wav");
        let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let wav_data = create_test_wav(&samples, 44100);
        fs::write(&wav_path, &wav_data).unwrap();

        let out_path = tmp.path().join("results.json");

        let code = run(
            None,
            None,
            Some(subdir.to_str().unwrap()),
            Some(out_path.to_str().unwrap()),
            false,
            "json",
            true, // include embeddings
        )
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);

        // Check output has embeddings
        let content = fs::read_to_string(&out_path).unwrap();
        let parsed: BatchAnalyzeOutput = serde_json::from_str(&content).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.results.len(), 1);
        assert!(parsed.results[0].embedding.is_some());
        assert_eq!(
            parsed.results[0].embedding.as_ref().unwrap().len(),
            embeddings::AUDIO_EMBEDDING_DIM
        );
    }

    #[test]
    fn test_batch_deterministic_output() {
        let tmp = tempfile::tempdir().unwrap();
        let subdir = tmp.path().join("assets");
        fs::create_dir(&subdir).unwrap();

        // Create test files
        let wav_path = subdir.join("a_test.wav");
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.01).sin()).collect();
        let wav_data = create_test_wav(&samples, 44100);
        fs::write(&wav_path, &wav_data).unwrap();

        let png_path = subdir.join("b_test.png");
        let pixels: Vec<u8> = vec![128, 128, 128, 255];
        let png_data = create_test_png(1, 1, &pixels);
        fs::write(&png_path, &png_data).unwrap();

        let out1_path = tmp.path().join("results1.json");
        let out2_path = tmp.path().join("results2.json");

        // Run twice
        run(
            None,
            None,
            Some(subdir.to_str().unwrap()),
            Some(out1_path.to_str().unwrap()),
            false,
            "json",
            false,
        )
        .unwrap();

        run(
            None,
            None,
            Some(subdir.to_str().unwrap()),
            Some(out2_path.to_str().unwrap()),
            false,
            "json",
            false,
        )
        .unwrap();

        // Compare outputs - should be identical
        let content1 = fs::read_to_string(&out1_path).unwrap();
        let content2 = fs::read_to_string(&out2_path).unwrap();
        assert_eq!(content1, content2);
    }
}
