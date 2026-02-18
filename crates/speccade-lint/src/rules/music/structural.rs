//! Structural music lint rules.
//!
//! Rules that check pattern structure, arrangement, tempo, and channel usage.

use crate::report::{AssetType, LintIssue, Severity};
use crate::rules::{AssetData, LintRule};
use speccade_spec::Spec;
use std::collections::HashSet;

use super::helpers::{extract_music_params, is_pattern_empty, note_to_midi, notes_by_row};

/// Rule: music/empty-pattern
/// Detects patterns with no notes.
pub struct EmptyPatternRule;

impl LintRule for EmptyPatternRule {
    fn id(&self) -> &'static str {
        "music/empty-pattern"
    }

    fn description(&self) -> &'static str {
        "Pattern has no notes"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Music]
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, _asset: &AssetData, spec: Option<&Spec>) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let Some(params) = extract_music_params(spec) else {
            return issues;
        };

        for (name, pattern) in &params.patterns {
            if is_pattern_empty(pattern) {
                issues.push(
                    LintIssue::new(
                        self.id(),
                        self.default_severity(),
                        format!("Pattern '{}' has no notes", name),
                        "Add notes or remove pattern",
                    )
                    .with_spec_path(format!("recipe.params.patterns[\"{}\"]", name))
                    .with_asset_location(format!("pattern:{}", name)),
                );
            }
        }

        issues
    }
}

/// Rule: music/empty-arrangement
/// Detects songs with no patterns in the arrangement.
pub struct EmptyArrangementRule;

impl LintRule for EmptyArrangementRule {
    fn id(&self) -> &'static str {
        "music/empty-arrangement"
    }

    fn description(&self) -> &'static str {
        "No patterns in arrangement"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Music]
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, _asset: &AssetData, spec: Option<&Spec>) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let Some(params) = extract_music_params(spec) else {
            return issues;
        };

        if params.arrangement.is_empty() {
            issues.push(
                LintIssue::new(
                    self.id(),
                    self.default_severity(),
                    "Song arrangement is empty",
                    "Add patterns to arrangement",
                )
                .with_spec_path("recipe.params.arrangement"),
            );
        }

        issues
    }
}

/// Rule: music/dense-pattern
/// Detects rows with more than 8 simultaneous notes.
pub struct DensePatternRule;

impl LintRule for DensePatternRule {
    fn id(&self) -> &'static str {
        "music/dense-pattern"
    }

    fn description(&self) -> &'static str {
        "More than 8 simultaneous notes in a row"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Music]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, _asset: &AssetData, spec: Option<&Spec>) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let Some(params) = extract_music_params(spec) else {
            return issues;
        };

        const MAX_POLYPHONY: usize = 8;

        for (pattern_name, pattern) in &params.patterns {
            let by_row = notes_by_row(pattern);

            for (row, notes) in &by_row {
                // Count actual notes (not note-offs or empty)
                let note_count = notes
                    .iter()
                    .filter(|(_, n)| note_to_midi(&n.note).is_some())
                    .count();

                if note_count > MAX_POLYPHONY {
                    issues.push(
                        LintIssue::new(
                            self.id(),
                            self.default_severity(),
                            format!(
                                "Row {} has {} simultaneous notes (max {})",
                                row, note_count, MAX_POLYPHONY
                            ),
                            "Reduce polyphony by removing or offsetting notes",
                        )
                        .with_spec_path(format!("recipe.params.patterns[\"{}\"]", pattern_name))
                        .with_asset_location(format!("pattern:{}:row{}", pattern_name, row))
                        .with_actual_value(note_count.to_string())
                        .with_expected_range(format!("1-{}", MAX_POLYPHONY)),
                    );
                }
            }
        }

        issues
    }
}

/// Rule: music/sparse-pattern
/// Detects patterns with less than 5% cell occupancy.
pub struct SparsePatternRule;

impl LintRule for SparsePatternRule {
    fn id(&self) -> &'static str {
        "music/sparse-pattern"
    }

    fn description(&self) -> &'static str {
        "Pattern has less than 5% cell occupancy"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Music]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, _asset: &AssetData, spec: Option<&Spec>) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let Some(params) = extract_music_params(spec) else {
            return issues;
        };

        const MIN_OCCUPANCY: f64 = 0.05; // 5%

        for (pattern_name, pattern) in &params.patterns {
            let rows = pattern.rows as usize;
            let channels = params.channels as usize;

            if rows == 0 || channels == 0 {
                continue;
            }

            let total_cells = rows * channels;
            let filled_cells = pattern
                .flat_notes()
                .iter()
                .filter(|(_, n)| note_to_midi(&n.note).is_some())
                .count();

            let occupancy = filled_cells as f64 / total_cells as f64;

            if occupancy < MIN_OCCUPANCY && filled_cells > 0 {
                issues.push(
                    LintIssue::new(
                        self.id(),
                        self.default_severity(),
                        format!(
                            "Pattern '{}' has only {:.1}% cell occupancy ({}/{} cells)",
                            pattern_name,
                            occupancy * 100.0,
                            filled_cells,
                            total_cells
                        ),
                        "Add more notes or reduce pattern size",
                    )
                    .with_spec_path(format!("recipe.params.patterns[\"{}\"]", pattern_name))
                    .with_asset_location(format!("pattern:{}", pattern_name))
                    .with_actual_value(format!("{:.1}%", occupancy * 100.0))
                    .with_expected_range(format!(">={:.0}%", MIN_OCCUPANCY * 100.0)),
                );
            }
        }

        issues
    }
}

