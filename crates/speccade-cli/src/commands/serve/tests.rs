//! Tests for the WebSocket analysis server.

use base64::Engine;

use crate::commands::json_output::{error_codes, AnalyzeOutput};

use super::handler::{analyze_data, analyze_path, handle_request};
use super::types::ErrorResponse;

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

#[test]
fn test_analyze_path_success() {
    let tmp = tempfile::tempdir().unwrap();
    let wav_path = tmp.path().join("test.wav");

    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    std::fs::write(&wav_path, &wav_data).unwrap();

    let output = analyze_path(wav_path.to_str().unwrap(), false);
    assert!(output.success);
    assert!(output.result.is_some());
    assert_eq!(output.result.as_ref().unwrap().asset_type, "audio");
}

#[test]
fn test_analyze_path_not_found() {
    let output = analyze_path("/nonexistent/file.wav", false);
    assert!(!output.success);
    assert!(!output.errors.is_empty());
    assert_eq!(output.errors[0].code, error_codes::FILE_READ);
}

#[test]
fn test_analyze_path_unsupported_format() {
    let tmp = tempfile::tempdir().unwrap();
    let txt_path = tmp.path().join("test.txt");
    std::fs::write(&txt_path, "hello").unwrap();

    let output = analyze_path(txt_path.to_str().unwrap(), false);
    assert!(!output.success);
    assert!(!output.errors.is_empty());
    assert_eq!(output.errors[0].code, error_codes::UNSUPPORTED_FORMAT);
}

#[test]
fn test_analyze_data_success() {
    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&wav_data);

    let output = analyze_data(&base64_data, "test.wav", false);
    assert!(output.success);
    assert!(output.result.is_some());
    assert_eq!(output.result.as_ref().unwrap().asset_type, "audio");
}

#[test]
fn test_analyze_data_invalid_base64() {
    let output = analyze_data("not valid base64!!!", "test.wav", false);
    assert!(!output.success);
    assert!(!output.errors.is_empty());
    assert_eq!(output.errors[0].code, "CLI_016");
}

#[test]
fn test_analyze_data_unsupported_format() {
    let base64_data = base64::engine::general_purpose::STANDARD.encode(b"hello");
    let output = analyze_data(&base64_data, "test.txt", false);
    assert!(!output.success);
    assert!(!output.errors.is_empty());
    assert_eq!(output.errors[0].code, error_codes::UNSUPPORTED_FORMAT);
}

#[test]
fn test_handle_request_analyze_path() {
    let tmp = tempfile::tempdir().unwrap();
    let wav_path = tmp.path().join("test.wav");

    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    std::fs::write(&wav_path, &wav_data).unwrap();

    let request = format!(
        r#"{{"type":"analyze_path","path":"{}"}}"#,
        wav_path.to_str().unwrap().replace('\\', "\\\\")
    );
    let response = handle_request(&request);
    let output: AnalyzeOutput = serde_json::from_str(&response).unwrap();

    assert!(output.success);
    assert_eq!(output.result.as_ref().unwrap().asset_type, "audio");
}

#[test]
fn test_handle_request_analyze_data() {
    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&wav_data);

    let request = format!(
        r#"{{"type":"analyze_data","data":"{}","filename":"test.wav"}}"#,
        base64_data
    );
    let response = handle_request(&request);
    let output: AnalyzeOutput = serde_json::from_str(&response).unwrap();

    assert!(output.success);
    assert_eq!(output.result.as_ref().unwrap().asset_type, "audio");
}

#[test]
fn test_handle_request_invalid_json() {
    let response = handle_request("not json");
    let output: ErrorResponse = serde_json::from_str(&response).unwrap();
    assert!(!output.success);
    assert_eq!(output.errors[0].code, "CLI_015");
}

#[test]
fn test_handle_request_unknown_type() {
    let response = handle_request(r#"{"type":"unknown"}"#);
    let output: ErrorResponse = serde_json::from_str(&response).unwrap();
    assert!(!output.success);
}

#[test]
fn test_analyze_with_embeddings() {
    let tmp = tempfile::tempdir().unwrap();
    let wav_path = tmp.path().join("test.wav");

    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    std::fs::write(&wav_path, &wav_data).unwrap();

    let output = analyze_path(wav_path.to_str().unwrap(), true);
    assert!(output.success);
    assert!(output.result.as_ref().unwrap().embedding.is_some());
}

#[test]
fn test_analyze_request_with_embeddings_flag() {
    let tmp = tempfile::tempdir().unwrap();
    let wav_path = tmp.path().join("test.wav");

    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    std::fs::write(&wav_path, &wav_data).unwrap();

    let request = format!(
        r#"{{"type":"analyze_path","path":"{}","embeddings":true}}"#,
        wav_path.to_str().unwrap().replace('\\', "\\\\")
    );
    let response = handle_request(&request);
    let output: AnalyzeOutput = serde_json::from_str(&response).unwrap();

    assert!(output.success);
    assert!(output.result.as_ref().unwrap().embedding.is_some());
}
