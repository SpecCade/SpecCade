//! Integration tests for the WebSocket analysis server.
//!
//! These tests verify the WebSocket server functionality by:
//! - Starting a server on a dynamic port
//! - Connecting as a WebSocket client
//! - Sending analyze requests and validating responses
//!
//! ## Running Tests
//!
//! ```bash
//! cargo test -p speccade-tests --test analyze_serve
//! ```

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::TcpListener;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

/// AnalyzeOutput structure for deserializing responses.
#[derive(Debug, Deserialize)]
struct AnalyzeOutput {
    success: bool,
    errors: Vec<JsonError>,
    result: Option<AnalyzeResult>,
}

#[derive(Debug, Deserialize)]
struct AnalyzeResult {
    input: String,
    asset_type: String,
    input_hash: String,
    metrics: BTreeMap<String, serde_json::Value>,
    embedding: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonError {
    code: String,
    message: String,
}

/// Request types for the server.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnalyzeRequest {
    AnalyzePath {
        path: String,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        embeddings: bool,
    },
    AnalyzeData {
        data: String,
        filename: String,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        embeddings: bool,
    },
}

/// Find an available port for testing.
fn find_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

/// Create a test WAV file with a sine wave.
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

/// Create a simple test PNG image.
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

/// Helper to start a server in the background.
async fn start_server(port: u16) -> tokio::task::JoinHandle<()> {
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio::sync::broadcast;

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tokio::spawn(async move {
        let listener = TcpListener::bind(&addr).await.unwrap();
        let (shutdown_tx, _) = broadcast::channel::<()>(1);
        let _shutdown_tx = Arc::new(shutdown_tx);

        // Only handle a few connections for testing
        for _ in 0..5 {
            if let Ok((stream, _)) = listener.accept().await {
                let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
                let (mut write, mut read) = ws_stream.split();

                while let Some(Ok(msg)) = read.next().await {
                    if let Message::Text(text) = msg {
                        // Process request using the serve module
                        let response = process_request(&text);
                        if write.send(Message::Text(response)).await.is_err() {
                            break;
                        }
                    } else if let Message::Close(_) = msg {
                        break;
                    }
                }
            }
        }
    })
}

/// Process a request directly using the serve module functions.
fn process_request(json_text: &str) -> String {
    use speccade_cli::commands::serve::{analyze_data, analyze_path};

    #[derive(Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum Request {
        AnalyzePath {
            path: String,
            #[serde(default)]
            embeddings: bool,
        },
        AnalyzeData {
            data: String,
            filename: String,
            #[serde(default)]
            embeddings: bool,
        },
    }

    match serde_json::from_str::<Request>(json_text) {
        Ok(Request::AnalyzePath { path, embeddings }) => {
            let output = analyze_path(&path, embeddings);
            serde_json::to_string(&output).unwrap()
        }
        Ok(Request::AnalyzeData {
            data,
            filename,
            embeddings,
        }) => {
            let output = analyze_data(&data, &filename, embeddings);
            serde_json::to_string(&output).unwrap()
        }
        Err(e) => {
            format!(
                r#"{{"success":false,"errors":[{{"code":"CLI_015","message":"{}"}}]}}"#,
                e.to_string().replace('"', "\\\"")
            )
        }
    }
}

