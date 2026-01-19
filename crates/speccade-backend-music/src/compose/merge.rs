//! Cell merging and transformation logic.

use std::collections::BTreeMap;

use rand::Rng;
use speccade_spec::recipe::audio::parse_note_name;
use speccade_spec::recipe::music::{MergePolicy, QuantizeScale, TransformOp};

use super::error::ExpandError;
use super::utils::{midi_to_note_name, rng_for_cell, transpose_note};

pub(super) type CellKey = (i32, u8);
pub(super) type CellMap = BTreeMap<CellKey, Cell>;

/// A tracker cell with optional fields.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Cell {
    pub note: Option<String>,
    pub inst: Option<u8>,
    pub vol: Option<u8>,
    pub effect: Option<u8>,
    pub param: Option<u8>,
    pub effect_name: Option<String>,
    pub effect_xy: Option<[u8; 2]>,
}

impl Cell {
    pub fn from_template(
        template: &speccade_spec::recipe::music::CellTemplate,
        inst: Option<u8>,
    ) -> Self {
        Self {
            note: template.note.clone(),
            inst,
            vol: template.vol,
            effect: template.effect,
            param: template.param,
            effect_name: template.effect_name.clone(),
            effect_xy: template.effect_xy,
        }
    }

    pub fn from_pattern_note(note: &speccade_spec::recipe::music::PatternNote) -> Self {
        Self {
            note: Some(note.note.clone()),
            inst: Some(note.inst),
            vol: note.vol,
            effect: note.effect,
            param: note.param,
            effect_name: note.effect_name.clone(),
            effect_xy: note.effect_xy,
        }
    }
}

/// Shift all rows in a cell map by an offset.
pub(super) fn shift_map(map: &mut CellMap, offset: i32) {
    if offset == 0 {
        return;
    }
    let moved = std::mem::take(map);
    let shifted: CellMap = moved
        .into_iter()
        .map(|((row, channel), cell)| ((row + offset, channel), cell))
        .collect();
    *map = shifted;
}

/// Insert a cell into a map, applying merge policy.
pub(super) fn insert_cell_merge(
    map: &mut CellMap,
    key: CellKey,
    cell: Cell,
    policy: MergePolicy,
    pattern_name: &str,
) -> Result<(), ExpandError> {
    if let Some(existing) = map.get_mut(&key) {
        if policy == MergePolicy::Error {
            return Err(ExpandError::MergeConflict {
                pattern: pattern_name.to_string(),
                row: key.0,
                channel: key.1,
                field: "cell",
            });
        }
        merge_cell_fields(existing, &cell, policy, key, pattern_name)?;
    } else {
        map.insert(key, cell);
    }
    Ok(())
}

/// Merge individual fields from incoming cell into existing cell.
fn merge_cell_fields(
    existing: &mut Cell,
    incoming: &Cell,
    policy: MergePolicy,
    key: CellKey,
    pattern_name: &str,
) -> Result<(), ExpandError> {
    merge_field(
        &mut existing.note,
        &incoming.note,
        policy,
        key,
        pattern_name,
        "note",
    )?;
    merge_field(
        &mut existing.inst,
        &incoming.inst,
        policy,
        key,
        pattern_name,
        "inst",
    )?;
    merge_field(
        &mut existing.vol,
        &incoming.vol,
        policy,
        key,
        pattern_name,
        "vol",
    )?;
    merge_field(
        &mut existing.effect,
        &incoming.effect,
        policy,
        key,
        pattern_name,
        "effect",
    )?;
    merge_field(
        &mut existing.param,
        &incoming.param,
        policy,
        key,
        pattern_name,
        "param",
    )?;
    merge_field(
        &mut existing.effect_name,
        &incoming.effect_name,
        policy,
        key,
        pattern_name,
        "effect_name",
    )?;
    merge_field(
        &mut existing.effect_xy,
        &incoming.effect_xy,
        policy,
        key,
        pattern_name,
        "effect_xy",
    )?;
    Ok(())
}

