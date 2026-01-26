//! Music quality lint rules.
//!
//! Rules for detecting perceptual problems in generated music assets (tracker patterns).

use crate::report::{AssetType, LintIssue, Severity};
use crate::rules::{AssetData, LintRule};
use speccade_spec::recipe::music::{MusicTrackerSongV1Params, PatternNote, TrackerPattern};
use speccade_spec::Spec;
use std::collections::{HashMap, HashSet};

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

// =============================================================================
// Helper functions
// =============================================================================

/// Convert note name to MIDI number.
/// Returns None for non-note values like "---" (note off) or "..." (no note).
fn note_to_midi(note: &str) -> Option<i32> {
    let note = note.trim();

    // Skip empty or special notes
    if note.is_empty() || note == "---" || note == "..." || note == "===" {
        return None;
    }

    // Try to parse as direct MIDI number first
    if let Ok(midi) = note.parse::<i32>() {
        return Some(midi);
    }

    // Parse note name format: e.g., "C4", "C#4", "Db4"
    let bytes = note.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    // Get the base note (C, D, E, F, G, A, B)
    let base_note = bytes[0].to_ascii_uppercase();
    let semitone = match base_note {
        b'C' => 0,
        b'D' => 2,
        b'E' => 4,
        b'F' => 5,
        b'G' => 7,
        b'A' => 9,
        b'B' => 11,
        _ => return None,
    };

    let mut idx = 1;
    let mut modifier = 0i32;

    // Check for sharps/flats
    if idx < bytes.len() {
        match bytes[idx] {
            b'#' | b's' | b'S' => {
                modifier = 1;
                idx += 1;
            }
            b'b' | b'f' | b'F' => {
                modifier = -1;
                idx += 1;
            }
            _ => {}
        }
    }

    // Parse octave number
    let octave_str = &note[idx..];
    let octave: i32 = octave_str.parse().ok()?;

    // MIDI note number: C4 = 60 (middle C)
    // Formula: (octave + 1) * 12 + semitone + modifier
    Some((octave + 1) * 12 + semitone + modifier)
}

/// Extract music params from spec.
fn extract_music_params(spec: Option<&Spec>) -> Option<MusicTrackerSongV1Params> {
    let spec = spec?;
    let recipe = spec.recipe.as_ref()?;

    // Try both music recipe kinds
    if recipe.kind == "music.tracker_song_v1" || recipe.kind == "music.tracker_song_compose_v1" {
        recipe.as_music_tracker_song().ok()
    } else {
        None
    }
}

/// Get all notes from a pattern organized by row.
fn notes_by_row(pattern: &TrackerPattern) -> HashMap<u16, Vec<(u8, &PatternNote)>> {
    let mut by_row: HashMap<u16, Vec<(u8, &PatternNote)>> = HashMap::new();
    for (channel, note) in pattern.flat_notes() {
        by_row.entry(note.row).or_default().push((channel, note));
    }
    by_row
}

/// Check if a pattern is empty (has no notes).
fn is_pattern_empty(pattern: &TrackerPattern) -> bool {
    let flat = pattern.flat_notes();
    flat.is_empty() || flat.iter().all(|(_, n)| {
        n.note.is_empty() || n.note == "---" || n.note == "..." || n.note == "==="
    })
}

/// Calculate interval in semitones between two MIDI notes.
fn interval(note1: i32, note2: i32) -> i32 {
    (note2 - note1).abs()
}

/// Check if an interval is dissonant (tritone = 6, minor 2nd = 1, major 7th = 11).
fn is_dissonant_interval(semitones: i32) -> bool {
    let normalized = semitones % 12;
    matches!(normalized, 1 | 6 | 11)
}

// =============================================================================
// Error-level rules (3)
// =============================================================================

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

// =============================================================================
// Warning-level rules (6)
// =============================================================================

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

                        let Some(midi1_a) = note_to_midi(&n1_a.note) else { continue };
                        let Some(midi1_b) = note_to_midi(&n1_b.note) else { continue };

                        // Find same channels in next row
                        let n2_a = notes2.iter().find(|(ch, _)| ch == ch1_a);
                        let n2_b = notes2.iter().find(|(ch, _)| ch == ch1_b);

                        if let (Some((_, n2_a)), Some((_, n2_b))) = (n2_a, n2_b) {
                            let Some(midi2_a) = note_to_midi(&n2_a.note) else { continue };
                            let Some(midi2_b) = note_to_midi(&n2_b.note) else { continue };

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

                        let Some(midi1_a) = note_to_midi(&n1_a.note) else { continue };
                        let Some(midi1_b) = note_to_midi(&n1_b.note) else { continue };

                        let n2_a = notes2.iter().find(|(ch, _)| ch == ch1_a);
                        let n2_b = notes2.iter().find(|(ch, _)| ch == ch1_b);

                        if let (Some((_, n2_a)), Some((_, n2_b))) = (n2_a, n2_b) {
                            let Some(midi2_a) = note_to_midi(&n2_a.note) else { continue };
                            let Some(midi2_b) = note_to_midi(&n2_b.note) else { continue };

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
                let mut sorted_notes: Vec<_> = notes.iter()
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
                let note_count = notes.iter()
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
                        .with_spec_path(format!(
                            "recipe.params.patterns[\"{}\"]",
                            pattern_name
                        ))
                        .with_asset_location(format!(
                            "pattern:{}:row{}",
                            pattern_name, row
                        ))
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
            let filled_cells = pattern.flat_notes().iter()
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
                            pattern_name, occupancy * 100.0, filled_cells, total_cells
                        ),
                        "Add more notes or reduce pattern size",
                    )
                    .with_spec_path(format!(
                        "recipe.params.patterns[\"{}\"]",
                        pattern_name
                    ))
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