/// Connect to a WebSocket server.
async fn connect_ws(
    port: u16,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::Error> {
    let url = format!("ws://127.0.0.1:{}", port);
    let (ws_stream, _) = connect_async(&url).await?;
    Ok(ws_stream)
}

/// Send a request and receive a response.
async fn send_request(
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    request: &AnalyzeRequest,
) -> Result<AnalyzeOutput, Box<dyn std::error::Error>> {
    let request_json = serde_json::to_string(request)?;
    ws.send(Message::Text(request_json)).await?;

    match timeout(Duration::from_secs(5), ws.next()).await {
        Ok(Some(Ok(Message::Text(response)))) => {
            let output: AnalyzeOutput = serde_json::from_str(&response)?;
            Ok(output)
        }
        Ok(Some(Ok(msg))) => Err(format!("Unexpected message type: {:?}", msg).into()),
        Ok(Some(Err(e))) => Err(e.into()),
        Ok(None) => Err("Connection closed".into()),
        Err(_) => Err("Timeout waiting for response".into()),
    }
}

// ============================================================================
// Tests
// ============================================================================

/// Test analyzing a WAV file by path.
#[tokio::test]
async fn test_analyze_path_wav() {
    let port = find_available_port();

    // Create test file
    let tmp = tempfile::tempdir().unwrap();
    let wav_path = tmp.path().join("test.wav");
    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    std::fs::write(&wav_path, &wav_data).unwrap();

    // Start server
    let server_handle = start_server(port).await;

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and send request
    let mut ws = connect_ws(port).await.expect("Failed to connect");
    let request = AnalyzeRequest::AnalyzePath {
        path: wav_path.to_str().unwrap().to_string(),
        embeddings: false,
    };
    let output = send_request(&mut ws, &request).await.unwrap();

    assert!(output.success, "Expected success, got errors: {:?}", output.errors);
    let result = output.result.unwrap();
    assert_eq!(result.asset_type, "audio");
    assert!(!result.input_hash.is_empty());
    assert!(!result.metrics.is_empty());

    // Clean up
    ws.close(None).await.ok();
    server_handle.abort();
}

/// Test analyzing a PNG file by path.
#[tokio::test]
async fn test_analyze_path_png() {
    let port = find_available_port();

    // Create test file
    let tmp = tempfile::tempdir().unwrap();
    let png_path = tmp.path().join("test.png");
    let pixels: Vec<u8> = (0..4 * 4 * 4).map(|i| (i % 256) as u8).collect();
    let png_data = create_test_png(4, 4, &pixels);
    std::fs::write(&png_path, &png_data).unwrap();

    // Start server
    let server_handle = start_server(port).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and send request
    let mut ws = connect_ws(port).await.expect("Failed to connect");
    let request = AnalyzeRequest::AnalyzePath {
        path: png_path.to_str().unwrap().to_string(),
        embeddings: false,
    };
    let output = send_request(&mut ws, &request).await.unwrap();

    assert!(output.success, "Expected success, got errors: {:?}", output.errors);
    let result = output.result.unwrap();
    assert_eq!(result.asset_type, "texture");
    assert!(!result.input_hash.is_empty());

    ws.close(None).await.ok();
    server_handle.abort();
}

/// Test analyzing WAV data sent as base64.
#[tokio::test]
async fn test_analyze_data_wav() {
    let port = find_available_port();

    // Create test data
    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&wav_data);

    // Start server
    let server_handle = start_server(port).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and send request
    let mut ws = connect_ws(port).await.expect("Failed to connect");
    let request = AnalyzeRequest::AnalyzeData {
        data: base64_data,
        filename: "test.wav".to_string(),
        embeddings: false,
    };
    let output = send_request(&mut ws, &request).await.unwrap();

    assert!(output.success, "Expected success, got errors: {:?}", output.errors);
    let result = output.result.unwrap();
    assert_eq!(result.asset_type, "audio");
    assert_eq!(result.input, "test.wav");

    ws.close(None).await.ok();
    server_handle.abort();
}

/// Test analyzing PNG data sent as base64.
#[tokio::test]
async fn test_analyze_data_png() {
    let port = find_available_port();

    // Create test data
    let pixels: Vec<u8> = (0..4 * 4 * 4).map(|i| (i % 256) as u8).collect();
    let png_data = create_test_png(4, 4, &pixels);
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&png_data);

    // Start server
    let server_handle = start_server(port).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and send request
    let mut ws = connect_ws(port).await.expect("Failed to connect");
    let request = AnalyzeRequest::AnalyzeData {
        data: base64_data,
        filename: "test.png".to_string(),
        embeddings: false,
    };
    let output = send_request(&mut ws, &request).await.unwrap();

    assert!(output.success, "Expected success, got errors: {:?}", output.errors);
    let result = output.result.unwrap();
    assert_eq!(result.asset_type, "texture");
    assert_eq!(result.input, "test.png");

    ws.close(None).await.ok();
    server_handle.abort();
}

/// Test analyzing with embeddings flag.
#[tokio::test]
async fn test_analyze_with_embeddings() {
    let port = find_available_port();

    // Create test file
    let tmp = tempfile::tempdir().unwrap();
    let wav_path = tmp.path().join("test.wav");
    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    std::fs::write(&wav_path, &wav_data).unwrap();

    // Start server
    let server_handle = start_server(port).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and send request with embeddings
    let mut ws = connect_ws(port).await.expect("Failed to connect");
    let request = AnalyzeRequest::AnalyzePath {
        path: wav_path.to_str().unwrap().to_string(),
        embeddings: true,
    };
    let output = send_request(&mut ws, &request).await.unwrap();

    assert!(output.success, "Expected success, got errors: {:?}", output.errors);
    let result = output.result.unwrap();
    assert!(result.embedding.is_some(), "Expected embedding to be present");
    let embedding = result.embedding.unwrap();
    assert!(!embedding.is_empty(), "Expected non-empty embedding");

    ws.close(None).await.ok();
    server_handle.abort();
}

/// Test error handling for file not found.
#[tokio::test]
async fn test_analyze_path_not_found() {
    let port = find_available_port();

    // Start server
    let server_handle = start_server(port).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and send request for non-existent file
    let mut ws = connect_ws(port).await.expect("Failed to connect");
    let request = AnalyzeRequest::AnalyzePath {
        path: "/nonexistent/path/to/file.wav".to_string(),
        embeddings: false,
    };
    let output = send_request(&mut ws, &request).await.unwrap();

    assert!(!output.success, "Expected failure for non-existent file");
    assert!(!output.errors.is_empty());
    assert_eq!(output.errors[0].code, "CLI_001"); // FILE_READ error

    ws.close(None).await.ok();
    server_handle.abort();
}