/// Merge a single field according to policy.
fn merge_field<T: Clone + PartialEq>(
    target: &mut Option<T>,
    incoming: &Option<T>,
    policy: MergePolicy,
    key: CellKey,
    pattern_name: &str,
    field: &'static str,
) -> Result<(), ExpandError> {
    let Some(value) = incoming else {
        return Ok(());
    };

    match target {
        None => {
            *target = Some(value.clone());
        }
        Some(existing) => {
            if existing != value {
                match policy {
                    MergePolicy::LastWins => {
                        *existing = value.clone();
                    }
                    MergePolicy::MergeFields | MergePolicy::Error => {
                        return Err(ExpandError::MergeConflict {
                            pattern: pattern_name.to_string(),
                            row: key.0,
                            channel: key.1,
                            field,
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

/// Context needed for transforms that use randomization.
pub(super) struct TransformContext<'a> {
    pub seed: u32,
    pub pattern_name: &'a str,
    pub key: CellKey,
}

/// Apply a list of transforms to a cell.
pub(super) fn apply_transforms(
    cell: &mut Cell,
    ops: &[TransformOp],
    ctx: &TransformContext<'_>,
) -> Result<(), ExpandError> {
    for op in ops {
        match op {
            TransformOp::TransposeSemitones { semitones } => {
                if let Some(ref note) = cell.note {
                    if let Some(transposed) = transpose_note(note, *semitones, ctx.pattern_name)? {
                        cell.note = Some(transposed);
                    }
                }
            }
            TransformOp::VolMul { mul, div } => {
                if *div == 0 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: ctx.pattern_name.to_string(),
                        message: "vol_mul div must be > 0".to_string(),
                    });
                }
                if let Some(vol) = cell.vol {
                    let scaled = (vol as u32).saturating_mul(*mul) / *div;
                    cell.vol = Some(std::cmp::min(scaled, 64) as u8);
                }
            }
            TransformOp::Set {
                inst,
                vol,
                effect,
                param,
                effect_name,
                effect_xy,
            } => {
                if cell.inst.is_none() {
                    cell.inst = *inst;
                }
                if cell.vol.is_none() {
                    cell.vol = *vol;
                }
                if cell.effect.is_none() {
                    cell.effect = *effect;
                }
                if cell.param.is_none() {
                    cell.param = *param;
                }
                if cell.effect_name.is_none() {
                    cell.effect_name = effect_name.clone();
                }
                if cell.effect_xy.is_none() {
                    cell.effect_xy = *effect_xy;
                }
            }
            TransformOp::HumanizeVol {
                min_vol,
                max_vol,
                seed_salt,
            } => {
                if min_vol > max_vol {
                    return Err(ExpandError::InvalidExpr {
                        pattern: ctx.pattern_name.to_string(),
                        message: format!(
                            "humanize_vol min_vol ({}) must be <= max_vol ({})",
                            min_vol, max_vol
                        ),
                    });
                }
                let (row, channel) = ctx.key;
                let mut rng = rng_for_cell(ctx.seed, ctx.pattern_name, seed_salt, row, channel);
                let vol = if min_vol == max_vol {
                    *min_vol
                } else {
                    rng.gen_range(*min_vol..=*max_vol)
                };
                cell.vol = Some(vol.min(64));
            }
            TransformOp::Swing {
                amount_permille,
                stride,
                seed_salt,
            } => {
                if *amount_permille > 1000 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: ctx.pattern_name.to_string(),
                        message: format!(
                            "swing amount_permille ({}) must be <= 1000",
                            amount_permille
                        ),
                    });
                }
                if *stride == 0 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: ctx.pattern_name.to_string(),
                        message: "swing stride must be > 0".to_string(),
                    });
                }
                let (row, channel) = ctx.key;
                // Check if this is an offbeat position (row % stride != 0)
                let is_offbeat = !((row as u32).is_multiple_of(*stride));
                if is_offbeat && *amount_permille > 0 {
                    // Apply note delay effect.
                    // The delay is expressed in ticks (0-15 for most trackers).
                    // We map amount_permille (0-1000) to delay ticks (0-15).
                    // At 1000 permille, we apply maximum delay (15 ticks).
                    let mut rng = rng_for_cell(ctx.seed, ctx.pattern_name, seed_salt, row, channel);
                    // Add some randomization to the delay for natural feel
                    let base_delay = (*amount_permille as u32 * 15) / 1000;
                    let jitter = if base_delay > 0 {
                        rng.gen_range(0..=1)
                    } else {
                        0
                    };
                    let delay = (base_delay + jitter).min(15) as u8;
                    if delay > 0 {
                        // Apply EDx (note delay) effect
                        // effect_name = "note_delay" and effect_xy = [delay, 0]
                        // Only apply if no effect is already set
                        if cell.effect_name.is_none() && cell.effect_xy.is_none() {
                            cell.effect_name = Some("note_delay".to_string());
                            cell.effect_xy = Some([delay, 0]);
                        }
                    }
                }
            }
            TransformOp::InvertPitch { pivot } => {
                if let Some(ref note) = cell.note {
                    // Skip special notes
                    let upper = note.to_uppercase();
                    if matches!(
                        upper.as_str(),
                        "---" | "..." | "OFF" | "===" | "^^^" | "CUT" | "FADE" | "~~~"
                    ) {
                        continue;
                    }

                    // Parse pivot note
                    let pivot_midi = parse_note_name(pivot).ok_or_else(|| ExpandError::InvalidExpr {
                        pattern: ctx.pattern_name.to_string(),
                        message: format!("invert_pitch: invalid pivot note '{}'", pivot),
                    })? as i32;

                    // Parse cell note
                    let Some(note_midi) = parse_note_name(note) else {
                        continue;
                    };
                    let note_midi = note_midi as i32;

                    // Calculate inverted position: new_midi = pivot - (note - pivot) = 2*pivot - note
                    let inverted = 2 * pivot_midi - note_midi;

                    // Clamp to valid MIDI range
                    if inverted < 0 || inverted > 127 {
                        return Err(ExpandError::InvalidExpr {
                            pattern: ctx.pattern_name.to_string(),
                            message: format!(
                                "invert_pitch produced out-of-range MIDI note {}",
                                inverted
                            ),
                        });
                    }

                    cell.note = Some(midi_to_note_name(inverted as u8));
                }
            }
            TransformOp::QuantizePitch { scale, root } => {
                if let Some(ref note) = cell.note {
                    // Skip special notes
                    let upper = note.to_uppercase();
                    if matches!(
                        upper.as_str(),
                        "---" | "..." | "OFF" | "===" | "^^^" | "CUT" | "FADE" | "~~~"
                    ) {
                        continue;
                    }

                    // Chromatic scale is a no-op
                    if *scale == QuantizeScale::Chromatic {
                        continue;
                    }

                    // Parse root note to get pitch class
                    let root_pc = parse_root_pitch_class(root).ok_or_else(|| {
                        ExpandError::InvalidExpr {
                            pattern: ctx.pattern_name.to_string(),
                            message: format!("quantize_pitch: invalid root note '{}'", root),
                        }
                    })?;

                    // Parse cell note
                    let Some(note_midi) = parse_note_name(note) else {
                        continue;
                    };
                    let note_midi = note_midi as i32;

                    // Get pitch class of note (relative to root)
                    let note_pc = (note_midi - root_pc as i32).rem_euclid(12) as u8;

                    // Find nearest scale degree
                    let intervals = scale.intervals();
                    let quantized_pc = find_nearest_scale_degree(note_pc, intervals);

                    // Calculate the quantized MIDI note
                    let octave = note_midi / 12;
                    let original_pc_in_oct = (note_midi % 12) as u8;
                    let quantized_abs = (root_pc + quantized_pc) % 12;
                    let quantized_midi = if quantized_abs <= original_pc_in_oct {
                        octave * 12 + quantized_abs as i32
                    } else {
                        // Quantized note is higher than original, keep in same octave
                        octave * 12 + quantized_abs as i32
                    };

                    // Clamp to valid MIDI range
                    let quantized_midi = quantized_midi.clamp(0, 127);

                    cell.note = Some(midi_to_note_name(quantized_midi as u8));
                }
            }
            TransformOp::Ratchet {
                divisions,
                seed_salt,
            } => {
                if *divisions == 0 || *divisions > 16 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: ctx.pattern_name.to_string(),
                        message: format!("ratchet divisions ({}) must be 1-16", divisions),
                    });
                }
                // Only apply if no effect is already set
                if cell.effect_name.is_none() && cell.effect_xy.is_none() {
                    let (row, channel) = ctx.key;
                    let mut rng = rng_for_cell(ctx.seed, ctx.pattern_name, seed_salt, row, channel);
                    // Decide whether to apply ratchet (50% chance by default)
                    if rng.gen_bool(0.5) {
                        // Apply E9x (retrigger) effect
                        // divisions maps to retrigger speed: 1=fast, 16=slow
                        // XM/IT E9x: x is the interval in ticks
                        let interval = (16 / *divisions).max(1);
                        cell.effect_name = Some("retrig".to_string());
                        cell.effect_xy = Some([interval, 0]);
                    }
                }
            }
            TransformOp::Arpeggiate {
                semitones_up,
                semitones_down,
            } => {
                if *semitones_up > 15 || *semitones_down > 15 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: ctx.pattern_name.to_string(),
                        message: format!(
                            "arpeggiate semitones must be 0-15 (got up={}, down={})",
                            semitones_up, semitones_down
                        ),
                    });
                }
                // Only apply if no effect is already set
                if cell.effect_name.is_none() && cell.effect_xy.is_none() {
                    cell.effect_name = Some("arpeggio".to_string());
                    cell.effect_xy = Some([*semitones_up, *semitones_down]);
                }
            }
        }
    }
    Ok(())
}

