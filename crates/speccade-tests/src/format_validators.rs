//! Binary format validators for test infrastructure.
//!
//! This module provides validators that parse binary file headers and return
//! structured information about file contents. Used for validating that
//! generated assets are correctly formatted.

use std::fmt;

/// Error type for format validation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatError {
    /// The format being validated.
    pub format: &'static str,
    /// Description of what went wrong.
    pub message: String,
    /// Byte offset where the error occurred, if applicable.
    pub offset: Option<usize>,
}

impl FormatError {
    /// Create a new format error.
    pub fn new(format: &'static str, message: impl Into<String>) -> Self {
        Self {
            format,
            message: message.into(),
            offset: None,
        }
    }

    /// Create a format error with a byte offset.
    pub fn at_offset(format: &'static str, message: impl Into<String>, offset: usize) -> Self {
        Self {
            format,
            message: message.into(),
            offset: Some(offset),
        }
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(offset) = self.offset {
            write!(f, "{} error at offset {}: {}", self.format, offset, self.message)
        } else {
            write!(f, "{} error: {}", self.format, self.message)
        }
    }
}

impl std::error::Error for FormatError {}

/// Information extracted from a WAV file header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WavInfo {
    /// Number of audio channels (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Sample rate in Hz (e.g., 44100, 48000).
    pub sample_rate: u32,
    /// Bits per sample (e.g., 8, 16, 24, 32).
    pub bits_per_sample: u16,
    /// Total number of samples (per channel).
    pub num_samples: usize,
    /// Audio format code (1 = PCM, 3 = IEEE float).
    pub audio_format: u16,
    /// Byte rate (sample_rate * channels * bits_per_sample / 8).
    pub byte_rate: u32,
    /// Block alignment (channels * bits_per_sample / 8).
    pub block_align: u16,
}

