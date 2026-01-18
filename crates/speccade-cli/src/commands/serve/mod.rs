//! WebSocket analysis server for real-time editor integration.
//!
//! This module provides a WebSocket server that accepts analyze requests
//! and returns JSON responses with quality metrics. Designed for editor
//! plugins and iterative workflow tools.
//!
//! ## Protocol
//!
//! Requests are JSON objects with a `type` field:
//!
//! - `analyze_path`: Analyze a file by absolute path
//!   ```json
//!   {"type": "analyze_path", "path": "/abs/path/to/file.wav"}
//!   ```
//!
//! - `analyze_data`: Analyze raw data sent as base64
//!   ```json
//!   {"type": "analyze_data", "data": "<base64>", "filename": "test.wav"}
//!   ```
//!
//! Responses are the standard AnalyzeOutput JSON structure.

mod handler;
mod types;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;

pub use handler::{analyze_data, analyze_path};
pub use types::{AnalyzeRequest, ErrorResponse};

/// Default port for the WebSocket server.
pub const DEFAULT_PORT: u16 = 9123;

/// Run the WebSocket analysis server.
///
/// # Arguments
/// * `port` - Port to listen on
///
/// # Returns
/// Exit code: 0 on clean shutdown, 1 on error
pub fn run(port: u16) -> Result<ExitCode> {
    // Build tokio runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to create tokio runtime")?;

    rt.block_on(async move { run_server(port).await })
}

/// Run the WebSocket server (async entry point).
async fn run_server(port: u16) -> Result<ExitCode> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {}", addr))?;

    eprintln!("WebSocket analysis server listening on ws://{}", addr);
    eprintln!("Press Ctrl+C to shutdown");

    // Create shutdown channel
    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let shutdown_tx = Arc::new(shutdown_tx);

    // Set up SIGINT handler
    let shutdown_tx_clone = Arc::clone(&shutdown_tx);
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            eprintln!("\nShutting down...");
            let _ = shutdown_tx_clone.send(());
        }
    });

    let mut shutdown_rx = shutdown_tx.subscribe();

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, peer_addr)) => {
                        eprintln!("New connection from {}", peer_addr);
                        let shutdown_rx = shutdown_tx.subscribe();
                        tokio::spawn(handle_connection(stream, peer_addr, shutdown_rx));
                    }
                    Err(e) => {
                        eprintln!("Accept error: {}", e);
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                eprintln!("Server shutdown complete");
                break;
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}

/// Handle a single WebSocket connection.
async fn handle_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake failed for {}: {}", peer_addr, e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    loop {
        tokio::select! {
            msg_opt = read.next() => {
                match msg_opt {
                    Some(Ok(msg)) => {
                        if let Some(response) = handler::process_message(msg).await {
                            if let Err(e) = write.send(Message::Text(response)).await {
                                eprintln!("Send error for {}: {}", peer_addr, e);
                                break;
                            }
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("Receive error for {}: {}", peer_addr, e);
                        break;
                    }
                    None => {
                        // Connection closed
                        break;
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                // Server shutting down
                let _ = write.send(Message::Close(None)).await;
                break;
            }
        }
    }

    eprintln!("Connection closed: {}", peer_addr);
}
