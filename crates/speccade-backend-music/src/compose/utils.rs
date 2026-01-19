//! Utility functions for compose expansion.

use rand::SeedableRng;
use rand_pcg::Pcg32;

use speccade_spec::recipe::audio::parse_note_name;

use super::error::ExpandError;

/// Convert a MIDI note number to a note name (e.g., 60 -> "C4").
pub fn midi_to_note_name(midi: u8) -> String {
    const NOTES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (midi / 12) as i32 - 1;
    let note_idx = (midi % 12) as usize;
    format!("{}{}", NOTES[note_idx], octave)
}

/// Transpose a note name by a given number of semitones.
pub fn transpose_note(
    note: &str,
    semitones: i32,
    pattern_name: &str,
) -> Result<Option<String>, ExpandError> {
    let trimmed = note.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let upper = trimmed.to_uppercase();
    if matches!(
        upper.as_str(),
        "---" | "..." | "OFF" | "===" | "^^^" | "CUT" | "FADE" | "~~~"
    ) {
        return Ok(None);
    }
    let midi = parse_note_name(trimmed)
        .or_else(|| parse_note_name(&trimmed.replace('-', "")))
        .map(|v| v as i32);
    let Some(midi) = midi else {
        return Ok(None);
    };
    let transposed = midi + semitones;
    if !(0..=127).contains(&transposed) {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: format!("transpose produced out-of-range MIDI note {}", transposed),
        });
    }
    Ok(Some(midi_to_note_name(transposed as u8)))
}

/// Generate a Euclidean rhythm using Bjorklund's algorithm.
pub fn bjorklund(steps: usize, pulses: usize) -> Vec<bool> {
    if pulses == 0 {
        return vec![false; steps];
    }
    if pulses >= steps {
        return vec![true; steps];
    }

    let mut pattern = Vec::new();
    let mut counts = Vec::new();
    let mut remainders = Vec::new();
    remainders.push(pulses);
    let mut divisor = steps - pulses;
    let mut level = 0usize;

    while remainders[level] > 1 {
        counts.push(divisor / remainders[level]);
        remainders.push(divisor % remainders[level]);
        divisor = remainders[level];
        level += 1;
    }
    counts.push(divisor);

    fn build(level: isize, counts: &[usize], remainders: &[usize], pattern: &mut Vec<bool>) {
        if level == -1 {
            pattern.push(false);
        } else if level == -2 {
            pattern.push(true);
        } else {
            for _ in 0..counts[level as usize] {
                build(level - 1, counts, remainders, pattern);
            }
            if remainders[level as usize] != 0 {
                build(level - 2, counts, remainders, pattern);
            }
        }
    }

    build(level as isize, &counts, &remainders, &mut pattern);
    pattern.truncate(steps);
    pattern
}

/// Euclidean modulo operation (always returns a positive result).
pub fn modulo(value: i32, modulus: i32) -> i32 {
    if modulus == 0 {
        return 0;
    }
    let mut v = value % modulus;
    if v < 0 {
        v += modulus;
    }
    v
}

/// Create a deterministic RNG from a seed, pattern name, and salt.
pub fn rng_for(seed: u32, pattern_name: &str, seed_salt: &str) -> Pcg32 {
    let mut input = Vec::with_capacity(8 + pattern_name.len() + seed_salt.len() + 2);
    input.extend_from_slice(&seed.to_le_bytes());
    input.push(0);
    input.extend_from_slice(pattern_name.as_bytes());
    input.push(0);
    input.extend_from_slice(seed_salt.as_bytes());

    let hash = blake3::hash(&input);
    let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
    let derived = u32::from_le_bytes(bytes);
    let seed64 = (derived as u64) | ((derived as u64) << 32);
    Pcg32::seed_from_u64(seed64)
}

/// Create a deterministic RNG from a seed, pattern name, salt, row, and channel.
///
/// This provides per-cell randomization that is stable across runs with the same inputs.
pub fn rng_for_cell(
    seed: u32,
    pattern_name: &str,
    seed_salt: &str,
    row: i32,
    channel: u8,
) -> Pcg32 {
    let mut input = Vec::with_capacity(8 + pattern_name.len() + seed_salt.len() + 4 + 1 + 3);
    input.extend_from_slice(&seed.to_le_bytes());
    input.push(0);
    input.extend_from_slice(pattern_name.as_bytes());
    input.push(0);
    input.extend_from_slice(seed_salt.as_bytes());
    input.push(0);
    input.extend_from_slice(&row.to_le_bytes());
    input.push(channel);

    let hash = blake3::hash(&input);
    let bytes: [u8; 4] = hash.as_bytes()[0..4].try_into().unwrap();
    let derived = u32::from_le_bytes(bytes);
    let seed64 = (derived as u64) | ((derived as u64) << 32);
    Pcg32::seed_from_u64(seed64)
}
