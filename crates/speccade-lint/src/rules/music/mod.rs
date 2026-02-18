//! Music quality lint rules.
//!
//! Rules for detecting perceptual problems in generated music assets (tracker patterns).

pub mod harmonic;
mod helpers;
pub mod structural;

#[cfg(test)]
mod tests;

use crate::rules::LintRule;

pub use harmonic::{
    InvalidNoteRule, ParallelFifthsRule, ParallelOctavesRule, UnresolvedTensionRule,
    VoiceCrossingRule,
};
pub use structural::{
    DensePatternRule, EmptyArrangementRule, EmptyPatternRule, ExtremeTempoRule, NoVariationRule,
    SparsePatternRule, UnusedChannelRule,
};

/// Returns all music lint rules.
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        // Error-level rules (3)
        Box::new(EmptyPatternRule),
        Box::new(InvalidNoteRule),
        Box::new(EmptyArrangementRule),
        // Warning-level rules (6)
        Box::new(ParallelOctavesRule),
        Box::new(ParallelFifthsRule),
        Box::new(VoiceCrossingRule),
        Box::new(DensePatternRule),
        Box::new(SparsePatternRule),
        Box::new(ExtremeTempoRule),
        // Info-level rules (3)
        Box::new(UnusedChannelRule),
        Box::new(NoVariationRule),
        Box::new(UnresolvedTensionRule),
    ]
}
