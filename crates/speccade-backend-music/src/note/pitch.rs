//! Pitch correction calculations for XM and IT tracker formats.

use super::constants::XM_BASE_FREQ;

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

/// Compute pitch deviation in cents for XM pitch correction parameters.
///
/// Simulates the XM playback engine's frequency calculation and compares
/// it to the sample's native rate. Returns deviation in cents (positive = sharp).
pub fn xm_pitch_deviation_cents(
    sample_rate: u32,
    base_midi_note: u8,
    finetune: i8,
    relative_note: i8,
) -> f64 {
    let base_xm_note_0indexed = base_midi_note as f64 - 12.0;
    let semitones = (base_xm_note_0indexed + relative_note as f64 - 48.0)
        + (finetune as f64 / 128.0);
    let playback_rate = XM_BASE_FREQ * 2.0_f64.powf(semitones / 12.0);
    1200.0 * (playback_rate / sample_rate as f64).log2()
}

/// Compute pitch deviation in cents for IT c5_speed parameter.
///
/// Simulates the IT playback engine's frequency calculation and compares
/// it to the sample's native rate. Returns deviation in cents (positive = sharp).
pub fn it_pitch_deviation_cents(sample_rate: u32, base_midi_note: u8, c5_speed: u32) -> f64 {
    let base_it_note = base_midi_note as f64 - 12.0;
    let playback_rate = c5_speed as f64 * 2.0_f64.powf((base_it_note - 60.0) / 12.0);
    1200.0 * (playback_rate / sample_rate as f64).log2()
}
