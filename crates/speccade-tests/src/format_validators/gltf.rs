//! glTF and GLB file format validators.

use serde_json::Value;

use super::FormatError;

/// Information extracted from a GLB (glTF Binary) file header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlbInfo {
    /// glTF version (should be 2).
    pub version: u32,
    /// Total file length in bytes.
    pub length: u32,
    /// Length of the JSON chunk.
    pub json_chunk_length: u32,
    /// Length of the binary chunk (if present).
    pub bin_chunk_length: Option<u32>,
    /// Number of chunks in the file.
    pub num_chunks: u32,
}

/// Information extracted from a glTF JSON file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GltfInfo {
    /// The glTF asset version string (e.g., "2.0").
    pub version: String,
}

/// Validate glTF (JSON) file format and extract basic information.
pub fn validate_gltf(data: &[u8]) -> Result<GltfInfo, FormatError> {
    let text = std::str::from_utf8(data)
        .map_err(|e| FormatError::new("glTF", format!("Invalid UTF-8: {}", e)))?;

    let json: Value =
        serde_json::from_str(text).map_err(|e| FormatError::new("glTF", e.to_string()))?;

    let asset = json
        .get("asset")
        .and_then(|v| v.as_object())
        .ok_or_else(|| FormatError::new("glTF", "Missing or invalid 'asset' object"))?;

    let version = asset
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| FormatError::new("glTF", "Missing or invalid 'asset.version'"))?;

    Ok(GltfInfo {
        version: version.to_string(),
    })
}

/// Validate GLB (glTF Binary) file format and extract header information.
///
/// Parses the GLB header and chunk structure to validate:
/// - GLB magic number ("glTF")
/// - Version 2 format
/// - Valid chunk structure
///
/// # Arguments
/// * `data` - Raw bytes of the GLB file
///
/// # Returns
/// * `Ok(GlbInfo)` - Successfully parsed GLB file information
/// * `Err(FormatError)` - Invalid or corrupted GLB file
pub fn validate_glb(data: &[u8]) -> Result<GlbInfo, FormatError> {
    const GLB_MAGIC: &[u8; 4] = b"glTF";
    const MIN_HEADER_SIZE: usize = 12;
    const CHUNK_HEADER_SIZE: usize = 8;
    const JSON_CHUNK_TYPE: u32 = 0x4E4F534A; // "JSON" in little-endian
    const BIN_CHUNK_TYPE: u32 = 0x004E4942; // "BIN\0" in little-endian

    if data.len() < MIN_HEADER_SIZE {
        return Err(FormatError::new(
            "GLB",
            format!(
                "File too short: {} bytes (minimum {} required)",
                data.len(),
                MIN_HEADER_SIZE
            ),
        ));
    }

    // Check magic number
    if &data[0..4] != GLB_MAGIC {
        return Err(FormatError::at_offset(
            "GLB",
            format!("Invalid GLB magic: expected 'glTF', got {:?}", &data[0..4]),
            0,
        ));
    }

    // Version (bytes 4-7, little-endian)
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if version != 2 {
        return Err(FormatError::at_offset(
            "GLB",
            format!("Unsupported GLB version: {} (expected 2)", version),
            4,
        ));
    }

    // Total length (bytes 8-11, little-endian)
    let length = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    if length as usize > data.len() {
        return Err(FormatError::new(
            "GLB",
            format!(
                "Declared length {} exceeds actual file size {}",
                length,
                data.len()
            ),
        ));
    }

    // Parse chunks
    let mut offset = 12;
    let mut num_chunks = 0u32;
    let mut json_chunk_length = 0u32;
    let mut bin_chunk_length: Option<u32> = None;

    while offset + CHUNK_HEADER_SIZE <= data.len() {
        let chunk_length = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        let chunk_type = u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);

        if num_chunks == 0 {
            // First chunk must be JSON
            if chunk_type != JSON_CHUNK_TYPE {
                return Err(FormatError::at_offset(
                    "GLB",
                    format!("First chunk must be JSON, got type 0x{:08X}", chunk_type),
                    offset,
                ));
            }
            json_chunk_length = chunk_length;
        } else if chunk_type == BIN_CHUNK_TYPE {
            bin_chunk_length = Some(chunk_length);
        }

        num_chunks += 1;

        // Move to next chunk (chunks are 4-byte aligned)
        let padded_length = (chunk_length + 3) & !3;
        offset += CHUNK_HEADER_SIZE + padded_length as usize;
    }

    if num_chunks == 0 {
        return Err(FormatError::new("GLB", "No chunks found"));
    }

    Ok(GlbInfo {
        version,
        length,
        json_chunk_length,
        bin_chunk_length,
        num_chunks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_glb_valid() {
        let glb = create_test_glb(r#"{"asset":{"version":"2.0"}}"#, Some(100));
        let info = validate_glb(&glb).expect("Should parse valid GLB");

        assert_eq!(info.version, 2);
        assert_eq!(info.num_chunks, 2);
        assert!(info.bin_chunk_length.is_some());
    }

    #[test]
    fn test_validate_glb_json_only() {
        let glb = create_test_glb(r#"{"asset":{"version":"2.0"}}"#, None);
        let info = validate_glb(&glb).expect("Should parse GLB with JSON only");

        assert_eq!(info.version, 2);
        assert_eq!(info.num_chunks, 1);
        assert!(info.bin_chunk_length.is_none());
    }

    #[test]
    fn test_validate_glb_too_short() {
        let data = vec![0u8; 8];
        let err = validate_glb(&data).unwrap_err();
        assert_eq!(err.format, "GLB");
        assert!(err.message.contains("too short"));
    }

    #[test]
    fn test_validate_glb_invalid_magic() {
        let mut glb = create_test_glb("{}", None);
        glb[0..4].copy_from_slice(b"XXXX");
        let err = validate_glb(&glb).unwrap_err();
        assert!(err.message.contains("magic") || err.message.contains("glTF"));
    }

    #[test]
    fn test_validate_glb_invalid_version() {
        let mut glb = create_test_glb("{}", None);
        glb[4..8].copy_from_slice(&1u32.to_le_bytes()); // Version 1
        let err = validate_glb(&glb).unwrap_err();
        assert!(err.message.contains("version"));
    }

    fn create_test_glb(json: &str, bin_size: Option<usize>) -> Vec<u8> {
        let json_bytes = json.as_bytes();
        let json_padded_len = (json_bytes.len() + 3) & !3; // 4-byte aligned

        let bin_padded_len = bin_size.map(|s| (s + 3) & !3).unwrap_or(0);

        let total_len = 12
            + 8
            + json_padded_len
            + if bin_size.is_some() {
                8 + bin_padded_len
            } else {
                0
            };

        let mut glb = Vec::with_capacity(total_len);

        // Header
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2u32.to_le_bytes()); // version
        glb.extend_from_slice(&(total_len as u32).to_le_bytes());

        // JSON chunk
        glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // "JSON"
        glb.extend_from_slice(json_bytes);
        // Padding
        glb.resize(glb.len() + (json_padded_len - json_bytes.len()), 0x20); // Space padding for JSON

        // Binary chunk (optional)
        if let Some(size) = bin_size {
            glb.extend_from_slice(&(size as u32).to_le_bytes());
            glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // "BIN\0"
            glb.resize(glb.len() + bin_padded_len, 0);
        }

        glb
    }
}
