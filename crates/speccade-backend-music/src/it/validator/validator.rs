//! Main IT validator implementation.

use super::error::{ItErrorCategory, ItFormatError};
use super::helpers::extract_string;
use super::info::{ItFlags, ItHeaderInfo, ItSpecialFlags};
use super::instrument_validator::validate_instruments;
use super::pattern_validator::validate_patterns;
use super::report::ItValidationReport;
use super::sample_validator::validate_samples;

/// IT format validator.
///
/// Validates IT files against the ITTECH.TXT specification.
pub struct ItValidator;

impl ItValidator {
    /// Validate an IT file and return a comprehensive report.
    ///
    /// This performs complete validation including:
    /// - Header magic and all fields
    /// - Order list validity
    /// - Instrument headers and envelopes
    /// - Sample headers and data references
    /// - Pattern headers and packed data
    ///
    /// # Arguments
    /// * `data` - Raw bytes of the IT file
    ///
    /// # Returns
    /// * `Ok(ItValidationReport)` - Validation completed (check `is_valid` field)
    /// * `Err(ItFormatError)` - Fatal error preventing validation
    pub fn validate(data: &[u8]) -> Result<ItValidationReport, ItFormatError> {
        let mut report = ItValidationReport::new();

        // Validate minimum size
        if data.len() < 192 {
            return Err(ItFormatError::new(
                ItErrorCategory::Structure,
                format!(
                    "File too small: {} bytes (minimum 192 bytes for header)",
                    data.len()
                ),
            ));
        }

        // Validate and extract header
        let header = Self::validate_header(data, &mut report)?;
        report.header = Some(header.clone());

        // Validate order list
        Self::validate_order_list(data, &header, &mut report)?;

        // Calculate offset table location
        let offset_table_start = 192 + header.order_count as usize;

        // Validate instruments
        validate_instruments(
            data,
            header.instrument_count,
            offset_table_start,
            &mut report,
        )?;

        // Validate samples
        let sample_offsets_start = offset_table_start + (header.instrument_count as usize * 4);
        validate_samples(data, header.sample_count, sample_offsets_start, &mut report)?;

        // Validate patterns
        let pattern_offsets_start = sample_offsets_start + (header.sample_count as usize * 4);
        validate_patterns(
            data,
            header.pattern_count,
            pattern_offsets_start,
            &mut report,
        )?;

        // Validate message if present
        if header.special.message_attached && header.message_length > 0 {
            Self::validate_message(data, &header, &mut report)?;
        }

        // Set validity based on errors
        report.is_valid = report.errors.is_empty();

        Ok(report)
    }

    /// Validate just the header and return basic info.
    pub fn validate_header_only(data: &[u8]) -> Result<ItHeaderInfo, ItFormatError> {
        if data.len() < 192 {
            return Err(ItFormatError::new(
                ItErrorCategory::Structure,
                format!(
                    "File too small: {} bytes (minimum 192 bytes for header)",
                    data.len()
                ),
            ));
        }

        let mut report = ItValidationReport::new();
        Self::validate_header(data, &mut report)
    }

