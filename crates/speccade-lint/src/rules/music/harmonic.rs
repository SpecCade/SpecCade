//! Harmonic music lint rules.
//!
//! Rules that check note validity, voice leading, parallel motion, and harmonic tension.

use crate::report::{AssetType, LintIssue, Severity};
use crate::rules::{AssetData, LintRule};
use speccade_spec::Spec;

use super::helpers::{
    extract_music_params, interval, is_dissonant_interval, note_to_midi, notes_by_row,
};

/// Rule: music/invalid-note
/// Detects notes outside valid MIDI range (0-127, C-1 to G9).
pub struct InvalidNoteRule;

impl LintRule for InvalidNoteRule {
    fn id(&self) -> &'static str {
        "music/invalid-note"
    }

    fn description(&self) -> &'static str {
        "Note outside MIDI range (0-127)"
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

        for (pattern_name, pattern) in &params.patterns {
            for (channel, note) in pattern.flat_notes() {
                if let Some(midi) = note_to_midi(&note.note) {
                    if !(0..=127).contains(&midi) {
                        issues.push(
                            LintIssue::new(
                                self.id(),
                                self.default_severity(),
                                format!(
                                    "Note '{}' at row {} channel {} is outside MIDI range",
                                    note.note, note.row, channel
                                ),
                                "Transpose to valid range (0-127, C-1 to G9)",
                            )
                            .with_spec_path(format!(
                                "recipe.params.patterns[\"{}\"].notes[\"{}\"]",
                                pattern_name, channel
                            ))
                            .with_asset_location(format!(
                                "pattern:{}:row{}:ch{}",
                                pattern_name, note.row, channel
                            ))
                            .with_actual_value(midi.to_string())
                            .with_expected_range("0-127"),
                        );
                    }
                }
            }
        }

        issues
    }
}

/// Rule: music/parallel-octaves
/// Detects consecutive parallel octaves between voices.
pub struct ParallelOctavesRule;

