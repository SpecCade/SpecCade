//! Shared helper functions for music lint rules.

use speccade_spec::recipe::music::{MusicTrackerSongV1Params, PatternNote, TrackerPattern};
use speccade_spec::Spec;
use std::collections::HashMap;

/// Convert note name to MIDI number.
/// Returns None for non-note values like "---" (note off) or "..." (no note).
pub(super) fn note_to_midi(note: &str) -> Option<i32> {
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
pub(super) fn extract_music_params(spec: Option<&Spec>) -> Option<MusicTrackerSongV1Params> {
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
pub(super) fn notes_by_row(pattern: &TrackerPattern) -> HashMap<u16, Vec<(u8, &PatternNote)>> {
    let mut by_row: HashMap<u16, Vec<(u8, &PatternNote)>> = HashMap::new();
    for (channel, note) in pattern.flat_notes() {
        by_row.entry(note.row).or_default().push((channel, note));
    }
    by_row
}

/// Check if a pattern is empty (has no notes).
pub(super) fn is_pattern_empty(pattern: &TrackerPattern) -> bool {
    let flat = pattern.flat_notes();
    flat.is_empty()
        || flat.iter().all(|(_, n)| {
            n.note.is_empty() || n.note == "---" || n.note == "..." || n.note == "==="
        })
}

/// Calculate interval in semitones between two MIDI notes.
pub(super) fn interval(note1: i32, note2: i32) -> i32 {
    (note2 - note1).abs()
}

/// Check if an interval is dissonant (tritone = 6, minor 2nd = 1, major 7th = 11).
pub(super) fn is_dissonant_interval(semitones: i32) -> bool {
    let normalized = semitones % 12;
    matches!(normalized, 1 | 6 | 11)
}
