//! Note name parsing and conversion between XM and IT formats.

use super::constants::{it, xm, SEMITONE_MAP};

/// Convert a note name (e.g., "C4", "A#3", "Bb5") to an XM note value.
///
/// # Arguments
/// * `name` - Note name string (e.g., "C4", "A#3", "Bb5", "---", "OFF")
///
/// # Returns
/// XM note value (0 for no note, 1-96 for C-0..B-7, 97 for note off)
///
/// # Examples
/// ```
/// use speccade_backend_music::note::note_name_to_xm;
///
/// assert_eq!(note_name_to_xm("C4"), 49);
/// assert_eq!(note_name_to_xm("A#3"), 47);
/// assert_eq!(note_name_to_xm("---"), 0);
/// assert_eq!(note_name_to_xm("OFF"), 97);
/// ```
pub fn note_name_to_xm(name: &str) -> u8 {
    let name = name.trim().to_uppercase();

    // Handle special cases
    if name.is_empty() || name == "---" || name == "..." {
        return xm::NOTE_NONE;
    }
    if name == "===" || name == "OFF" {
        return xm::NOTE_OFF;
    }

    // Remove dashes for parsing (e.g., "C-4" -> "C4")
    let name = name.replace('-', "");

    parse_note_name(&name)
        .map(|(semitone, octave)| {
            let note = octave * 12 + semitone + 1;
            note.clamp(xm::NOTE_MIN as i32, xm::NOTE_MAX as i32) as u8
        })
        .unwrap_or(xm::NOTE_NONE)
}

/// Convert a note name (e.g., "C4", "A#3", "Bb5") to an IT note value.
///
/// # Arguments
/// * `name` - Note name string (e.g., "C4", "A#3", "Bb5", "---", "^^^", "===")
///
/// # Returns
/// IT note value (0-119 for C-0..B-9, 254 for cut, 255 for off)
///
/// # Examples
/// ```
/// use speccade_backend_music::note::note_name_to_it;
///
/// assert_eq!(note_name_to_it("C4"), 48);
/// assert_eq!(note_name_to_it("A#5"), 70);
/// assert_eq!(note_name_to_it("^^^"), 255);
/// assert_eq!(note_name_to_it("==="), 254);
/// ```
pub fn note_name_to_it(name: &str) -> u8 {
    let name = name.trim().to_uppercase();

    // Handle special cases
    if name.is_empty() || name == "---" || name == "..." {
        return 0;
    }
    if name == "^^^" || name == "OFF" {
        return it::NOTE_OFF;
    }
    if name == "===" || name == "CUT" {
        return it::NOTE_CUT;
    }
    if name == "~~~" || name == "FADE" {
        return it::NOTE_FADE;
    }

    // Remove dashes for parsing (e.g., "C-4" -> "C4")
    let name = name.replace('-', "");

    parse_note_name(&name)
        .map(|(semitone, octave)| {
            let note = octave * 12 + semitone;
            note.clamp(it::NOTE_MIN as i32, it::NOTE_MAX as i32) as u8
        })
        .unwrap_or(0)
}

/// Parse a note name into semitone offset and octave.
fn parse_note_name(name: &str) -> Option<(i32, i32)> {
    let chars: Vec<char> = name.chars().collect();
    if chars.is_empty() {
        return None;
    }

    // Get note letter
    let note_char = chars[0];
    let semitone = SEMITONE_MAP
        .iter()
        .find(|(c, _)| *c == note_char)
        .map(|(_, s)| *s as i32)?;

    let mut idx = 1;

    // Check for sharp or flat
    // Sharp is '#', flat is 'b' or 'B' (uppercase B when input was uppercased)
    let semitone = if idx < chars.len() {
        match chars[idx] {
            '#' => {
                idx += 1;
                semitone + 1
            }
            'b' | 'B' if idx + 1 < chars.len() && chars[idx + 1].is_ascii_digit() => {
                // 'b' or 'B' followed by a digit means flat (e.g., "Bb3" -> "BB3")
                idx += 1;
                semitone - 1
            }
            _ => semitone,
        }
    } else {
        semitone
    };

    // Get octave
    let octave_str: String = chars[idx..].iter().collect();
    let octave: i32 = octave_str.parse().ok()?;

    Some((semitone, octave))
}