/// Validate WAV file format and extract header information.
///
/// Parses the RIFF/WAVE header structure and validates:
/// - RIFF chunk identifier
/// - WAVE format identifier
/// - fmt sub-chunk with audio parameters
/// - data sub-chunk presence
///
/// # Arguments
/// * `data` - Raw bytes of the WAV file
///
/// # Returns
/// * `Ok(WavInfo)` - Successfully parsed WAV file information
/// * `Err(FormatError)` - Invalid or corrupted WAV file
pub fn validate_wav(data: &[u8]) -> Result<WavInfo, FormatError> {
    const MIN_HEADER_SIZE: usize = 44;

    if data.len() < MIN_HEADER_SIZE {
        return Err(FormatError::new(
            "WAV",
            format!("File too short: {} bytes (minimum {} required)", data.len(), MIN_HEADER_SIZE),
        ));
    }

    // Check RIFF header
    if &data[0..4] != b"RIFF" {
        return Err(FormatError::at_offset(
            "WAV",
            format!("Invalid RIFF header: expected 'RIFF', got {:?}", &data[0..4]),
            0,
        ));
    }

    // Check WAVE format
    if &data[8..12] != b"WAVE" {
        return Err(FormatError::at_offset(
            "WAV",
            format!("Invalid WAVE format: expected 'WAVE', got {:?}", &data[8..12]),
            8,
        ));
    }

    // Find and parse fmt chunk
    let mut offset = 12;
    let mut fmt_found = false;
    let mut audio_format = 0u16;
    let mut channels = 0u16;
    let mut sample_rate = 0u32;
    let mut byte_rate = 0u32;
    let mut block_align = 0u16;
    let mut bits_per_sample = 0u16;

    while offset + 8 <= data.len() {
        let chunk_id = &data[offset..offset + 4];
        let chunk_size = u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as usize;

        if chunk_id == b"fmt " {
            if chunk_size < 16 {
                return Err(FormatError::at_offset(
                    "WAV",
                    format!("fmt chunk too small: {} bytes", chunk_size),
                    offset,
                ));
            }

            if offset + 8 + 16 > data.len() {
                return Err(FormatError::at_offset("WAV", "Truncated fmt chunk", offset));
            }

            let fmt_data = &data[offset + 8..];
            audio_format = u16::from_le_bytes([fmt_data[0], fmt_data[1]]);
            channels = u16::from_le_bytes([fmt_data[2], fmt_data[3]]);
            sample_rate = u32::from_le_bytes([fmt_data[4], fmt_data[5], fmt_data[6], fmt_data[7]]);
            byte_rate = u32::from_le_bytes([fmt_data[8], fmt_data[9], fmt_data[10], fmt_data[11]]);
            block_align = u16::from_le_bytes([fmt_data[12], fmt_data[13]]);
            bits_per_sample = u16::from_le_bytes([fmt_data[14], fmt_data[15]]);

            fmt_found = true;
        }

        if chunk_id == b"data" {
            if !fmt_found {
                return Err(FormatError::at_offset(
                    "WAV",
                    "data chunk found before fmt chunk",
                    offset,
                ));
            }

            let num_samples = if block_align > 0 {
                chunk_size / block_align as usize
            } else {
                0
            };

            return Ok(WavInfo {
                channels,
                sample_rate,
                bits_per_sample,
                num_samples,
                audio_format,
                byte_rate,
                block_align,
            });
        }

        // Move to next chunk (chunks are word-aligned)
        let padded_size = (chunk_size + 1) & !1;
        offset += 8 + padded_size;
    }

    if !fmt_found {
        return Err(FormatError::new("WAV", "Missing fmt chunk"));
    }

    Err(FormatError::new("WAV", "Missing data chunk"))
}

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
            format!("File too short: {} bytes (minimum {} required)", data.len(), MIN_HEADER_SIZE),
        ));
    }

    // Check PNG signature
    if data[0..8] != PNG_SIGNATURE {
        return Err(FormatError::at_offset(
            "PNG",
            "Invalid PNG signature",
            0,
        ));
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
        2 => matches!(bit_depth, 8 | 16),              // RGB
        3 => matches!(bit_depth, 1 | 2 | 4 | 8),       // Indexed
        4 => matches!(bit_depth, 8 | 16),              // Grayscale+Alpha
        6 => matches!(bit_depth, 8 | 16),              // RGBA
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

/// Information extracted from an XM (Extended Module) file header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmInfo {
    /// Module name (up to 20 characters).
    pub name: String,
    /// Tracker name that created the file.
    pub tracker_name: String,
    /// XM format version (typically 0x0104 for v1.04).
    pub version: u16,
    /// Number of channels (1-32 typical).
    pub num_channels: u8,
    /// Number of patterns in the module.
    pub num_patterns: u16,
    /// Number of instruments in the module.
    pub num_instruments: u16,
    /// Number of orders in the pattern order table.
    pub num_orders: u16,
    /// Default tempo (ticks per row).
    pub default_tempo: u16,
    /// Default BPM.
    pub default_bpm: u16,
}

