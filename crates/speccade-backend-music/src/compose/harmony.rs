//! Harmony, chord, and pitch parsing logic.

use speccade_spec::recipe::audio::parse_note_name;
use speccade_spec::recipe::music::{BeatPos, ChordSpec, Harmony, HarmonyScale};

use super::error::ExpandError;
use super::utils::midi_to_note_name;

/// Key context with root pitch class and scale intervals.
#[derive(Debug, Clone)]
pub(super) struct KeyContext {
    pub root_pc: u8,
    pub _scale: HarmonyScale,
    pub scale_intervals: [i32; 7],
}

/// Parsed chord with root, intervals, and optional bass note.
#[derive(Debug, Clone)]
pub(super) struct ParsedChord {
    pub root_pc: u8,
    pub intervals: Vec<u8>,
    pub _bass_pc: Option<u8>,
}

/// Chord positioned at a specific row in the pattern.
#[derive(Debug, Clone)]
pub(super) struct ChordAt {
    pub at_row: i64,
    pub idx: usize,
    pub chord: ParsedChord,
}

/// Parse a pitch class name (e.g., "C", "F#", "Bb") into a semitone value (0-11).
pub(super) fn parse_pitch_class(name: &str, pattern_name: &str) -> Result<u8, ExpandError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: "pitch class cannot be empty".to_string(),
        });
    }

    let bytes = trimmed.as_bytes();
    let note_letter = bytes[0] as char;
    let base_semitone: i32 = match note_letter.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => -1,
    };

    if base_semitone >= 0 {
        let mut idx = 1usize;
        let mut accidental: i32 = 0;
        if idx < bytes.len() {
            match bytes[idx] as char {
                '#' | 's' => {
                    accidental = 1;
                    idx += 1;
                }
                'b' => {
                    accidental = -1;
                    idx += 1;
                }
                _ => {}
            }
        }
        if idx == bytes.len() {
            let pc = (base_semitone + accidental).rem_euclid(12) as u8;
            return Ok(pc);
        }
    }

    let midi = parse_note_name(trimmed).or_else(|| parse_note_name(&trimmed.replace('-', "")));
    let Some(midi) = midi else {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: format!("invalid pitch class '{}'", trimmed),
        });
    };

    Ok(midi % 12)
}

/// Build a key context from harmony settings.
pub(super) fn build_key_context(
    harmony: &Harmony,
    pattern_name: &str,
) -> Result<KeyContext, ExpandError> {
    let root_pc = parse_pitch_class(&harmony.key.root, pattern_name)?;
    let scale_intervals = match harmony.key.scale {
        HarmonyScale::Major => [0, 2, 4, 5, 7, 9, 11],
        HarmonyScale::Minor => [0, 2, 3, 5, 7, 8, 10],
    };
    Ok(KeyContext {
        root_pc,
        _scale: harmony.key.scale,
        scale_intervals,
    })
}

/// Parse a chord specification (symbol or interval form).
pub(super) fn parse_chord_spec(
    spec: &ChordSpec,
    pattern_name: &str,
) -> Result<ParsedChord, ExpandError> {
    match spec {
        ChordSpec::Symbol(s) => parse_chord_symbol(&s.symbol, pattern_name),
        ChordSpec::Intervals(i) => {
            let root_pc = parse_pitch_class(&i.root, pattern_name)?;
            if !i.intervals.contains(&0) {
                return Err(ExpandError::InvalidExpr {
                    pattern: pattern_name.to_string(),
                    message: "chord interval form must include 0".to_string(),
                });
            }
            let mut intervals = i.intervals.clone();
            intervals.sort_unstable();
            intervals.dedup();
            let bass_pc = i
                .bass
                .as_ref()
                .map(|b| parse_pitch_class(b, pattern_name))
                .transpose()?;
            Ok(ParsedChord {
                root_pc,
                intervals,
                _bass_pc: bass_pc,
            })
        }
    }
}

