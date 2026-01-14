//! XM (Extended Module) file format validator.

use super::{extract_string, FormatError};

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
            format!(
                "File too short: {} bytes (minimum {} required)",
                data.len(),
                MIN_HEADER_SIZE
            ),
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
            format!(
                "Truncated header: declared size {} exceeds file",
                header_size
            ),
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