/// Validate XM (Extended Module) file format and extract header information.
///
/// Parses the XM header to validate:
/// - XM identification string
/// - Version number
/// - Module parameters
///
/// # Arguments
/// * `data` - Raw bytes of the XM file
///
/// # Returns
/// * `Ok(XmInfo)` - Successfully parsed XM file information
/// * `Err(FormatError)` - Invalid or corrupted XM file
pub fn validate_xm(data: &[u8]) -> Result<XmInfo, FormatError> {
    const XM_HEADER_ID: &[u8; 17] = b"Extended Module: ";
    const MIN_HEADER_SIZE: usize = 80;

    if data.len() < MIN_HEADER_SIZE {
        return Err(FormatError::new(
            "XM",
            format!("File too short: {} bytes (minimum {} required)", data.len(), MIN_HEADER_SIZE),
        ));
    }

    // Check XM header ID
    if &data[0..17] != XM_HEADER_ID {
        return Err(FormatError::at_offset(
            "XM",
            "Invalid XM header identifier",
            0,
        ));
    }

    // Module name (bytes 17-36, null-padded)
    let name = extract_string(&data[17..37]);

    // Check for 0x1A marker at offset 37
    if data[37] != 0x1A {
        return Err(FormatError::at_offset(
            "XM",
            format!("Missing 0x1A marker, got 0x{:02X}", data[37]),
            37,
        ));
    }

    // Tracker name (bytes 38-57)
    let tracker_name = extract_string(&data[38..58]);

    // Version number (bytes 58-59, little-endian)
    let version = u16::from_le_bytes([data[58], data[59]]);

    // Header size (bytes 60-63)
    let header_size = u32::from_le_bytes([data[60], data[61], data[62], data[63]]) as usize;

    if 60 + header_size > data.len() {
        return Err(FormatError::new(
            "XM",
            format!("Truncated header: declared size {} exceeds file", header_size),
        ));
    }

    // Song length / number of orders (bytes 64-65)
    let num_orders = u16::from_le_bytes([data[64], data[65]]);

    // Restart position (bytes 66-67) - skipped

    // Number of channels (bytes 68-69)
    let num_channels = u16::from_le_bytes([data[68], data[69]]);

    // Number of patterns (bytes 70-71)
    let num_patterns = u16::from_le_bytes([data[70], data[71]]);

    // Number of instruments (bytes 72-73)
    let num_instruments = u16::from_le_bytes([data[72], data[73]]);

    // Flags (bytes 74-75) - skipped

    // Default tempo (bytes 76-77)
    let default_tempo = u16::from_le_bytes([data[76], data[77]]);

    // Default BPM (bytes 78-79)
    let default_bpm = u16::from_le_bytes([data[78], data[79]]);

    // Validate reasonable values
    if num_channels == 0 || num_channels > 128 {
        return Err(FormatError::new(
            "XM",
            format!("Invalid channel count: {}", num_channels),
        ));
    }

    Ok(XmInfo {
        name,
        tracker_name,
        version,
        num_channels: num_channels as u8,
        num_patterns,
        num_instruments,
        num_orders,
        default_tempo,
        default_bpm,
    })
}

/// Information extracted from an IT (Impulse Tracker) file header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItInfo {
    /// Module name (up to 26 characters).
    pub name: String,
    /// Number of orders in the order list.
    pub num_orders: u16,
    /// Number of instruments.
    pub num_instruments: u16,
    /// Number of samples.
    pub num_samples: u16,
    /// Number of patterns.
    pub num_patterns: u16,
    /// Tracker version that created the file.
    pub created_with: u16,
    /// Minimum compatible tracker version.
    pub compatible_with: u16,
    /// Global volume (0-128).
    pub global_volume: u8,
    /// Mix volume (0-128).
    pub mix_volume: u8,
    /// Initial tempo.
    pub initial_tempo: u8,
    /// Initial speed.
    pub initial_speed: u8,
}

