//! PNG file format validator.

use super::FormatError;

/// Information extracted from a PNG file header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PngInfo {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Bit depth (1, 2, 4, 8, or 16).
    pub bit_depth: u8,
    /// Color type (0=grayscale, 2=RGB, 3=indexed, 4=grayscale+alpha, 6=RGBA).
    pub color_type: u8,
    /// Compression method (0 = deflate).
    pub compression_method: u8,
    /// Filter method (0 = adaptive).
    pub filter_method: u8,
    /// Interlace method (0 = none, 1 = Adam7).
    pub interlace_method: u8,
}

impl PngInfo {
    /// Returns the number of channels for this color type.
    pub fn channels(&self) -> u8 {
        match self.color_type {
            0 => 1, // Grayscale
            2 => 3, // RGB
            3 => 1, // Indexed (palette)
            4 => 2, // Grayscale + Alpha
            6 => 4, // RGBA
            _ => 0,
        }
    }

    /// Returns a human-readable description of the color type.
    pub fn color_type_name(&self) -> &'static str {
        match self.color_type {
            0 => "Grayscale",
            2 => "RGB",
            3 => "Indexed",
            4 => "Grayscale+Alpha",
            6 => "RGBA",
            _ => "Unknown",
        }
    }
}

/// Validate PNG file format and extract header information.
///
/// Parses the PNG signature and IHDR chunk to validate:
/// - PNG magic signature (8 bytes)
/// - IHDR chunk presence and validity
/// - Valid color type and bit depth combinations
///
/// # Arguments
/// * `data` - Raw bytes of the PNG file
///
/// # Returns
/// * `Ok(PngInfo)` - Successfully parsed PNG file information
/// * `Err(FormatError)` - Invalid or corrupted PNG file
pub fn validate_png(data: &[u8]) -> Result<PngInfo, FormatError> {
    const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    const MIN_HEADER_SIZE: usize = 8 + 8 + 13; // signature + chunk header + IHDR data

    if data.len() < MIN_HEADER_SIZE {
        return Err(FormatError::new(
            "PNG",
            format!(
                "File too short: {} bytes (minimum {} required)",
                data.len(),
                MIN_HEADER_SIZE
            ),
        ));
    }

    // Check PNG signature
    if data[0..8] != PNG_SIGNATURE {
        return Err(FormatError::at_offset("PNG", "Invalid PNG signature", 0));
    }

    // Parse IHDR chunk (must be first chunk after signature)
    let chunk_length = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as usize;
    let chunk_type = &data[12..16];

    if chunk_type != b"IHDR" {
        return Err(FormatError::at_offset(
            "PNG",
            format!("First chunk must be IHDR, got {:?}", chunk_type),
            12,
        ));
    }

    if chunk_length != 13 {
        return Err(FormatError::at_offset(
            "PNG",
            format!("IHDR chunk must be 13 bytes, got {}", chunk_length),
            8,
        ));
    }

    // Parse IHDR data
    let ihdr_data = &data[16..29];
    let width = u32::from_be_bytes([ihdr_data[0], ihdr_data[1], ihdr_data[2], ihdr_data[3]]);
    let height = u32::from_be_bytes([ihdr_data[4], ihdr_data[5], ihdr_data[6], ihdr_data[7]]);
    let bit_depth = ihdr_data[8];
    let color_type = ihdr_data[9];
    let compression_method = ihdr_data[10];
    let filter_method = ihdr_data[11];
    let interlace_method = ihdr_data[12];

    // Validate dimensions
    if width == 0 || height == 0 {
        return Err(FormatError::new(
            "PNG",
            format!("Invalid dimensions: {}x{}", width, height),
        ));
    }

    // Validate color type and bit depth combination
    let valid_combination = match color_type {
        0 => matches!(bit_depth, 1 | 2 | 4 | 8 | 16), // Grayscale
        2 => matches!(bit_depth, 8 | 16),             // RGB
        3 => matches!(bit_depth, 1 | 2 | 4 | 8),      // Indexed
        4 => matches!(bit_depth, 8 | 16),             // Grayscale+Alpha
        6 => matches!(bit_depth, 8 | 16),             // RGBA
        _ => false,
    };

    if !valid_combination {
        return Err(FormatError::new(
            "PNG",
            format!(
                "Invalid color type ({}) and bit depth ({}) combination",
                color_type, bit_depth
            ),
        ));
    }

    // Validate compression method
    if compression_method != 0 {
        return Err(FormatError::new(
            "PNG",
            format!("Unknown compression method: {}", compression_method),
        ));
    }

    // Validate filter method
    if filter_method != 0 {
        return Err(FormatError::new(
            "PNG",
            format!("Unknown filter method: {}", filter_method),
        ));
    }

    // Validate interlace method
    if interlace_method > 1 {
        return Err(FormatError::new(
            "PNG",
            format!("Unknown interlace method: {}", interlace_method),
        ));
    }

    Ok(PngInfo {
        width,
        height,
        bit_depth,
        color_type,
        compression_method,
        filter_method,
        interlace_method,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_png_valid_rgba() {
        let png = create_test_png(256, 256, 8, 6); // RGBA
        let info = validate_png(&png).expect("Should parse valid PNG");

        assert_eq!(info.width, 256);
        assert_eq!(info.height, 256);
        assert_eq!(info.bit_depth, 8);
        assert_eq!(info.color_type, 6);
        assert_eq!(info.channels(), 4);
        assert_eq!(info.color_type_name(), "RGBA");
    }

    #[test]
    fn test_validate_png_grayscale() {
        let png = create_test_png(128, 64, 8, 0); // Grayscale
        let info = validate_png(&png).expect("Should parse grayscale PNG");

        assert_eq!(info.width, 128);
        assert_eq!(info.height, 64);
        assert_eq!(info.color_type, 0);
        assert_eq!(info.channels(), 1);
    }

    #[test]
    fn test_validate_png_too_short() {
        let data = vec![0u8; 10];
        let err = validate_png(&data).unwrap_err();
        assert_eq!(err.format, "PNG");
        assert!(err.message.contains("too short"));
    }

    #[test]
    fn test_validate_png_invalid_signature() {
        let mut png = create_test_png(64, 64, 8, 6);
        png[0] = 0x00; // Corrupt signature
        let err = validate_png(&png).unwrap_err();
        assert!(err.message.contains("signature"));
    }

    #[test]
    fn test_validate_png_invalid_color_bit_combo() {
        // RGB with 1-bit depth is invalid
        let mut png = create_test_png(64, 64, 1, 2);
        // Note: The test helper creates a valid combination, so we need to manually corrupt it
        png[24] = 1; // bit_depth
        png[25] = 2; // color_type RGB
        let err = validate_png(&png).unwrap_err();
        assert!(err.message.contains("color type") || err.message.contains("bit depth"));
    }

    fn create_test_png(width: u32, height: u32, bit_depth: u8, color_type: u8) -> Vec<u8> {
        let mut png = Vec::new();

        // PNG signature
        png.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

        // IHDR chunk
        png.extend_from_slice(&13u32.to_be_bytes()); // length
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&width.to_be_bytes());
        png.extend_from_slice(&height.to_be_bytes());
        png.push(bit_depth);
        png.push(color_type);
        png.push(0); // compression
        png.push(0); // filter
        png.push(0); // interlace
                     // CRC (simplified - just zeros for testing)
        png.extend_from_slice(&[0u8; 4]);

        // IEND chunk
        png.extend_from_slice(&0u32.to_be_bytes()); // length
        png.extend_from_slice(b"IEND");
        png.extend_from_slice(&[0u8; 4]); // CRC

        png
    }
}
