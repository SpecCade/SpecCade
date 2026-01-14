//! PNG encoding for graph values.

use crate::png;

use super::super::GenerateError;
use super::GraphValue;

/// Encode a graph value as PNG bytes (deterministic) and return `(bytes, blake3_hash)`.
pub fn encode_graph_value_png(value: &GraphValue) -> Result<(Vec<u8>, String), GenerateError> {
    let config = crate::png::PngConfig::default();

    match value {
        GraphValue::Grayscale(buf) => {
            let (data, hash) = png::write_grayscale_to_vec_with_hash(buf, &config)?;
            Ok((data, hash))
        }
        GraphValue::Color(buf) => {
            let (data, hash) = png::write_rgba_to_vec_with_hash(buf, &config)?;
            Ok((data, hash))
        }
    }
}