/// Validate IT (Impulse Tracker) file format and extract header information.
///
/// Parses the IT header to validate:
/// - IT magic number ("IMPM")
/// - Module parameters
/// - Version information
///
/// # Arguments
/// * `data` - Raw bytes of the IT file
///
/// # Returns
/// * `Ok(ItInfo)` - Successfully parsed IT file information
/// * `Err(FormatError)` - Invalid or corrupted IT file
pub fn validate_it(data: &[u8]) -> Result<ItInfo, FormatError> {
    const IT_MAGIC: &[u8; 4] = b"IMPM";
    const MIN_HEADER_SIZE: usize = 192;

    if data.len() < MIN_HEADER_SIZE {
        return Err(FormatError::new(
            "IT",
            format!("File too short: {} bytes (minimum {} required)", data.len(), MIN_HEADER_SIZE),
        ));
    }

    // Check IT magic number
    if &data[0..4] != IT_MAGIC {
        return Err(FormatError::at_offset(
            "IT",
            format!("Invalid IT magic: expected 'IMPM', got {:?}", &data[0..4]),
            0,
        ));
    }

    // Song name (bytes 4-29, null-terminated)
    let name = extract_string(&data[4..30]);

    // Pattern highlight (bytes 30-31) - skipped

    // Number of orders (bytes 32-33)
    let num_orders = u16::from_le_bytes([data[32], data[33]]);

    // Number of instruments (bytes 34-35)
    let num_instruments = u16::from_le_bytes([data[34], data[35]]);

    // Number of samples (bytes 36-37)
    let num_samples = u16::from_le_bytes([data[36], data[37]]);

    // Number of patterns (bytes 38-39)
    let num_patterns = u16::from_le_bytes([data[38], data[39]]);

    // Created with tracker (bytes 40-41)
    let created_with = u16::from_le_bytes([data[40], data[41]]);

    // Compatible with tracker (bytes 42-43)
    let compatible_with = u16::from_le_bytes([data[42], data[43]]);

    // Flags (bytes 44-45) - skipped
    // Special (bytes 46-47) - skipped

    // Global volume (byte 48)
    let global_volume = data[48];

    // Mix volume (byte 49)
    let mix_volume = data[49];

    // Initial speed (byte 50)
    let initial_speed = data[50];

    // Initial tempo (byte 51)
    let initial_tempo = data[51];

    Ok(ItInfo {
        name,
        num_orders,
        num_instruments,
        num_samples,
        num_patterns,
        created_with,
        compatible_with,
        global_volume,
        mix_volume,
        initial_tempo,
        initial_speed,
    })
}

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
    const BIN_CHUNK_TYPE: u32 = 0x004E4942;  // "BIN\0" in little-endian

    if data.len() < MIN_HEADER_SIZE {
        return Err(FormatError::new(
            "GLB",
            format!("File too short: {} bytes (minimum {} required)", data.len(), MIN_HEADER_SIZE),
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
            format!("Declared length {} exceeds actual file size {}", length, data.len()),
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

/// Extract a null-terminated or space-padded string from a byte slice.
fn extract_string(data: &[u8]) -> String {
    // Find the end of the string (first null byte or end of slice)
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());

    // Convert to string and trim trailing spaces
    String::from_utf8_lossy(&data[..end])
        .trim_end()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== WAV Tests ==========

    #[test]
    fn test_validate_wav_valid_pcm16() {
        // Create a minimal valid WAV file (16-bit PCM, mono, 44100 Hz)
        let wav = create_test_wav(1, 44100, 16, 100);
        let info = validate_wav(&wav).expect("Should parse valid WAV");

        assert_eq!(info.channels, 1);
        assert_eq!(info.sample_rate, 44100);
        assert_eq!(info.bits_per_sample, 16);
        assert_eq!(info.audio_format, 1); // PCM
        assert_eq!(info.num_samples, 100);
    }

    #[test]
    fn test_validate_wav_stereo() {
        let wav = create_test_wav(2, 48000, 16, 50);
        let info = validate_wav(&wav).expect("Should parse stereo WAV");

        assert_eq!(info.channels, 2);
        assert_eq!(info.sample_rate, 48000);
        assert_eq!(info.num_samples, 50);
    }

    #[test]
    fn test_validate_wav_too_short() {
        let data = vec![0u8; 10];
        let err = validate_wav(&data).unwrap_err();
        assert_eq!(err.format, "WAV");
        assert!(err.message.contains("too short"));
    }

    #[test]
    fn test_validate_wav_invalid_riff() {
        let mut wav = create_test_wav(1, 44100, 16, 10);
        wav[0..4].copy_from_slice(b"XXXX");
        let err = validate_wav(&wav).unwrap_err();
        assert!(err.message.contains("RIFF"));
    }

    #[test]
    fn test_validate_wav_invalid_wave() {
        let mut wav = create_test_wav(1, 44100, 16, 10);
        wav[8..12].copy_from_slice(b"XXXX");
        let err = validate_wav(&wav).unwrap_err();
        assert!(err.message.contains("WAVE"));
    }

    // ========== PNG Tests ==========

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
        png[24] = 1;  // bit_depth
        png[25] = 2;  // color_type RGB
        let err = validate_png(&png).unwrap_err();
        assert!(err.message.contains("color type") || err.message.contains("bit depth"));
    }

    // ========== XM Tests ==========

    #[test]
    fn test_validate_xm_valid() {
        let xm = create_test_xm("Test Song", 8, 4, 2);
        let info = validate_xm(&xm).expect("Should parse valid XM");

        assert_eq!(info.name, "Test Song");
        assert_eq!(info.num_channels, 8);
        assert_eq!(info.num_patterns, 4);
        assert_eq!(info.num_instruments, 2);
    }

    #[test]
    fn test_validate_xm_too_short() {
        let data = vec![0u8; 30];
        let err = validate_xm(&data).unwrap_err();
        assert_eq!(err.format, "XM");
        assert!(err.message.contains("too short"));
    }

    #[test]
    fn test_validate_xm_invalid_header() {
        let mut xm = create_test_xm("Test", 4, 1, 1);
        xm[0..10].copy_from_slice(b"Not an XM!");
        let err = validate_xm(&xm).unwrap_err();
        assert!(err.message.contains("header"));
    }

    // ========== IT Tests ==========

    #[test]
    fn test_validate_it_valid() {
        let it = create_test_it("Test Module", 64, 8, 16, 4);
        let info = validate_it(&it).expect("Should parse valid IT");

        assert_eq!(info.name, "Test Module");
        assert_eq!(info.num_orders, 64);
        assert_eq!(info.num_instruments, 8);
        assert_eq!(info.num_samples, 16);
        assert_eq!(info.num_patterns, 4);
    }

    #[test]
    fn test_validate_it_too_short() {
        let data = vec![0u8; 50];
        let err = validate_it(&data).unwrap_err();
        assert_eq!(err.format, "IT");
        assert!(err.message.contains("too short"));
    }

    #[test]
    fn test_validate_it_invalid_magic() {
        let mut it = create_test_it("Test", 16, 2, 4, 2);
        it[0..4].copy_from_slice(b"XXXX");
        let err = validate_it(&it).unwrap_err();
        assert!(err.message.contains("magic") || err.message.contains("IMPM"));
    }

    // ========== GLB Tests ==========

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

    // ========== Helper Functions ==========

    fn create_test_wav(channels: u16, sample_rate: u32, bits_per_sample: u16, num_samples: usize) -> Vec<u8> {
        let block_align = channels * bits_per_sample / 8;
        let byte_rate = sample_rate * block_align as u32;
        let data_size = num_samples * block_align as usize;
        let file_size = 36 + data_size;

        let mut wav = Vec::with_capacity(44 + data_size);

        // RIFF header
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(file_size as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        // fmt chunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
        wav.extend_from_slice(&1u16.to_le_bytes());  // audio format (PCM)
        wav.extend_from_slice(&channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());

        // data chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(data_size as u32).to_le_bytes());
        wav.resize(wav.len() + data_size, 0);

        wav
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

    fn create_test_xm(name: &str, channels: u8, patterns: u16, instruments: u16) -> Vec<u8> {
        let mut xm = vec![0u8; 336]; // Minimum header + pattern order table

        // Header ID
        xm[0..17].copy_from_slice(b"Extended Module: ");

        // Module name (17-36)
        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(20);
        xm[17..17 + copy_len].copy_from_slice(&name_bytes[..copy_len]);

        // 0x1A marker
        xm[37] = 0x1A;

        // Tracker name (38-57)
        xm[38..49].copy_from_slice(b"SpecCade   ");

        // Version (1.04)
        xm[58..60].copy_from_slice(&0x0104u16.to_le_bytes());

        // Header size (from offset 60)
        xm[60..64].copy_from_slice(&276u32.to_le_bytes());

        // Song length (number of orders)
        xm[64..66].copy_from_slice(&16u16.to_le_bytes());

        // Restart position
        xm[66..68].copy_from_slice(&0u16.to_le_bytes());

        // Number of channels
        xm[68..70].copy_from_slice(&(channels as u16).to_le_bytes());

        // Number of patterns
        xm[70..72].copy_from_slice(&patterns.to_le_bytes());

        // Number of instruments
        xm[72..74].copy_from_slice(&instruments.to_le_bytes());

        // Flags
        xm[74..76].copy_from_slice(&1u16.to_le_bytes());

        // Default tempo
        xm[76..78].copy_from_slice(&6u16.to_le_bytes());

        // Default BPM
        xm[78..80].copy_from_slice(&125u16.to_le_bytes());

        xm
    }

    fn create_test_it(name: &str, orders: u16, instruments: u16, samples: u16, patterns: u16) -> Vec<u8> {
        let mut it = vec![0u8; 192 + orders as usize];

        // Magic
        it[0..4].copy_from_slice(b"IMPM");

        // Name (4-29)
        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(26);
        it[4..4 + copy_len].copy_from_slice(&name_bytes[..copy_len]);

        // Pattern highlight
        it[30..32].copy_from_slice(&0x1004u16.to_le_bytes());

        // OrdNum
        it[32..34].copy_from_slice(&orders.to_le_bytes());

        // InsNum
        it[34..36].copy_from_slice(&instruments.to_le_bytes());

        // SmpNum
        it[36..38].copy_from_slice(&samples.to_le_bytes());

        // PatNum
        it[38..40].copy_from_slice(&patterns.to_le_bytes());

        // Created with (Impulse Tracker 2.14)
        it[40..42].copy_from_slice(&0x0214u16.to_le_bytes());

        // Compatible with
        it[42..44].copy_from_slice(&0x0200u16.to_le_bytes());

        // Flags
        it[44..46].copy_from_slice(&0x0009u16.to_le_bytes());

        // Special
        it[46..48].copy_from_slice(&0x0006u16.to_le_bytes());

        // Global volume
        it[48] = 128;

        // Mix volume
        it[49] = 48;

        // Initial speed
        it[50] = 6;

        // Initial tempo
        it[51] = 125;

        // Pan separation
        it[52] = 128;

        // Pitch wheel depth
        it[53] = 0;

        // Message length and offset (54-59)
        // Reserved (60-63)

        // Channel pan (64-127)
        for i in 64..128 {
            it[i] = 32; // Center pan
        }

        // Channel volume (128-191)
        for i in 128..192 {
            it[i] = 64; // Full volume
        }

        it
    }

    fn create_test_glb(json: &str, bin_size: Option<usize>) -> Vec<u8> {
        let json_bytes = json.as_bytes();
        let json_padded_len = (json_bytes.len() + 3) & !3; // 4-byte aligned

        let bin_padded_len = bin_size.map(|s| (s + 3) & !3).unwrap_or(0);

        let total_len = 12 + 8 + json_padded_len + if bin_size.is_some() { 8 + bin_padded_len } else { 0 };

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
        for _ in json_bytes.len()..json_padded_len {
            glb.push(0x20); // Space padding for JSON
        }

        // Binary chunk (optional)
        if let Some(size) = bin_size {
            glb.extend_from_slice(&(size as u32).to_le_bytes());
            glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // "BIN\0"
            glb.resize(glb.len() + bin_padded_len, 0);
        }

        glb
    }

    // ========== FormatError Tests ==========

    #[test]
    fn test_format_error_display() {
        let err = FormatError::new("TEST", "something went wrong");
        assert_eq!(format!("{}", err), "TEST error: something went wrong");

        let err_offset = FormatError::at_offset("WAV", "bad header", 12);
        assert_eq!(format!("{}", err_offset), "WAV error at offset 12: bad header");
    }

    #[test]
    fn test_extract_string_null_terminated() {
        let data = b"Hello\0World";
        assert_eq!(extract_string(data), "Hello");
    }

    #[test]
    fn test_extract_string_space_padded() {
        let data = b"Hello     ";
        assert_eq!(extract_string(data), "Hello");
    }

    #[test]
    fn test_extract_string_no_terminator() {
        let data = b"Hello";
        assert_eq!(extract_string(data), "Hello");
    }
}
