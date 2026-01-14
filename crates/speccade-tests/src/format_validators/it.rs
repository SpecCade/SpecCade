//! IT (Impulse Tracker) file format validator.

use super::{extract_string, FormatError};

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
            format!(
                "File too short: {} bytes (minimum {} required)",
                data.len(),
                MIN_HEADER_SIZE
            ),
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

#[cfg(test)]
mod tests {
    use super::*;

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

    fn create_test_it(
        name: &str,
        orders: u16,
        instruments: u16,
        samples: u16,
        patterns: u16,
    ) -> Vec<u8> {
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
        it[64..128].fill(32); // Center pan

        // Channel volume (128-191)
        it[128..192].fill(64); // Full volume

        it
    }
}
