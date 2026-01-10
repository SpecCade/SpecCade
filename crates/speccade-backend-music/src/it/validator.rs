//! Comprehensive IT (Impulse Tracker) format validator.
//!
//! This module provides thorough validation of IT files against the official
//! ITTECH.TXT specification from Impulse Tracker.
//!
//! Reference: <https://github.com/schismtracker/schismtracker/wiki/ITTECH.TXT>

use std::fmt;

// ============================================================================
// Error Types
// ============================================================================

/// Validation error for IT format.
#[derive(Debug, Clone, PartialEq)]
pub struct ItFormatError {
    /// Category of the error.
    pub category: ItErrorCategory,
    /// Detailed error message.
    pub message: String,
    /// Byte offset where error occurred (if applicable).
    pub offset: Option<usize>,
    /// Field name that failed validation (if applicable).
    pub field: Option<&'static str>,
}

impl ItFormatError {
    /// Create a new format error.
    pub fn new(category: ItErrorCategory, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
            offset: None,
            field: None,
        }
    }

    /// Create an error at a specific offset.
    pub fn at_offset(category: ItErrorCategory, message: impl Into<String>, offset: usize) -> Self {
        Self {
            category,
            message: message.into(),
            offset: Some(offset),
            field: None,
        }
    }

    /// Create an error for a specific field.
    pub fn for_field(
        category: ItErrorCategory,
        field: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            message: message.into(),
            offset: None,
            field: Some(field),
        }
    }

    /// Create an error at offset for a specific field.
    pub fn field_at_offset(
        category: ItErrorCategory,
        field: &'static str,
        message: impl Into<String>,
        offset: usize,
    ) -> Self {
        Self {
            category,
            message: message.into(),
            offset: Some(offset),
            field: Some(field),
        }
    }
}

impl fmt::Display for ItFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IT {} error", self.category)?;
        if let Some(field) = self.field {
            write!(f, " in field '{}'", field)?;
        }
        if let Some(offset) = self.offset {
            write!(f, " at offset 0x{:04X}", offset)?;
        }
        write!(f, ": {}", self.message)
    }
}

impl std::error::Error for ItFormatError {}

/// Category of IT format error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItErrorCategory {
    /// File structure error (too small, truncated).
    Structure,
    /// Header field error.
    Header,
    /// Order list error.
    OrderList,
    /// Instrument error.
    Instrument,
    /// Sample error.
    Sample,
    /// Pattern error.
    Pattern,
    /// Offset table error.
    OffsetTable,
    /// Message error.
    Message,
}

impl fmt::Display for ItErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Structure => write!(f, "structure"),
            Self::Header => write!(f, "header"),
            Self::OrderList => write!(f, "order list"),
            Self::Instrument => write!(f, "instrument"),
            Self::Sample => write!(f, "sample"),
            Self::Pattern => write!(f, "pattern"),
            Self::OffsetTable => write!(f, "offset table"),
            Self::Message => write!(f, "message"),
        }
    }
}

// ============================================================================
// Validation Report
// ============================================================================

/// Warning during IT validation (non-fatal issue).
#[derive(Debug, Clone)]
pub struct ItValidationWarning {
    /// Warning message.
    pub message: String,
    /// Byte offset (if applicable).
    pub offset: Option<usize>,
}

/// Comprehensive validation report for an IT file.
#[derive(Debug, Clone)]
pub struct ItValidationReport {
    /// Whether the file is valid.
    pub is_valid: bool,
    /// Header information extracted during validation.
    pub header: Option<ItHeaderInfo>,
    /// Instrument information.
    pub instruments: Vec<ItInstrumentInfo>,
    /// Sample information.
    pub samples: Vec<ItSampleInfo>,
    /// Pattern information.
    pub patterns: Vec<ItPatternInfo>,
    /// Order list.
    pub orders: Vec<u8>,
    /// Validation warnings (non-fatal issues).
    pub warnings: Vec<ItValidationWarning>,
    /// Validation errors (fatal issues).
    pub errors: Vec<ItFormatError>,
}