/// Convert an XM note value to IT note value.
///
/// # Arguments
/// * `xm_note` - XM note value (0=none, 1-96=C-0..B-7, 97=off)
///
/// # Returns
/// IT note value (0-119 for notes, 255 for off)
pub fn xm_note_to_it(xm_note: u8) -> u8 {
    match xm_note {
        0 => 0,                       // No note
        xm::NOTE_OFF => it::NOTE_OFF, // Note off
        1..=96 => xm_note - 1,        // XM is 1-based, IT is 0-based
        _ => 0,
    }
}

/// Convert an IT note value to XM note value.
///
/// # Arguments
/// * `it_note` - IT note value (0-119 for notes, 254=cut, 255=off)
///
/// # Returns
/// XM note value (0=none, 1-96=C-0..B-7, 97=off)
pub fn it_note_to_xm(it_note: u8) -> u8 {
    match it_note {
        0..=95 => it_note + 1,         // IT is 0-based, XM is 1-based
        it::NOTE_OFF => xm::NOTE_OFF,  // Note off
        it::NOTE_CUT => xm::NOTE_OFF,  // Treat cut as off in XM
        it::NOTE_FADE => xm::NOTE_OFF, // Treat fade as off in XM
        _ => xm::NOTE_NONE,
    }
}

/// Generate a note name from an XM note value.
///
/// # Arguments
/// * `note` - XM note value
///
/// # Returns
/// Note name string (e.g., "C-4", "OFF", "---")
pub fn xm_note_to_name(note: u8) -> String {
    match note {
        xm::NOTE_NONE => "---".to_string(),
        xm::NOTE_OFF => "OFF".to_string(),
        1..=96 => {
            let n = note - 1;
            let octave = n / 12;
            let semitone = n % 12;
            let note_names = [
                "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
            ];
            format!("{}{}", note_names[semitone as usize], octave)
        }
        _ => "---".to_string(),
    }
}

/// Generate a note name from an IT note value.
///
/// # Arguments
/// * `note` - IT note value
///
/// # Returns
/// Note name string (e.g., "C-4", "^^^", "===", "---")
pub fn it_note_to_name(note: u8) -> String {
    match note {
        it::NOTE_OFF => "^^^".to_string(),
        it::NOTE_CUT => "===".to_string(),
        it::NOTE_FADE => "~~~".to_string(),
        0..=119 => {
            let octave = note / 12;
            let semitone = note % 12;
            let note_names = [
                "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
            ];
            format!("{}{}", note_names[semitone as usize], octave)
        }
        _ => "---".to_string(),
    }
}

/// Convert a note name string to MIDI note number.
///
/// This converts tracker-style note names (where C-4 = middle C) to MIDI
/// note numbers (where C4 = middle C = MIDI 60).
///
/// **Conversion:** MIDI note = IT note + 12
/// - Tracker "C4" (IT note 48) → MIDI 60 (261.6 Hz)
/// - Tracker "C5" (IT note 60) → MIDI 72 (523.25 Hz)
///
/// # Arguments
/// * `name` - Note name (e.g., "C4", "A#3", "Bb5")
///
/// # Returns
/// MIDI note number (0-127)
///
/// # Examples
/// ```
/// use speccade_backend_music::note::note_name_to_midi;
///
/// assert_eq!(note_name_to_midi("C4"), 60);  // Middle C
/// assert_eq!(note_name_to_midi("C5"), 72);  // One octave above
/// assert_eq!(note_name_to_midi("A4"), 69);  // A440
/// ```
pub fn note_name_to_midi(name: &str) -> u8 {
    let it_note = note_name_to_it(name);

    // Handle special cases (note off, cut, etc.)
    if it_note >= 254 {
        return 60; // Default to middle C for invalid notes
    }

    // Convert IT note to MIDI note: MIDI = IT + 12
    // IT note 48 (C-4) = MIDI 60 (C4) = 261.6 Hz
    (it_note as i32 + 12).clamp(0, 127) as u8
}