// =============================================================================
// Info-level rules (3)
// =============================================================================

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
        let midi_values: Vec<i32> = last_notes.iter()
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::music::ArrangementEntry;
    use std::path::Path;

    /// Helper to create test asset data.
    fn test_asset_data() -> AssetData<'static> {
        AssetData {
            path: Path::new("test.xm"),
            bytes: &[],
        }
    }

    /// Helper to create a basic spec with music params.
    fn make_music_spec(params: MusicTrackerSongV1Params) -> Spec {
        use speccade_spec::recipe::Recipe;
        use speccade_spec::spec::AssetType;

        Spec {
            spec_version: 1,
            asset_id: "test-music".to_string(),
            asset_type: AssetType::Music,
            license: "CC0-1.0".to_string(),
            seed: 42,
            outputs: vec![],
            description: None,
            style_tags: None,
            engine_targets: None,
            migration_notes: None,
            variants: None,
            recipe: Some(Recipe::new(
                "music.tracker_song_v1",
                serde_json::to_value(params).unwrap(),
            )),
        }
    }

    /// Helper to create a pattern with notes.
    fn make_pattern(rows: u16, notes: Vec<(u8, u16, &str)>) -> TrackerPattern {
        let mut notes_map: HashMap<String, Vec<PatternNote>> = HashMap::new();

        for (channel, row, note) in notes {
            notes_map
                .entry(channel.to_string())
                .or_default()
                .push(PatternNote {
                    row,
                    channel: None,
                    note: note.to_string(),
                    inst: 1,
                    vol: None,
                    effect: None,
                    param: None,
                    effect_name: None,
                    effect_xy: None,
                });
        }

        TrackerPattern {
            rows,
            notes: Some(notes_map),
            data: None,
        }
    }

    // -------------------------------------------------------------------------
    // note_to_midi tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_note_to_midi_basic() {
        assert_eq!(note_to_midi("C4"), Some(60));
        assert_eq!(note_to_midi("A4"), Some(69));
        assert_eq!(note_to_midi("C0"), Some(12));
        assert_eq!(note_to_midi("C-1"), Some(0));
    }

    #[test]
    fn test_note_to_midi_sharps_flats() {
        assert_eq!(note_to_midi("C#4"), Some(61));  // C4=60 + 1 = 61
        assert_eq!(note_to_midi("Db4"), Some(61));  // D4=62 - 1 = 61 (enharmonic with C#4)
        assert_eq!(note_to_midi("F#3"), Some(54));  // F3=53 + 1 = 54
    }

    #[test]
    fn test_note_to_midi_special() {
        assert_eq!(note_to_midi("---"), None);
        assert_eq!(note_to_midi("..."), None);
        assert_eq!(note_to_midi("==="), None);
        assert_eq!(note_to_midi(""), None);
    }

    #[test]
    fn test_note_to_midi_numeric() {
        assert_eq!(note_to_midi("60"), Some(60));
        assert_eq!(note_to_midi("0"), Some(0));
        assert_eq!(note_to_midi("127"), Some(127));
    }

    // -------------------------------------------------------------------------
    // EmptyPatternRule tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_empty_pattern_rule_detects_empty() {
        let rule = EmptyPatternRule;
        let asset = test_asset_data();

        let mut patterns = HashMap::new();
        patterns.insert("intro".to_string(), TrackerPattern {
            rows: 64,
            notes: Some(HashMap::new()),
            data: None,
        });

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns,
            arrangement: vec![ArrangementEntry {
                pattern: "intro".to_string(),
                repeat: 1,
            }],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "music/empty-pattern");
    }

    #[test]
    fn test_empty_pattern_rule_passes_with_notes() {
        let rule = EmptyPatternRule;
        let asset = test_asset_data();

        let mut patterns = HashMap::new();
        patterns.insert("intro".to_string(), make_pattern(64, vec![
            (0, 0, "C4"),
            (0, 16, "E4"),
        ]));

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns,
            arrangement: vec![ArrangementEntry {
                pattern: "intro".to_string(),
                repeat: 1,
            }],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert!(issues.is_empty());
    }

    // -------------------------------------------------------------------------
    // InvalidNoteRule tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_invalid_note_rule_detects_out_of_range() {
        let rule = InvalidNoteRule;
        let asset = test_asset_data();

        let mut patterns = HashMap::new();
        patterns.insert("test".to_string(), make_pattern(64, vec![
            (0, 0, "C12"),  // Invalid: too high (C12 = 156)
        ]));

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns,
            arrangement: vec![],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "music/invalid-note");
    }

    #[test]
    fn test_invalid_note_rule_passes_valid_notes() {
        let rule = InvalidNoteRule;
        let asset = test_asset_data();

        let mut patterns = HashMap::new();
        patterns.insert("test".to_string(), make_pattern(64, vec![
            (0, 0, "C4"),
            (1, 0, "G9"),  // G9 = 127 (highest valid)
            (2, 0, "C-1"), // C-1 = 0 (lowest valid)
        ]));

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns,
            arrangement: vec![],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert!(issues.is_empty());
    }

    // -------------------------------------------------------------------------
    // EmptyArrangementRule tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_empty_arrangement_rule_detects_empty() {
        let rule = EmptyArrangementRule;
        let asset = test_asset_data();

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns: HashMap::new(),
            arrangement: vec![],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "music/empty-arrangement");
    }

    #[test]
    fn test_empty_arrangement_rule_passes_with_entries() {
        let rule = EmptyArrangementRule;
        let asset = test_asset_data();

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns: HashMap::new(),
            arrangement: vec![ArrangementEntry {
                pattern: "intro".to_string(),
                repeat: 1,
            }],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert!(issues.is_empty());
    }

    // -------------------------------------------------------------------------
    // ExtremeTempoRule tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_extreme_tempo_rule_detects_too_slow() {
        let rule = ExtremeTempoRule;
        let asset = test_asset_data();

        let params = MusicTrackerSongV1Params {
            bpm: 30, // Too slow
            speed: 6,
            channels: 4,
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "music/extreme-tempo");
    }

    #[test]
    fn test_extreme_tempo_rule_detects_too_fast() {
        let rule = ExtremeTempoRule;
        let asset = test_asset_data();

        let params = MusicTrackerSongV1Params {
            bpm: 350, // Too fast
            speed: 6,
            channels: 4,
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "music/extreme-tempo");
    }

    #[test]
    fn test_extreme_tempo_rule_passes_normal() {
        let rule = ExtremeTempoRule;
        let asset = test_asset_data();

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert!(issues.is_empty());
    }

    // -------------------------------------------------------------------------
    // DensePatternRule tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_dense_pattern_rule_detects_too_many_notes() {
        let rule = DensePatternRule;
        let asset = test_asset_data();

        let mut patterns = HashMap::new();
        // 10 simultaneous notes on row 0
        patterns.insert("dense".to_string(), make_pattern(64, vec![
            (0, 0, "C4"), (1, 0, "D4"), (2, 0, "E4"), (3, 0, "F4"),
            (4, 0, "G4"), (5, 0, "A4"), (6, 0, "B4"), (7, 0, "C5"),
            (8, 0, "D5"), (9, 0, "E5"),
        ]));

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 16,
            patterns,
            arrangement: vec![],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "music/dense-pattern");
    }

    #[test]
    fn test_dense_pattern_rule_passes_normal() {
        let rule = DensePatternRule;
        let asset = test_asset_data();

        let mut patterns = HashMap::new();
        patterns.insert("normal".to_string(), make_pattern(64, vec![
            (0, 0, "C4"), (1, 0, "E4"), (2, 0, "G4"), (3, 0, "C5"),
        ]));

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns,
            arrangement: vec![],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert!(issues.is_empty());
    }

    // -------------------------------------------------------------------------
    // SparsePatternRule tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sparse_pattern_rule_detects_sparse() {
        let rule = SparsePatternRule;
        let asset = test_asset_data();

        let mut patterns = HashMap::new();
        // 64 rows * 8 channels = 512 cells, only 2 notes = 0.4%
        patterns.insert("sparse".to_string(), make_pattern(64, vec![
            (0, 0, "C4"),
            (0, 32, "E4"),
        ]));

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 8,
            patterns,
            arrangement: vec![],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "music/sparse-pattern");
    }

    // -------------------------------------------------------------------------
    // NoVariationRule tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_no_variation_rule_detects_repetition() {
        let rule = NoVariationRule;
        let asset = test_asset_data();

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns: HashMap::new(),
            arrangement: vec![
                ArrangementEntry { pattern: "A".to_string(), repeat: 8 },
            ],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "music/no-variation");
    }

    #[test]
    fn test_no_variation_rule_passes_with_variation() {
        let rule = NoVariationRule;
        let asset = test_asset_data();

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns: HashMap::new(),
            arrangement: vec![
                ArrangementEntry { pattern: "A".to_string(), repeat: 2 },
                ArrangementEntry { pattern: "B".to_string(), repeat: 2 },
                ArrangementEntry { pattern: "A".to_string(), repeat: 2 },
            ],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert!(issues.is_empty());
    }

    // -------------------------------------------------------------------------
    // UnusedChannelRule tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_unused_channel_rule_detects_unused() {
        let rule = UnusedChannelRule;
        let asset = test_asset_data();

        let mut patterns = HashMap::new();
        // Only channels 0 and 1 used, but 4 channels declared
        patterns.insert("test".to_string(), make_pattern(64, vec![
            (0, 0, "C4"),
            (1, 0, "E4"),
        ]));

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns,
            arrangement: vec![],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        // Channels 2 and 3 are unused
        assert_eq!(issues.len(), 2);
        assert!(issues.iter().all(|i| i.rule_id == "music/unused-channel"));
    }

    // -------------------------------------------------------------------------
    // UnresolvedTensionRule tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_unresolved_tension_rule_detects_tritone() {
        let rule = UnresolvedTensionRule;
        let asset = test_asset_data();

        let mut patterns = HashMap::new();
        // Ends with C4 and F#4 (tritone = 6 semitones)
        patterns.insert("ending".to_string(), make_pattern(64, vec![
            (0, 63, "C4"),
            (1, 63, "F#4"),
        ]));

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns,
            arrangement: vec![ArrangementEntry {
                pattern: "ending".to_string(),
                repeat: 1,
            }],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "music/unresolved-tension");
        assert!(issues[0].message.contains("tritone"));
    }

    #[test]
    fn test_unresolved_tension_rule_passes_consonant() {
        let rule = UnresolvedTensionRule;
        let asset = test_asset_data();

        let mut patterns = HashMap::new();
        // Ends with C major chord (consonant)
        patterns.insert("ending".to_string(), make_pattern(64, vec![
            (0, 63, "C4"),
            (1, 63, "E4"),
            (2, 63, "G4"),
        ]));

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns,
            arrangement: vec![ArrangementEntry {
                pattern: "ending".to_string(),
                repeat: 1,
            }],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert!(issues.is_empty());
    }

    // -------------------------------------------------------------------------
    // VoiceCrossingRule tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_voice_crossing_rule_detects_crossing() {
        let rule = VoiceCrossingRule;
        let asset = test_asset_data();

        let mut patterns = HashMap::new();
        // Channel 0 (soprano) at C3, Channel 1 (alto) at C5 - crossing!
        patterns.insert("test".to_string(), make_pattern(64, vec![
            (0, 0, "C3"),  // Soprano too low
            (1, 0, "C5"),  // Alto too high
        ]));

        let params = MusicTrackerSongV1Params {
            bpm: 120,
            speed: 6,
            channels: 4,
            patterns,
            arrangement: vec![],
            ..Default::default()
        };

        let spec = make_music_spec(params);
        let issues = rule.check(&asset, Some(&spec));

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "music/voice-crossing");
    }

    // -------------------------------------------------------------------------
    // all_rules test
    // -------------------------------------------------------------------------

    #[test]
    fn test_all_rules_returns_12_rules() {
        let rules = all_rules();
        assert_eq!(rules.len(), 12);

        // Verify each rule has the correct prefix
        for rule in &rules {
            assert!(rule.id().starts_with("music/"));
            assert!(rule.applies_to().contains(&AssetType::Music));
        }
    }

    #[test]
    fn test_all_rules_unique_ids() {
        let rules = all_rules();
        let ids: HashSet<_> = rules.iter().map(|r| r.id()).collect();
        assert_eq!(ids.len(), rules.len(), "All rule IDs should be unique");
    }

    #[test]
    fn test_severity_distribution() {
        let rules = all_rules();

        let errors = rules.iter().filter(|r| r.default_severity() == Severity::Error).count();
        let warnings = rules.iter().filter(|r| r.default_severity() == Severity::Warning).count();
        let infos = rules.iter().filter(|r| r.default_severity() == Severity::Info).count();

        assert_eq!(errors, 3, "Expected 3 error-level rules");
        assert_eq!(warnings, 6, "Expected 6 warning-level rules");
        assert_eq!(infos, 3, "Expected 3 info-level rules");
    }
}
