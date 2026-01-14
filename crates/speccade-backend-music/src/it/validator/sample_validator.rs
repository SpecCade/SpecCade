//! Sample validation logic.

use super::error::{ItErrorCategory, ItFormatError};
use super::helpers::extract_string;
use super::info::{ItConvertFlags, ItSampleFlags, ItSampleInfo};
use super::report::ItValidationReport;

pub(super) fn validate_samples(
    data: &[u8],
    sample_count: u16,
    offset_table_start: usize,
    report: &mut ItValidationReport,
) -> Result<(), ItFormatError> {
    if sample_count == 0 {
        return Ok(());
    }

    let table_end = offset_table_start + (sample_count as usize * 4);
    if table_end > data.len() {
        return Err(ItFormatError::at_offset(
            ItErrorCategory::OffsetTable,
            format!(
                "Sample offset table extends beyond file: needs {} bytes at offset {}, file has {}",
                sample_count * 4,
                offset_table_start,
                data.len()
            ),
            offset_table_start,
        ));
    }

    for i in 0..sample_count as usize {
        let offset_pos = offset_table_start + i * 4;
        let sample_offset = u32::from_le_bytes([
            data[offset_pos],
            data[offset_pos + 1],
            data[offset_pos + 2],
            data[offset_pos + 3],
        ]);

        if sample_offset == 0 {
            continue;
        }

        match validate_single_sample(data, i + 1, sample_offset, report) {
            Ok(info) => report.samples.push(info),
            Err(e) => report.add_error(e),
        }
    }

    Ok(())
}

