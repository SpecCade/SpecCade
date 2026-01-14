//! Sequence accessor utilities for emit_seq operations.

use speccade_spec::recipe::music::{PitchSeq, Seq, SeqMode};

use super::error::ExpandError;

/// Accessor for generic sequences (notes, instruments, volumes, etc.).
pub(super) struct SeqAccessor<'a, T> {
    seq: Option<&'a Seq<T>>,
    len: usize,
    pattern_name: &'a str,
}

impl<'a, T: Clone> SeqAccessor<'a, T> {
    pub fn new(
        seq: Option<&'a Seq<T>>,
        len: usize,
        pattern_name: &'a str,
    ) -> Result<Self, ExpandError> {
        if let Some(seq) = seq {
            match seq.mode {
                SeqMode::Cycle => {
                    if seq.values.is_empty() {
                        return Err(ExpandError::InvalidExpr {
                            pattern: pattern_name.to_string(),
                            message: "emit_seq cycle values must be non-empty".to_string(),
                        });
                    }
                }
                SeqMode::Once => {
                    if seq.values.len() != len {
                        return Err(ExpandError::InvalidExpr {
                            pattern: pattern_name.to_string(),
                            message: format!(
                                "emit_seq once values length {} != {}",
                                seq.values.len(),
                                len
                            ),
                        });
                    }
                }
            }
        }
        Ok(Self {
            seq,
            len,
            pattern_name,
        })
    }

    pub fn value(&self, index: usize) -> Result<Option<T>, ExpandError> {
        let Some(seq) = self.seq else {
            return Ok(None);
        };
        match seq.mode {
            SeqMode::Cycle => {
                if seq.values.is_empty() {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "emit_seq cycle values must be non-empty".to_string(),
                    });
                }
                Ok(Some(seq.values[index % seq.values.len()].clone()))
            }
            SeqMode::Once => {
                if seq.values.len() != self.len {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: format!(
                            "emit_seq once values length {} != {}",
                            seq.values.len(),
                            self.len
                        ),
                    });
                }
                Ok(Some(seq.values[index].clone()))
            }
        }
    }
}

/// Accessor for pitch sequences (scale degrees or chord tones).
pub(super) struct PitchSeqAccessor<'a> {
    seq: Option<&'a PitchSeq>,
    len: usize,
    pattern_name: &'a str,
}

impl<'a> PitchSeqAccessor<'a> {
    pub fn new(
        seq: Option<&'a PitchSeq>,
        len: usize,
        pattern_name: &'a str,
    ) -> Result<Self, ExpandError> {
        if let Some(seq) = seq {
            match seq.mode {
                SeqMode::Cycle => {
                    if seq.values.is_empty() {
                        return Err(ExpandError::InvalidExpr {
                            pattern: pattern_name.to_string(),
                            message: "emit_seq pitch_seq cycle values must be non-empty"
                                .to_string(),
                        });
                    }
                }
                SeqMode::Once => {
                    if seq.values.len() != len {
                        return Err(ExpandError::InvalidExpr {
                            pattern: pattern_name.to_string(),
                            message: format!(
                                "emit_seq pitch_seq once values length {} != {}",
                                seq.values.len(),
                                len
                            ),
                        });
                    }
                }
            }
        }

        Ok(Self {
            seq,
            len,
            pattern_name,
        })
    }

    pub fn seq(&self) -> Option<&'a PitchSeq> {
        self.seq
    }

    pub fn value(&self, index: usize) -> Result<Option<String>, ExpandError> {
        let Some(seq) = self.seq else {
            return Ok(None);
        };
        match seq.mode {
            SeqMode::Cycle => {
                if seq.values.is_empty() {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "emit_seq pitch_seq cycle values must be non-empty".to_string(),
                    });
                }
                Ok(Some(seq.values[index % seq.values.len()].clone()))
            }
            SeqMode::Once => {
                if seq.values.len() != self.len {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: format!(
                            "emit_seq pitch_seq once values length {} != {}",
                            seq.values.len(),
                            self.len
                        ),
                    });
                }
                Ok(Some(seq.values[index].clone()))
            }
        }
    }
}