/// Parse a root note name (e.g., "C", "F#") to a pitch class (0-11).
fn parse_root_pitch_class(root: &str) -> Option<u8> {
    let root = root.trim().to_uppercase();
    match root.as_str() {
        "C" => Some(0),
        "C#" | "DB" => Some(1),
        "D" => Some(2),
        "D#" | "EB" => Some(3),
        "E" => Some(4),
        "F" => Some(5),
        "F#" | "GB" => Some(6),
        "G" => Some(7),
        "G#" | "AB" => Some(8),
        "A" => Some(9),
        "A#" | "BB" => Some(10),
        "B" => Some(11),
        _ => None,
    }
}

/// Find the nearest scale degree to a given pitch class.
/// Ties snap down (toward lower pitch).
fn find_nearest_scale_degree(note_pc: u8, intervals: &[u8]) -> u8 {
    let mut best_interval = intervals[0];
    let mut best_distance = 12u8; // Max possible distance

    for &interval in intervals {
        // Distance in semitones (wrapping around octave)
        let dist_up = (12 + interval - note_pc) % 12;
        let dist_down = (12 + note_pc - interval) % 12;
        let distance = dist_up.min(dist_down);

        if distance < best_distance || (distance == best_distance && interval < best_interval) {
            best_distance = distance;
            best_interval = interval;
        }
    }

    best_interval
}
