//! MIDI note number and frequency conversion utilities.

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
