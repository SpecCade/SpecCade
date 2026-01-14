//! Error types for compose expansion.

use thiserror::Error;

use speccade_spec::BackendError;

/// Errors that can occur during compose expansion.
#[derive(Debug, Error)]
pub enum ExpandError {
    #[error("unknown ref '{name}' in pattern '{pattern}'")]
    UnknownRef { pattern: String, name: String },
    #[error("ref cycle detected: {cycle}")]
    RefCycle { cycle: String },
    #[error("recursion depth exceeded (max {max}) in pattern '{pattern}'")]
    RecursionLimit { pattern: String, max: usize },
    #[error("invalid time expression in pattern '{pattern}': {message}")]
    InvalidTime { pattern: String, message: String },
    #[error("invalid pattern expression in pattern '{pattern}': {message}")]
    InvalidExpr { pattern: String, message: String },
    #[error("unknown channel alias '{alias}' in pattern '{pattern}'")]
    UnknownChannelAlias { pattern: String, alias: String },
    #[error("unknown instrument alias '{alias}' in pattern '{pattern}'")]
    UnknownInstrumentAlias { pattern: String, alias: String },
    #[error(
        "merge conflict at row {row}, channel {channel} on field '{field}' in pattern '{pattern}'"
    )]
    MergeConflict {
        pattern: String,
        row: i32,
        channel: u8,
        field: &'static str,
    },
    #[error("cell out of bounds (row {row}, channel {channel}) in pattern '{pattern}'")]
    CellOutOfBounds {
        pattern: String,
        row: i32,
        channel: u8,
    },
    #[error("instrument index {inst} out of range (len {len}) in pattern '{pattern}'")]
    InvalidInstrument {
        pattern: String,
        inst: u8,
        len: usize,
    },
    #[error("missing instrument for cell at row {row}, channel {channel} in pattern '{pattern}'")]
    MissingInstrument {
        pattern: String,
        row: i32,
        channel: u8,
    },
    #[error("expanded cell limit exceeded in pattern '{pattern}' (max {max})")]
    CellLimit { pattern: String, max: usize },
    #[error("time list limit exceeded in pattern '{pattern}' (max {max})")]
    TimeListLimit { pattern: String, max: usize },
    #[error("pattern string too long in pattern '{pattern}' (max {max})")]
    PatternStringLimit { pattern: String, max: usize },
}

impl BackendError for ExpandError {
    fn code(&self) -> &'static str {
        match self {
            ExpandError::UnknownRef { .. } => "MUSIC_COMPOSE_001",
            ExpandError::RefCycle { .. } => "MUSIC_COMPOSE_002",
            ExpandError::RecursionLimit { .. } => "MUSIC_COMPOSE_003",
            ExpandError::InvalidTime { .. } => "MUSIC_COMPOSE_004",
            ExpandError::InvalidExpr { .. } => "MUSIC_COMPOSE_005",
            ExpandError::UnknownChannelAlias { .. } => "MUSIC_COMPOSE_013",
            ExpandError::UnknownInstrumentAlias { .. } => "MUSIC_COMPOSE_014",
            ExpandError::MergeConflict { .. } => "MUSIC_COMPOSE_006",
            ExpandError::CellOutOfBounds { .. } => "MUSIC_COMPOSE_007",
            ExpandError::InvalidInstrument { .. } => "MUSIC_COMPOSE_008",
            ExpandError::MissingInstrument { .. } => "MUSIC_COMPOSE_009",
            ExpandError::CellLimit { .. } => "MUSIC_COMPOSE_010",
            ExpandError::TimeListLimit { .. } => "MUSIC_COMPOSE_011",
            ExpandError::PatternStringLimit { .. } => "MUSIC_COMPOSE_012",
        }
    }

    fn category(&self) -> &'static str {
        "music"
    }
}
