//! Binary format validators for test infrastructure.
//!
//! This module provides validators that parse binary file headers and return
//! structured information about file contents. Used for validating that
//! generated assets are correctly formatted.

use std::fmt;

mod gltf;
mod it;
mod png;
mod wav;
mod xm;

// Re-export public types and functions
pub use gltf::{validate_glb, validate_gltf, GlbInfo, GltfInfo};
pub use it::{validate_it, ItInfo};
pub use png::{validate_png, PngInfo};
pub use wav::{validate_wav, WavInfo};
pub use xm::{validate_xm, XmInfo};

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
            write!(
                f,
                "{} error at offset {}: {}",
                self.format, offset, self.message
            )
        } else {
            write!(f, "{} error: {}", self.format, self.message)
        }
    }
}

impl std::error::Error for FormatError {}

/// Extract a null-terminated or space-padded string from a byte slice.
pub(crate) fn extract_string(data: &[u8]) -> String {
    // Find the end of the string (first null byte or end of slice)
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());

    // Convert to string and trim trailing spaces
    String::from_utf8_lossy(&data[..end]).trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_error_display() {
        let err = FormatError::new("TEST", "something went wrong");
        assert_eq!(format!("{}", err), "TEST error: something went wrong");

        let err_offset = FormatError::at_offset("WAV", "bad header", 12);
        assert_eq!(
            format!("{}", err_offset),
            "WAV error at offset 12: bad header"
        );
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