impl LintRule for ParallelOctavesRule {
    fn id(&self) -> &'static str {
        "music/parallel-octaves"
    }

    fn description(&self) -> &'static str {
        "Consecutive parallel octaves between voices"
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

        for (pattern_name, pattern) in &params.patterns {
            let by_row = notes_by_row(pattern);
            let mut rows: Vec<u16> = by_row.keys().copied().collect();
            rows.sort();

            for window in rows.windows(2) {
                let (row1, row2) = (window[0], window[1]);
                let notes1 = &by_row[&row1];
                let notes2 = &by_row[&row2];

                // Check all pairs of voices between consecutive rows
                for (ch1_a, n1_a) in notes1 {
                    for (ch1_b, n1_b) in notes1 {
                        if ch1_a >= ch1_b {
                            continue;
                        }

                        let Some(midi1_a) = note_to_midi(&n1_a.note) else {
                            continue;
                        };
                        let Some(midi1_b) = note_to_midi(&n1_b.note) else {
                            continue;
                        };

                        // Find same channels in next row
                        let n2_a = notes2.iter().find(|(ch, _)| ch == ch1_a);
                        let n2_b = notes2.iter().find(|(ch, _)| ch == ch1_b);

                        if let (Some((_, n2_a)), Some((_, n2_b))) = (n2_a, n2_b) {
                            let Some(midi2_a) = note_to_midi(&n2_a.note) else {
                                continue;
                            };
                            let Some(midi2_b) = note_to_midi(&n2_b.note) else {
                                continue;
                            };

                            let interval1 = interval(midi1_a, midi1_b);
                            let interval2 = interval(midi2_a, midi2_b);

                            // Both intervals are octaves (12 semitones) and both voices moved
                            if interval1 == 12 && interval2 == 12 {
                                let motion_a = midi2_a - midi1_a;
                                let motion_b = midi2_b - midi1_b;

                                // Parallel motion: same direction and same interval
                                if motion_a == motion_b && motion_a != 0 {
                                    issues.push(
                                        LintIssue::new(
                                            self.id(),
                                            self.default_severity(),
                                            format!(
                                                "Parallel octaves between channels {} and {} at rows {}-{}",
                                                ch1_a, ch1_b, row1, row2
                                            ),
                                            "Use contrary motion or different intervals",
                                        )
                                        .with_spec_path(format!(
                                            "recipe.params.patterns[\"{}\"]",
                                            pattern_name
                                        ))
                                        .with_asset_location(format!(
                                            "pattern:{}:rows{}-{}",
                                            pattern_name, row1, row2
                                        )),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        issues
    }
}

/// Rule: music/parallel-fifths
/// Detects consecutive parallel fifths between voices.
pub struct ParallelFifthsRule;

impl LintRule for ParallelFifthsRule {
    fn id(&self) -> &'static str {
        "music/parallel-fifths"
    }

    fn description(&self) -> &'static str {
        "Consecutive parallel fifths between voices"
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

        for (pattern_name, pattern) in &params.patterns {
            let by_row = notes_by_row(pattern);
            let mut rows: Vec<u16> = by_row.keys().copied().collect();
            rows.sort();

            for window in rows.windows(2) {
                let (row1, row2) = (window[0], window[1]);
                let notes1 = &by_row[&row1];
                let notes2 = &by_row[&row2];

                // Check all pairs of voices
                for (ch1_a, n1_a) in notes1 {
                    for (ch1_b, n1_b) in notes1 {
                        if ch1_a >= ch1_b {
                            continue;
                        }

                        let Some(midi1_a) = note_to_midi(&n1_a.note) else {
                            continue;
                        };
                        let Some(midi1_b) = note_to_midi(&n1_b.note) else {
                            continue;
                        };

                        let n2_a = notes2.iter().find(|(ch, _)| ch == ch1_a);
                        let n2_b = notes2.iter().find(|(ch, _)| ch == ch1_b);

                        if let (Some((_, n2_a)), Some((_, n2_b))) = (n2_a, n2_b) {
                            let Some(midi2_a) = note_to_midi(&n2_a.note) else {
                                continue;
                            };
                            let Some(midi2_b) = note_to_midi(&n2_b.note) else {
                                continue;
                            };

                            let interval1 = interval(midi1_a, midi1_b) % 12;
                            let interval2 = interval(midi2_a, midi2_b) % 12;

                            // Both intervals are fifths (7 semitones)
                            if interval1 == 7 && interval2 == 7 {
                                let motion_a = midi2_a - midi1_a;
                                let motion_b = midi2_b - midi1_b;

                                // Parallel motion
                                if motion_a == motion_b && motion_a != 0 {
                                    issues.push(
                                        LintIssue::new(
                                            self.id(),
                                            self.default_severity(),
                                            format!(
                                                "Parallel fifths between channels {} and {} at rows {}-{}",
                                                ch1_a, ch1_b, row1, row2
                                            ),
                                            "Use different intervals or contrary motion",
                                        )
                                        .with_spec_path(format!(
                                            "recipe.params.patterns[\"{}\"]",
                                            pattern_name
                                        ))
                                        .with_asset_location(format!(
                                            "pattern:{}:rows{}-{}",
                                            pattern_name, row1, row2
                                        )),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        issues
    }
}

/// Rule: music/voice-crossing
/// Detects when a lower voice is above a higher voice.
pub struct VoiceCrossingRule;

impl LintRule for VoiceCrossingRule {
    fn id(&self) -> &'static str {
        "music/voice-crossing"
    }

    fn description(&self) -> &'static str {
        "Lower voice above higher voice"
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

        for (pattern_name, pattern) in &params.patterns {
            let by_row = notes_by_row(pattern);

            for (row, notes) in &by_row {
                // Sort notes by channel (lower channel = higher voice typically)
                let mut sorted_notes: Vec<_> = notes
                    .iter()
                    .filter_map(|(ch, n)| note_to_midi(&n.note).map(|midi| (*ch, midi)))
                    .collect();
                sorted_notes.sort_by_key(|(ch, _)| *ch);

                // Check for crossing (lower channel should have higher or equal pitch)
                for window in sorted_notes.windows(2) {
                    let (ch1, midi1) = window[0];
                    let (ch2, midi2) = window[1];

                    // If lower channel number (higher voice) has lower pitch than higher channel
                    if midi1 < midi2 {
                        issues.push(
                            LintIssue::new(
                                self.id(),
                                self.default_severity(),
                                format!(
                                    "Voice crossing at row {}: channel {} (pitch {}) below channel {} (pitch {})",
                                    row, ch1, midi1, ch2, midi2
                                ),
                                "Adjust voicing to maintain proper voice leading",
                            )
                            .with_spec_path(format!(
                                "recipe.params.patterns[\"{}\"]",
                                pattern_name
                            ))
                            .with_asset_location(format!(
                                "pattern:{}:row{}",
                                pattern_name, row
                            )),
                        );
                    }
                }
            }
        }

        issues
    }
}

/// Rule: music/unresolved-tension
/// Detects songs ending on dissonant intervals.
pub struct UnresolvedTensionRule;

impl LintRule for UnresolvedTensionRule {
    fn id(&self) -> &'static str {
        "music/unresolved-tension"
    }

    fn description(&self) -> &'static str {
        "Song ends on dissonant interval"
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

        // Get the last pattern in the arrangement
        let last_entry = params.arrangement.last().unwrap();
        let Some(last_pattern) = params.patterns.get(&last_entry.pattern) else {
            return issues;
        };

        // Find the last row with notes
        let by_row = notes_by_row(last_pattern);
        if by_row.is_empty() {
            return issues;
        }

        let last_row = *by_row.keys().max().unwrap();
        let last_notes = &by_row[&last_row];

        // Get all MIDI values in the last row
        let midi_values: Vec<i32> = last_notes
            .iter()
            .filter_map(|(_, n)| note_to_midi(&n.note))
            .collect();

        if midi_values.len() < 2 {
            return issues;
        }

        // Check all intervals in the final chord
        for i in 0..midi_values.len() {
            for j in (i + 1)..midi_values.len() {
                let int = interval(midi_values[i], midi_values[j]);
                if is_dissonant_interval(int) {
                    let interval_name = match int % 12 {
                        1 => "minor 2nd",
                        6 => "tritone",
                        11 => "major 7th",
                        _ => "dissonant interval",
                    };

                    issues.push(
                        LintIssue::new(
                            self.id(),
                            self.default_severity(),
                            format!(
                                "Song ends with {} ({} semitones) between notes",
                                interval_name, int
                            ),
                            "Resolve to consonant interval (unison, 3rd, 5th, octave)",
                        )
                        .with_spec_path(format!(
                            "recipe.params.patterns[\"{}\"]",
                            last_entry.pattern
                        ))
                        .with_asset_location(format!(
                            "pattern:{}:row{}",
                            last_entry.pattern, last_row
                        )),
                    );

                    // Only report one dissonance per ending
                    return issues;
                }
            }
        }

        issues
    }
}
