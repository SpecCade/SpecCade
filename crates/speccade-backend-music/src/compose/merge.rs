//! Cell merging and transformation logic.

use std::collections::BTreeMap;

use rand::Rng;
use speccade_spec::recipe::music::{MergePolicy, TransformOp};

use super::error::ExpandError;
use super::utils::{rng_for_cell, transpose_note};

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
        }
    }
    Ok(())
}