/// Parse a chord symbol (e.g., "Cmaj7", "Am7b5", "G7#11") into intervals.
pub(super) fn parse_chord_symbol(
    symbol: &str,
    pattern_name: &str,
) -> Result<ParsedChord, ExpandError> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: "chord symbol cannot be empty".to_string(),
        });
    }

    let (main, bass) = match symbol.split_once('/') {
        Some((a, b)) => {
            if b.contains('/') {
                return Err(ExpandError::InvalidExpr {
                    pattern: pattern_name.to_string(),
                    message: format!("invalid chord symbol '{}'", symbol),
                });
            }
            (a.trim(), Some(b.trim()))
        }
        None => (symbol, None),
    };

    let bass_pc = bass
        .map(|b| parse_pitch_class(b, pattern_name))
        .transpose()?;

    let bytes = main.as_bytes();
    if bytes.is_empty() {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: format!("invalid chord symbol '{}'", symbol),
        });
    }
    let mut root_len = 1usize;
    if root_len < bytes.len() {
        match bytes[root_len] as char {
            '#' | 'b' | 's' => root_len += 1,
            _ => {}
        }
    }
    let root_str = &main[..root_len];
    let root_pc = parse_pitch_class(root_str, pattern_name)?;

    let rest_lower = main[root_len..].trim().to_ascii_lowercase();
    let mut rest = rest_lower.as_str();

    let mut intervals: Vec<u8> = vec![0, 4, 7];
    let mut base_third: Option<u8> = Some(4);
    let mut base_fifth: Option<u8> = Some(7);

    // Special combined forms (imply triad + seventh).
    if let Some(stripped) = rest.strip_prefix("m7b5") {
        intervals = vec![0, 3, 6, 10];
        base_third = Some(3);
        base_fifth = Some(6);
        rest = stripped;
    } else if let Some(stripped) = rest.strip_prefix("ø7") {
        intervals = vec![0, 3, 6, 10];
        base_third = Some(3);
        base_fifth = Some(6);
        rest = stripped;
    } else if let Some(stripped) = rest.strip_prefix("dim7") {
        intervals = vec![0, 3, 6, 9];
        base_third = Some(3);
        base_fifth = Some(6);
        rest = stripped;
    } else {
        // Quality (triad).
        if let Some(stripped) = rest.strip_prefix("sus2") {
            intervals = vec![0, 2, 7];
            base_third = Some(2);
            base_fifth = Some(7);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("sus4") {
            intervals = vec![0, 5, 7];
            base_third = Some(5);
            base_fifth = Some(7);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("sus") {
            intervals = vec![0, 5, 7];
            base_third = Some(5);
            base_fifth = Some(7);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("dim") {
            intervals = vec![0, 3, 6];
            base_third = Some(3);
            base_fifth = Some(6);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("aug") {
            intervals = vec![0, 4, 8];
            base_third = Some(4);
            base_fifth = Some(8);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("+") {
            intervals = vec![0, 4, 8];
            base_third = Some(4);
            base_fifth = Some(8);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("5") {
            intervals = vec![0, 7];
            base_third = None;
            base_fifth = Some(7);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("min") {
            intervals = vec![0, 3, 7];
            base_third = Some(3);
            base_fifth = Some(7);
            rest = stripped;
        } else if rest.starts_with('m') && !rest.starts_with("maj") {
            intervals = vec![0, 3, 7];
            base_third = Some(3);
            base_fifth = Some(7);
            rest = &rest[1..];
        }

        // Sevenths.
        if let Some(stripped) = rest.strip_prefix("maj7") {
            intervals.push(11);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("7") {
            intervals.push(10);
            rest = stripped;
        }
    }

    let ensure_seventh = |intervals: &mut Vec<u8>| {
        if !intervals.iter().any(|v| matches!(*v, 9..=11)) {
            intervals.push(10);
        }
    };
    let ensure_ninth = |intervals: &mut Vec<u8>| {
        if !intervals.iter().any(|v| matches!(*v, 13..=15)) {
            intervals.push(14);
        }
    };
    let ensure_eleven = |intervals: &mut Vec<u8>| {
        if !intervals.iter().any(|v| matches!(*v, 17 | 18)) {
            intervals.push(17);
        }
    };

    // Extensions / adds / alterations / omissions (prefix scan).
    while !rest.is_empty() {
        if let Some(stripped) = rest.strip_prefix("add13") {
            intervals.push(21);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("add11") {
            intervals.push(17);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("add9") {
            intervals.push(14);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("no3") {
            if let Some(third) = base_third {
                intervals.retain(|v| *v != third);
            }
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("no5") {
            if let Some(fifth) = base_fifth {
                intervals.retain(|v| *v != fifth);
            }
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("#11") {
            intervals.retain(|v| *v != 17 && *v != 18);
            intervals.push(18);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("b13") {
            intervals.retain(|v| *v != 20 && *v != 21);
            intervals.push(20);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("b9") {
            intervals.retain(|v| !matches!(*v, 13..=15));
            intervals.push(13);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("#9") {
            intervals.retain(|v| !matches!(*v, 13..=15));
            intervals.push(15);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("b5") {
            intervals.retain(|v| !matches!(*v, 6..=8));
            intervals.push(6);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("#5") {
            intervals.retain(|v| !matches!(*v, 6..=8));
            intervals.push(8);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("maj7") {
            intervals.push(11);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("7") {
            intervals.push(10);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("6") {
            intervals.push(9);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("13") {
            ensure_seventh(&mut intervals);
            ensure_ninth(&mut intervals);
            ensure_eleven(&mut intervals);
            intervals.push(21);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("11") {
            ensure_seventh(&mut intervals);
            ensure_ninth(&mut intervals);
            intervals.push(17);
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix("9") {
            ensure_seventh(&mut intervals);
            intervals.push(14);
            rest = stripped;
        } else {
            return Err(ExpandError::InvalidExpr {
                pattern: pattern_name.to_string(),
                message: format!("unsupported chord token '{}' in symbol '{}'", rest, symbol),
            });
        }
    }

    intervals.sort_unstable();
    intervals.dedup();
    if !intervals.contains(&0) {
        intervals.insert(0, 0);
    }

    Ok(ParsedChord {
        root_pc,
        intervals,
        _bass_pc: bass_pc,
    })
}

/// Parse a degree value (e.g., "1", "b3", "#5") into degree and accidental.
pub(super) fn parse_degree_value(
    value: &str,
    pattern_name: &str,
) -> Result<(i32, i32), ExpandError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: "degree value cannot be empty".to_string(),
        });
    }

    let mut accidental = 0i32;
    let mut rest = trimmed;
    while let Some(stripped) = rest.strip_prefix('b') {
        accidental -= 1;
        rest = stripped;
    }
    while let Some(stripped) = rest.strip_prefix('#') {
        accidental += 1;
        rest = stripped;
    }

    let degree = rest.parse::<i32>().map_err(|_| ExpandError::InvalidExpr {
        pattern: pattern_name.to_string(),
        message: format!("invalid degree value '{}'", trimmed),
    })?;

    Ok((degree, accidental))
}

/// Convert pitch (root + octave + semitones) to MIDI note number.
pub(super) fn midi_from_pitch(
    root_pc: u8,
    octave: i32,
    semitones: i32,
    pattern_name: &str,
) -> Result<u8, ExpandError> {
    let midi = (octave + 1)
        .checked_mul(12)
        .and_then(|v| v.checked_add(root_pc as i32))
        .and_then(|v| v.checked_add(semitones))
        .ok_or_else(|| ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: "pitch -> MIDI overflow".to_string(),
        })?;
    if !(0..=127).contains(&midi) {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: format!("pitch produced out-of-range MIDI note {}", midi),
        });
    }
    Ok(midi as u8)
}

/// Convert a scale degree to a note name.
pub(super) fn scale_degree_note_name(
    key: &KeyContext,
    value: &str,
    octave: i32,
    allow_accidentals: bool,
    pattern_name: &str,
) -> Result<String, ExpandError> {
    let (degree, accidental) = parse_degree_value(value, pattern_name)?;
    if !(1..=7).contains(&degree) {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: format!("scale_degree value '{}' must be 1..=7", value),
        });
    }
    if accidental != 0 && !allow_accidentals {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: format!(
                "scale_degree value '{}' uses accidentals (set allow_accidentals=true to permit)",
                value
            ),
        });
    }

    let base = key.scale_intervals[(degree - 1) as usize];
    let semitones = base + accidental;
    if semitones < 0 {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: format!("scale_degree value '{}' underflows", value),
        });
    }

    let midi = midi_from_pitch(key.root_pc, octave, semitones, pattern_name)?;
    Ok(midi_to_note_name(midi))
}

/// Convert a chord tone degree to a note name.
pub(super) fn chord_tone_note_name(
    chord: &ParsedChord,
    value: &str,
    octave: i32,
    pattern_name: &str,
) -> Result<String, ExpandError> {
    let (degree, accidental) = parse_degree_value(value, pattern_name)?;
    if accidental != 0 {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: format!("chord_tone value '{}' does not support accidentals", value),
        });
    }

    let candidates: &'static [u8] = match degree {
        1 => &[0],
        3 => &[4, 3],
        5 => &[7, 6, 8],
        7 => &[10, 11, 9],
        9 => &[14, 13, 15],
        11 => &[17, 18],
        13 => &[21, 20],
        _ => {
            return Err(ExpandError::InvalidExpr {
                pattern: pattern_name.to_string(),
                message: format!(
                    "chord_tone value '{}' must be one of 1,3,5,7,9,11,13",
                    value
                ),
            })
        }
    };

    let Some(interval) = candidates
        .iter()
        .copied()
        .find(|cand| chord.intervals.contains(cand))
    else {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: format!(
                "chord_tone value '{}' not present in chord intervals {:?}",
                value, chord.intervals
            ),
        });
    };

    let midi = midi_from_pitch(chord.root_pc, octave, interval as i32, pattern_name)?;
    Ok(midi_to_note_name(midi))
}

/// Select the active chord at a given row from a sorted list of chords.
pub(super) fn select_chord<'c>(
    chords: &'c [ChordAt],
    row: i32,
    pattern_name: &str,
) -> Result<&'c ParsedChord, ExpandError> {
    if chords.is_empty() {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: "no chords available".to_string(),
        });
    }

    let row_i64 = row as i64;
    let mut selected: Option<&ChordAt> = None;
    for chord in chords {
        if chord.at_row <= row_i64 {
            selected = Some(chord);
        } else {
            break;
        }
    }

    selected
        .map(|c| &c.chord)
        .ok_or_else(|| ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: format!("no chord defined at or before row {}", row),
        })
}

/// Build a chord context (list of chords with row positions) from harmony settings.
pub(super) fn build_chord_context<F>(
    harmony: &Harmony,
    beat_pos_to_row: F,
    pattern_name: &str,
) -> Result<Vec<ChordAt>, ExpandError>
where
    F: Fn(&BeatPos) -> Result<i64, ExpandError>,
{
    if harmony.chords.is_empty() {
        return Err(ExpandError::InvalidExpr {
            pattern: pattern_name.to_string(),
            message: "pitch_seq chord_tone requires harmony.chords".to_string(),
        });
    }

    let mut chords = Vec::with_capacity(harmony.chords.len());
    for (idx, entry) in harmony.chords.iter().enumerate() {
        let at_row = beat_pos_to_row(&entry.at)?;
        let chord = parse_chord_spec(&entry.chord, pattern_name)?;
        chords.push(ChordAt { at_row, idx, chord });
    }
    chords.sort_by(|a, b| a.at_row.cmp(&b.at_row).then(a.idx.cmp(&b.idx)));
    Ok(chords)
}