/// Test error handling for unsupported format.
#[tokio::test]
async fn test_analyze_unsupported_format() {
    let port = find_available_port();

    // Create test file with unsupported extension
    let tmp = tempfile::tempdir().unwrap();
    let txt_path = tmp.path().join("test.txt");
    std::fs::write(&txt_path, "hello").unwrap();

    // Start server
    let server_handle = start_server(port).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and send request
    let mut ws = connect_ws(port).await.expect("Failed to connect");
    let request = AnalyzeRequest::AnalyzePath {
        path: txt_path.to_str().unwrap().to_string(),
        embeddings: false,
    };
    let output = send_request(&mut ws, &request).await.unwrap();

    assert!(!output.success, "Expected failure for unsupported format");
    assert!(!output.errors.is_empty());
    assert_eq!(output.errors[0].code, "CLI_011"); // UNSUPPORTED_FORMAT error

    ws.close(None).await.ok();
    server_handle.abort();
}

/// Test error handling for invalid base64.
#[tokio::test]
async fn test_analyze_data_invalid_base64() {
    let port = find_available_port();

    // Start server
    let server_handle = start_server(port).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and send request with invalid base64
    let mut ws = connect_ws(port).await.expect("Failed to connect");
    let request = AnalyzeRequest::AnalyzeData {
        data: "not valid base64!!!".to_string(),
        filename: "test.wav".to_string(),
        embeddings: false,
    };
    let output = send_request(&mut ws, &request).await.unwrap();

    assert!(!output.success, "Expected failure for invalid base64");
    assert!(!output.errors.is_empty());
    assert_eq!(output.errors[0].code, "CLI_016"); // Invalid base64 error

    ws.close(None).await.ok();
    server_handle.abort();
}

/// Test multiple requests on same connection.
#[tokio::test]
async fn test_multiple_requests() {
    let port = find_available_port();

    // Create test files
    let tmp = tempfile::tempdir().unwrap();
    let wav_path = tmp.path().join("test.wav");
    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    std::fs::write(&wav_path, &wav_data).unwrap();

    let png_path = tmp.path().join("test.png");
    let pixels: Vec<u8> = (0..4 * 4 * 4).map(|i| (i % 256) as u8).collect();
    let png_data = create_test_png(4, 4, &pixels);
    std::fs::write(&png_path, &png_data).unwrap();

    // Start server
    let server_handle = start_server(port).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and send multiple requests
    let mut ws = connect_ws(port).await.expect("Failed to connect");

    // First request (WAV)
    let request1 = AnalyzeRequest::AnalyzePath {
        path: wav_path.to_str().unwrap().to_string(),
        embeddings: false,
    };
    let output1 = send_request(&mut ws, &request1).await.unwrap();
    assert!(output1.success);
    assert_eq!(output1.result.unwrap().asset_type, "audio");

    // Second request (PNG)
    let request2 = AnalyzeRequest::AnalyzePath {
        path: png_path.to_str().unwrap().to_string(),
        embeddings: false,
    };
    let output2 = send_request(&mut ws, &request2).await.unwrap();
    assert!(output2.success);
    assert_eq!(output2.result.unwrap().asset_type, "texture");

    ws.close(None).await.ok();
    server_handle.abort();
}

/// Test that responses are deterministic (same input = same output).
#[tokio::test]
async fn test_deterministic_responses() {
    let port = find_available_port();

    // Create test file
    let tmp = tempfile::tempdir().unwrap();
    let wav_path = tmp.path().join("test.wav");
    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    std::fs::write(&wav_path, &wav_data).unwrap();

    // Start server
    let server_handle = start_server(port).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and send same request twice
    let mut ws = connect_ws(port).await.expect("Failed to connect");
    let request = AnalyzeRequest::AnalyzePath {
        path: wav_path.to_str().unwrap().to_string(),
        embeddings: true,
    };

    let output1 = send_request(&mut ws, &request).await.unwrap();
    let output2 = send_request(&mut ws, &request).await.unwrap();

    assert!(output1.success);
    assert!(output2.success);

    let result1 = output1.result.unwrap();
    let result2 = output2.result.unwrap();

    // Hashes must be identical
    assert_eq!(result1.input_hash, result2.input_hash);

    // Metrics must be identical
    assert_eq!(result1.metrics, result2.metrics);

    // Embeddings must be identical
    assert_eq!(result1.embedding, result2.embedding);

    ws.close(None).await.ok();
    server_handle.abort();
}

/// Test direct function calls (unit test for serve module functions).
#[test]
fn test_analyze_path_direct() {
    use speccade_cli::commands::serve::analyze_path;

    let tmp = tempfile::tempdir().unwrap();
    let wav_path = tmp.path().join("test.wav");
    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    std::fs::write(&wav_path, &wav_data).unwrap();

    let output = analyze_path(wav_path.to_str().unwrap(), false);
    assert!(output.success);
    assert_eq!(output.result.as_ref().unwrap().asset_type, "audio");
}

/// Test direct function calls for base64 data.
#[test]
fn test_analyze_data_direct() {
    use speccade_cli::commands::serve::analyze_data;

    let samples: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let wav_data = create_test_wav(&samples, 44100);
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&wav_data);

    let output = analyze_data(&base64_data, "test.wav", false);
    assert!(output.success);
    assert_eq!(output.result.as_ref().unwrap().asset_type, "audio");
}