impl ItValidationReport {
    fn new() -> Self {
        Self {
            is_valid: false,
            header: None,
            instruments: Vec::new(),
            samples: Vec::new(),
            patterns: Vec::new(),
            orders: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn add_warning(&mut self, message: impl Into<String>, offset: Option<usize>) {
        self.warnings.push(ItValidationWarning {
            message: message.into(),
            offset,
        });
    }

    fn add_error(&mut self, error: ItFormatError) {
        self.errors.push(error);
    }
}

// ============================================================================
// Extracted Information Structures
// ============================================================================

/// Information extracted from IT header.
#[derive(Debug, Clone)]
pub struct ItHeaderInfo {
    /// Song name (up to 26 characters).
    pub name: String,
    /// Pattern highlight (minor, major).
    pub pattern_highlight: (u8, u8),
    /// Number of orders.
    pub order_count: u16,
    /// Number of instruments.
    pub instrument_count: u16,
    /// Number of samples.
    pub sample_count: u16,
    /// Number of patterns.
    pub pattern_count: u16,
    /// Created with tracker version.
    pub created_with: u16,
    /// Compatible with tracker version.
    pub compatible_with: u16,
    /// Flags.
    pub flags: ItFlags,
    /// Special flags.
    pub special: ItSpecialFlags,
    /// Global volume (0-128).
    pub global_volume: u8,
    /// Mix volume (0-128).
    pub mix_volume: u8,
    /// Initial speed (ticks per row).
    pub initial_speed: u8,
    /// Initial tempo (BPM).
    pub initial_tempo: u8,
    /// Panning separation (0-128).
    pub panning_separation: u8,
    /// Pitch wheel depth.
    pub pitch_wheel_depth: u8,
    /// Message length.
    pub message_length: u16,
    /// Message offset.
    pub message_offset: u32,
    /// Channel panning values.
    pub channel_pan: [u8; 64],
    /// Channel volume values.
    pub channel_vol: [u8; 64],
}

/// IT header flags parsed from the flags field.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItFlags {
    /// Stereo output.
    pub stereo: bool,
    /// Vol0MixOptimizations.
    pub vol0_mix_optimizations: bool,
    /// Use instruments (vs samples only).
    pub use_instruments: bool,
    /// Linear slides.
    pub linear_slides: bool,
    /// Old effects.
    pub old_effects: bool,
    /// Link effect G memory with E/F.
    pub link_g_memory: bool,
    /// Use MIDI pitch controller.
    pub midi_pitch_controller: bool,
    /// Embedded MIDI configuration.
    pub embedded_midi_config: bool,
}

impl ItFlags {
    fn from_u16(value: u16) -> Self {
        Self {
            stereo: value & 0x01 != 0,
            vol0_mix_optimizations: value & 0x02 != 0,
            use_instruments: value & 0x04 != 0,
            linear_slides: value & 0x08 != 0,
            old_effects: value & 0x10 != 0,
            link_g_memory: value & 0x20 != 0,
            midi_pitch_controller: value & 0x40 != 0,
            embedded_midi_config: value & 0x80 != 0,
        }
    }
}

/// IT special flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItSpecialFlags {
    /// Song message attached.
    pub message_attached: bool,
    /// Embedded MIDI configuration.
    pub midi_configuration: bool,
}

impl ItSpecialFlags {
    fn from_u16(value: u16) -> Self {
        Self {
            message_attached: value & 0x01 != 0,
            midi_configuration: value & 0x08 != 0,
        }
    }
}

/// Information extracted from IT instrument.
#[derive(Debug, Clone)]
pub struct ItInstrumentInfo {
    /// Instrument index (1-based).
    pub index: usize,
    /// File offset.
    pub offset: u32,
    /// Instrument name.
    pub name: String,
    /// DOS filename.
    pub filename: String,
    /// New Note Action.
    pub nna: u8,
    /// Duplicate Check Type.
    pub dct: u8,
    /// Duplicate Check Action.
    pub dca: u8,
    /// Fadeout value.
    pub fadeout: u16,
    /// Pitch-pan separation.
    pub pps: i8,
    /// Pitch-pan center note.
    pub ppc: u8,
    /// Global volume.
    pub global_volume: u8,
    /// Default pan (None if not set).
    pub default_pan: Option<u8>,
    /// Random volume variation.
    pub random_volume: u8,
    /// Random panning variation.
    pub random_pan: u8,
    /// Volume envelope info.
    pub volume_envelope: ItEnvelopeInfo,
    /// Panning envelope info.
    pub panning_envelope: ItEnvelopeInfo,
    /// Pitch envelope info.
    pub pitch_envelope: ItEnvelopeInfo,
}

/// Information about an IT envelope.
#[derive(Debug, Clone, Default)]
pub struct ItEnvelopeInfo {
    /// Envelope enabled.
    pub enabled: bool,
    /// Loop enabled.
    pub loop_enabled: bool,
    /// Sustain loop enabled.
    pub sustain_loop_enabled: bool,
    /// Carry envelope.
    pub carry: bool,
    /// Filter envelope (for pitch envelope).
    pub is_filter: bool,
    /// Number of nodes.
    pub num_nodes: u8,
    /// Loop begin node.
    pub loop_begin: u8,
    /// Loop end node.
    pub loop_end: u8,
    /// Sustain loop begin node.
    pub sustain_begin: u8,
    /// Sustain loop end node.
    pub sustain_end: u8,
}

