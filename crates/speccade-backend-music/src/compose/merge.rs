//! Cell merging and transformation logic.

use std::collections::BTreeMap;

use speccade_spec::recipe::music::{MergePolicy, TransformOp};

use super::error::ExpandError;
use super::utils::transpose_note;

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

/// Apply a list of transforms to a cell.
pub(super) fn apply_transforms(
    cell: &mut Cell,
    ops: &[TransformOp],
    pattern_name: &str,
) -> Result<(), ExpandError> {
    for op in ops {
        match op {
            TransformOp::TransposeSemitones { semitones } => {
                if let Some(ref note) = cell.note {
                    if let Some(transposed) = transpose_note(note, *semitones, pattern_name)? {
                        cell.note = Some(transposed);
                    }
                }
            }
            TransformOp::VolMul { mul, div } => {
                if *div == 0 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: pattern_name.to_string(),
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
        }
    }
    Ok(())
}
