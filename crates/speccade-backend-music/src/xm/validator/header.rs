//! XM header validation.

use super::constants::*;
use super::error::{XmFormatError, XmWarning};
use super::types::{XmHeaderInfo, XmValidationReport};
use crate::xm::header::{XM_MAX_INSTRUMENTS, XM_MAX_PATTERNS};

/// Extract a null-terminated or space-padded string from a byte slice.
pub(super) fn extract_string(data: &[u8]) -> String {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..end]).trim_end().to_string()
}

/// Validate the XM header (offsets 0x00-0x3C and extended header).
pub(super) fn validate_header(
    data: &[u8],
    report: &mut XmValidationReport,
) -> Result<XmHeaderInfo, XmFormatError> {
    // Check minimum file size
    if data.len() < XM_MIN_FILE_SIZE {
        return Err(XmFormatError::FileTooSmall {
            size: data.len(),
            minimum: XM_MIN_FILE_SIZE,
        });
    }

    // Offset 0x00-0x10: ID text "Extended Module: "
    if &data[0..17] != XM_ID_TEXT {
        return Err(XmFormatError::InvalidIdText {
            found: data[0..17].to_vec(),
        });
    }

    // Offset 0x11-0x24: Module name (20 bytes)
    let name = extract_string(&data[17..37]);

    // Offset 0x25: Magic byte 0x1A
    if data[37] != XM_MAGIC_BYTE {
        return Err(XmFormatError::MissingMagicByte {
            found: data[37],
            offset: 37,
        });
    }

    // Offset 0x26-0x39: Tracker name (20 bytes)
    let tracker_name = extract_string(&data[38..58]);

    // Check for non-standard tracker name
    if !tracker_name.starts_with("FastTracker") && !tracker_name.is_empty() {
        report.add_warning(XmWarning::NonStandardTrackerName {
            name: tracker_name.clone(),
        });
    }

    // Offset 0x3A-0x3B: Version number
    let version = u16::from_le_bytes([data[58], data[59]]);
    if version != XM_VERSION_104 {
        // We'll accept it but log if it's not 1.04
        if !(0x0100..=0x0104).contains(&version) {
            return Err(XmFormatError::UnsupportedVersion { version });
        }
    }

    // Check if we have enough data for extended header
    if data.len() < XM_FULL_HEADER_SIZE {
        return Err(XmFormatError::FileTooSmall {
            size: data.len(),
            minimum: XM_FULL_HEADER_SIZE,
        });
    }

    // Offset 0x3C-0x3F: Header size (from offset 60)
    let header_size = u32::from_le_bytes([data[60], data[61], data[62], data[63]]);
    if header_size != XM_STANDARD_HEADER_SIZE {
        report.add_warning(XmWarning::NonStandardHeaderSize {
            size: header_size,
            standard: XM_STANDARD_HEADER_SIZE,
        });
    }

    // Extended header fields (offsets are relative to offset 60)
    // Song length at offset 64-65
    let song_length = u16::from_le_bytes([data[64], data[65]]);
    if song_length == 0 || song_length > 256 {
        return Err(XmFormatError::InvalidSongLength {
            length: song_length,
        });
    }

    // Restart position at offset 66-67
    let restart_position = u16::from_le_bytes([data[66], data[67]]);
    if restart_position >= song_length {
        return Err(XmFormatError::InvalidRestartPosition {
            restart: restart_position,
            song_length,
        });
    }

    // Number of channels at offset 68-69
    let num_channels = u16::from_le_bytes([data[68], data[69]]);
    if !(XM_MIN_CHANNELS..=XM_VALIDATOR_MAX_CHANNELS).contains(&num_channels) {
        return Err(XmFormatError::InvalidChannelCount {
            channels: num_channels,
            min: XM_MIN_CHANNELS,
            max: XM_VALIDATOR_MAX_CHANNELS,
        });
    }

    // Number of patterns at offset 70-71
    let num_patterns = u16::from_le_bytes([data[70], data[71]]);
    if num_patterns > XM_MAX_PATTERNS {
        return Err(XmFormatError::InvalidPatternCount {
            patterns: num_patterns,
            max: XM_MAX_PATTERNS,
        });
    }

    // Number of instruments at offset 72-73
    let num_instruments = u16::from_le_bytes([data[72], data[73]]);
    if num_instruments > XM_MAX_INSTRUMENTS {
        return Err(XmFormatError::InvalidInstrumentCount {
            instruments: num_instruments,
            max: XM_MAX_INSTRUMENTS,
        });
    }

    // Flags at offset 74-75
    let flags = u16::from_le_bytes([data[74], data[75]]);
    let linear_frequency_table = (flags & 1) != 0;

    // Default tempo at offset 76-77
    let default_tempo = u16::from_le_bytes([data[76], data[77]]);
    if !(XM_MIN_TEMPO..=XM_MAX_TEMPO).contains(&default_tempo) {
        return Err(XmFormatError::InvalidTempo {
            tempo: default_tempo,
            min: XM_MIN_TEMPO,
            max: XM_MAX_TEMPO,
        });
    }

    // Default BPM at offset 78-79
    let default_bpm = u16::from_le_bytes([data[78], data[79]]);
    if !(XM_MIN_BPM..=XM_MAX_BPM).contains(&default_bpm) {
        return Err(XmFormatError::InvalidBpm {
            bpm: default_bpm,
            min: XM_MIN_BPM,
            max: XM_MAX_BPM,
        });
    }

    // Check for unusual tempo/BPM combinations
    if default_tempo > 20 || !(50..=200).contains(&default_bpm) {
        report.add_warning(XmWarning::UnusualTempoBpm {
            tempo: default_tempo,
            bpm: default_bpm,
        });
    }

    // Pattern order table at offset 80-335 (256 bytes)
    let mut pattern_order = [0u8; 256];
    pattern_order.copy_from_slice(&data[80..336]);

    // Validate pattern order entries
    for (i, pattern) in pattern_order
        .iter()
        .copied()
        .take(song_length as usize)
        .enumerate()
    {
        if pattern as u16 >= num_patterns && num_patterns > 0 {
            return Err(XmFormatError::InvalidPatternOrder {
                position: i,
                pattern,
                max_pattern: num_patterns,
            });
        }
    }

    Ok(XmHeaderInfo {
        name,
        tracker_name,
        version,
        header_size,
        song_length,
        restart_position,
        num_channels,
        num_patterns,
        num_instruments,
        flags,
        linear_frequency_table,
        default_tempo,
        default_bpm,
        pattern_order,
    })
}