    fn validate_header(
        data: &[u8],
        report: &mut ItValidationReport,
    ) -> Result<ItHeaderInfo, ItFormatError> {
        // Check magic "IMPM" at offset 0x00
        if &data[0..4] != b"IMPM" {
            return Err(ItFormatError::field_at_offset(
                ItErrorCategory::Header,
                "magic",
                format!(
                    "Invalid magic: expected 'IMPM' (0x49 0x4D 0x50 0x4D), got {:02X?}",
                    &data[0..4]
                ),
                0,
            ));
        }

        // Extract song name (26 bytes at 0x04)
        let name = extract_string(&data[0x04..0x1E]);

        // Pattern highlight at 0x1E (PHilight_Minor, PHilight_Major)
        let pattern_highlight = (data[0x1E], data[0x1F]);

        // Order count at 0x20
        let order_count = u16::from_le_bytes([data[0x20], data[0x21]]);

        // Instrument count at 0x22
        let instrument_count = u16::from_le_bytes([data[0x22], data[0x23]]);
        if instrument_count > 99 {
            report.add_warning(
                format!(
                    "Instrument count {} exceeds IT maximum of 99",
                    instrument_count
                ),
                Some(0x22),
            );
        }

        // Sample count at 0x24
        let sample_count = u16::from_le_bytes([data[0x24], data[0x25]]);
        if sample_count > 99 {
            report.add_warning(
                format!("Sample count {} exceeds IT maximum of 99", sample_count),
                Some(0x24),
            );
        }

        // Pattern count at 0x26
        let pattern_count = u16::from_le_bytes([data[0x26], data[0x27]]);
        if pattern_count > 200 {
            report.add_warning(
                format!("Pattern count {} exceeds IT maximum of 200", pattern_count),
                Some(0x26),
            );
        }

        // Created with tracker version at 0x28
        let created_with = u16::from_le_bytes([data[0x28], data[0x29]]);

        // Compatible with version at 0x2A
        let compatible_with = u16::from_le_bytes([data[0x2A], data[0x2B]]);

        // Flags at 0x2C
        let flags_raw = u16::from_le_bytes([data[0x2C], data[0x2D]]);
        let flags = ItFlags::from_u16(flags_raw);

        // Special at 0x2E
        let special_raw = u16::from_le_bytes([data[0x2E], data[0x2F]]);
        let special = ItSpecialFlags::from_u16(special_raw);

        // Global volume at 0x30
        let global_volume = data[0x30];
        if global_volume > 128 {
            report.add_error(ItFormatError::field_at_offset(
                ItErrorCategory::Header,
                "global_volume",
                format!("Global volume {} exceeds maximum of 128", global_volume),
                0x30,
            ));
        }

        // Mix volume at 0x31
        let mix_volume = data[0x31];
        if mix_volume > 128 {
            report.add_warning(
                format!("Mix volume {} exceeds typical maximum of 128", mix_volume),
                Some(0x31),
            );
        }

        // Initial speed at 0x32
        let initial_speed = data[0x32];
        if initial_speed == 0 {
            report.add_error(ItFormatError::field_at_offset(
                ItErrorCategory::Header,
                "initial_speed",
                "Initial speed cannot be 0",
                0x32,
            ));
        }

        // Initial tempo at 0x33
        let initial_tempo = data[0x33];
        if initial_tempo < 32 {
            report.add_warning(
                format!("Initial tempo {} is below minimum of 32", initial_tempo),
                Some(0x33),
            );
        }

        // Panning separation at 0x34
        let panning_separation = data[0x34];
        if panning_separation > 128 {
            report.add_warning(
                format!(
                    "Panning separation {} exceeds maximum of 128",
                    panning_separation
                ),
                Some(0x34),
            );
        }

        // Pitch wheel depth at 0x35
        let pitch_wheel_depth = data[0x35];

        // Message length at 0x36
        let message_length = u16::from_le_bytes([data[0x36], data[0x37]]);

        // Message offset at 0x38
        let message_offset = u32::from_le_bytes([data[0x38], data[0x39], data[0x3A], data[0x3B]]);

        // Reserved at 0x3C-0x3F (should be 0, but we just skip it)

        // Channel pan positions at 0x40 (64 bytes)
        let mut channel_pan = [0u8; 64];
        channel_pan.copy_from_slice(&data[0x40..0x80]);

        // Validate channel panning values
        for (i, &pan) in channel_pan.iter().enumerate() {
            // Values 0-64 are valid panning, 100 = surround, 128+ = channel disabled
            if pan > 64 && pan < 100 && pan < 128 {
                report.add_warning(
                    format!(
                        "Channel {} panning value {} is in undefined range (65-99)",
                        i, pan
                    ),
                    Some(0x40 + i),
                );
            }
        }

        // Channel volume at 0x80 (64 bytes)
        let mut channel_vol = [0u8; 64];
        channel_vol.copy_from_slice(&data[0x80..0xC0]);

        // Validate channel volumes
        for (i, &vol) in channel_vol.iter().enumerate() {
            if vol > 64 {
                report.add_warning(
                    format!("Channel {} volume {} exceeds maximum of 64", i, vol),
                    Some(0x80 + i),
                );
            }
        }

        Ok(ItHeaderInfo {
            name,
            pattern_highlight,
            order_count,
            instrument_count,
            sample_count,
            pattern_count,
            created_with,
            compatible_with,
            flags,
            special,
            global_volume,
            mix_volume,
            initial_speed,
            initial_tempo,
            panning_separation,
            pitch_wheel_depth,
            message_length,
            message_offset,
            channel_pan,
            channel_vol,
        })
    }

    fn validate_order_list(
        data: &[u8],
        header: &ItHeaderInfo,
        report: &mut ItValidationReport,
    ) -> Result<(), ItFormatError> {
        let order_start = 0xC0;
        let order_end = order_start + header.order_count as usize;

        if order_end > data.len() {
            return Err(ItFormatError::at_offset(
                ItErrorCategory::OrderList,
                format!(
                    "Order list extends beyond file: needs {} bytes, file has {}",
                    order_end,
                    data.len()
                ),
                order_start,
            ));
        }

        let orders = &data[order_start..order_end];
        report.orders = orders.to_vec();

        for (i, &order) in orders.iter().enumerate() {
            // 254 = skip marker (+++), 255 = end of order list (---)
            if order != 254 && order != 255 && order as u16 >= header.pattern_count {
                report.add_warning(
                    format!(
                        "Order {} references pattern {} but only {} patterns exist",
                        i, order, header.pattern_count
                    ),
                    Some(order_start + i),
                );
            }
        }

        Ok(())
    }

    fn validate_message(
        data: &[u8],
        header: &ItHeaderInfo,
        report: &mut ItValidationReport,
    ) -> Result<(), ItFormatError> {
        let msg_offset = header.message_offset as usize;
        let msg_length = header.message_length as usize;

        if msg_offset + msg_length > data.len() {
            report.add_error(ItFormatError::at_offset(
                ItErrorCategory::Message,
                format!(
                    "Message extends beyond file: {} bytes at offset {}, file has {}",
                    msg_length,
                    msg_offset,
                    data.len()
                ),
                msg_offset,
            ));
        }

        Ok(())
    }
}