/// Rule: music/extreme-tempo
/// Detects BPM values outside reasonable range (40-300).
pub struct ExtremeTempoRule;

impl LintRule for ExtremeTempoRule {
    fn id(&self) -> &'static str {
        "music/extreme-tempo"
    }

    fn description(&self) -> &'static str {
        "BPM outside reasonable range (40-300)"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Music]
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, _asset: &AssetData, spec: Option<&Spec>) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let Some(params) = extract_music_params(spec) else {
            return issues;
        };

        const MIN_BPM: u16 = 40;
        const MAX_BPM: u16 = 300;

        if params.bpm < MIN_BPM || params.bpm > MAX_BPM {
            issues.push(
                LintIssue::new(
                    self.id(),
                    self.default_severity(),
                    format!("Tempo {} BPM is outside reasonable range", params.bpm),
                    format!("Adjust tempo to {}-{} BPM", MIN_BPM, MAX_BPM),
                )
                .with_spec_path("recipe.params.bpm")
                .with_actual_value(params.bpm.to_string())
                .with_expected_range(format!("{}-{}", MIN_BPM, MAX_BPM))
                .with_fix_param("bpm"),
            );
        }

        issues
    }
}

/// Rule: music/unused-channel
/// Detects channels with no notes across all patterns.
pub struct UnusedChannelRule;

impl LintRule for UnusedChannelRule {
    fn id(&self) -> &'static str {
        "music/unused-channel"
    }

    fn description(&self) -> &'static str {
        "Channel has no notes across all patterns"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Music]
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, _asset: &AssetData, spec: Option<&Spec>) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let Some(params) = extract_music_params(spec) else {
            return issues;
        };

        let total_channels = params.channels;
        if total_channels == 0 {
            return issues;
        }

        // Collect all channels that have at least one note
        let mut used_channels: HashSet<u8> = HashSet::new();

        for pattern in params.patterns.values() {
            for (channel, note) in pattern.flat_notes() {
                if note_to_midi(&note.note).is_some() {
                    used_channels.insert(channel);
                }
            }
        }

        // Report unused channels
        for ch in 0..total_channels {
            if !used_channels.contains(&ch) {
                issues.push(
                    LintIssue::new(
                        self.id(),
                        self.default_severity(),
                        format!("Channel {} has no notes in any pattern", ch),
                        "Remove channel or add content to it",
                    )
                    .with_spec_path(format!("recipe.params.patterns[*].notes[\"{}\"]", ch))
                    .with_asset_location(format!("channel:{}", ch)),
                );
            }
        }

        issues
    }
}

/// Rule: music/no-variation
/// Detects patterns that repeat more than 4 times consecutively.
pub struct NoVariationRule;

impl LintRule for NoVariationRule {
    fn id(&self) -> &'static str {
        "music/no-variation"
    }

    fn description(&self) -> &'static str {
        "Same pattern repeats more than 4 times consecutively"
    }

    fn applies_to(&self) -> &[AssetType] {
        &[AssetType::Music]
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, _asset: &AssetData, spec: Option<&Spec>) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let Some(params) = extract_music_params(spec) else {
            return issues;
        };

        if params.arrangement.is_empty() {
            return issues;
        }

        const MAX_CONSECUTIVE: u16 = 4;

        // Expand arrangement with repeats
        let mut expanded: Vec<&str> = Vec::new();
        for entry in &params.arrangement {
            for _ in 0..entry.repeat {
                expanded.push(&entry.pattern);
            }
        }

        // Find consecutive repeats
        let mut i = 0;
        while i < expanded.len() {
            let pattern = expanded[i];
            let mut count = 1;

            while i + count < expanded.len() && expanded[i + count] == pattern {
                count += 1;
            }

            if count > MAX_CONSECUTIVE as usize {
                issues.push(
                    LintIssue::new(
                        self.id(),
                        self.default_severity(),
                        format!(
                            "Pattern '{}' repeats {} times consecutively",
                            pattern, count
                        ),
                        "Add variation (B-section) or reduce repeats",
                    )
                    .with_spec_path("recipe.params.arrangement")
                    .with_asset_location(format!("arrangement:index{}", i))
                    .with_actual_value(count.to_string())
                    .with_expected_range(format!("<={}", MAX_CONSECUTIVE)),
                );
            }

            i += count;
        }

        issues
    }
}
