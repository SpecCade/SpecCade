//! Error types for IT format validation.

use std::fmt;

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
