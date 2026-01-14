//! Validation report structures.

use super::error::ItFormatError;
use super::info::{ItHeaderInfo, ItInstrumentInfo, ItPatternInfo, ItSampleInfo};

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
    pub(super) fn new() -> Self {
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

    pub(super) fn add_warning(&mut self, message: impl Into<String>, offset: Option<usize>) {
        self.warnings.push(ItValidationWarning {
            message: message.into(),
            offset,
        });
    }

    pub(super) fn add_error(&mut self, error: ItFormatError) {
        self.errors.push(error);
    }
}