/// Information extracted from IT sample.
#[derive(Debug, Clone)]
pub struct ItSampleInfo {
    /// Sample index (1-based).
    pub index: usize,
    /// File offset.
    pub offset: u32,
    /// Sample name.
    pub name: String,
    /// DOS filename.
    pub filename: String,
    /// Global volume.
    pub global_volume: u8,
    /// Sample flags.
    pub flags: ItSampleFlags,
    /// Default volume.
    pub default_volume: u8,
    /// Default pan (None if not set).
    pub default_pan: Option<u8>,
    /// Sample length in samples.
    pub length: u32,
    /// Loop begin.
    pub loop_begin: u32,
    /// Loop end.
    pub loop_end: u32,
    /// C5 speed.
    pub c5_speed: u32,
    /// Sustain loop begin.
    pub sustain_loop_begin: u32,
    /// Sustain loop end.
    pub sustain_loop_end: u32,
    /// Sample data offset.
    pub data_offset: u32,
    /// Vibrato speed.
    pub vibrato_speed: u8,
    /// Vibrato depth.
    pub vibrato_depth: u8,
    /// Vibrato rate.
    pub vibrato_rate: u8,
    /// Vibrato type.
    pub vibrato_type: u8,
    /// Conversion flags.
    pub convert_flags: ItConvertFlags,
}

/// IT sample flags parsed.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItSampleFlags {
    /// Sample has data.
    pub has_data: bool,
    /// 16-bit sample.
    pub is_16bit: bool,
    /// Stereo sample.
    pub is_stereo: bool,
    /// Compressed sample.
    pub is_compressed: bool,
    /// Loop enabled.
    pub loop_enabled: bool,
    /// Sustain loop enabled.
    pub sustain_loop_enabled: bool,
    /// Ping-pong loop.
    pub ping_pong_loop: bool,
    /// Ping-pong sustain loop.
    pub ping_pong_sustain: bool,
}

impl ItSampleFlags {
    fn from_u8(value: u8) -> Self {
        Self {
            has_data: value & 0x01 != 0,
            is_16bit: value & 0x02 != 0,
            is_stereo: value & 0x04 != 0,
            is_compressed: value & 0x08 != 0,
            loop_enabled: value & 0x10 != 0,
            sustain_loop_enabled: value & 0x20 != 0,
            ping_pong_loop: value & 0x40 != 0,
            ping_pong_sustain: value & 0x80 != 0,
        }
    }
}

/// IT sample conversion flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItConvertFlags {
    /// Signed samples.
    pub signed: bool,
    /// Big endian (Motorola byte order).
    pub big_endian: bool,
    /// Delta encoded.
    pub delta: bool,
    /// Byte delta (8-bit only).
    pub byte_delta: bool,
    /// TX-Wave 12-bit.
    pub tx_wave: bool,
    /// Stereo prompt.
    pub stereo_prompt: bool,
}

impl ItConvertFlags {
    fn from_u8(value: u8) -> Self {
        Self {
            signed: value & 0x01 != 0,
            big_endian: value & 0x02 != 0,
            delta: value & 0x04 != 0,
            byte_delta: value & 0x08 != 0,
            tx_wave: value & 0x10 != 0,
            stereo_prompt: value & 0x20 != 0,
        }
    }
}

/// Information extracted from IT pattern.
#[derive(Debug, Clone)]
pub struct ItPatternInfo {
    /// Pattern index.
    pub index: usize,
    /// File offset.
    pub offset: u32,
    /// Packed data length.
    pub packed_length: u16,
    /// Number of rows.
    pub num_rows: u16,
}

