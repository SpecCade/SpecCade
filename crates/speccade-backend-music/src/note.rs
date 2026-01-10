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

/// Calculate the C-5 speed for IT samples given a base MIDI note.
///
/// IT's reference pitch is C-5 (IT note 60, ~523.25 Hz). If a sample is
/// recorded/generated at a different pitch, the C-5 speed must be adjusted
/// so that playing C-5 produces the correct frequency.
///
/// **Octave Convention Note:**
/// - IT note numbers are offset from MIDI by 12: IT note = MIDI note - 12
/// - IT "C-4" (note 48) = MIDI "C4" (note 60) = 261.6 Hz (middle C)
/// - IT "C-5" (note 60) = MIDI "C5" (note 72) = 523.25 Hz
///
/// # Arguments
/// * `sample_rate` - The sample rate of the audio
/// * `base_midi_note` - The MIDI note number the sample was recorded/generated at
///
/// # Returns
/// C-5 speed value adjusted for the base note
///
/// # Examples
/// ```
/// use speccade_backend_music::note::calculate_c5_speed_for_base_note;
///
/// // Sample at MIDI 72 (C5 standard = C-5 in IT) = 523.25 Hz, the IT reference
/// let c5_speed = calculate_c5_speed_for_base_note(22050, 72);
/// assert_eq!(c5_speed, 22050); // No adjustment needed - at reference pitch
///
/// // Sample at MIDI 60 (C4 standard = C-4 in IT) = 261.6 Hz, one octave below reference
/// let c5_speed = calculate_c5_speed_for_base_note(22050, 60);
/// assert_eq!(c5_speed, 44100); // Need 2x speed to reach C-5 pitch
/// ```
pub fn calculate_c5_speed_for_base_note(sample_rate: u32, base_midi_note: u8) -> u32 {
    // Convert MIDI note to IT note number.
    // IT note 48 (C-4) = MIDI 60 (middle C) = 261.6 Hz
    // So: it_note = midi_note - 12
    //
    // IT's reference is C-5 (IT note 60 = MIDI 72 = 523.25 Hz)
    let base_it_note = base_midi_note as i32 - 12;
    let it_reference_note: i32 = 60; // C-5 in IT = MIDI 72 = 523.25 Hz

    let semitone_diff = it_reference_note - base_it_note;

    // Adjust sample rate: if base note is below IT C-5, we need higher c5_speed
    // so that C-5 playback produces the correct pitch.
    //
    // Example: MIDI 60 (C4) → IT note 48 (C-4)
    //   semitone_diff = 60 - 48 = 12
    //   c5_speed = 22050 * 2^(12/12) = 44100
    //   This means C-5 plays at 44100 Hz, doubling our 261.6 Hz sample to 523.25 Hz ✓
    (sample_rate as f64 * 2.0_f64.powf(semitone_diff as f64 / 12.0)) as u32
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

/// Calculate XM pitch correction for a sample at a given base MIDI note.
///
/// XM uses `relative_note` and `finetune` to adjust pitch. This function
/// calculates the correct values for a sample generated at a specific MIDI note.
///
/// # Arguments
/// * `sample_rate` - The sample rate of the audio
/// * `base_midi_note` - The MIDI note the sample was generated at (default: 60 = C4)
///
/// # Returns
/// Tuple of (finetune, relative_note) for XM sample header
///
/// # Examples
/// ```
/// use speccade_backend_music::note::calculate_xm_pitch_correction;
///
/// // Sample at MIDI 60 (middle C), 22050 Hz sample rate
/// let (finetune, relative_note) = calculate_xm_pitch_correction(22050, 60);
/// // relative_note compensates for 22050 Hz vs 8363 Hz reference
/// assert_eq!(relative_note, 16);
/// ```
pub fn calculate_xm_pitch_correction(sample_rate: u32, base_midi_note: u8) -> (i8, i8) {
    // Convert MIDI note to XM note number (0-indexed for calculations).
    // XM note 49 (C-4, 1-indexed) = XM note 48 (0-indexed) = MIDI 60 = 261.6 Hz
    // So: xm_note_0indexed = midi_note - 12
    //
    // XM reference for pitch calculation is note 48 (C-4, 0-indexed) at 8363 Hz.
    // This means a sample at 8363 Hz containing C-4 (261.6 Hz) plays correctly.
    //
    // The relative_note formula (from existing sample-based code):
    //   relative_note = 48 + rate_correction - base_note_0indexed
    //
    // This ensures that when playing the base note, the sample plays at correct pitch.

    // Get the rate correction (semitones to compensate for sample rate vs 8363 Hz)
    let (finetune, rate_correction) = calculate_pitch_correction(sample_rate);

    // Convert MIDI note to XM 0-indexed note
    let base_xm_note_0indexed = base_midi_note as i8 - 12;

    // Calculate relative_note
    // Example: MIDI 60 (XM note 48, 0-indexed) at 22050 Hz:
    //   rate_correction = 16
    //   relative_note = 48 + 16 - 48 = 16
    //   When playing C-4 (XM note 49), sample plays at correct 22050 Hz rate ✓
    let relative_note = 48 + rate_correction - base_xm_note_0indexed;

    (finetune, relative_note)
}

/// Default MIDI note for synthesized samples (middle C = C4 = 261.6 Hz)
/// This is used by XM format, which uses C-4 as its reference pitch.
pub const DEFAULT_SYNTH_MIDI_NOTE: u8 = 60;

/// Default MIDI note for IT format synthesized samples (C5 = 523.25 Hz)
/// IT uses C-5 (IT note 60 = MIDI 72) as its reference pitch.
/// When no base_note is specified, IT samples are assumed to contain C-5 audio.
pub const DEFAULT_IT_SYNTH_MIDI_NOTE: u8 = 72;

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

    // Tests for IT format pitch calculation (c5_speed)
    // Context: MIDI and IT note numbers are offset by 12:
    // - MIDI 60 (C4, middle C, 261.6 Hz) = IT note 48 (C-4)
    // - MIDI 72 (C5, 523.25 Hz) = IT note 60 (C-5, the IT reference)

    #[test]
    fn test_calculate_c5_speed_midi_60_middle_c() {
        // MIDI 60 = C4 (middle C) = IT C-4 (note 48)
        // IT reference is C-5 (note 60), which is one octave (12 semitones) higher
        // So we need 2x speed to reach C-5 pitch from C-4
        let c5_speed = calculate_c5_speed_for_base_note(22050, 60);
        assert_eq!(c5_speed, 44100);
    }

    #[test]
    fn test_calculate_c5_speed_midi_72_c5() {
        // MIDI 72 = C5 = IT C-5 (note 60), the IT reference pitch
        // No adjustment needed since we're already at the reference
        let c5_speed = calculate_c5_speed_for_base_note(22050, 72);
        assert_eq!(c5_speed, 22050);
    }

    #[test]
    fn test_calculate_c5_speed_with_it_default_constant() {
        // Verify that using DEFAULT_IT_SYNTH_MIDI_NOTE (72) results in c5_speed = sample_rate
        // This confirms that when no base_note is specified for IT, c5_speed = sample_rate
        let c5_speed = calculate_c5_speed_for_base_note(22050, DEFAULT_IT_SYNTH_MIDI_NOTE);
        assert_eq!(c5_speed, 22050, "IT default (MIDI 72) should produce c5_speed = sample_rate");

        // Also verify that DEFAULT_SYNTH_MIDI_NOTE (60) requires 2x adjustment
        let c5_speed_xm_default = calculate_c5_speed_for_base_note(22050, DEFAULT_SYNTH_MIDI_NOTE);
        assert_eq!(c5_speed_xm_default, 44100, "XM default (MIDI 60) should produce c5_speed = 2x sample_rate");
    }

    #[test]
    fn test_calculate_c5_speed_midi_48_c3() {
        // MIDI 48 = C3 = IT C-3 (note 36)
        // IT reference C-5 is 24 semitones (2 octaves) higher
        // So we need 4x speed to reach C-5 pitch from C-3
        let c5_speed = calculate_c5_speed_for_base_note(22050, 48);
        assert_eq!(c5_speed, 88200);
    }

    #[test]
    fn test_note_name_to_midi_middle_c() {
        // C4 is middle C = MIDI 60
        assert_eq!(note_name_to_midi("C4"), 60);
    }

    #[test]
    fn test_note_name_to_midi_c5() {
        // C5 is one octave above middle C = MIDI 72
        assert_eq!(note_name_to_midi("C5"), 72);
    }

    #[test]
    fn test_note_name_to_midi_a4() {
        // A4 is the A440 reference = MIDI 69
        assert_eq!(note_name_to_midi("A4"), 69);
    }

    #[test]
    fn test_note_name_to_midi_with_accidentals() {
        // Test sharp and flat notes
        assert_eq!(note_name_to_midi("C#4"), 61);
        assert_eq!(note_name_to_midi("Bb4"), 70); // Bb4 = A#4 = MIDI 70
        assert_eq!(note_name_to_midi("F#5"), 78);
    }

    #[test]
    fn test_note_name_to_midi_with_dash() {
        // Tracker format with dashes should work too
        assert_eq!(note_name_to_midi("C-4"), 60);
        assert_eq!(note_name_to_midi("A-4"), 69);
    }

    // =========================================================================
    // Tests for XM format pitch calculation
    // =========================================================================
    // Context: MIDI and XM note numbers are offset by 12:
    // - MIDI 60 (C4, middle C, 261.6 Hz) = XM note 49 (C-4, 1-indexed) = XM note 48 (0-indexed)
    // - XM reference is note 48 (0-indexed) at 8363 Hz sample rate

    #[test]
    fn test_calculate_xm_pitch_correction_midi_60_at_22050hz() {
        // Sample at MIDI 60 (C4, middle C = 261.6 Hz), 22050 Hz sample rate
        // XM note 48 (0-indexed) = MIDI 60
        // rate_correction = 16 (from 22050/8363 ratio: log2(2.637)*12 = 16.79)
        // relative_note = 48 + 16 - 48 = 16
        let (finetune, relative_note) = calculate_xm_pitch_correction(22050, 60);
        assert_eq!(relative_note, 16);
        // Finetune should be around 101 (0.79 * 128 rounded)
        // 22050/8363 = 2.637, log2(2.637)*12 = 16.79 semitones
        // fractional part: 0.79 * 128 = 101.12
        assert!(
            finetune >= 100 && finetune <= 102,
            "finetune should be ~101, got {}",
            finetune
        );
    }

    #[test]
    fn test_calculate_xm_pitch_correction_midi_72_at_22050hz() {
        // Sample at MIDI 72 (C5 = 523.25 Hz), 22050 Hz sample rate
        // XM note 60 (0-indexed) = MIDI 72
        // rate_correction = 16 (from 22050/8363 ratio)
        // relative_note = 48 + 16 - 60 = 4
        let (finetune, relative_note) = calculate_xm_pitch_correction(22050, 72);
        assert_eq!(relative_note, 4);
        // Same finetune as before since sample rate is the same
        assert!(
            finetune >= 100 && finetune <= 102,
            "finetune should be ~101, got {}",
            finetune
        );
    }

    #[test]
    fn test_calculate_xm_pitch_correction_at_reference_rate_midi_60() {
        // Sample at 8363 Hz (XM reference rate), MIDI 60
        // At reference rate, rate_correction = 0, finetune = 0
        // base_xm_note_0indexed = 60 - 12 = 48
        // relative_note = 48 + 0 - 48 = 0
        let (finetune, relative_note) = calculate_xm_pitch_correction(8363, 60);
        assert_eq!(finetune, 0);
        assert_eq!(relative_note, 0);
    }

    #[test]
    fn test_calculate_xm_pitch_correction_at_reference_rate_midi_72() {
        // Sample at 8363 Hz (XM reference rate), MIDI 72 (C5)
        // rate_correction = 0
        // base_xm_note_0indexed = 72 - 12 = 60
        // relative_note = 48 + 0 - 60 = -12
        // This means playing C-5 (XM note 61, 1-indexed) will shift down 12 semitones
        // to match the C5 content in the sample
        let (finetune, relative_note) = calculate_xm_pitch_correction(8363, 72);
        assert_eq!(finetune, 0);
        assert_eq!(relative_note, -12);
    }

    #[test]
    fn test_midi_xm_note_relationship() {
        // Verify the MIDI to XM note offset is 12
        // MIDI 60 (C4) = XM note 49 (C-4, 1-indexed) = XM note 48 (0-indexed)
        let xm_note = note_name_to_xm("C4");
        assert_eq!(xm_note, 49); // 1-indexed XM note

        let midi_note = note_name_to_midi("C4");
        assert_eq!(midi_note, 60);

        // The relationship: xm_note_0indexed = midi_note - 12
        assert_eq!((xm_note - 1) as i32, midi_note as i32 - 12);
    }

    #[test]
    fn test_xm_pitch_correction_consistency_with_base_pitch_correction() {
        // Verify that calculate_xm_pitch_correction for MIDI 60 at reference rate
        // is equivalent to calculate_pitch_correction but adjusted for base note
        let (rate_ft, rate_rn) = calculate_pitch_correction(22050);
        let (xm_ft, xm_rn) = calculate_xm_pitch_correction(22050, 60);

        // For MIDI 60 (XM note 48, 0-indexed), the formula is:
        // relative_note = 48 + rate_correction - 48 = rate_correction
        // So xm_rn should equal rate_rn
        assert_eq!(xm_rn, rate_rn);
        assert_eq!(xm_ft, rate_ft);
    }

    #[test]
    fn test_xm_pitch_correction_higher_base_note_less_adjustment() {
        // A higher base note should result in a lower relative_note
        // because we're closer to or above the reference pitch
        let (_ft1, rn1) = calculate_xm_pitch_correction(22050, 60); // C4
        let (_ft2, rn2) = calculate_xm_pitch_correction(22050, 72); // C5

        // C5 is 12 semitones higher than C4, so relative_note should be 12 lower
        assert_eq!(rn1 - rn2, 12);
    }

    // =========================================================================
    // Tests for IT format with different sample rates and base notes
    // =========================================================================

    #[test]
    fn test_calculate_c5_speed_48khz_midi_36_c2() {
        // Sample at 48000 Hz, base note C2 (MIDI 36)
        // MIDI 36 = C2 = IT note 24 (C-2 in IT notation)
        // IT reference is C-5 (IT note 60)
        // semitone_diff = 60 - 24 = 36 (3 octaves up to C-5)
        // c5_speed = 48000 * 2^(36/12) = 48000 * 2^3 = 48000 * 8 = 384000
        let c5_speed = calculate_c5_speed_for_base_note(48000, 36);
        assert_eq!(c5_speed, 384000);
    }

    #[test]
    fn test_calculate_c5_speed_44100hz_midi_60() {
        // Sample at 44100 Hz (CD quality), base note C4 (MIDI 60)
        // MIDI 60 = C4 = IT note 48 (C-4 in IT notation)
        // IT reference is C-5 (IT note 60)
        // semitone_diff = 60 - 48 = 12 (1 octave up to C-5)
        // c5_speed = 44100 * 2^(12/12) = 44100 * 2 = 88200
        let c5_speed = calculate_c5_speed_for_base_note(44100, 60);
        assert_eq!(c5_speed, 88200);
    }

    #[test]
    fn test_calculate_c5_speed_preserves_sample_rate_ratio() {
        // Verify that doubling sample rate doubles c5_speed for the same base note
        let c5_speed_22050 = calculate_c5_speed_for_base_note(22050, 60);
        let c5_speed_44100 = calculate_c5_speed_for_base_note(44100, 60);

        // 44100 / 22050 = 2, so c5_speed should also double
        assert_eq!(c5_speed_44100, c5_speed_22050 * 2);

        // Test another pair: 24000 Hz vs 48000 Hz
        let c5_speed_24000 = calculate_c5_speed_for_base_note(24000, 48);
        let c5_speed_48000 = calculate_c5_speed_for_base_note(48000, 48);
        assert_eq!(c5_speed_48000, c5_speed_24000 * 2);
    }

    // =========================================================================
    // Tests for XM format with different sample rates and base notes
    // =========================================================================

    #[test]
    fn test_calculate_xm_pitch_correction_48khz_midi_36() {
        // Sample at 48000 Hz, base note C2 (MIDI 36)
        //
        // Step 1: Rate correction (from 48000 Hz vs 8363 Hz reference)
        //   rate_ratio = 48000 / 8363 = 5.739
        //   rate_semitones = 12 * log2(5.739) = 12 * 2.521 = 30.25
        //   rate_correction = 30 (floor)
        //   finetune = (0.25 * 128) = 32
        //
        // Step 2: Base note adjustment
        //   base_xm_note_0indexed = 36 - 12 = 24 (C-2 in XM)
        //   relative_note = 48 + 30 - 24 = 54
        let (finetune, relative_note) = calculate_xm_pitch_correction(48000, 36);
        assert_eq!(relative_note, 54);
        // finetune should be around 32 (0.25 * 128)
        assert!(
            finetune >= 30 && finetune <= 34,
            "finetune should be ~32, got {}",
            finetune
        );
    }

    #[test]
    fn test_calculate_xm_pitch_correction_44100hz_midi_60() {
        // Sample at 44100 Hz (CD quality), base note C4 (MIDI 60)
        //
        // Step 1: Rate correction (from 44100 Hz vs 8363 Hz reference)
        //   rate_ratio = 44100 / 8363 = 5.274
        //   rate_semitones = 12 * log2(5.274) = 12 * 2.399 = 28.79
        //   rate_correction = 28 (floor)
        //   finetune = (0.79 * 128) = 101
        //
        // Step 2: Base note adjustment
        //   base_xm_note_0indexed = 60 - 12 = 48 (C-4 in XM)
        //   relative_note = 48 + 28 - 48 = 28
        let (finetune, relative_note) = calculate_xm_pitch_correction(44100, 60);
        assert_eq!(relative_note, 28);
        // finetune should be around 101
        assert!(
            finetune >= 99 && finetune <= 103,
            "finetune should be ~101, got {}",
            finetune
        );
    }

    #[test]
    fn test_calculate_xm_pitch_different_rates_same_base() {
        // Verify relative_note scales with sample rate for the same base note
        // Doubling sample rate adds 12 semitones (one octave) to rate_correction

        // 22050 Hz at MIDI 60
        let (_, rn_22050) = calculate_xm_pitch_correction(22050, 60);
        // 44100 Hz at MIDI 60 (exactly 2x sample rate)
        let (_, rn_44100) = calculate_xm_pitch_correction(44100, 60);

        // Doubling sample rate: log2(2) * 12 = 12 semitones difference
        assert_eq!(rn_44100 - rn_22050, 12);

        // Test another pair: 24000 Hz vs 48000 Hz at MIDI 48
        let (_, rn_24000) = calculate_xm_pitch_correction(24000, 48);
        let (_, rn_48000) = calculate_xm_pitch_correction(48000, 48);
        assert_eq!(rn_48000 - rn_24000, 12);
    }

    #[test]
    fn test_calculate_xm_pitch_same_rate_different_base() {
        // Verify relative_note changes by 12 for each octave of base note change
        // (Base note is subtracted, so higher base = lower relative_note)

        // 48000 Hz at MIDI 36 (C2)
        let (_, rn_c2) = calculate_xm_pitch_correction(48000, 36);
        // 48000 Hz at MIDI 48 (C3)
        let (_, rn_c3) = calculate_xm_pitch_correction(48000, 48);
        // 48000 Hz at MIDI 60 (C4)
        let (_, rn_c4) = calculate_xm_pitch_correction(48000, 60);

        // Each octave higher base note reduces relative_note by 12
        assert_eq!(rn_c2 - rn_c3, 12);
        assert_eq!(rn_c3 - rn_c4, 12);
    }

    #[test]
    fn test_xm_and_it_consistency_for_same_sample() {
        // For the same sample (sample_rate, base_note), XM and IT should
        // produce equivalent pitch mappings even though they use different
        // mechanisms (relative_note vs c5_speed)

        // Test case: 48000 Hz sample at MIDI 36 (C2)
        let sample_rate = 48000u32;
        let base_midi = 36u8;

        let c5_speed = calculate_c5_speed_for_base_note(sample_rate, base_midi);
        let (_finetune, relative_note) = calculate_xm_pitch_correction(sample_rate, base_midi);

        // IT: c5_speed = 384000
        // This means playing IT C-5 (note 60) at 384000 Hz produces the original pitch
        // Since the sample is at 48000 Hz, this is 8x speed, raising pitch by 3 octaves
        // So IT C-5 plays what should be C5 (MIDI 72), but the sample contains C2 (MIDI 36)
        // This is correct: C-5 in IT should sound like the note 3 octaves up from C2
        assert_eq!(c5_speed, 384000);

        // XM: relative_note = 54, finetune ~= 32
        // When playing XM C-4 (note 49, 1-indexed = note 48, 0-indexed),
        // the effective note is 48 + 54 = 102 (very high adjustment)
        // This compensates for both the high sample rate (30 semitones) and low base note (24 semitones down from reference)
        assert_eq!(relative_note, 54);

        // Both formats should produce the same pitch when playing their reference notes
        // IT: C-5 at c5_speed=384000 = base pitch
        // XM: Note that maps to effective pitch 48 = base pitch
        // The relationship holds because both account for sample_rate and base_note
    }
}
