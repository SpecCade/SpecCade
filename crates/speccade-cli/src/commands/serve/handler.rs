//! Request handler logic for the WebSocket analysis server.

use std::path::Path;

use base64::Engine;
use tokio_tungstenite::tungstenite::Message;

use crate::analysis::{audio, detect_asset_type, embeddings, texture, AssetAnalysisType};
use crate::commands::json_output::{error_codes, AnalyzeOutput, AnalyzeResult, JsonError};

use super::types::{AnalyzeRequest, ErrorResponse};

/// Process a single WebSocket message and return a response.
pub async fn process_message(msg: Message) -> Option<String> {
    match msg {
        Message::Text(text) => {
            let response = handle_request(&text);
            Some(response)
        }
        Message::Binary(data) => {
            // Try to parse binary as UTF-8 JSON
            match String::from_utf8(data) {
                Ok(text) => {
                    let response = handle_request(&text);
                    Some(response)
                }
                Err(_) => {
                    let error =
                        ErrorResponse::new("CLI_014", "Binary message must be valid UTF-8 JSON");
                    Some(serde_json::to_string(&error).unwrap_or_else(|_| {
                        r#"{"success":false,"errors":[{"code":"CLI_014","message":"Binary message must be valid UTF-8 JSON"}]}"#.to_string()
                    }))
                }
            }
        }
        Message::Ping(_) | Message::Pong(_) => {
            // Handled automatically by tungstenite
            None
        }
        Message::Close(_) => {
            // Connection closing
            None
        }
        Message::Frame(_) => {
            // Raw frame, shouldn't happen in normal operation
            None
        }
    }
}

/// Handle a JSON request and return a JSON response.
pub fn handle_request(json_text: &str) -> String {
    // Parse request
    let request: AnalyzeRequest = match serde_json::from_str(json_text) {
        Ok(req) => req,
        Err(e) => {
            let error = ErrorResponse::new("CLI_015", format!("Invalid request JSON: {}", e));
            return serde_json::to_string(&error).unwrap_or_else(|_| {
                r#"{"success":false,"errors":[{"code":"CLI_015","message":"Invalid request JSON"}]}"#.to_string()
            });
        }
    };

    // Process request
    let output = match request {
        AnalyzeRequest::AnalyzePath { path, embeddings } => analyze_path(&path, embeddings),
        AnalyzeRequest::AnalyzeData {
            data,
            filename,
            embeddings,
        } => analyze_data(&data, &filename, embeddings),
    };

    // Serialize response
    serde_json::to_string(&output).unwrap_or_else(|e| {
        let error = ErrorResponse::new(error_codes::JSON_SERIALIZE, format!("Failed to serialize response: {}", e));
        serde_json::to_string(&error).unwrap_or_else(|_| {
            r#"{"success":false,"errors":[{"code":"CLI_009","message":"Failed to serialize response"}]}"#.to_string()
        })
    })
}

/// Analyze a file at the given path.
pub fn analyze_path(path: &str, include_embeddings: bool) -> AnalyzeOutput {
    let file_path = Path::new(path);

    // Detect asset type
    let asset_type = match detect_asset_type(file_path) {
        Some(t) => t,
        None => {
            let error = JsonError::new(
                error_codes::UNSUPPORTED_FORMAT,
                format!(
                    "Unsupported file format: {}",
                    file_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("(none)")
                ),
            )
            .with_file(path);
            return AnalyzeOutput::failure(vec![error]);
        }
    };

    // Read file
    let data = match std::fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            let error = JsonError::new(
                error_codes::FILE_READ,
                format!("Failed to read file: {}", e),
            )
            .with_file(path);
            return AnalyzeOutput::failure(vec![error]);
        }
    };

    analyze_bytes(&data, path, asset_type, include_embeddings)
}

/// Analyze data provided as base64.
pub fn analyze_data(base64_data: &str, filename: &str, include_embeddings: bool) -> AnalyzeOutput {
    let file_path = Path::new(filename);

    // Detect asset type from filename
    let asset_type = match detect_asset_type(file_path) {
        Some(t) => t,
        None => {
            let error = JsonError::new(
                error_codes::UNSUPPORTED_FORMAT,
                format!(
                    "Unsupported file format: {}",
                    file_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("(none)")
                ),
            );
            return AnalyzeOutput::failure(vec![error]);
        }
    };

    // Decode base64
    let data = match base64::engine::general_purpose::STANDARD.decode(base64_data) {
        Ok(d) => d,
        Err(e) => {
            let error = JsonError::new("CLI_016", format!("Invalid base64 data: {}", e));
            return AnalyzeOutput::failure(vec![error]);
        }
    };

    analyze_bytes(&data, filename, asset_type, include_embeddings)
}

/// Analyze raw bytes and return the analysis output.
fn analyze_bytes(
    data: &[u8],
    input_name: &str,
    asset_type: AssetAnalysisType,
    include_embeddings: bool,
) -> AnalyzeOutput {
    // Compute input hash
    let input_hash = blake3::hash(data).to_hex().to_string();

    // Analyze based on type
    let (metrics, embedding) = match asset_type {
        AssetAnalysisType::Audio => match audio::analyze_wav(data) {
            Ok(m) => {
                let emb = if include_embeddings {
                    match audio::extract_wav_samples(data) {
                        Ok((samples, sample_rate)) => {
                            Some(embeddings::compute_audio_embedding(&samples, sample_rate))
                        }
                        Err(e) => {
                            let error = JsonError::new(
                                error_codes::AUDIO_ANALYSIS,
                                format!("Audio extraction for embedding failed: {}", e),
                            );
                            return AnalyzeOutput::failure(vec![error]);
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
                );
                return AnalyzeOutput::failure(vec![error]);
            }
        },
        AssetAnalysisType::Texture => match texture::analyze_png(data) {
            Ok(m) => {
                let emb = if include_embeddings {
                    match texture::extract_png_pixels(data) {
                        Ok((pixels, width, height, channels)) => Some(
                            embeddings::compute_texture_embedding(&pixels, width, height, channels),
                        ),
                        Err(e) => {
                            let error = JsonError::new(
                                error_codes::TEXTURE_ANALYSIS,
                                format!("Texture extraction for embedding failed: {}", e),
                            );
                            return AnalyzeOutput::failure(vec![error]);
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
                );
                return AnalyzeOutput::failure(vec![error]);
            }
        },
    };

    // Build result
    let result = AnalyzeResult {
        input: input_name.to_string(),
        asset_type: asset_type.as_str().to_string(),
        input_hash,
        metrics,
        embedding,
    };

    AnalyzeOutput::success(result)
}
