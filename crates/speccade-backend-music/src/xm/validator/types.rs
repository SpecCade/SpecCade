//! Data structure types for XM validation.

use super::error::{XmFormatError, XmWarning};

/// Detailed information extracted from XM header.
#[derive(Debug, Clone)]
pub struct XmHeaderInfo {
    /// Module name (up to 20 characters).
    pub name: String,
    /// Tracker name (up to 20 characters).
    pub tracker_name: String,
    /// XM format version.
    pub version: u16,
    /// Header size (from offset 60).
    pub header_size: u32,
    /// Song length (number of pattern positions).
    pub song_length: u16,
    /// Restart position.
    pub restart_position: u16,
    /// Number of channels.
    pub num_channels: u16,
    /// Number of patterns.
    pub num_patterns: u16,
    /// Number of instruments.
    pub num_instruments: u16,
    /// Flags (bit 0: frequency table type).
    pub flags: u16,
    /// Uses linear frequency table (vs Amiga).
    pub linear_frequency_table: bool,
    /// Default tempo (ticks per row).
    pub default_tempo: u16,
    /// Default BPM.
    pub default_bpm: u16,
    /// Pattern order table (256 entries).
    pub pattern_order: [u8; 256],
}

/// Information about a pattern.
#[derive(Debug, Clone)]
pub struct XmPatternInfo {
    /// Pattern index.
    pub index: usize,
    /// Header length.
    pub header_length: u32,
    /// Packing type (should be 0).
    pub packing_type: u8,
    /// Number of rows.
    pub num_rows: u16,
    /// Packed data size.
    pub packed_size: u16,
    /// Is pattern empty.
    pub is_empty: bool,
}

/// Information about an envelope.
#[derive(Debug, Clone)]
pub struct XmEnvelopeInfo {
    /// Envelope type ("volume" or "panning").
    pub envelope_type: &'static str,
    /// Number of points.
    pub num_points: u8,
    /// Sustain point index.
    pub sustain_point: u8,
    /// Loop start point.
    pub loop_start: u8,
    /// Loop end point.
    pub loop_end: u8,
    /// Envelope enabled.
    pub enabled: bool,
    /// Sustain enabled.
    pub sustain_enabled: bool,
    /// Loop enabled.
    pub loop_enabled: bool,
    /// Envelope points (frame, value).
    pub points: Vec<(u16, u16)>,
}

/// Information about a sample.
#[derive(Debug, Clone)]
pub struct XmSampleInfo {
    /// Sample index within instrument.
    pub index: usize,
    /// Sample name (up to 22 characters).
    pub name: String,
    /// Sample length in bytes.
    pub length: u32,
    /// Loop start position.
    pub loop_start: u32,
    /// Loop length.
    pub loop_length: u32,
    /// Default volume (0-64).
    pub volume: u8,
    /// Finetune (-128 to +127).
    pub finetune: i8,
    /// Sample type/flags.
    pub flags: u8,
    /// Is 16-bit sample.
    pub is_16bit: bool,
    /// Loop type (0=none, 1=forward, 2=ping-pong).
    pub loop_type: u8,
    /// Panning (0-255).
    pub panning: u8,
    /// Relative note number.
    pub relative_note: i8,
}

/// Information about an instrument.
#[derive(Debug, Clone)]
pub struct XmInstrumentInfo {
    /// Instrument index (1-based in file, 0-based here).
    pub index: usize,
    /// Instrument name (up to 22 characters).
    pub name: String,
    /// Instrument type (should be 0).
    pub instrument_type: u8,
    /// Number of samples.
    pub num_samples: u16,
    /// Sample header size.
    pub sample_header_size: u32,
    /// Volume envelope info.
    pub volume_envelope: XmEnvelopeInfo,
    /// Panning envelope info.
    pub panning_envelope: XmEnvelopeInfo,
    /// Vibrato type.
    pub vibrato_type: u8,
    /// Vibrato sweep.
    pub vibrato_sweep: u8,
    /// Vibrato depth.
    pub vibrato_depth: u8,
    /// Vibrato rate.
    pub vibrato_rate: u8,
    /// Volume fadeout.
    pub volume_fadeout: u16,
    /// Sample information.
    pub samples: Vec<XmSampleInfo>,
}

/// Complete XM validation report.
#[derive(Debug, Clone)]
pub struct XmValidationReport {
    /// Whether the file is valid.
    pub valid: bool,
    /// Header information (if parsed successfully).
    pub header: Option<XmHeaderInfo>,
    /// Pattern information.
    pub patterns: Vec<XmPatternInfo>,
    /// Instrument information.
    pub instruments: Vec<XmInstrumentInfo>,
    /// Validation errors.
    pub errors: Vec<XmFormatError>,
    /// Validation warnings.
    pub warnings: Vec<XmWarning>,
    /// Total file size.
    pub file_size: usize,
    /// Calculated expected file size.
    pub expected_size: usize,
}

impl XmValidationReport {
    /// Create a new empty report.
    pub(super) fn new(file_size: usize) -> Self {
        Self {
            valid: true,
            header: None,
            patterns: Vec::new(),
            instruments: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            file_size,
            expected_size: 0,
        }
    }

    /// Add an error to the report.
    pub(super) fn add_error(&mut self, error: XmFormatError) {
        self.errors.push(error);
        self.valid = false;
    }

    /// Add a warning to the report.
    pub(super) fn add_warning(&mut self, warning: XmWarning) {
        self.warnings.push(warning);
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}
