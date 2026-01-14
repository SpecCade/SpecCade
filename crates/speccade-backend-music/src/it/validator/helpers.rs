//! Helper functions for IT validation.

/// Extract a null-terminated string from a byte slice.
pub(super) fn extract_string(data: &[u8]) -> String {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..end]).trim_end().to_string()
}