fn validate_single_sample(
    data: &[u8],
    index: usize,
    offset: u32,
    report: &mut ItValidationReport,
) -> Result<ItSampleInfo, ItFormatError> {
    let offset = offset as usize;

    // Sample header is 80 bytes
    if offset + 80 > data.len() {
        return Err(ItFormatError::at_offset(
            ItErrorCategory::Sample,
            format!("Sample {} header extends beyond file", index),
            offset,
        ));
    }

    let smp = &data[offset..];

    // Check magic "IMPS" at offset 0x00
    if &smp[0..4] != b"IMPS" {
        return Err(ItFormatError::field_at_offset(
            ItErrorCategory::Sample,
            "magic",
            format!(
                "Sample {} has invalid magic: expected 'IMPS', got {:02X?}",
                index,
                &smp[0..4]
            ),
            offset,
        ));
    }

    // DOS filename at 0x04 (12 bytes)
    let filename = extract_string(&smp[0x04..0x10]);

    // Reserved byte at 0x10

    // Global volume at 0x11
    let global_volume = smp[0x11];
    if global_volume > 64 {
        report.add_warning(
            format!(
                "Sample {} global volume {} exceeds maximum of 64",
                index, global_volume
            ),
            Some(offset + 0x11),
        );
    }

    // Flags at 0x12
    let flags_raw = smp[0x12];
    let flags = ItSampleFlags::from_u8(flags_raw);

    // Default volume at 0x13
    let default_volume = smp[0x13];
    if default_volume > 64 {
        report.add_warning(
            format!(
                "Sample {} default volume {} exceeds maximum of 64",
                index, default_volume
            ),
            Some(offset + 0x13),
        );
    }

    // Sample name at 0x14 (26 bytes)
    let name = extract_string(&smp[0x14..0x2E]);

    // Convert flags at 0x2E
    let convert_raw = smp[0x2E];
    let convert_flags = ItConvertFlags::from_u8(convert_raw);

    // Default pan at 0x2F (bit 7 = use pan)
    let dfp = smp[0x2F];
    let default_pan = if dfp & 0x80 != 0 {
        Some(dfp & 0x7F)
    } else {
        None
    };

    // Length at 0x30 (4 bytes)
    let length = u32::from_le_bytes([smp[0x30], smp[0x31], smp[0x32], smp[0x33]]);

    // Loop begin at 0x34 (4 bytes)
    let loop_begin = u32::from_le_bytes([smp[0x34], smp[0x35], smp[0x36], smp[0x37]]);

    // Loop end at 0x38 (4 bytes)
    let loop_end = u32::from_le_bytes([smp[0x38], smp[0x39], smp[0x3A], smp[0x3B]]);

    // Validate loop points
    if flags.loop_enabled {
        if loop_begin > loop_end {
            report.add_error(ItFormatError::field_at_offset(
                ItErrorCategory::Sample,
                "loop_begin",
                format!(
                    "Sample {} loop begin ({}) > end ({})",
                    index, loop_begin, loop_end
                ),
                offset + 0x34,
            ));
        }
        if loop_end > length {
            report.add_warning(
                format!(
                    "Sample {} loop end ({}) > length ({})",
                    index, loop_end, length
                ),
                Some(offset + 0x38),
            );
        }
    }

    // C5 speed at 0x3C (4 bytes)
    let c5_speed = u32::from_le_bytes([smp[0x3C], smp[0x3D], smp[0x3E], smp[0x3F]]);
    if c5_speed > 9_999_999 {
        report.add_warning(
            format!(
                "Sample {} C5 speed {} exceeds maximum of 9,999,999",
                index, c5_speed
            ),
            Some(offset + 0x3C),
        );
    }

    // Sustain loop begin at 0x40 (4 bytes)
    let sustain_loop_begin = u32::from_le_bytes([smp[0x40], smp[0x41], smp[0x42], smp[0x43]]);

    // Sustain loop end at 0x44 (4 bytes)
    let sustain_loop_end = u32::from_le_bytes([smp[0x44], smp[0x45], smp[0x46], smp[0x47]]);

    // Validate sustain loop points
    if flags.sustain_loop_enabled && sustain_loop_begin > sustain_loop_end {
        report.add_error(ItFormatError::field_at_offset(
            ItErrorCategory::Sample,
            "sustain_loop_begin",
            format!(
                "Sample {} sustain loop begin ({}) > end ({})",
                index, sustain_loop_begin, sustain_loop_end
            ),
            offset + 0x40,
        ));
    }

    // Sample data pointer at 0x48 (4 bytes)
    let data_offset = u32::from_le_bytes([smp[0x48], smp[0x49], smp[0x4A], smp[0x4B]]);

    // Validate sample data pointer
    if flags.has_data && data_offset > 0 {
        let bytes_per_sample = if flags.is_16bit { 2 } else { 1 };
        let channels = if flags.is_stereo { 2 } else { 1 };
        let expected_size = length as usize * bytes_per_sample * channels;

        if data_offset as usize + expected_size > data.len() {
            report.add_warning(
                format!(
                    "Sample {} data ({} bytes at offset {}) extends beyond file ({})",
                    index,
                    expected_size,
                    data_offset,
                    data.len()
                ),
                Some(offset + 0x48),
            );
        }
    }

    // Vibrato speed at 0x4C
    let vibrato_speed = smp[0x4C];
    if vibrato_speed > 64 {
        report.add_warning(
            format!(
                "Sample {} vibrato speed {} exceeds maximum of 64",
                index, vibrato_speed
            ),
            Some(offset + 0x4C),
        );
    }

    // Vibrato depth at 0x4D
    let vibrato_depth = smp[0x4D];
    if vibrato_depth > 64 {
        report.add_warning(
            format!(
                "Sample {} vibrato depth {} exceeds maximum of 64",
                index, vibrato_depth
            ),
            Some(offset + 0x4D),
        );
    }

    // Vibrato rate at 0x4E
    let vibrato_rate = smp[0x4E];

    // Vibrato type at 0x4F
    let vibrato_type = smp[0x4F];
    if vibrato_type > 3 {
        report.add_warning(
            format!(
                "Sample {} vibrato type {} is invalid (should be 0-3)",
                index, vibrato_type
            ),
            Some(offset + 0x4F),
        );
    }

    Ok(ItSampleInfo {
        index,
        offset: offset as u32,
        name,
        filename,
        global_volume,
        flags,
        default_volume,
        default_pan,
        length,
        loop_begin,
        loop_end,
        c5_speed,
        sustain_loop_begin,
        sustain_loop_end,
        data_offset,
        vibrato_speed,
        vibrato_depth,
        vibrato_rate,
        vibrato_type,
        convert_flags,
    })
}
