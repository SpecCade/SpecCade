//! Pattern validation logic.

use super::error::{ItErrorCategory, ItFormatError};
use super::info::ItPatternInfo;
use super::report::ItValidationReport;

pub(super) fn validate_patterns(
    data: &[u8],
    pattern_count: u16,
    offset_table_start: usize,
    report: &mut ItValidationReport,
) -> Result<(), ItFormatError> {
    if pattern_count == 0 {
        return Ok(());
    }

    let table_end = offset_table_start + (pattern_count as usize * 4);
    if table_end > data.len() {
        return Err(ItFormatError::at_offset(
            ItErrorCategory::OffsetTable,
            format!(
                "Pattern offset table extends beyond file: needs {} bytes at offset {}, file has {}",
                pattern_count * 4,
                offset_table_start,
                data.len()
            ),
            offset_table_start,
        ));
    }

    for i in 0..pattern_count as usize {
        let offset_pos = offset_table_start + i * 4;
        let pattern_offset = u32::from_le_bytes([
            data[offset_pos],
            data[offset_pos + 1],
            data[offset_pos + 2],
            data[offset_pos + 3],
        ]);

        if pattern_offset == 0 {
            // Null pattern (empty)
            report.patterns.push(ItPatternInfo {
                index: i,
                offset: 0,
                packed_length: 0,
                num_rows: 64, // Default
            });
            continue;
        }

        match validate_single_pattern(data, i, pattern_offset, report) {
            Ok(info) => report.patterns.push(info),
            Err(e) => report.add_error(e),
        }
    }

    Ok(())
}

fn validate_single_pattern(
    data: &[u8],
    index: usize,
    offset: u32,
    report: &mut ItValidationReport,
) -> Result<ItPatternInfo, ItFormatError> {
    let offset = offset as usize;

    // Pattern header is 8 bytes
    if offset + 8 > data.len() {
        return Err(ItFormatError::at_offset(
            ItErrorCategory::Pattern,
            format!("Pattern {} header extends beyond file", index),
            offset,
        ));
    }

    let pat = &data[offset..];

    // Packed length at offset 0 (2 bytes)
    let packed_length = u16::from_le_bytes([pat[0], pat[1]]);

    // Number of rows at offset 2 (2 bytes)
    let num_rows = u16::from_le_bytes([pat[2], pat[3]]);

    // Validate row count
    if num_rows == 0 {
        report.add_error(ItFormatError::field_at_offset(
            ItErrorCategory::Pattern,
            "num_rows",
            format!("Pattern {} has 0 rows", index),
            offset + 2,
        ));
    }
    if num_rows > 200 {
        report.add_warning(
            format!("Pattern {} has {} rows (maximum is 200)", index, num_rows),
            Some(offset + 2),
        );
    }

    // Reserved bytes at offset 4 (4 bytes)

    // Validate packed data fits in file
    if offset + 8 + packed_length as usize > data.len() {
        return Err(ItFormatError::at_offset(
            ItErrorCategory::Pattern,
            format!(
                "Pattern {} data ({} bytes) extends beyond file",
                index, packed_length
            ),
            offset + 8,
        ));
    }

    // Validate pattern packing structure
    validate_pattern_packing(
        &data[offset + 8..offset + 8 + packed_length as usize],
        index,
        num_rows,
        offset + 8,
        report,
    );

    Ok(ItPatternInfo {
        index,
        offset: offset as u32,
        packed_length,
        num_rows,
    })
}

fn validate_pattern_packing(
    packed_data: &[u8],
    pattern_index: usize,
    expected_rows: u16,
    base_offset: usize,
    report: &mut ItValidationReport,
) {
    let mut pos = 0;
    let mut row_count = 0;
    let mut channel_masks = [0u8; 64];

    while pos < packed_data.len() {
        let channel_var = packed_data[pos];
        pos += 1;

        if channel_var == 0 {
            // End of row
            row_count += 1;
            continue;
        }

        let channel = (channel_var - 1) & 63;

        // Check if mask variable follows
        let mask = if channel_var & 0x80 != 0 {
            if pos >= packed_data.len() {
                report.add_error(ItFormatError::at_offset(
                    ItErrorCategory::Pattern,
                    format!("Pattern {} truncated: expected mask byte", pattern_index),
                    base_offset + pos,
                ));
                return;
            }
            let m = packed_data[pos];
            pos += 1;
            channel_masks[channel as usize] = m;
            m
        } else {
            channel_masks[channel as usize]
        };

        // Count bytes to skip based on mask
        let mut bytes_to_skip = 0;
        if mask & 0x01 != 0 {
            bytes_to_skip += 1;
        } // Note
        if mask & 0x02 != 0 {
            bytes_to_skip += 1;
        } // Instrument
        if mask & 0x04 != 0 {
            bytes_to_skip += 1;
        } // Volume
        if mask & 0x08 != 0 {
            bytes_to_skip += 2;
        } // Effect + param

        if pos + bytes_to_skip > packed_data.len() {
            report.add_error(ItFormatError::at_offset(
                ItErrorCategory::Pattern,
                format!(
                    "Pattern {} truncated: expected {} more bytes for channel data",
                    pattern_index, bytes_to_skip
                ),
                base_offset + pos,
            ));
            return;
        }

        // Validate note value if present
        if mask & 0x01 != 0 {
            let note = packed_data[pos];
            // Valid notes: 0-119 (C-0 to B-9), 254 (note cut), 255 (note off)
            if note > 119 && note != 254 && note != 255 {
                report.add_warning(
                    format!(
                        "Pattern {} has invalid note value {} on channel {}",
                        pattern_index, note, channel
                    ),
                    Some(base_offset + pos),
                );
            }
        }

        pos += bytes_to_skip;
    }

    if row_count != expected_rows {
        report.add_warning(
            format!(
                "Pattern {} has {} rows in packed data but header declares {}",
                pattern_index, row_count, expected_rows
            ),
            Some(base_offset),
        );
    }
}
