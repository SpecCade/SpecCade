//! Constants for XM format validation.

// Re-use constants from header module to avoid duplication
use crate::xm::header::{
    XM_HEADER_SIZE, XM_MAGIC, XM_MAX_CHANNELS as XM_MAX_CHANNELS_U8, XM_VERSION,
};

// Aliases for consistency within validator
/// XM ID text (alias for XM_MAGIC).
pub const XM_ID_TEXT: &[u8; 17] = XM_MAGIC;

/// XM version 1.04 (alias for XM_VERSION).
pub const XM_VERSION_104: u16 = XM_VERSION;

/// Standard header size (alias for XM_HEADER_SIZE).
pub const XM_STANDARD_HEADER_SIZE: u32 = XM_HEADER_SIZE;

/// Maximum number of channels as u16 for comparison.
pub const XM_VALIDATOR_MAX_CHANNELS: u16 = XM_MAX_CHANNELS_U8 as u16;

// Additional validation-specific constants

/// Required byte at offset 0x25 (37).
pub const XM_MAGIC_BYTE: u8 = 0x1A;

/// Minimum number of channels.
pub const XM_MIN_CHANNELS: u16 = 1;

/// Minimum pattern rows.
pub const XM_MIN_PATTERN_ROWS: u16 = 1;

/// Maximum envelope points.
pub const XM_MAX_ENVELOPE_POINTS: u8 = 12;

/// Maximum volume value.
pub const XM_MAX_VOLUME: u8 = 64;

/// Maximum panning value.
pub const XM_MAX_PANNING: u8 = 255;

/// Maximum BPM value.
pub const XM_MAX_BPM: u16 = 255;

/// Minimum BPM value.
pub const XM_MIN_BPM: u16 = 32;

/// Maximum tempo (ticks per row).
pub const XM_MAX_TEMPO: u16 = 31;

/// Minimum tempo.
pub const XM_MIN_TEMPO: u16 = 1;

/// Note off value.
pub const XM_NOTE_OFF: u8 = 97;

/// Maximum note value (excluding note-off).
pub const XM_MAX_NOTE: u8 = 96;

/// Minimum note value.
pub const XM_MIN_NOTE: u8 = 1;

/// Minimum file size for a valid XM file (header only).
pub const XM_MIN_FILE_SIZE: usize = 60;

/// Full header size including pattern order table.
pub const XM_FULL_HEADER_SIZE: usize = 336; // 60 + 276 = 336

/// Pattern header size.
pub const XM_PATTERN_HEADER_SIZE: u32 = 9;

/// Packing byte high bit.
pub const XM_PACKING_FLAG: u8 = 0x80;
