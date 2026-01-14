//! Tests for note conversion and pitch calculation functions.

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
    assert_eq!(
        c5_speed, 22050,
        "IT default (MIDI 72) should produce c5_speed = sample_rate"
    );

    // Also verify that DEFAULT_SYNTH_MIDI_NOTE (60) requires 2x adjustment
    let c5_speed_xm_default = calculate_c5_speed_for_base_note(22050, DEFAULT_SYNTH_MIDI_NOTE);
    assert_eq!(
        c5_speed_xm_default, 44100,
        "XM default (MIDI 60) should produce c5_speed = 2x sample_rate"
    );
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
        (100..=102).contains(&finetune),
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
        (100..=102).contains(&finetune),
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
        (30..=34).contains(&finetune),
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
        (99..=103).contains(&finetune),
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

#[test]
fn test_xm_pitch_correction_produces_expected_playback_rate_for_base_note() {
    // Verify that (finetune, relative_note) make the base note play back at the
    // sample's natural sample_rate. If the math is reversed, this will be far off.
    let cases = [
        (22050u32, 33u8), // A1 (low note)
        (22050u32, 60u8), // C4 (middle C)
        (22050u32, 72u8), // C5
        (44100u32, 33u8), // A1 at higher sample rate
        (44100u32, 60u8), // C4 at higher sample rate
    ];

    for (sample_rate, base_midi) in cases {
        let (finetune, relative_note) = calculate_xm_pitch_correction(sample_rate, base_midi);

        // XM 0-indexed note number for this MIDI base note.
        let base_xm_note_0indexed = base_midi as i32 - 12;

        // XM reference: C-4 (0-indexed note 48) at XM_BASE_FREQ.
        let semitones = (base_xm_note_0indexed as f64 + relative_note as f64 - 48.0)
            + (finetune as f64 / 128.0);
        let playback_rate = XM_BASE_FREQ * 2.0_f64.powf(semitones / 12.0);

        let rel_err = (playback_rate - sample_rate as f64).abs() / sample_rate as f64;
        assert!(
            rel_err < 0.01,
            "XM pitch correction mismatch: sample_rate={}, base_midi={}, finetune={}, relative_note={}, playback_rate={:.2} (rel_err={:.5})",
            sample_rate,
            base_midi,
            finetune,
            relative_note,
            playback_rate,
            rel_err
        );
    }
}

#[test]
fn test_it_c5_speed_produces_expected_playback_rate_for_base_note() {
    // Verify that c5_speed makes the base note play back at the sample's natural
    // sample_rate. This checks the direction/sign of the semitone adjustment.
    let cases = [
        (22050u32, 33u8), // A1
        (22050u32, 72u8), // C5 (IT reference base)
        (44100u32, 60u8), // C4
    ];

    for (sample_rate, base_midi) in cases {
        let c5_speed = calculate_c5_speed_for_base_note(sample_rate, base_midi);

        // IT note number: it_note = midi - 12. Reference is C-5 (note 60).
        let base_it_note = base_midi as i32 - 12;
        let playback_rate = c5_speed as f64 * 2.0_f64.powf((base_it_note - 60) as f64 / 12.0);

        let rel_err = (playback_rate - sample_rate as f64).abs() / sample_rate as f64;
        assert!(
            rel_err < 0.01,
            "IT c5_speed mismatch: sample_rate={}, base_midi={}, c5_speed={}, playback_rate={:.2} (rel_err={:.5})",
            sample_rate,
            base_midi,
            c5_speed,
            playback_rate,
            rel_err
        );
    }
}