// ============================================================================
// Validator Implementation
// ============================================================================

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
        Self::validate_instruments(data, &header, offset_table_start, &mut report)?;

        // Validate samples
        let sample_offsets_start =
            offset_table_start + (header.instrument_count as usize * 4);
        Self::validate_samples(data, &header, sample_offsets_start, &mut report)?;

        // Validate patterns
        let pattern_offsets_start =
            sample_offsets_start + (header.sample_count as usize * 4);
        Self::validate_patterns(data, &header, pattern_offsets_start, &mut report)?;

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
                format!(
                    "Global volume {} exceeds maximum of 128",
                    global_volume
                ),
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
                format!(
                    "Initial tempo {} is below minimum of 32",
                    initial_tempo
                ),
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
                    format!(
                        "Channel {} volume {} exceeds maximum of 64",
                        i, vol
                    ),
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

    fn validate_instruments(
        data: &[u8],
        header: &ItHeaderInfo,
        offset_table_start: usize,
        report: &mut ItValidationReport,
    ) -> Result<(), ItFormatError> {
        if header.instrument_count == 0 {
            return Ok(());
        }

        let table_end = offset_table_start + (header.instrument_count as usize * 4);
        if table_end > data.len() {
            return Err(ItFormatError::at_offset(
                ItErrorCategory::OffsetTable,
                format!(
                    "Instrument offset table extends beyond file: needs {} bytes at offset {}, file has {}",
                    header.instrument_count * 4,
                    offset_table_start,
                    data.len()
                ),
                offset_table_start,
            ));
        }

        for i in 0..header.instrument_count as usize {
            let offset_pos = offset_table_start + i * 4;
            let instrument_offset = u32::from_le_bytes([
                data[offset_pos],
                data[offset_pos + 1],
                data[offset_pos + 2],
                data[offset_pos + 3],
            ]);

            if instrument_offset == 0 {
                // Null offset means no instrument data
                continue;
            }

            match Self::validate_single_instrument(data, i + 1, instrument_offset, report) {
                Ok(info) => report.instruments.push(info),
                Err(e) => report.add_error(e),
            }
        }

        Ok(())
    }

    fn validate_single_instrument(
        data: &[u8],
        index: usize,
        offset: u32,
        report: &mut ItValidationReport,
    ) -> Result<ItInstrumentInfo, ItFormatError> {
        let offset = offset as usize;

        // Instrument header is 554 bytes (per ITTECH.TXT)
        if offset + 554 > data.len() {
            return Err(ItFormatError::at_offset(
                ItErrorCategory::Instrument,
                format!(
                    "Instrument {} header extends beyond file",
                    index
                ),
                offset,
            ));
        }

        let inst = &data[offset..];

        // Check magic "IMPI" at offset 0x00
        if &inst[0..4] != b"IMPI" {
            return Err(ItFormatError::field_at_offset(
                ItErrorCategory::Instrument,
                "magic",
                format!(
                    "Instrument {} has invalid magic: expected 'IMPI', got {:02X?}",
                    index,
                    &inst[0..4]
                ),
                offset,
            ));
        }

        // DOS filename (12 bytes at 0x04)
        let filename = extract_string(&inst[0x04..0x10]);

        // Reserved byte at 0x10 (should be 0)

        // NNA at 0x11
        let nna = inst[0x11];
        if nna > 3 {
            report.add_warning(
                format!(
                    "Instrument {} NNA value {} is invalid (should be 0-3)",
                    index, nna
                ),
                Some(offset + 0x11),
            );
        }

        // DCT at 0x12
        let dct = inst[0x12];
        if dct > 3 {
            report.add_warning(
                format!(
                    "Instrument {} DCT value {} is invalid (should be 0-3)",
                    index, dct
                ),
                Some(offset + 0x12),
            );
        }

        // DCA at 0x13
        let dca = inst[0x13];
        if dca > 2 {
            report.add_warning(
                format!(
                    "Instrument {} DCA value {} is invalid (should be 0-2)",
                    index, dca
                ),
                Some(offset + 0x13),
            );
        }

        // Fadeout at 0x14 (2 bytes)
        let fadeout = u16::from_le_bytes([inst[0x14], inst[0x15]]);
        if fadeout > 1024 {
            report.add_warning(
                format!(
                    "Instrument {} fadeout {} exceeds typical maximum of 1024",
                    index, fadeout
                ),
                Some(offset + 0x14),
            );
        }

        // PPS (Pitch-Pan Separation) at 0x16
        let pps = inst[0x16] as i8;

        // PPC (Pitch-Pan Center) at 0x17
        let ppc = inst[0x17];
        if ppc > 119 {
            report.add_warning(
                format!(
                    "Instrument {} PPC {} exceeds maximum note 119 (B-9)",
                    index, ppc
                ),
                Some(offset + 0x17),
            );
        }

        // Global volume at 0x18
        let global_volume = inst[0x18];
        if global_volume > 128 {
            report.add_warning(
                format!(
                    "Instrument {} global volume {} exceeds maximum of 128",
                    index, global_volume
                ),
                Some(offset + 0x18),
            );
        }

        // Default pan at 0x19 (bit 7 = use pan)
        let dfp = inst[0x19];
        let default_pan = if dfp & 0x80 != 0 {
            Some(dfp & 0x7F)
        } else {
            None
        };

        // Random volume variation at 0x1A
        let random_volume = inst[0x1A];
        if random_volume > 100 {
            report.add_warning(
                format!(
                    "Instrument {} random volume {} exceeds 100%",
                    index, random_volume
                ),
                Some(offset + 0x1A),
            );
        }

        // Random panning variation at 0x1B
        let random_pan = inst[0x1B];
        if random_pan > 64 {
            report.add_warning(
                format!(
                    "Instrument {} random pan {} exceeds maximum of 64",
                    index, random_pan
                ),
                Some(offset + 0x1B),
            );
        }

        // Tracker version at 0x1C (2 bytes) - only for instrument files
        // Number of samples at 0x1E - only for instrument files
        // Reserved byte at 0x1F

        // Instrument name at 0x20 (26 bytes)
        let name = extract_string(&inst[0x20..0x3A]);

        // Initial filter cutoff at 0x3A (bit 7 = use filter)
        // Initial filter resonance at 0x3B
        // MIDI channel at 0x3C
        // MIDI program at 0x3D
        // MIDI bank at 0x3E (2 bytes)

        // Note-sample table at 0x40 (240 bytes = 120 pairs)
        // We skip detailed validation here but could check that sample indices are valid

        // Volume envelope starts at 0x130 (offset 304)
        let volume_envelope = Self::parse_envelope(&inst[0x130..], "volume", index, offset + 0x130, report);

        // Panning envelope at 0x182 (offset 386)
        let panning_envelope = Self::parse_envelope(&inst[0x182..], "panning", index, offset + 0x182, report);

        // Pitch envelope at 0x1D4 (offset 468)
        let pitch_envelope = Self::parse_envelope(&inst[0x1D4..], "pitch", index, offset + 0x1D4, report);

        Ok(ItInstrumentInfo {
            index,
            offset: offset as u32,
            name,
            filename,
            nna,
            dct,
            dca,
            fadeout,
            pps,
            ppc,
            global_volume,
            default_pan,
            random_volume,
            random_pan,
            volume_envelope,
            panning_envelope,
            pitch_envelope,
        })
    }

    fn parse_envelope(
        data: &[u8],
        env_name: &str,
        inst_index: usize,
        base_offset: usize,
        report: &mut ItValidationReport,
    ) -> ItEnvelopeInfo {
        // Envelope structure: 82 bytes
        // Flags at offset 0
        let flags = data[0];
        let enabled = flags & 0x01 != 0;
        let loop_enabled = flags & 0x02 != 0;
        let sustain_loop_enabled = flags & 0x04 != 0;
        let carry = flags & 0x08 != 0;
        let is_filter = flags & 0x80 != 0; // For pitch envelope

        // Number of nodes at offset 1
        let num_nodes = data[1];
        if num_nodes > 25 {
            report.add_warning(
                format!(
                    "Instrument {} {} envelope has {} nodes (maximum is 25)",
                    inst_index, env_name, num_nodes
                ),
                Some(base_offset + 1),
            );
        }

        // Loop begin at offset 2
        let loop_begin = data[2];

        // Loop end at offset 3
        let loop_end = data[3];

        // Sustain begin at offset 4
        let sustain_begin = data[4];

        // Sustain end at offset 5
        let sustain_end = data[5];

        // Validate loop points
        if loop_enabled && loop_begin > loop_end {
            report.add_warning(
                format!(
                    "Instrument {} {} envelope loop begin ({}) > end ({})",
                    inst_index, env_name, loop_begin, loop_end
                ),
                Some(base_offset + 2),
            );
        }

        if sustain_loop_enabled && sustain_begin > sustain_end {
            report.add_warning(
                format!(
                    "Instrument {} {} envelope sustain begin ({}) > end ({})",
                    inst_index, env_name, sustain_begin, sustain_end
                ),
                Some(base_offset + 4),
            );
        }

        ItEnvelopeInfo {
            enabled,
            loop_enabled,
            sustain_loop_enabled,
            carry,
            is_filter,
            num_nodes,
            loop_begin,
            loop_end,
            sustain_begin,
            sustain_end,
        }
    }

    fn validate_samples(
        data: &[u8],
        header: &ItHeaderInfo,
        offset_table_start: usize,
        report: &mut ItValidationReport,
    ) -> Result<(), ItFormatError> {
        if header.sample_count == 0 {
            return Ok(());
        }

        let table_end = offset_table_start + (header.sample_count as usize * 4);
        if table_end > data.len() {
            return Err(ItFormatError::at_offset(
                ItErrorCategory::OffsetTable,
                format!(
                    "Sample offset table extends beyond file: needs {} bytes at offset {}, file has {}",
                    header.sample_count * 4,
                    offset_table_start,
                    data.len()
                ),
                offset_table_start,
            ));
        }

        for i in 0..header.sample_count as usize {
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

            match Self::validate_single_sample(data, i + 1, sample_offset, report) {
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
        let sustain_loop_begin =
            u32::from_le_bytes([smp[0x40], smp[0x41], smp[0x42], smp[0x43]]);

        // Sustain loop end at 0x44 (4 bytes)
        let sustain_loop_end = u32::from_le_bytes([smp[0x44], smp[0x45], smp[0x46], smp[0x47]]);

        // Validate sustain loop points
        if flags.sustain_loop_enabled {
            if sustain_loop_begin > sustain_loop_end {
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
                        index, expected_size, data_offset, data.len()
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

    fn validate_patterns(
        data: &[u8],
        header: &ItHeaderInfo,
        offset_table_start: usize,
        report: &mut ItValidationReport,
    ) -> Result<(), ItFormatError> {
        if header.pattern_count == 0 {
            return Ok(());
        }

        let table_end = offset_table_start + (header.pattern_count as usize * 4);
        if table_end > data.len() {
            return Err(ItFormatError::at_offset(
                ItErrorCategory::OffsetTable,
                format!(
                    "Pattern offset table extends beyond file: needs {} bytes at offset {}, file has {}",
                    header.pattern_count * 4,
                    offset_table_start,
                    data.len()
                ),
                offset_table_start,
            ));
        }

        for i in 0..header.pattern_count as usize {
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

            match Self::validate_single_pattern(data, i, pattern_offset, report) {
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
                format!(
                    "Pattern {} has {} rows (maximum is 200)",
                    index, num_rows
                ),
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
        Self::validate_pattern_packing(
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
                        format!(
                            "Pattern {} truncated: expected mask byte",
                            pattern_index
                        ),
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
                    msg_length, msg_offset, data.len()
                ),
                msg_offset,
            ));
        }

        Ok(())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract a null-terminated string from a byte slice.
fn extract_string(data: &[u8]) -> String {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..end])
        .trim_end()
        .to_string()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a minimal valid IT file for testing.
    fn create_minimal_it() -> Vec<u8> {
        let mut it = vec![0u8; 192];

        // Magic
        it[0..4].copy_from_slice(b"IMPM");

        // Name
        it[4..14].copy_from_slice(b"Test Song ");

        // OrdNum = 1
        it[0x20..0x22].copy_from_slice(&1u16.to_le_bytes());

        // InsNum = 0
        it[0x22..0x24].copy_from_slice(&0u16.to_le_bytes());

        // SmpNum = 0
        it[0x24..0x26].copy_from_slice(&0u16.to_le_bytes());

        // PatNum = 0
        it[0x26..0x28].copy_from_slice(&0u16.to_le_bytes());

        // Cwt/v = 0x0214
        it[0x28..0x2A].copy_from_slice(&0x0214u16.to_le_bytes());

        // Cmwt = 0x0200
        it[0x2A..0x2C].copy_from_slice(&0x0200u16.to_le_bytes());

        // Flags
        it[0x2C..0x2E].copy_from_slice(&0x0009u16.to_le_bytes());

        // Global volume = 128
        it[0x30] = 128;

        // Mix volume = 48
        it[0x31] = 48;

        // Initial speed = 6
        it[0x32] = 6;

        // Initial tempo = 125
        it[0x33] = 125;

        // Pan separation = 128
        it[0x34] = 128;

        // Channel pan (all center)
        for i in 0x40..0x80 {
            it[i] = 32;
        }

        // Channel volume
        for i in 0x80..0xC0 {
            it[i] = 64;
        }

        // Order: 255 (end)
        it.push(255);

        it
    }

    /// Create an IT file with an instrument.
    fn create_it_with_instrument() -> Vec<u8> {
        let mut it = create_minimal_it();

        // Update header for 1 instrument
        it[0x22..0x24].copy_from_slice(&1u16.to_le_bytes());

        // Add instrument offset (right after order + offset table)
        let inst_offset = (it.len() + 4) as u32; // 4 bytes for offset entry
        it.extend_from_slice(&inst_offset.to_le_bytes());

        // Create instrument header (554 bytes)
        let mut inst = vec![0u8; 554];
        inst[0..4].copy_from_slice(b"IMPI");
        // Name
        inst[0x20..0x30].copy_from_slice(b"Test Inst       ");
        // Global volume
        inst[0x18] = 128;

        it.extend(inst);

        it
    }

    /// Create an IT file with a sample.
    fn create_it_with_sample() -> Vec<u8> {
        let mut it = create_minimal_it();

        // Update header for 1 sample
        it[0x24..0x26].copy_from_slice(&1u16.to_le_bytes());

        // Add sample offset
        let sample_offset = (it.len() + 4) as u32;
        it.extend_from_slice(&sample_offset.to_le_bytes());

        // Create sample header (80 bytes)
        let mut smp = vec![0u8; 80];
        smp[0..4].copy_from_slice(b"IMPS");
        // Global volume
        smp[0x11] = 64;
        // Flags (has data, 16-bit)
        smp[0x12] = 0x03;
        // Default volume
        smp[0x13] = 64;
        // Name
        smp[0x14..0x24].copy_from_slice(b"Test Sample     ");
        // C5 speed
        smp[0x3C..0x40].copy_from_slice(&22050u32.to_le_bytes());

        it.extend(smp);

        it
    }

    /// Create an IT file with a pattern.
    fn create_it_with_pattern() -> Vec<u8> {
        let mut it = create_minimal_it();

        // Update header for 1 pattern
        it[0x26..0x28].copy_from_slice(&1u16.to_le_bytes());

        // Add pattern offset
        let pattern_offset = (it.len() + 4) as u32;
        it.extend_from_slice(&pattern_offset.to_le_bytes());

        // Create minimal pattern
        // Header: 8 bytes
        let mut pattern = Vec::new();
        // Packed length (just end-of-row markers for 64 rows)
        pattern.extend_from_slice(&64u16.to_le_bytes());
        // Rows = 64
        pattern.extend_from_slice(&64u16.to_le_bytes());
        // Reserved
        pattern.extend_from_slice(&[0u8; 4]);
        // Packed data: 64 end-of-row markers
        pattern.extend(vec![0u8; 64]);

        it.extend(pattern);

        it
    }

    #[test]
    fn test_it_header_magic() {
        let it = create_minimal_it();
        let report = ItValidator::validate(&it).unwrap();

        assert!(report.is_valid);
        assert!(report.header.is_some());

        let header = report.header.unwrap();
        assert_eq!(header.name, "Test Song");
    }

    #[test]
    fn test_it_header_magic_invalid() {
        let mut it = create_minimal_it();
        it[0..4].copy_from_slice(b"XXXX");

        let result = ItValidator::validate(&it);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.category, ItErrorCategory::Header);
        assert!(err.message.contains("magic"));
    }

    #[test]
    fn test_it_header_fields() {
        let it = create_minimal_it();
        let report = ItValidator::validate(&it).unwrap();

        let header = report.header.unwrap();
        assert_eq!(header.initial_speed, 6);
        assert_eq!(header.initial_tempo, 125);
        assert_eq!(header.global_volume, 128);
        assert_eq!(header.mix_volume, 48);
        assert_eq!(header.panning_separation, 128);
        assert!(header.flags.stereo);
        assert!(header.flags.linear_slides);
    }

    #[test]
    fn test_it_header_global_volume_invalid() {
        let mut it = create_minimal_it();
        it[0x30] = 200; // Invalid: > 128

        let report = ItValidator::validate(&it).unwrap();
        assert!(!report.is_valid);
        assert!(!report.errors.is_empty());
        assert!(report.errors[0].message.contains("Global volume"));
    }

    #[test]
    fn test_it_header_speed_zero() {
        let mut it = create_minimal_it();
        it[0x32] = 0; // Invalid: speed cannot be 0

        let report = ItValidator::validate(&it).unwrap();
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.message.contains("speed")));
    }

    #[test]
    fn test_it_instrument_nna() {
        let it = create_it_with_instrument();
        let report = ItValidator::validate(&it).unwrap();

        assert!(report.is_valid);
        assert_eq!(report.instruments.len(), 1);
        assert_eq!(report.instruments[0].nna, 0); // Default: cut
    }

    #[test]
    fn test_it_instrument_invalid_magic() {
        let mut it = create_it_with_instrument();
        // Find instrument data and corrupt magic
        let inst_offset = u32::from_le_bytes([
            it[193],
            it[194],
            it[195],
            it[196],
        ]) as usize;
        it[inst_offset..inst_offset + 4].copy_from_slice(b"XXXX");

        let report = ItValidator::validate(&it).unwrap();
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.message.contains("magic")));
    }

    #[test]
    fn test_it_sample_validation() {
        let it = create_it_with_sample();
        let report = ItValidator::validate(&it).unwrap();

        assert!(report.is_valid);
        assert_eq!(report.samples.len(), 1);

        let sample = &report.samples[0];
        assert!(sample.flags.has_data);
        assert!(sample.flags.is_16bit);
        assert_eq!(sample.c5_speed, 22050);
        assert_eq!(sample.global_volume, 64);
    }

    #[test]
    fn test_it_sample_invalid_magic() {
        let mut it = create_it_with_sample();
        let sample_offset = u32::from_le_bytes([
            it[193],
            it[194],
            it[195],
            it[196],
        ]) as usize;
        it[sample_offset..sample_offset + 4].copy_from_slice(b"XXXX");

        let report = ItValidator::validate(&it).unwrap();
        assert!(!report.is_valid);
    }

    #[test]
    fn test_it_sample_compression() {
        let mut it = create_it_with_sample();
        let sample_offset = u32::from_le_bytes([
            it[193],
            it[194],
            it[195],
            it[196],
        ]) as usize;
        // Set compressed flag
        it[sample_offset + 0x12] |= 0x08;

        let report = ItValidator::validate(&it).unwrap();
        assert!(report.samples[0].flags.is_compressed);
    }

    #[test]
    fn test_it_pattern_packing() {
        let it = create_it_with_pattern();
        let report = ItValidator::validate(&it).unwrap();

        assert!(report.is_valid);
        assert_eq!(report.patterns.len(), 1);
        assert_eq!(report.patterns[0].num_rows, 64);
    }

    #[test]
    fn test_it_pattern_zero_rows() {
        let mut it = create_it_with_pattern();
        let pattern_offset = u32::from_le_bytes([
            it[193],
            it[194],
            it[195],
            it[196],
        ]) as usize;
        // Set rows to 0
        it[pattern_offset + 2..pattern_offset + 4].copy_from_slice(&0u16.to_le_bytes());

        let report = ItValidator::validate(&it).unwrap();
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.message.contains("0 rows")));
    }

    #[test]
    fn test_it_full_validation() {
        // Create a more complete IT file
        let mut it = create_minimal_it();

        // Add 1 instrument
        it[0x22..0x24].copy_from_slice(&1u16.to_le_bytes());
        // Add 1 sample
        it[0x24..0x26].copy_from_slice(&1u16.to_le_bytes());
        // Add 1 pattern
        it[0x26..0x28].copy_from_slice(&1u16.to_le_bytes());

        // Calculate offsets
        let base = it.len();
        let inst_offset = (base + 12) as u32; // After 3 offset entries
        let sample_offset = (inst_offset as usize + 554) as u32;
        let pattern_offset = (sample_offset as usize + 80) as u32;

        // Add offset table
        it.extend_from_slice(&inst_offset.to_le_bytes());
        it.extend_from_slice(&sample_offset.to_le_bytes());
        it.extend_from_slice(&pattern_offset.to_le_bytes());

        // Add instrument
        let mut inst = vec![0u8; 554];
        inst[0..4].copy_from_slice(b"IMPI");
        inst[0x18] = 128;
        it.extend(inst);

        // Add sample
        let mut smp = vec![0u8; 80];
        smp[0..4].copy_from_slice(b"IMPS");
        smp[0x11] = 64;
        smp[0x12] = 0x03;
        smp[0x13] = 64;
        smp[0x3C..0x40].copy_from_slice(&22050u32.to_le_bytes());
        it.extend(smp);

        // Add pattern
        let mut pattern = Vec::new();
        pattern.extend_from_slice(&64u16.to_le_bytes()); // packed length
        pattern.extend_from_slice(&64u16.to_le_bytes()); // rows
        pattern.extend_from_slice(&[0u8; 4]); // reserved
        pattern.extend(vec![0u8; 64]); // end-of-row markers
        it.extend(pattern);

        let report = ItValidator::validate(&it).unwrap();

        assert!(report.is_valid, "Errors: {:?}", report.errors);
        assert!(report.header.is_some());
        assert_eq!(report.instruments.len(), 1);
        assert_eq!(report.samples.len(), 1);
        assert_eq!(report.patterns.len(), 1);
    }

    #[test]
    fn test_it_file_too_small() {
        let data = vec![0u8; 100];
        let result = ItValidator::validate(&data);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.category, ItErrorCategory::Structure);
        assert!(err.message.contains("too small"));
    }

    #[test]
    fn test_it_order_list_validation() {
        let mut it = create_minimal_it();

        // Add more orders
        it[0x20..0x22].copy_from_slice(&5u16.to_le_bytes());
        // Remove the existing order (255) and add new ones
        it.pop();
        it.push(0);   // Pattern 0
        it.push(254); // Skip marker
        it.push(0);   // Pattern 0
        it.push(100); // Invalid: no pattern 100
        it.push(255); // End marker

        let report = ItValidator::validate(&it).unwrap();

        assert_eq!(report.orders.len(), 5);
        assert_eq!(report.orders[0], 0);
        assert_eq!(report.orders[1], 254);
        assert_eq!(report.orders[4], 255);

        // Should have a warning about referencing non-existent pattern
        assert!(!report.warnings.is_empty());
        assert!(report.warnings.iter().any(|w| w.message.contains("pattern 100")));
    }

    #[test]
    fn test_it_channel_pan_validation() {
        let mut it = create_minimal_it();

        // Set invalid panning value
        it[0x45] = 80; // Invalid: between 65-99

        let report = ItValidator::validate(&it).unwrap();

        assert!(report.warnings.iter().any(|w| w.message.contains("panning")));
    }

    #[test]
    fn test_it_channel_volume_validation() {
        let mut it = create_minimal_it();

        // Set invalid volume
        it[0x85] = 100; // Invalid: > 64

        let report = ItValidator::validate(&it).unwrap();

        assert!(report.warnings.iter().any(|w| w.message.contains("volume")));
    }

    #[test]
    fn test_format_error_display() {
        let err = ItFormatError::new(ItErrorCategory::Header, "test error");
        assert_eq!(format!("{}", err), "IT header error: test error");

        let err = ItFormatError::at_offset(ItErrorCategory::Sample, "at offset", 0x100);
        assert_eq!(format!("{}", err), "IT sample error at offset 0x0100: at offset");

        let err = ItFormatError::field_at_offset(
            ItErrorCategory::Pattern,
            "num_rows",
            "invalid",
            0x50,
        );
        assert!(format!("{}", err).contains("field 'num_rows'"));
        assert!(format!("{}", err).contains("0x0050"));
    }

    #[test]
    fn test_it_header_only_validation() {
        let it = create_minimal_it();
        let header = ItValidator::validate_header_only(&it).unwrap();

        assert_eq!(header.name, "Test Song");
        assert_eq!(header.initial_speed, 6);
        assert_eq!(header.initial_tempo, 125);
    }

    #[test]
    fn test_sample_loop_validation() {
        let mut it = create_it_with_sample();
        let sample_offset = u32::from_le_bytes([
            it[193],
            it[194],
            it[195],
            it[196],
        ]) as usize;

        // Enable loop
        it[sample_offset + 0x12] |= 0x10;
        // Set loop begin > end
        it[sample_offset + 0x34..sample_offset + 0x38].copy_from_slice(&100u32.to_le_bytes());
        it[sample_offset + 0x38..sample_offset + 0x3C].copy_from_slice(&50u32.to_le_bytes());

        let report = ItValidator::validate(&it).unwrap();
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.message.contains("loop begin")));
    }

    #[test]
    fn test_envelope_validation() {
        let it = create_it_with_instrument();
        let report = ItValidator::validate(&it).unwrap();

        let inst = &report.instruments[0];
        // Default envelope should not be enabled
        assert!(!inst.volume_envelope.enabled);
        assert!(!inst.panning_envelope.enabled);
        assert!(!inst.pitch_envelope.enabled);
    }

    #[test]
    fn test_instrument_fadeout_warning() {
        let mut it = create_it_with_instrument();
        let inst_offset = u32::from_le_bytes([
            it[193],
            it[194],
            it[195],
            it[196],
        ]) as usize;

        // Set fadeout > 1024
        it[inst_offset + 0x14..inst_offset + 0x16].copy_from_slice(&2000u16.to_le_bytes());

        let report = ItValidator::validate(&it).unwrap();
        assert!(report.warnings.iter().any(|w| w.message.contains("fadeout")));
    }
}
