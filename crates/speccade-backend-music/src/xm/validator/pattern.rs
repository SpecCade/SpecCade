//! XM pattern validation.

use super::constants::*;
use super::error::{XmFormatError, XmWarning};
use super::types::{XmHeaderInfo, XmPatternInfo, XmValidationReport};
use crate::xm::header::XM_MAX_PATTERN_ROWS;

/// Validate all patterns and return the offset after the last pattern.
pub(super) fn validate_patterns(
    data: &[u8],
    start_offset: usize,
    header: &XmHeaderInfo,
    report: &mut XmValidationReport,
) -> Result<usize, XmFormatError> {
    let mut offset = start_offset;

    for pattern_idx in 0..header.num_patterns as usize {
        // Check if we have enough data for pattern header
        if offset + 9 > data.len() {
            return Err(XmFormatError::FileTruncated {
                expected: offset + 9,
                actual: data.len(),
            });
        }

        // Pattern header length (4 bytes)
        let header_length = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);

        // Packing type (1 byte, should be 0)
        let packing_type = data[offset + 4];
        if packing_type != 0 {
            return Err(XmFormatError::InvalidPackingType {
                found: packing_type,
                pattern: pattern_idx,
            });
        }

        // Number of rows (2 bytes)
        let num_rows = u16::from_le_bytes([data[offset + 5], data[offset + 6]]);
        if !(XM_MIN_PATTERN_ROWS..=XM_MAX_PATTERN_ROWS).contains(&num_rows) {
            return Err(XmFormatError::PatternError {
                pattern_index: pattern_idx,
                message: format!(
                    "Invalid row count {} (must be {}-{})",
                    num_rows, XM_MIN_PATTERN_ROWS, XM_MAX_PATTERN_ROWS
                ),
            });
        }

        // Packed pattern data size (2 bytes)
        let packed_size = u16::from_le_bytes([data[offset + 7], data[offset + 8]]);

        let is_empty = packed_size == 0;
        if is_empty {
            report.add_warning(XmWarning::EmptyPattern {
                pattern_index: pattern_idx,
            });
        }

        // Validate pattern data if not empty
        if packed_size > 0 {
            let data_offset = offset + header_length as usize;
            let data_end = data_offset + packed_size as usize;

            if data_end > data.len() {
                return Err(XmFormatError::FileTruncated {
                    expected: data_end,
                    actual: data.len(),
                });
            }

            // Validate packed pattern data
            validate_pattern_data(
                &data[data_offset..data_end],
                pattern_idx,
                num_rows,
                header.num_channels,
                report,
            )?;
        }

        report.patterns.push(XmPatternInfo {
            index: pattern_idx,
            header_length,
            packing_type,
            num_rows,
            packed_size,
            is_empty,
        });

        // Move to next pattern
        offset += header_length as usize + packed_size as usize;
    }

    Ok(offset)
}

/// Validate packed pattern data.
fn validate_pattern_data(
    data: &[u8],
    pattern_idx: usize,
    num_rows: u16,
    num_channels: u16,
    report: &mut XmValidationReport,
) -> Result<(), XmFormatError> {
    let mut pos = 0;
    let mut row = 0;
    let mut channel = 0;

    while pos < data.len() && row < num_rows {
        let byte = data[pos];
        pos += 1;

        // Check if this is a packing byte or a note value
        if byte & XM_PACKING_FLAG != 0 {
            // Packing byte: extract flags
            let has_note = (byte & 0x01) != 0;
            let has_instrument = (byte & 0x02) != 0;
            let has_volume = (byte & 0x04) != 0;
            let has_effect = (byte & 0x08) != 0;
            let has_effect_param = (byte & 0x10) != 0;

            // Validate note if present
            if has_note {
                if pos >= data.len() {
                    return Err(XmFormatError::FileTruncated {
                        expected: pos + 1,
                        actual: data.len(),
                    });
                }
                let note = data[pos];
                pos += 1;

                // Note must be 0 (none), 1-96 (C-0 to B-7), or 97 (key off)
                if note != 0 && (note > XM_NOTE_OFF) {
                    report.add_error(XmFormatError::InvalidNoteValue {
                        value: note,
                        pattern: pattern_idx,
                        row: row as usize,
                        channel: channel as usize,
                    });
                }
            }

            // Skip instrument if present
            if has_instrument {
                if pos >= data.len() {
                    return Err(XmFormatError::FileTruncated {
                        expected: pos + 1,
                        actual: data.len(),
                    });
                }
                pos += 1;
            }

            // Validate volume if present
            if has_volume {
                if pos >= data.len() {
                    return Err(XmFormatError::FileTruncated {
                        expected: pos + 1,
                        actual: data.len(),
                    });
                }
                let _volume = data[pos];
                pos += 1;
                // Volume column can have various commands, so we just validate it exists
            }

            // Skip effect if present
            if has_effect {
                if pos >= data.len() {
                    return Err(XmFormatError::FileTruncated {
                        expected: pos + 1,
                        actual: data.len(),
                    });
                }
                pos += 1;
            }

            // Skip effect parameter if present
            if has_effect_param {
                if pos >= data.len() {
                    return Err(XmFormatError::FileTruncated {
                        expected: pos + 1,
                        actual: data.len(),
                    });
                }
                pos += 1;
            }
        } else {
            // Uncompressed note: byte is the note value itself
            // This path is less common in properly packed XM files
            // We need 4 more bytes: instrument, volume, effect, effect_param
            if pos + 4 > data.len() {
                return Err(XmFormatError::FileTruncated {
                    expected: pos + 4,
                    actual: data.len(),
                });
            }

            // Validate note
            if byte != 0 && byte > XM_NOTE_OFF {
                report.add_error(XmFormatError::InvalidNoteValue {
                    value: byte,
                    pattern: pattern_idx,
                    row: row as usize,
                    channel: channel as usize,
                });
            }

            pos += 4; // Skip instrument, volume, effect, effect_param
        }

        // Move to next channel
        channel += 1;
        if channel >= num_channels {
            channel = 0;
            row += 1;
        }
    }

    Ok(())
}
