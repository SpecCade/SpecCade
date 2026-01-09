//! Note and frequency conversion utilities for tracker modules.
//!
//! This module provides deterministic conversion between note names, MIDI numbers,
//! and frequencies for XM and IT tracker formats.


/// XM reference frequency (C-4 in XM format).
pub const XM_BASE_FREQ: f64 = 8363.0;

/// IT reference frequency (C-5 in IT format, plays at sample rate).
pub const IT_BASE_FREQ: f64 = 8363.0;

/// Default sample rate for generated samples.
pub const DEFAULT_SAMPLE_RATE: u32 = 22050;

/// Note values for XM format.
pub mod xm {
    /// No note present.
    pub const NOTE_NONE: u8 = 0;
    /// Note off command.
    pub const NOTE_OFF: u8 = 97;
    /// Minimum valid note (C-0).
    pub const NOTE_MIN: u8 = 1;
    /// Maximum valid note (B-7).
    pub const NOTE_MAX: u8 = 96;
}

/// Note values for IT format.
pub mod it {
    /// Minimum valid note (C-0).
    pub const NOTE_MIN: u8 = 0;
    /// Maximum valid note (B-9).
    pub const NOTE_MAX: u8 = 119;
    /// Note fade command.
    pub const NOTE_FADE: u8 = 253;
    /// Note cut command.
    pub const NOTE_CUT: u8 = 254;
    /// Note off command.
    pub const NOTE_OFF: u8 = 255;
}

/// Semitone offsets for note names (C=0, D=2, E=4, F=5, G=7, A=9, B=11).
const SEMITONE_MAP: [(char, i8); 7] = [
    ('C', 0),
    ('D', 2),
    ('E', 4),
    ('F', 5),
    ('G', 7),
    ('A', 9),
    ('B', 11),
];

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

/// Convert a MIDI note number to frequency in Hz.
///
/// Uses the standard formula: f = 440 * 2^((n-69)/12)
/// where n is the MIDI note number and 69 is A4.
///
/// # Arguments
/// * `midi_note` - MIDI note number (0-127)
///
/// # Returns
/// Frequency in Hz
///
/// # Examples
/// ```
/// use speccade_backend_music::note::midi_to_freq;
///
/// let a4 = midi_to_freq(69);
/// assert!((a4 - 440.0).abs() < 0.001);
///
/// let c4 = midi_to_freq(60);
/// assert!((c4 - 261.626).abs() < 0.01);
/// ```
pub fn midi_to_freq(midi_note: u8) -> f64 {
    440.0 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0)
}

/// Convert a frequency in Hz to the nearest MIDI note number.
///
/// # Arguments
/// * `freq` - Frequency in Hz
///
/// # Returns
/// MIDI note number (0-127)
///
/// # Examples
/// ```
/// use speccade_backend_music::note::freq_to_midi;
///
/// assert_eq!(freq_to_midi(440.0), 69); // A4
/// assert_eq!(freq_to_midi(261.626), 60); // C4
/// ```
pub fn freq_to_midi(freq: f64) -> u8 {
    let note = 69.0 + 12.0 * (freq / 440.0).log2();
    note.round().clamp(0.0, 127.0) as u8
}

/// Calculate pitch correction (finetune and relative note) for a given sample rate.
///
/// XM/IT expect samples tuned for 8363 Hz at the reference pitch. This function
/// calculates the pitch correction needed to play a sample at the correct pitch.
///
/// # Arguments
/// * `sample_rate` - The sample rate of the audio (e.g., 22050)
///
/// # Returns
/// Tuple of (finetune, relative_note) where finetune is -128 to 127 and
/// relative_note is the semitone offset from the reference pitch.
///
/// # Examples
/// ```
/// use speccade_backend_music::note::calculate_pitch_correction;
///
/// let (finetune, relative_note) = calculate_pitch_correction(22050);
/// assert_eq!(relative_note, 16);
/// ```
pub fn calculate_pitch_correction(sample_rate: u32) -> (i8, i8) {
    let semitones = 12.0 * (sample_rate as f64 / XM_BASE_FREQ).log2();
    let relative_note = semitones.floor() as i32;
    let finetune = ((semitones - relative_note as f64) * 128.0).round() as i32;

    // Handle overflow
    let (finetune, relative_note) = if finetune >= 128 {
        (finetune - 128, relative_note + 1)
    } else {
        (finetune, relative_note)
    };

    (
        (finetune as i8).clamp(-128, 127),
        (relative_note as i8).clamp(-128, 127),
    )
}

/// Calculate the C-5 speed for IT samples.
///
/// IT uses "C-5 speed" which is the sample rate at which C-5 plays at its
/// natural frequency. For most samples, this equals the sample rate.
///
/// # Arguments
/// * `sample_rate` - The sample rate of the audio
///
/// # Returns
/// C-5 speed value for IT sample header
pub fn calculate_c5_speed(sample_rate: u32) -> u32 {
    sample_rate
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
        0..=95 => it_note + 1,       // IT is 0-based, XM is 1-based
        it::NOTE_OFF => xm::NOTE_OFF, // Note off
        it::NOTE_CUT => xm::NOTE_OFF, // Treat cut as off in XM
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
            let note_names = ["C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-"];
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
            let note_names = ["C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-"];
            format!("{}{}", note_names[semitone as usize], octave)
        }
        _ => "---".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_name_to_xm() {
        assert_eq!(note_name_to_xm("C4"), 49);
        assert_eq!(note_name_to_xm("C-4"), 49);
        assert_eq!(note_name_to_xm("A#3"), 47);
        assert_eq!(note_name_to_xm("Bb3"), 47); // Bb = A#
        assert_eq!(note_name_to_xm("C0"), 1);
        assert_eq!(note_name_to_xm("B7"), 96);
        assert_eq!(note_name_to_xm("---"), 0);
        assert_eq!(note_name_to_xm("OFF"), 97);
    }

    #[test]
    fn test_note_name_to_it() {
        assert_eq!(note_name_to_it("C4"), 48);
        assert_eq!(note_name_to_it("C-4"), 48);
        assert_eq!(note_name_to_it("A#5"), 70);
        assert_eq!(note_name_to_it("C0"), 0);
        assert_eq!(note_name_to_it("B9"), 119);
        assert_eq!(note_name_to_it("^^^"), 255);
        assert_eq!(note_name_to_it("==="), 254);
    }

    #[test]
    fn test_midi_to_freq() {
        assert!((midi_to_freq(69) - 440.0).abs() < 0.001);
        assert!((midi_to_freq(60) - 261.626).abs() < 0.01);
        assert!((midi_to_freq(57) - 220.0).abs() < 0.001);
    }

    #[test]
    fn test_freq_to_midi() {
        assert_eq!(freq_to_midi(440.0), 69);
        assert_eq!(freq_to_midi(261.626), 60);
        assert_eq!(freq_to_midi(220.0), 57);
    }

    #[test]
    fn test_pitch_correction() {
        let (_finetune, relative_note) = calculate_pitch_correction(22050);
        // 22050 / 8363 = ~2.637, log2(2.637) * 12 = ~16.66 semitones
        assert_eq!(relative_note, 16);

        let (finetune, relative_note) = calculate_pitch_correction(8363);
        assert_eq!(relative_note, 0);
        assert_eq!(finetune, 0);
    }

    #[test]
    fn test_note_name_roundtrip() {
        for note in 1..=96u8 {
            let name = xm_note_to_name(note);
            let parsed = note_name_to_xm(&name);
            assert_eq!(note, parsed, "Roundtrip failed for note {}: {}", note, name);
        }
    }
}
